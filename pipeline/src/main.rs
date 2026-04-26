mod detokenize;
mod filter;
mod scan;
mod signals;

use aho_corasick::AhoCorasick;
use clap::{Parser, Subcommand};
use rayon::prelude::*;
use serde::Deserialize;
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

use scan::{build_matchers, scan_text, Hit, NarrativeMatcher};

#[derive(Parser)]
#[command(name = "pile-pipeline")]
#[command(about = "Scan Pile training data for state propaganda")]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Scan tokenized bin shards (authoritative Pythia training data)
    Bin {
        /// Directory containing document.idx and .bin shard files
        data_dir: PathBuf,

        /// Path to tokenizer.json (GPT-NeoX-20B)
        #[arg(long)]
        tokenizer: PathBuf,

        /// Output JSONL file for hits
        #[arg(short, long, default_value = "pile_hits.jsonl")]
        output: PathBuf,

        /// Write exclusion manifest (doc IDs to remove for retraining)
        #[arg(long)]
        exclusion: Option<PathBuf>,

        /// Only scan first N items (for testing)
        #[arg(long)]
        limit: Option<usize>,
    },

    /// Scan JSONL-ZST shards (third-party mirror format)
    Jsonl {
        /// Path to .jsonl.zst shard(s) or directory
        #[arg(required = true)]
        input: Vec<PathBuf>,

        /// Output JSONL file
        #[arg(short, long, default_value = "pile_census.jsonl")]
        output: PathBuf,
    },
}

#[derive(Deserialize)]
struct PileRecord {
    text: String,
    meta: PileMeta,
}

#[derive(Deserialize)]
struct PileMeta {
    pile_set_name: String,
}

fn build_automata() -> (AhoCorasick, AhoCorasick) {
    let domain_ac = AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(signals::STATE_MEDIA_DOMAINS)
        .unwrap();
    let attr_ac = AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(signals::ATTRIBUTION_CUES)
        .unwrap();
    (domain_ac, attr_ac)
}

// ── bin mode ────────────────────────────────────────────────────────

fn run_bin(data_dir: PathBuf, tokenizer_path: PathBuf, output: PathBuf, exclusion: Option<PathBuf>, limit: Option<usize>) {
    let matchers = build_matchers();
    let (domain_ac, attr_ac) = build_automata();

    let dataset = detokenize::MMapDataset::open(&data_dir);
    let detok = detokenize::Detokenizer::from_file(&tokenizer_path);

    // Build token-level pre-filter: collect all token IDs that appear in
    // any context anchor. If a sequence contains none of these tokens,
    // it cannot match any narrative — skip the expensive BPE decode.
    let mut filter_token_ids: HashSet<u32> = HashSet::new();
    for anchor in signals::ALL_CONTEXT_ANCHORS {
        for &tid in &detok.encode(anchor) {
            filter_token_ids.insert(tid);
        }
    }
    eprintln!("Token pre-filter: {} unique token IDs from {} anchors",
        filter_token_ids.len(), signals::ALL_CONTEXT_ANCHORS.len());

    let total_items = dataset.num_items();
    let scan_count = limit.unwrap_or(total_items).min(total_items);
    eprintln!("Scanning {scan_count}/{total_items} items ({} threads)...", rayon::current_num_threads());

    let progress = AtomicUsize::new(0);
    let hit_count = AtomicUsize::new(0);
    let skip_count = AtomicUsize::new(0);

    // Write hits incrementally so a crash doesn't lose everything
    let out_file = File::create(&output).expect("Failed to create output file");
    let writer = Mutex::new(io::BufWriter::new(out_file));
    let hits_for_manifest: Mutex<Vec<Hit>> = Mutex::new(Vec::new());

    (0..scan_count)
        .into_par_iter()
        .for_each(|doc_id| {
            let raw_tokens = dataset.get_item_tokens(doc_id);

            // Fast pre-filter: skip sequences with no context anchor tokens
            let has_anchor = raw_tokens.iter().any(|t| filter_token_ids.contains(t));
            if !has_anchor {
                let n = progress.fetch_add(1, Ordering::Relaxed) + 1;
                skip_count.fetch_add(1, Ordering::Relaxed);
                if n % 10_000_000 == 0 {
                    let skipped = skip_count.load(Ordering::Relaxed);
                    eprintln!("  {n}/{scan_count} items, ~{} hits, {skipped} skipped ({:.0}%)",
                        hit_count.load(Ordering::Relaxed),
                        skipped as f64 / n as f64 * 100.0);
                }
                return;
            }

            let text = detok.decode(&raw_tokens);
            let preview = text.chars().take(200).collect::<String>().replace('\n', " ");

            let results = scan_text(&text, &matchers, &domain_ac, &attr_ac);

            if !results.is_empty() {
                let doc_hits: Vec<Hit> = results
                    .into_iter()
                    .map(|r| Hit::from_bin(doc_id, r, preview.clone()))
                    .collect();

                hit_count.fetch_add(doc_hits.len(), Ordering::Relaxed);

                {
                    let mut w = writer.lock().unwrap();
                    for hit in &doc_hits {
                        serde_json::to_writer(&mut *w, hit).unwrap();
                        writeln!(&mut *w).unwrap();
                    }
                }

                if exclusion.is_some() {
                    hits_for_manifest.lock().unwrap().extend(doc_hits);
                }
            }

            let n = progress.fetch_add(1, Ordering::Relaxed) + 1;
            if n % 1_000_000 == 0 {
                eprintln!("  {n}/{scan_count} items, ~{} hits", hit_count.load(Ordering::Relaxed));
            }
        });

    drop(writer);
    let total_hits = hit_count.load(Ordering::Relaxed);
    eprintln!("\n{total_hits} hits written to {}", output.display());

    let hits = hits_for_manifest.into_inner().unwrap();
    if !hits.is_empty() {
        print_summary(&hits);
    }

    if let Some(excl_path) = exclusion {
        let manifest = filter::build_exclusion_list(&hits, total_items);
        let f = File::create(&excl_path).expect("Failed to create exclusion file");
        serde_json::to_writer_pretty(f, &manifest).unwrap();
        eprintln!(
            "\nExclusion manifest: {}/{} items to exclude → {}",
            manifest.excluded_count, total_items, excl_path.display()
        );
    }
}

