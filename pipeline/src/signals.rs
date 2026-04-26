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
/// - US State Dept Office of Foreign Missions: Chinese media designated as
///   foreign missions (Feb 2020, Jun 2020, Oct 2020)
/// - GEC "How the PRC Seeks to Reshape the Global Information Environment" (Sep 2023)
pub const STATE_MEDIA_DOMAINS: &[&str] = &[
    // ── Russian state media ──
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
    // ── Chinese state media ──
    // Designated as foreign missions by US State Dept (Feb–Oct 2020)
    "cgtn.com",                 // China Global Television Network (FARA-registered 2019)
    "xinhuanet.com",            // Xinhua News Agency (FARA-registered 2021)
    "xinhua.net",               // Xinhua alternate domain
    "globaltimes.cn",           // Global Times (Jun 2020 designation)
    "chinadaily.com.cn",        // China Daily (Feb 2020 designation)
    "en.people.cn",             // People's Daily English (Jun 2020 designation)
    "ecns.cn",                  // China News Service (Jun 2020 designation)
    "china.org.cn",             // China Internet Information Center (State Council)
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
    "free fall speed",
    "free-fall speed",
    // Contradict 9/11 Commission Report attribution
    "inside job",
    "9/11 truth",
    // WTC 7 conspiracy (NIST NCSTAR 1A addressed this specifically)
    "building 7",
    "wtc 7",
    "tower 7",
    // Removed: "thermite" (matches chemistry/propellant contexts)
    // Removed: "pull it" (common English phrase, massive FP rate)
    // Removed: "false flag" (too generic without 9/11 context)
];

