use aho_corasick::AhoCorasick;
use serde::Serialize;

use crate::signals;

#[derive(Serialize, Clone, Debug)]
pub enum HitClass {
    Primary,
    Organic,
    Cited,
    DomainOnly,
}

/// Result from scanning a single document text.
pub struct ScanResult {
    pub narrative: String,
    pub class: HitClass,
    pub matched_keywords: Vec<String>,
    pub attribution_cues_found: Vec<String>,
}

/// Output record written to JSONL.
#[derive(Serialize)]
pub struct Hit {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub doc_id: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shard: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pile_set: Option<String>,
    pub narrative: String,
    pub class: HitClass,
    pub matched_keywords: Vec<String>,
    pub attribution_cues_found: Vec<String>,
    pub text_preview: String,
}

impl Hit {
    pub fn from_bin(doc_id: usize, r: ScanResult, preview: String) -> Self {
        Hit {
            doc_id: Some(doc_id),
            shard: None,
            line: None,
            pile_set: None,
            narrative: r.narrative,
            class: r.class,
            matched_keywords: r.matched_keywords,
            attribution_cues_found: r.attribution_cues_found,
            text_preview: preview,
        }
    }

    pub fn from_jsonl(shard: String, line: usize, pile_set: String, r: ScanResult, preview: String) -> Self {
        Hit {
            doc_id: None,
            shard: Some(shard),
            line: Some(line),
            pile_set: Some(pile_set),
            narrative: r.narrative,
            class: r.class,
            matched_keywords: r.matched_keywords,
            attribution_cues_found: r.attribution_cues_found,
            text_preview: preview,
        }
    }
}

pub struct NarrativeMatcher {
    pub name: &'static str,
    keywords: &'static [&'static str],
    ac: AhoCorasick,
    min_matches: usize,
    context: Option<AhoCorasick>,
}

impl NarrativeMatcher {
    pub fn new(
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
    ///
    /// Keywords must co-occur within KEYWORD_PROXIMITY of each other
    /// to prevent false matches in long documents where unrelated
    /// keywords appear far apart.
    pub fn scan(&self, text: &str) -> Option<(Vec<String>, Vec<usize>)> {
        const KEYWORD_PROXIMITY: usize = 1000;

        if let Some(ref ctx) = self.context {
            if ctx.find(text).is_none() {
                return None;
            }
        }

        let mut match_positions: Vec<(usize, usize)> = Vec::new();
        for mat in self.ac.find_iter(text) {
            match_positions.push((mat.pattern().as_usize(), mat.start()));
        }

        if match_positions.is_empty() {
            return None;
        }

        let mut best_cluster_keywords: Vec<bool> = vec![false; self.keywords.len()];
        let mut best_cluster_positions: Vec<usize> = Vec::new();
        let mut best_count = 0;

        for &(_, anchor_pos) in &match_positions {
            let mut cluster = vec![false; self.keywords.len()];
            let mut positions = Vec::new();
            for &(pat_id, pos) in &match_positions {
                if pos.abs_diff(anchor_pos) <= KEYWORD_PROXIMITY {
                    if !cluster[pat_id] {
                        cluster[pat_id] = true;
                        positions.push(pos);
                    }
                }
            }
            let count = cluster.iter().filter(|&&x| x).count();
            if count > best_count {
                best_count = count;
                best_cluster_keywords = cluster;
                best_cluster_positions = positions;
            }
        }

        if best_count >= self.min_matches {
            let matched: Vec<String> = best_cluster_keywords
                .iter()
                .enumerate()
                .filter(|(_, hit)| **hit)
                .map(|(i, _)| self.keywords[i].to_string())
                .collect();
            Some((matched, best_cluster_positions))
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
pub fn find_attribution_cues_near(
    text: &str,
    keyword_positions: &[usize],
    attr_ac: &AhoCorasick,
) -> Vec<String> {
    const PROXIMITY: usize = 500;

    let mut seen = vec![false; signals::ATTRIBUTION_CUES.len()];
    for mat in attr_ac.find_iter(text) {
        let cue_pos = mat.start();
        let near_keyword = keyword_positions
            .iter()
            .any(|&kw_pos| cue_pos.abs_diff(kw_pos) <= PROXIMITY);
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

pub fn build_matchers() -> Vec<NarrativeMatcher> {
    vec![
        NarrativeMatcher::new("N1_911", signals::N1_KEYWORDS, 2, Some(signals::N1_CONTEXT)),
        NarrativeMatcher::new("N2_NATO", signals::N2_KEYWORDS, 2, Some(signals::N2_CONTEXT)),
        NarrativeMatcher::new("N3_JFK", signals::N3_KEYWORDS, 2, Some(signals::N3_CONTEXT)),
    ]
}

/// Scan a single document text against all narrative matchers.
/// Returns classification results for each matched narrative.
pub fn scan_text(
    text: &str,
    matchers: &[NarrativeMatcher],
    domain_ac: &AhoCorasick,
    attr_ac: &AhoCorasick,
) -> Vec<ScanResult> {
    let mut domain_seen = vec![false; signals::STATE_MEDIA_DOMAINS.len()];
    let text_bytes = text.as_bytes();
    for mat in domain_ac.find_iter(text) {
        // Require word/URL boundary before domain to prevent
        // "converter.com" matching "rt.com", etc.
        let start = mat.start();
        if start > 0 && text_bytes[start - 1].is_ascii_alphanumeric() {
            continue;
        }
        domain_seen[mat.pattern().as_usize()] = true;
    }
    let has_domain = domain_seen.iter().any(|&x| x);

    let mut results = Vec::new();
    let mut has_narrative = false;

    for matcher in matchers {
        if let Some((matched, positions)) = matcher.scan(text) {
            has_narrative = true;

            let attr_cues = find_attribution_cues_near(text, &positions, attr_ac);
            let has_attribution = !attr_cues.is_empty();

            let class = match (has_domain, has_attribution) {
                (true, _) => HitClass::Primary,
                (false, false) => HitClass::Organic,
                (false, true) => HitClass::Cited,
            };

            results.push(ScanResult {
                narrative: matcher.name.to_string(),
                class,
                matched_keywords: matched,
                attribution_cues_found: attr_cues,
            });
        }
    }

    if has_domain && !has_narrative {
        let matched_domains: Vec<String> = domain_seen
            .iter()
            .enumerate()
            .filter(|(_, hit)| **hit)
            .map(|(i, _)| signals::STATE_MEDIA_DOMAINS[i].to_string())
            .collect();

        results.push(ScanResult {
            narrative: "DOMAIN".to_string(),
            class: HitClass::DomainOnly,
            matched_keywords: matched_domains,
            attribution_cues_found: vec![],
        });
    }

    results
}
