//! Narrative signal definitions for propaganda detection in The Pile.
//!
//! SOURCING POLICY: Every keyword list must cite its derivation source.
//! We use only official US/international sources and peer-reviewed research:
//!
//! - NIST NCSTAR 1 (2005) — defines the factual consensus on WTC collapse,
//!   any claims contradicting it are conspiracy framing
//! - 9/11 Commission Report (2004) — official US government investigation
//! - NATO official communiqués and founding treaty text
//! - US State Department GEC reports on Russian disinformation pillars
//! - EUvsDisinfo database (EU East StratCom Task Force) — catalogued
//!   disinformation cases with identified narratives
//! - OFAC SDN List / EU Council Regulation 2022/879 — sanctioned entities
//! - US State Department designations of state media outlets
//!
//! Keywords are NOT invented — they are terms that either:
//! (a) contradict official findings (conspiracy), or
//! (b) appear in documented disinformation campaigns per GEC/EUvsDisinfo
//!
//! ATTRIBUTION DETECTION uses markers from:
//! - PARC 3.0 (Pareti, 2016, LREC) — 527 validated attribution cue verbs
//!   from ~20K annotated relations in Wall Street Journal text
//! - Thompson & Ye (1991, Applied Linguistics) — factive/non-factive/
//!   counter-factive reporting verb taxonomy
//! - BioScope corpus (Vincze et al., 2008) — hedging markers,
//!   validated in CoNLL-2010 shared task
//! - Stance categories follow Ferreira & Vlachos (2016, NAACL):
//!   for/against/observing (where "observing" = reporting without endorsing)

/// State media domains.
///
/// Sources:
/// - US State Department: RT designated as Russian state-controlled media (2017, updated 2022)
/// - OFAC SDN List: sanctions on Russian media entities
/// - EU Council: RT and Sputnik broadcasting banned in EU (Regulation 2022/879)
/// - GEC "Pillars of Russia's Disinformation" (2020): identifies key outlets
pub const STATE_MEDIA_DOMAINS: &[&str] = &[
    // Designated by US State Dept / sanctioned by EU
    "rt.com",
    "russian.rt.com",
    "sputniknews.com",
    "sputnikglobe.com",
    "tass.com",
    "tass.ru",
    // Russian government official
    "mid.ru",       // Ministry of Foreign Affairs
    "mfa.gov.ru",   // MFA English
    "eng.mil.ru",   // Ministry of Defence English
    // GEC-identified proxy outlets
    "strategic-culture.org",    // GEC Pillars report (Aug 2020), p.29
    "journal-neo.org",          // GEC Pillars report (Aug 2020), p.31
    "globalresearch.ca",        // GEC Pillars report (Aug 2020), p.34
    "southfront.org",           // GEC Pillars report (Aug 2020), p.32
    "news-front.info",          // EU-sanctioned (2022)
    "geopolitica.ru",           // GEC-identified
    "orientalreview.org",       // GEC Pillars report (Aug 2020), p.33
];

/// N1: 9/11 "inside job" conspiracy.
///
/// Source derivation: terms that contradict NIST NCSTAR 1 (2005) and
/// the 9/11 Commission Report (2004). NIST concluded fire-induced
/// progressive collapse; any claim of "controlled demolition" or
/// "thermite" directly contradicts the official finding.
///
/// EUvsDisinfo case IDs: multiple RT articles catalogued pushing
/// 9/11 conspiracy narratives (search "9/11" in euvsdisinfo.eu).
///
/// Minimum 2 keyword co-occurrence required to filter out casual mentions.
pub const N1_KEYWORDS: &[&str] = &[
    // Contradict NIST NCSTAR 1 finding of fire-induced collapse
    "controlled demolition",
    "nano-thermite",
    "thermite",
    "free fall speed",
    "free-fall speed",
    // Contradict 9/11 Commission Report attribution
    "inside job",
    "9/11 truth",
    "9/11 was an",
    // WTC 7 conspiracy (NIST NCSTAR 1A addressed this specifically)
    "building 7",
    "wtc 7",
    "tower 7",
    // General conspiracy framing co-occurring with above
    "false flag",
    "pull it",
];