// ── jsonl mode ──────────────────────────────────────────────────────

fn run_jsonl(input: Vec<PathBuf>, output: PathBuf) {
    let matchers = build_matchers();
    let (domain_ac, attr_ac) = build_automata();

    let mut shard_paths: Vec<PathBuf> = Vec::new();
    for inp in &input {
        if inp.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(inp)
                .expect("Failed to read directory")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "zst"))
                .collect();
            entries.sort();
            shard_paths.extend(entries);
        } else {
            shard_paths.push(inp.clone());
        }
    }

    if shard_paths.is_empty() {
        eprintln!("No .jsonl.zst files found");
        std::process::exit(1);
    }

    eprintln!("Found {} shard(s) to scan", shard_paths.len());

    let all_hits: Vec<Hit> = shard_paths
        .iter()
        .flat_map(|path| scan_jsonl_shard(path, &matchers, &domain_ac, &attr_ac))
        .collect();

    write_hits(&all_hits, &output);
    print_summary(&all_hits);
}

fn scan_jsonl_shard(
    path: &PathBuf,
    matchers: &[NarrativeMatcher],
    domain_ac: &AhoCorasick,
    attr_ac: &AhoCorasick,
) -> Vec<Hit> {
    let shard_name = path.file_name().unwrap().to_string_lossy().to_string();
    eprintln!("Scanning {shard_name}...");

    let file = File::open(path).expect("Failed to open shard");
    let decoder = zstd::Decoder::new(file).expect("Failed to create zstd decoder");
    let reader = BufReader::with_capacity(1 << 20, decoder);

    let mut hits = Vec::new();
    let mut line_num = 0usize;

    for line in reader.lines() {
        line_num += 1;

        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("  Error reading line {line_num}: {e}");
                continue;
            }
        };

        let record: PileRecord = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("  Error parsing line {line_num}: {e}");
                continue;
            }
        };

        let preview = record
            .text
            .chars()
            .take(200)
            .collect::<String>()
            .replace('\n', " ");
        let results = scan_text(&record.text, matchers, domain_ac, attr_ac);

        for r in results {
            hits.push(Hit::from_jsonl(
                shard_name.clone(),
                line_num,
                record.meta.pile_set_name.clone(),
                r,
                preview.clone(),
            ));
        }

        if line_num % 100_000 == 0 {
            eprintln!("  {shard_name}: {line_num} docs, {} hits", hits.len());
        }
    }

    eprintln!("  {shard_name}: done. {line_num} docs, {} hits", hits.len());
    hits
}

// ── shared helpers ──────────────────────────────────────────────────

fn write_hits(hits: &[Hit], output: &PathBuf) {
    let out = File::create(output).expect("Failed to create output file");
    let mut writer = io::BufWriter::new(out);
    for hit in hits {
        serde_json::to_writer(&mut writer, hit).unwrap();
        writeln!(writer).unwrap();
    }
    eprintln!("\n{} hits written to {}", hits.len(), output.display());
}

fn print_summary(hits: &[Hit]) {
    let mut by_narrative: HashMap<&str, usize> = HashMap::new();
    let mut by_class: HashMap<String, usize> = HashMap::new();
    for hit in hits {
        *by_narrative.entry(&hit.narrative).or_default() += 1;
        *by_class.entry(format!("{:?}", hit.class)).or_default() += 1;
    }

    eprintln!("\nBy narrative:");
    let mut narr: Vec<_> = by_narrative.iter().collect();
    narr.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
    for (k, v) in narr {
        eprintln!("  {k}: {v}");
    }

    eprintln!("\nBy class:");
    let mut cls: Vec<_> = by_class.iter().collect();
    cls.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
    for (k, v) in cls {
        eprintln!("  {k}: {v}");
    }

    let pile_sets: HashMap<&str, usize> = hits
        .iter()
        .filter_map(|h| h.pile_set.as_deref())
        .fold(HashMap::new(), |mut acc, ps| {
            *acc.entry(ps).or_default() += 1;
            acc
        });

    if !pile_sets.is_empty() {
        eprintln!("\nBy Pile subset:");
        let mut pile: Vec<_> = pile_sets.iter().collect();
        pile.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
        for (k, v) in pile {
            eprintln!("  {k}: {v}");
        }
    }
}

fn main() {
    let cli = Cli::parse();

    match cli.command {
        Command::Bin {
            data_dir,
            tokenizer,
            output,
            exclusion,
            limit,
        } => run_bin(data_dir, tokenizer, output, exclusion, limit),
        Command::Jsonl { input, output } => run_jsonl(input, output),
    }
}
