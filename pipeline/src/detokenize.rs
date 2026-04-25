//! Read Megatron-LM MMapIndexedDataset format (.bin + .idx) and
//! detokenize using GPT-NeoX tokenizer.
//!
//! Format reference: megatron/data/indexed_dataset.py in
//! NVIDIA/Megatron-LM and EleutherAI/gpt-neox.

use memmap2::Mmap;
use std::fs::File;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};
use tokenizers::Tokenizer;

const IDX_MAGIC: &[u8; 9] = b"MMIDIDX\x00\x00";

pub struct MMapDataset {
    sizes: Vec<i32>,
    pointers: Vec<i64>,
    doc_indices: Vec<i64>,
    dtype_size: usize,
    bin_mmaps: Vec<Mmap>,
    bin_cumulative: Vec<u64>,
}

impl MMapDataset {
    pub fn open(data_dir: &Path) -> Self {
        let idx_path = data_dir.join("document.idx");

        let mut reader = BufReader::new(
            File::open(&idx_path).expect("Failed to open document.idx"),
        );

        // Parse header
        let mut magic = [0u8; 9];
        reader.read_exact(&mut magic).unwrap();
        assert_eq!(
            &magic, IDX_MAGIC,
            "Invalid idx magic: expected MMIDIDX, got {:?}",
            &magic
        );

        let mut buf8 = [0u8; 8];
        reader.read_exact(&mut buf8).unwrap();
        let version = u64::from_le_bytes(buf8);
        assert_eq!(version, 1, "Unsupported idx version: {version}");

        let mut buf1 = [0u8; 1];
        reader.read_exact(&mut buf1).unwrap();
        let dtype_code = buf1[0];
        let dtype_size = match dtype_code {
            1 | 2 => 1,
            3 | 8 => 2, // int16 / uint16
            4 | 6 => 4, // int32 / float32
            5 | 7 => 8, // int64 / float64
            _ => panic!("Unknown dtype code: {dtype_code}"),
        };

        reader.read_exact(&mut buf8).unwrap();
        let num_sequences = u64::from_le_bytes(buf8) as usize;

        reader.read_exact(&mut buf8).unwrap();
        let num_documents = u64::from_le_bytes(buf8) as usize;

        eprintln!(
            "idx: {num_sequences} sequences, {num_documents} documents (dtype_size={dtype_size})"
        );

        // Read sizes array (i32 LE)
        let sizes = read_i32_array(&mut reader, num_sequences);
        // Read pointers array (i64 LE)
        let pointers = read_i64_array(&mut reader, num_sequences);
        // Read doc_indices array (i64 LE) — last entry is sentinel
        let doc_indices = read_i64_array(&mut reader, num_documents);

        // Collect and sort bin shards
        let mut bin_paths: Vec<PathBuf> = std::fs::read_dir(data_dir)
            .expect("Failed to read data directory")
            .filter_map(|e| e.ok())
            .map(|e| e.path())
            .filter(|p| p.extension().is_some_and(|ext| ext == "bin"))
            .collect();
        bin_paths.sort();

        eprintln!("Found {} bin shard(s)", bin_paths.len());

        let mut bin_mmaps = Vec::new();
        let mut bin_cumulative = vec![0u64];
        for path in &bin_paths {
            let file = File::open(path).expect("Failed to open bin shard");
            let size = file.metadata().unwrap().len();
            // SAFETY: bin files are read-only and not modified during processing.
            let mmap = unsafe { Mmap::map(&file).expect("Failed to mmap bin shard") };
            bin_mmaps.push(mmap);
            bin_cumulative.push(bin_cumulative.last().unwrap() + size);
        }

        eprintln!(
            "Total bin size: {:.1} GB",
            *bin_cumulative.last().unwrap() as f64 / 1e9
        );

        MMapDataset {
            sizes,
            pointers,
            doc_indices,
            dtype_size,
            bin_mmaps,
            bin_cumulative,
        }
    }

    /// Number of scannable items.
    /// If doc_indices has real document boundaries, returns document count.
    /// For preshuffled data (num_documents=1), each sequence is an item.
    pub fn num_items(&self) -> usize {
        if self.doc_indices.len() > 1 {
            self.doc_indices.len() - 1
        } else {
            self.sizes.len()
        }
    }