/// N2: NATO expansion as provocation / aggression.
///
/// Source derivation: GEC "Pillars of Russia's Disinformation and
/// Propaganda Ecosystem" (Aug 2020) identifies "NATO as aggressor"
/// as a core Russian narrative pillar. EUvsDisinfo catalogues 1,400+
/// cases of this narrative.
///
/// Ground truth: NATO Washington Treaty Art. 10 (open door policy),
/// Budapest Memorandum (1994), NATO-Russia Founding Act (1997).
///
/// Key distinction: "NATO expansion" alone is neutral terminology.
/// It becomes propaganda framing when paired with terms implying
/// aggression, broken promises, or justifying Russian response.
/// Minimum 2 co-occurring keywords required, AND document must
/// contain a context anchor ("russia" or "moscow" or "kremlin").
pub const N2_KEYWORDS: &[&str] = &[
    // GEC-identified framing: NATO as aggressor
    "nato aggression",
    "nato provocation",
    "nato encirclement",
    "nato threat to russia",
    "western aggression",
    "encirclement of russia",
    // "Broken promise" narrative (GEC Pillar 2)
    "not one inch eastward",
    "nato promised",
    "broken promise",
    // Justification framing
    "legitimate security concerns",
    "forced russia",
    "nato pushed russia",
    "defensive reaction",
    "provoked russia",
];

/// N2 context anchors — document must contain at least one of these
/// to qualify as NATO-provocation narrative (prevents false positives
/// on unrelated "broken promise" or "aggression" mentions).
pub const N2_CONTEXT: &[&str] = &[
    "russia",
    "moscow",
    "kremlin",
    "putin",
    "russian",
];

/// N3: JFK assassination conspiracy.
///
/// Source derivation: terms that contradict the Warren Commission
/// Report (1964), which concluded Lee Harvey Oswald acted alone.
/// The HSCA (1979) acknowledged a probable conspiracy but did NOT
/// endorse CIA involvement or "grassy knoll" shooter theories.
///
/// RT actively promotes JFK conspiracies as part of the GEC-identified
/// "eroding trust in US institutions" pillar. EUvsDisinfo catalogues
/// RT articles pushing CIA involvement narratives.
///
/// Ground truth: Warren Commission Report (1964), HSCA Final Report
/// (1979), National Archives JFK Records Collection.
///
/// Minimum 2 keyword co-occurrence required.
pub const N3_KEYWORDS: &[&str] = &[
    // Contradict Warren Commission finding of lone gunman
    "grassy knoll",
    "second shooter",
    "second gunman",
    "magic bullet",
    "single bullet theory",
    // CIA involvement claims
    "cia killed kennedy",
    "cia killed jfk",
    "cia assassination",
    "cia involved in jfk",
    // General conspiracy framing
    "jfk coverup",
    "jfk cover-up",
    "kennedy assassination conspiracy",
    "jfk conspiracy",
    "jfk truth",
    "kennedy was killed by",
    // Specific conspiracy theories
    "umbrella man",
    "dealey plaza conspiracy",
    "zapruder",
];

/// Attribution cues — signals that content is REPORTED, not ASSERTED.
///
/// Source: PARC 3.0 (Pareti, 2016, LREC). Top cues by frequency from
/// ~20,000 annotated attribution relations in WSJ news text. Verbal
/// cues account for 92% of all attribution signals in the corpus.
///
/// Supplemented with counter-factive markers from Thompson & Ye (1991)
/// factive/non-factive taxonomy, which signal the author DISTANCES
/// from the reported claim.
pub const ATTRIBUTION_CUES: &[&str] = &[
    // PARC 3.0 top verbal cues (by corpus frequency)
    "said that",
    "claimed that",
    "claims that",
    "alleged that",
    "alleges that",
    "reported that",
    "stated that",
    "argued that",
    "suggested that",
    "insisted that",
    "warned that",
    "denied that",
    "accused",
    "asserted that",
    "announced that",
    "maintained that",
    "contended that",
    // PARC 3.0 prepositional cues
    "according to",
    // PARC 3.0 adverbial cues
    "reportedly",
    "allegedly",
    "admittedly",
    // Thompson & Ye (1991) counter-factive markers —
    // signal author REJECTS the reported claim
    "debunked",
    "discredited",
    "disproven",
    "baseless",
    "unfounded",
    "false claim",
    "conspiracy theory",
    "conspiracy theories",
    "disinformation",
    "misinformation",
    "fact check",
    "fact-check",
    "has promoted",
    "has pushed",
    "widely rejected",
];
