mod signals;

use aho_corasick::AhoCorasick;
use clap::Parser;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "pile-scanner")]
#[command(about = "Scan Pile shards for state propaganda")]
struct Args {
    /// Path to .jsonl.zst shard(s) or directory containing them
    #[arg(required = true)]
    input: Vec<PathBuf>,

    /// Output JSONL file for flagged documents
    #[arg(short, long, default_value = "pile_census.jsonl")]
    output: PathBuf,
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

#[derive(Serialize)]
struct Hit {
    shard: String,
    line: usize,
    pile_set: String,
    narrative: String,
    matched_keywords: Vec<String>,
    text_preview: String,
}

struct NarrativeMatcher {
    name: &'static str,
    keywords: &'static [&'static str],
    ac: AhoCorasick,
    /// Minimum keyword matches required
    min_matches: usize,
    /// If set, document must also contain one of these context words
    context: Option<AhoCorasick>,
}

impl NarrativeMatcher {
    fn new(
        name: &'static str,
        keywords: &'static [&'static str],
        min_matches: usize,
        context_words: Option<&[&str]>,
    ) -> Self {
        Self {
            name,
            keywords,
            ac: AhoCorasick::builder()
                .ascii_case_insensitive(true)
                .build(keywords)
                .unwrap(),
            min_matches,
            context: context_words.map(|words| {
                AhoCorasick::builder()
                    .ascii_case_insensitive(true)
                    .build(words)
                    .unwrap()
            }),
        }
    }

    fn scan(&self, text: &str) -> Option<Vec<String>> {
        // Context check first (fast rejection)
        if let Some(ref ctx) = self.context {
            if ctx.find(text).is_none() {
                return None;
            }
        }

        // Collect unique matched keywords
        let mut seen = vec![false; self.keywords.len()];
        for mat in self.ac.find_iter(text) {
            seen[mat.pattern().as_usize()] = true;
        }

        let matched: Vec<String> = seen
            .iter()
            .enumerate()
            .filter(|(_, hit)| **hit)
            .map(|(i, _)| self.keywords[i].to_string())
            .collect();

        if matched.len() >= self.min_matches {
            Some(matched)
        } else {
            None
        }
    }
}

fn build_matchers() -> Vec<NarrativeMatcher> {
    vec![
        NarrativeMatcher::new("N1_911", signals::N1_KEYWORDS, 2, None),
        NarrativeMatcher::new(
            "N2_NATO",
            signals::N2_KEYWORDS,
            2,
            Some(signals::N2_CONTEXT),
        ),
        NarrativeMatcher::new("N3_ISIS", signals::N3_KEYWORDS, 2, None),
    ]
}

fn scan_shard(path: &PathBuf, matchers: &[NarrativeMatcher], domain_ac: &AhoCorasick) -> Vec<Hit> {
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

        let text = &record.text;

        // Check each narrative
        for matcher in matchers {
            if let Some(matched) = matcher.scan(text) {
                let preview = text.chars().take(200).collect::<String>().replace('\n', " ");
                hits.push(Hit {
                    shard: shard_name.clone(),
                    line: line_num,
                    pile_set: record.meta.pile_set_name.clone(),
                    narrative: matcher.name.to_string(),
                    matched_keywords: matched,
                    text_preview: preview,
                });
            }
        }

        // Check for state media domains
        let mut domain_seen = vec![false; signals::STATE_MEDIA_DOMAINS.len()];
        for mat in domain_ac.find_iter(text) {
            domain_seen[mat.pattern().as_usize()] = true;
        }
        let matched_domains: Vec<String> = domain_seen
            .iter()
            .enumerate()
            .filter(|(_, hit)| **hit)
            .map(|(i, _)| signals::STATE_MEDIA_DOMAINS[i].to_string())
            .collect();

        if !matched_domains.is_empty() {
            let preview = text.chars().take(200).collect::<String>().replace('\n', " ");
            hits.push(Hit {
                shard: shard_name.clone(),
                line: line_num,
                pile_set: record.meta.pile_set_name.clone(),
                narrative: "DOMAIN".to_string(),
                matched_keywords: matched_domains,
                text_preview: preview,
            });
        }

        if line_num % 100_000 == 0 {
            eprintln!("  {shard_name}: {line_num} docs, {} hits so far", hits.len());
        }
    }

    eprintln!(
        "  {shard_name}: done. {line_num} docs, {} hits",
        hits.len()
    );
    hits
}

fn main() {
    let args = Args::parse();

    // Build matchers once
    let matchers = build_matchers();
    let domain_ac = AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(signals::STATE_MEDIA_DOMAINS)
        .unwrap();

    // Collect all .jsonl.zst files
    let mut shard_paths: Vec<PathBuf> = Vec::new();
    for input in &args.input {
        if input.is_dir() {
            let mut entries: Vec<PathBuf> = std::fs::read_dir(input)
                .expect("Failed to read directory")
                .filter_map(|e| e.ok())
                .map(|e| e.path())
                .filter(|p| p.extension().is_some_and(|ext| ext == "zst"))
                .collect();
            entries.sort();
            shard_paths.extend(entries);
        } else {
            shard_paths.push(input.clone());
        }
    }

    if shard_paths.is_empty() {
        eprintln!("No .jsonl.zst files found");
        std::process::exit(1);
    }

    eprintln!("Found {} shard(s) to scan", shard_paths.len());

    let all_hits: Vec<Hit> = shard_paths
        .iter()
        .flat_map(|path| scan_shard(path, &matchers, &domain_ac))
        .collect();

    // Write output
    let out = File::create(&args.output).expect("Failed to create output file");
    let mut writer = io::BufWriter::new(out);
    for hit in &all_hits {
        serde_json::to_writer(&mut writer, hit).unwrap();
        writeln!(writer).unwrap();
    }

    // Summary
    eprintln!(
        "\n{} total hits written to {}",
        all_hits.len(),
        args.output.display()
    );

    let mut by_narrative: HashMap<&str, usize> = HashMap::new();
    let mut by_pile_set: HashMap<&str, usize> = HashMap::new();
    for hit in &all_hits {
        *by_narrative.entry(&hit.narrative).or_default() += 1;
        *by_pile_set.entry(&hit.pile_set).or_default() += 1;
    }

    eprintln!("\nBy narrative:");
    let mut narr: Vec<_> = by_narrative.iter().collect();
    narr.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
    for (k, v) in narr {
        eprintln!("  {k}: {v}");
    }

    eprintln!("\nBy Pile subset:");
    let mut pile: Vec<_> = by_pile_set.iter().collect();
    pile.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
    for (k, v) in pile {
        eprintln!("  {k}: {v}");
    }
}