/// N1 context anchors — document must mention 9/11 or WTC to qualify.
/// Prevents "inside job" matching corporate fraud, "controlled demolition"
/// matching actual demolition industry content, etc.
pub const N1_CONTEXT: &[&str] = &[
    "9/11",
    "september 11",
    "world trade center",
    "twin towers",
    "wtc",
    "pentagon attack",
    "flight 93",
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

/// N3 context anchors — document must mention JFK/Kennedy/Dallas to qualify.
/// Prevents "magic bullet" matching business/medical metaphors,
/// "second shooter" matching crime reports, etc.
pub const N3_CONTEXT: &[&str] = &[
    "kennedy",
    "jfk",
    "dealey plaza",
    "oswald",
    "dallas 1963",
    "assassination",
];

/// N4: US biolabs conspiracy.
///
/// Source derivation: GEC "Pillars of Russia's Disinformation" (Aug 2020),
/// GEC Special Report "The Kremlin's Efforts to Spread Chemical and Biological
/// Weapons Disinformation" (May 2022), Lentzos (2021, Nonproliferation Review)
/// "False allegations of biological weapons use from Putin's Russia."
///
/// Ground truth: US Cooperative Threat Reduction Program (Nunn-Lugar Act 1991)
/// funds biosafety and public health labs, not weapons development. The US
/// ratified the Biological Weapons Convention (BWC) in 1975.
///
/// Minimum 2 keyword co-occurrence required.
pub const N4_KEYWORDS: &[&str] = &[
    "pentagon biolabs",
    "pentagon bioweapons",
    "us bioweapons",
    "american bioweapons",
    "secret biolabs",
    "military biolabs",
    "bioweapons laboratory",
    "biological weapons lab",
    "us biological weapons program",
];

pub const N4_CONTEXT: &[&str] = &[
    "ukraine",
    "georgia",
    "kazakhstan",
    "lugar",
    "pentagon",
    "tbilisi",
];

/// N5: Syria chemical attacks denial / White Helmets conspiracy.
///
/// Source derivation: OPCW Fact-Finding Mission reports (2014–2019),
/// OPCW Investigation and Identification Team reports (2020, 2021, 2023).
/// IIT Third Report (Jan 2023) concluded Syrian Arab Air Forces dropped
/// chlorine cylinders on Douma on 7 April 2018, killing 43 people.
///
/// GEC "Pillars" report documents Russian disinformation on Syria.
/// EUvsDisinfo catalogues extensive White Helmets conspiracy cases.
///
/// Ground truth: OPCW confirmed Syrian government used chemical weapons
/// at Khan Shaykhun (2017), Douma (2018), and Ltamenah (2017).
///
/// Minimum 2 keyword co-occurrence required.
pub const N5_KEYWORDS: &[&str] = &[
    // The Syria Campaign report "Killing the Truth" (Dec 2017)
    // + Guardian investigation (Dec 2017) + EUvsDisinfo cases
    "white helmets staged",
    "white helmets fake",
    "white helmets terrorists",
    "white helmets propaganda",
    "white helmets al qaeda",      // Syria Campaign: "Al-Qaeda affiliates" claim
    "white helmets al-qaeda",
    "white helmets crisis actors",  // Syria Campaign: documented accusation
    "white helmets jihadist",       // Syria Campaign: documented accusation
    // Chemical attack denial (OPCW IIT reports as ground truth)
    "chemical attack staged",
    "chemical attack false flag",
    "chemical attack hoax",
    "douma staged",
    "douma false flag",
    "douma hoax",
    "ghouta false flag",
    "ghouta staged",
    "assad framed",
    // OPCW conspiracy
    "opcw coverup",
    "opcw cover-up",
];

pub const N5_CONTEXT: &[&str] = &[
    "syria",
    "syrian",
    "assad",
    "douma",
    "ghouta",
    "damascus",
    "chemical",
    "white helmets",
    "idlib",
];

/// N6: Moon landing hoax.
///
/// Source derivation: EUvsDisinfo database (tag: "moon-landing"), documenting
/// RT and Sputnik promotion of Apollo conspiracy theories around the 50th
/// anniversary (Jul 2019). Russian Senator Pushkov (Chair, Information Policy
/// Committee) promoted hoax since Nov 2017.
///
/// VOA Fact Check (May 2023) documented former Roscosmos head Rogozin
/// repeating moon landing conspiracy.
///
/// Ground truth: NASA Apollo missions (1969–1972), independently verified
/// by Lunar Reconnaissance Orbiter photographs of landing sites (2009–),
/// retroreflectors confirmed by observatories worldwide, 382 kg of lunar
/// samples verified by international laboratories.
///
/// Minimum 2 keyword co-occurrence required.
pub const N6_KEYWORDS: &[&str] = &[
    "moon landing hoax",
    "moon landing fake",
    "moon landing staged",
    "never landed on the moon",
    "never went to the moon",
    "apollo hoax",
    "faked the moon landing",
    "faked moon landing",
    "lunar landing hoax",
    "moon landing conspiracy",
    "moon hoax",
];

pub const N6_CONTEXT: &[&str] = &[
    "apollo",
    "nasa",
    "moon",
    "lunar",
    "astronaut",
];

/// N7: Anti-vaccination misinformation.
///
/// Source derivation: Broniatowski et al. (2018, AJPH vol.108 no.10)
/// "Weaponized Health Communication: Twitter Bots and Russian Trolls
/// Amplify the Vaccine Debate" — documents Russian state-sponsored
/// accounts amplifying anti-vax content on social media.
///
/// Ground truth:
/// - Lancet retraction of Wakefield (2010): original MMR-autism paper
///   retracted for fraud (Deer, 2011, BMJ)
/// - Taylor et al. (2014, Vaccine) meta-analysis: no link between
///   vaccines and autism across 1.27M children
/// - DeStefano et al. (2013, J Pediatr): no MMR-autism association
/// - WHO position papers on vaccine safety
///
/// Minimum 2 keyword co-occurrence required.
pub const N7_KEYWORDS: &[&str] = &[
    // Broniatowski et al. (2018, AJPH): exact IRA troll phrases
    "vaccines cause autism",       // documented troll tweet
    "vaccine autism link",         // core debunked claim (Taylor 2014 meta-analysis)
    "mmr autism",                  // Lancet-retracted Wakefield claim (2010)
    "thimerosal autism",           // from troll tweet "#vaccines contain mercury!"
    // Broniatowski 2018: documented IRA themes
    "vaccine injury coverup",     // "secret government database of vaccine-damaged children"
    "vaccine injury cover-up",
    "big pharma coverup",         // pharmaceutical profit conspiracy (documented IRA theme)
    "big pharma cover-up",
    // Frontiers in Microbiology (Vaccine Safety: Myths and Misinformation, 2020)
    "vaccines are not safe",       // core anti-vax claim
    "vaccines are unsafe",
    "too many vaccines",           // "too many vaccines too soon" documented myth
    "mercury in vaccines",         // documented ingredient fear (also Broniatowski troll tweet)
    "vaccine dangers",             // documented anti-vax framing
    "natural immunity is better",  // documented myth vs vaccination
];

pub const N7_CONTEXT: &[&str] = &[
    "vaccine",
    "vaccination",
    "immunization",
    "autism",
    "mmr",
    "thimerosal",
];

/// N8: Soros global conspiracy.
///
/// Source derivation: GEC "Pillars of Russia's Disinformation" (2020)
/// identifies "global conspiracy / shadow government" as a core Russian
/// disinformation pillar. EUvsDisinfo database catalogues 50+ cases of
/// Soros-related disinformation from Russian state media.
///
/// ADL (Anti-Defamation League) documents Soros conspiracy theories as
/// recycled antisemitic tropes about Jewish financial control.
///
/// Ground truth: Open Society Foundations is a legitimate philanthropic
/// network. Soros conspiracy narratives are not supported by evidence.
///
/// Minimum 2 keyword co-occurrence required.
pub const N8_KEYWORDS: &[&str] = &[
    // EUvsDisinfo case titles (220+ documented cases)
    "soros puppet",               // "Soros-sponsored puppets", "puppet of George Soros"
    "soros controls",             // "Soros fully controls Ukraine"
    "soros agenda",               // "Advancing Soros' Agenda to Undermine Nation States"
    "soros colour revolution",    // "Soros sponsors colour revolutions"
    "soros color revolution",     // US spelling variant
    "soros destabili",            // "Soros destabilises states", "Soros seeks to destabilise"
    "soros globalist",            // "globalist élite's ideology is based on Soros's vision"
    "soros open borders",         // "Soros openly admits his plan to destroy national borders"
    "soros conspiracy",           // "global Zionist conspiracy"
    "soros plunges",              // "Soros plunges countries into chaos and anarchy"
    "soros funded revolution",    // "Soros Foundation took an active part in...revolutions"
    // ADL (Anti-Defamation League): documented conspiracy claims
    "soros pays protesters",      // ADL: "funding protests" recurring claim
    "soros hires",                // ADL: variant of protest-funding claim
    "soros behind",               // ADL: "behind the scenes" manipulation framing
    "soros funds protests",       // ADL: documented recurring claim
    "soros undermining",          // ADL: "undermining societies"
    "soros manipulat",            // ADL: "manipulates national events"
];

/// N8 context — Soros keywords already contain "soros", but context
/// anchors ensure we only match in documents actually about Soros.
pub const N8_CONTEXT: &[&str] = &[
    "george soros",
    "open society",
    "soros",
];

/// N9: Uyghur / Xinjiang genocide denial.
///
/// Source derivation: GEC "How the PRC Seeks to Reshape the Global
/// Information Environment" (Sep 2023). UN OHCHR Assessment of Human
/// Rights Concerns in Xinjiang (Aug 2022) found "serious human rights
/// violations." Freedom House "Beijing's Global Megaphone" (2020, 2022).
///
/// Chinese state media uses euphemisms ("vocational training centers")
/// and deflection ("anti-China forces") to deny documented abuses.
///
/// Ground truth: UN OHCHR (2022), ASPI satellite analysis of camp
/// construction, leaked Xinjiang Police Files (2022), testimony from
/// former detainees before multiple national parliaments.
///
/// Minimum 2 keyword co-occurrence required.
pub const N9_KEYWORDS: &[&str] = &[
    // PRC State Council white paper (Aug 2019) exact euphemisms
    "vocational education and training center",  // official CCP term for camps
    "vocational training center",                // shortened form
    // GEC China Report (Sep 2023) documented denial terminology
    "anti-china forces",          // standard CCP dismissal of criticism
    "lies about xinjiang",        // CCP rebuttal language
    // CCP propaganda terminology (State Dept documented)
    "splittist",                  // CCP term for Uyghur/Tibet/Taiwan movements
    "separatist forces",          // CCP framing of Uyghur advocacy
    // PRC counter-narrative framing
    "xinjiang fabricat",          // "fabricated", "fabrication" — CCP denial language
];

pub const N9_CONTEXT: &[&str] = &[
    "xinjiang",
    "uyghur",
    "uighur",
    "east turkestan",
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
