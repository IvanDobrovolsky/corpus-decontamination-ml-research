# Findings

Running log of results, observations, and open items.

## Scan iterations

### v1 — Naive keyword matching

Shard `train/00.jsonl.zst` (5,899,215 docs):

| Category | Hits |
|---|---|
| DOMAIN (state media URLs) | 5,249 |
| N1 — 9/11 conspiracy | 221 |
| N3 — US biolabs | 33 |
| N2 — NATO provocation | 14 |

Problems: N2 false positives (golden rice matched "broken promise"), N3 almost
all PubMed biology papers matching "biolab".

### v2 — Aho-Corasick, tried ISIS

Replaced biolabs with "US created ISIS". Added Russia context filter for N2.
ISIS got **0 hits** — phrases too specific for natural text. N2 dropped to 2
(context filter working but very tight).

### v3 — Replaced ISIS with JFK

| Category | Hits | × 30 shards |
|---|---|---|
| DOMAIN | 5,226 | ~157,000 |
| N1 — 9/11 | 257 | ~7,700 |
| N3 — JFK | 46 | ~1,400 |
| N2 — NATO | 2 | ~60 |

JFK signal clean — "grassy knoll", "zapruder", "second shooter" are unambiguous.

### v4 — Attribution detection (current)

Added PARC 3.0 attribution cues to classify each narrative hit as PRIMARY
(state media asserting), ORGANIC (asserted without distancing), or CITED
(reported/discussed with attribution markers).

**Key result:**

| Class | Count | % of narrative hits | × 30 shards |
|---|---|---|---|
| DomainOnly | 5,211 | — | ~156,000 |
| **Cited** | **233** | **76%** | **~7,000** |
| **Organic** | **57** | **19%** | **~1,700** |
| **Primary** | **15** | **5%** | **~450** |

**76% of narrative-keyword hits are CITED — journalism discussing, debunking,
or reporting on propaganda — not the propaganda itself.** Only 19% are organic
(people asserting conspiracy claims) and 5% are primary state media sources.

This means the majority of "propaganda-adjacent" content in The Pile is
actually the counter-narrative. Removing it might DEGRADE the model's
resistance to propaganda rather than improve it.

### Spot-check quality

| Class | Quality | Example |
|---|---|---|
| Primary | Noisy — some Ubuntu IRC logs got classified due to long chats containing both domain URLs and keywords in different contexts | Needs refinement |
| Organic | Clean — conspiracy forums, Alex Jones content, people genuinely asserting claims | Good signal |
| Cited | Clean — articles with "according to", "conspiracy theory", "debunked" markers discussing 9/11 truth movement etc. | Good signal |

## Attribution detection methodology

Classification uses peer-reviewed frameworks, not ad hoc markers:

| Layer | Source | What it provides |
|---|---|---|
| Attribution cues | PARC 3.0 (Pareti, 2016, LREC) | 527 validated cue verbs from ~20K annotated relations in WSJ text |
| Factive/counter-factive | Thompson & Ye (1991, Applied Linguistics) | "Claimed" = distancing, "proved" = endorsing |
| Stance categories | Ferreira & Vlachos (2016, NAACL) | For/against/observing taxonomy — "observing" = reporting without endorsing |
| Hedging markers | BioScope corpus (Vincze et al., 2008), CoNLL-2010 | Validated hedging cues |

Gap we address: existing propaganda detection (Da San Martino et al., 2019;
SemEval-2020 Task 11) annotates technique presence at fragment level but does
NOT distinguish authorial assertion from reported speech. Our attribution-aware
classification addresses this.

## Narrative signal sourcing

Every keyword derived from official US/international sources:

| Source | What | Status |
|---|---|---|
| NIST NCSTAR 1 (2005) | WTC collapse — terms contradicting this are conspiracy | Available |
| 9/11 Commission Report (2004) | Official investigation | Available |
| Warren Commission Report (1964) | JFK — lone gunman conclusion | Available |
| HSCA Final Report (1979) | JFK — probable conspiracy but NOT CIA involvement | Available |
| GEC "Pillars of Russia's Disinformation" (Aug 2020) | Proxy outlets, narrative pillars | **TODO: download PDF** |
| EUvsDisinfo database | 16,000+ catalogued disinfo cases | **TODO: export relevant cases** |
| OFAC SDN List | Sanctioned media entities | Available |
| EU Council Regulation 2022/879 | RT/Sputnik ban | Available |
| US State Dept RT designation (2017, 2022) | State-controlled media | **TODO: find press release** |
| NATO Washington Treaty Art. 10 | Open door policy — ground truth for N2 | Available |
| Budapest Memorandum (1994) | Security assurances — ground truth for N2 | Available |

## Decomposition experiment

The v4 attribution results make the decomposition experiment much more
interesting and clearly worth pursuing in a single strong paper.

### Training conditions

| Condition | What's removed | Docs removed (est.) | What it tests |
|---|---|---|---|
| Baseline | Nothing | 0 | Published Pythia-1B checkpoint (free) |
| **-PRIMARY** | State media asserting propaganda | ~450 | Does removing the source fix it? |
| **-ORGANIC** | Organic amplification | ~1,700 | Does removing believers fix it? |
| **-CITED** | Journalism discussing/debunking | ~7,000 | Does removing counter-narrative HURT? |
| **-ALL** | Everything with narrative keywords | ~9,150 | Full narrative decontamination |
| **-DOMAIN** | All state media content (any topic) | ~156,000 | Source-level decontamination |

### Predicted findings and paper structure

1. **Removing PRIMARY+ORGANIC reduces propaganda bias** — expected, validates
   the causal chain from training data to model behavior.

2. **Removing CITED has no effect OR increases bias** — this would be the
   headline finding. It means journalism about disinformation doesn't
   contaminate models; it may actually inoculate them.

3. **Removing all DOMAIN content has the largest effect** — even state media
   articles not matching our 3 narratives carry framing bias on other topics.

4. **Mechanistic analysis** shows where each class is encoded — PRIMARY and
   ORGANIC create "framing directions" in representation space; CITED content
   may create counter-directions in the same subspace.

### Why this is one strong Q1 paper, not two

The decomposition IS the mechanistic analysis. You're not just showing "remove
propaganda → model changes" (which Deep Ignorance already did). You're showing:

- **What type of content actually causes bias** (source vs. organic vs. cited)
- **Where each type is encoded** (probing classifiers per class per layer)
- **Why journalism doesn't contaminate** (or does — either way it's a finding)
- **The RLHF gap explanation** follows naturally from the layer analysis

Title direction: "Not All Propaganda Is Equal: Decomposing the Causal Effect
of State Media, Organic Amplification, and Counter-Narrative in LLM Training Data"

Target: **EMNLP 2026** or **TACL** (Q1).

## TODO

### Before full scan (need SSD)
- [ ] Download remaining 29 Pile shards
- [ ] Run scanner on all 30, aggregate results
- [ ] Validate projected numbers hold across shards

### Before training
- [ ] Manual annotation: ~100 hits per class (PRIMARY/ORGANIC/CITED), report precision
- [ ] Decide on number of training conditions vs budget
- [ ] Fix PRIMARY classification noise (IRC logs issue)

### Before paper
- [ ] Download GEC Pillars PDF, add as supplementary
- [ ] Export EUvsDisinfo cases for 3 narratives
- [ ] Find US State Dept RT designation press release
- [ ] All source docs in `data/sources/`
- [ ] Every keyword traced to specific source document page