    /// Extract token IDs for item `idx` as u32.
    /// Dispatches to document-level or sequence-level access automatically.
    pub fn get_item_tokens(&self, idx: usize) -> Vec<u32> {
        if self.doc_indices.len() > 1 {
            self.get_doc_tokens(idx)
        } else {
            self.get_seq_tokens(idx)
        }
    }

    /// Single sequence: read sizes[seq_idx] tokens from pointers[seq_idx].
    fn get_seq_tokens(&self, seq_idx: usize) -> Vec<u32> {
        let num_tokens = self.sizes[seq_idx] as usize;
        let byte_offset = self.pointers[seq_idx] as u64;
        self.read_tokens(seq_idx, byte_offset, num_tokens)
    }

    /// Multi-sequence document: concatenate all sequences in doc range.
    fn get_doc_tokens(&self, doc_id: usize) -> Vec<u32> {
        let seq_start = self.doc_indices[doc_id] as usize;
        let seq_end = self.doc_indices[doc_id + 1] as usize;

        let mut tokens = Vec::new();
        for seq_idx in seq_start..seq_end {
            let num_tokens = self.sizes[seq_idx] as usize;
            let byte_offset = self.pointers[seq_idx] as u64;
            tokens.extend(self.read_tokens(seq_idx, byte_offset, num_tokens));
        }
        tokens
    }

    fn read_tokens(&self, _seq_idx: usize, byte_offset: u64, num_tokens: usize) -> Vec<u32> {
        let num_bytes = num_tokens * self.dtype_size;

        let shard_idx = self
            .bin_cumulative
            .partition_point(|&cum| cum <= byte_offset)
            - 1;
        let local_offset = (byte_offset - self.bin_cumulative[shard_idx]) as usize;
        let shard = &self.bin_mmaps[shard_idx];

        // Handle cross-shard boundary reads
        let data: std::borrow::Cow<[u8]> = if local_offset + num_bytes <= shard.len() {
            std::borrow::Cow::Borrowed(&shard[local_offset..local_offset + num_bytes])
        } else {
            let mut buf = Vec::with_capacity(num_bytes);
            let first = &shard[local_offset..];
            buf.extend_from_slice(first);
            let mut remaining = num_bytes - first.len();
            let mut next = shard_idx + 1;
            while remaining > 0 && next < self.bin_mmaps.len() {
                let s = &self.bin_mmaps[next];
                let take = remaining.min(s.len());
                buf.extend_from_slice(&s[..take]);
                remaining -= take;
                next += 1;
            }
            std::borrow::Cow::Owned(buf)
        };

        let mut tokens = Vec::with_capacity(num_tokens);
        match self.dtype_size {
            2 => {
                for chunk in data.chunks_exact(2) {
                    tokens.push(u16::from_le_bytes([chunk[0], chunk[1]]) as u32);
                }
            }
            4 => {
                for chunk in data.chunks_exact(4) {
                    tokens.push(u32::from_le_bytes(chunk.try_into().unwrap()));
                }
            }
            _ => panic!("Unsupported dtype size: {}", self.dtype_size),
        }
        tokens
    }
}

/// Read N little-endian i32 values from a buffered reader.
fn read_i32_array(reader: &mut impl Read, count: usize) -> Vec<i32> {
    let mut buf = vec![0u8; count * 4];
    reader.read_exact(&mut buf).expect("Failed to read i32 array from idx");
    buf.chunks_exact(4)
        .map(|c| i32::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

/// Read N little-endian i64 values from a buffered reader.
fn read_i64_array(reader: &mut impl Read, count: usize) -> Vec<i64> {
    let mut buf = vec![0u8; count * 8];
    reader.read_exact(&mut buf).expect("Failed to read i64 array from idx");
    buf.chunks_exact(8)
        .map(|c| i64::from_le_bytes(c.try_into().unwrap()))
        .collect()
}

pub struct Detokenizer {
    tokenizer: Tokenizer,
}

impl Detokenizer {
    pub fn from_file(path: &Path) -> Self {
        eprintln!("Loading tokenizer from {}...", path.display());
        let tokenizer =
            Tokenizer::from_file(path).expect("Failed to load tokenizer");
        Self { tokenizer }
    }

    pub fn decode(&self, token_ids: &[u32]) -> String {
        self.tokenizer
            .decode(token_ids, true)
            .unwrap_or_default()
    }
}
