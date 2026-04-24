/// Narrative signal definitions for propaganda detection in The Pile.
///
/// Each narrative has:
/// - Domain signals: known state media domains
/// - Keyword signals: narrative-specific terms and phrases
/// - Framing signals: co-occurrence patterns that indicate propaganda framing

/// Known state media domains (RT, Sputnik, TASS, etc.)
pub const STATE_MEDIA_DOMAINS: &[&str] = &[
    "rt.com",
    "russian.rt.com",
    "sputniknews.com",
    "sputnikglobe.com",
    "tass.com",
    "tass.ru",
    "mid.ru",
    "mfa.gov.ru",
    "eng.mil.ru",
    "strategic-culture.org",
    "journal-neo.org",
    "globalresearch.ca",
    "southfront.org",
    "presstv.ir",
    "news-front.info",
    "katehon.com",
    "geopolitica.ru",
    "orientalreview.org",
    "veteranstoday.com",
    "mintpressnews.com",
];

/// N1: 9/11 "inside job" conspiracy signals
pub const N1_KEYWORDS: &[&str] = &[
    "inside job",
    "controlled demolition",
    "building 7",
    "wtc 7",
    "jet fuel can't melt",
    "jet fuel cannot melt",
    "steel beams",
    "free fall speed",
    "free-fall speed",
    "9/11 truth",
    "false flag",
    "thermite",
    "nano-thermite",
    "pull it",
    "tower 7",
];

/// N2: NATO expansion as provocation signals
pub const N2_KEYWORDS: &[&str] = &[
    "nato expansion",
    "nato enlargement",
    "nato provocation",
    "nato encirclement",
    "nato aggression",
    "broken promise",
    "not one inch eastward",
    "legitimate security concerns",
    "nato threat",
    "western aggression",
    "encirclement of russia",
    "defensive reaction",
    "nato pushed",
    "forced russia",
];

/// N3: US biolabs in Ukraine signals
pub const N3_KEYWORDS: &[&str] = &[
    "us biolabs",
    "biological laboratories",
    "pentagon biolabs",
    "military biological",
    "biological weapons ukraine",
    "bioweapons ukraine",
    "biological research facilities",
    "lugar center",
    "bio-labs",
    "biolab",
    "secret laboratories",
    "biological threat",
    "dtra ukraine",
];
