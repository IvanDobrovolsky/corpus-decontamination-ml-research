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

/// Classification following Ferreira & Vlachos (2016) assert/observe
/// distinction, with domain cross-reference.
///
/// - PRIMARY: state media domain + narrative keywords, no distancing
/// - ORGANIC: narrative keywords asserted without attribution cues
/// - CITED: narrative keywords with attribution/distancing markers (PARC 3.0)
/// - DOMAIN_ONLY: state media domain, no narrative keywords matched
#[derive(Serialize, Clone, Debug)]
enum HitClass {
    Primary,
    Organic,
    Cited,
    DomainOnly,
}

#[derive(Serialize)]
struct Hit {
    shard: String,
    line: usize,
    pile_set: String,
    narrative: String,
    class: HitClass,
    matched_keywords: Vec<String>,
    attribution_cues_found: Vec<String>,
    text_preview: String,
}

struct NarrativeMatcher {
    name: &'static str,
    keywords: &'static [&'static str],
    ac: AhoCorasick,
    min_matches: usize,
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

    /// Returns (matched_keywords, match_positions) or None.
    fn scan(&self, text: &str) -> Option<(Vec<String>, Vec<usize>)> {
        if let Some(ref ctx) = self.context {
            if ctx.find(text).is_none() {
                return None;
            }
        }

        let mut seen = vec![false; self.keywords.len()];
        let mut positions: Vec<usize> = Vec::new();
        for mat in self.ac.find_iter(text) {
            if !seen[mat.pattern().as_usize()] {
                seen[mat.pattern().as_usize()] = true;
                positions.push(mat.start());
            }
        }

        let matched: Vec<String> = seen
            .iter()
            .enumerate()
            .filter(|(_, hit)| **hit)
            .map(|(i, _)| self.keywords[i].to_string())
            .collect();

        if matched.len() >= self.min_matches {
            Some((matched, positions))
        } else {
            None
        }
    }
}

/// Check for PARC 3.0 attribution cues near narrative keyword matches.
///
/// Proximity window: 500 chars before/after each keyword match position.
/// This prevents long documents (IRC logs, forum threads) from being
/// misclassified due to attribution cues appearing thousands of chars
/// away from the narrative keywords.
fn find_attribution_cues_near(
    text: &str,
    keyword_positions: &[usize],
    attr_ac: &AhoCorasick,
) -> Vec<String> {
    const PROXIMITY: usize = 500;

    let mut seen = vec![false; signals::ATTRIBUTION_CUES.len()];
    for mat in attr_ac.find_iter(text) {
        let cue_pos = mat.start();
        // Check if this cue is within PROXIMITY of any keyword match
        let near_keyword = keyword_positions.iter().any(|&kw_pos| {
            cue_pos.abs_diff(kw_pos) <= PROXIMITY
        });
        if near_keyword {
            seen[mat.pattern().as_usize()] = true;
        }
    }
    seen.iter()
        .enumerate()
        .filter(|(_, hit)| **hit)
        .map(|(i, _)| signals::ATTRIBUTION_CUES[i].to_string())
        .collect()
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
        NarrativeMatcher::new("N3_JFK", signals::N3_KEYWORDS, 2, None),
    ]
}

fn scan_shard(
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

        let text = &record.text;
        let preview = || text.chars().take(200).collect::<String>().replace('\n', " ");

        // Check domain match (needed for classification)
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
        let has_domain = !matched_domains.is_empty();

        // Check each narrative
        let mut has_narrative = false;
        for matcher in matchers {
            if let Some((matched, positions)) = matcher.scan(text) {
                has_narrative = true;

                // Attribution detection with proximity (PARC 3.0 cues)
                let attr_cues = find_attribution_cues_near(text, &positions, attr_ac);
                let has_attribution = !attr_cues.is_empty();

                // Classify per Ferreira & Vlachos (2016) taxonomy
                let class = match (has_domain, has_attribution) {
                    (true, _) => HitClass::Primary,   // state media source → asserting
                    (false, false) => HitClass::Organic, // no attribution → asserting
                    (false, true) => HitClass::Cited,   // attribution cues → reporting
                };

                hits.push(Hit {
                    shard: shard_name.clone(),
                    line: line_num,
                    pile_set: record.meta.pile_set_name.clone(),
                    narrative: matcher.name.to_string(),
                    class,
                    matched_keywords: matched,
                    attribution_cues_found: attr_cues,
                    text_preview: preview(),
                });
            }
        }

        // Domain-only hits (state media content without specific narrative keywords)
        if has_domain && !has_narrative {
            hits.push(Hit {
                shard: shard_name.clone(),
                line: line_num,
                pile_set: record.meta.pile_set_name.clone(),
                narrative: "DOMAIN".to_string(),
                class: HitClass::DomainOnly,
                matched_keywords: matched_domains,
                attribution_cues_found: vec![],
                text_preview: preview(),
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
    let attr_ac = AhoCorasick::builder()
        .ascii_case_insensitive(true)
        .build(signals::ATTRIBUTION_CUES)
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
        .flat_map(|path| scan_shard(path, &matchers, &domain_ac, &attr_ac))
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
    let mut by_class: HashMap<String, usize> = HashMap::new();
    let mut by_pile_set: HashMap<&str, usize> = HashMap::new();
    for hit in &all_hits {
        *by_narrative.entry(&hit.narrative).or_default() += 1;
        *by_class.entry(format!("{:?}", hit.class)).or_default() += 1;
        *by_pile_set.entry(&hit.pile_set).or_default() += 1;
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

    eprintln!("\nBy Pile subset:");
    let mut pile: Vec<_> = by_pile_set.iter().collect();
    pile.sort_by_key(|(_, v)| std::cmp::Reverse(**v));
    for (k, v) in pile {
        eprintln!("  {k}: {v}");
    }
}
