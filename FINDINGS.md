# Findings

Running log of results, observations, and open items as we go.

## Scan v1 — Shard 00 (naive keyword matching)

First pass on `train/00.jsonl.zst` (5,899,215 docs):

| Category | Hits |
|---|---|
| DOMAIN (state media URLs) | 5,249 |
| N1 — 9/11 conspiracy | 221 |
| N3 — US biolabs | 33 |
| N2 — NATO provocation | 14 |

- N1 signal is clean — hits are genuine conspiracy content
- N2 too broad — "broken promise" matched unrelated docs (golden rice article)
- N3 almost all false positives — "biolab" matches PubMed biology papers

## Scan v2 — Shard 00 (Aho-Corasick, replaced biolabs with ISIS)

| Category | Hits |
|---|---|
| DOMAIN | 5,226 |
| N1 — 9/11 | 257 |
| N2 — NATO (with Russia context filter) | 2 |
| N3 — ISIS | 0 |

- ISIS keywords too specific — exact phrases don't appear in natural text
- N2 context filter works but signal is very thin

## Scan v3 — Shard 00 (replaced ISIS with JFK)

| Category | Hits | × 30 shards (projected) |
|---|---|---|
| DOMAIN | 5,226 | ~157,000 |
| N1 — 9/11 | 257 | ~7,700 |
| N3 — JFK | 46 | ~1,400 |
| N2 — NATO | 2 | ~60 |

- JFK signal is clean — "grassy knoll", "zapruder", "second shooter" are unambiguous
- N2 thin but defensible — NATO provocation framing is rare in The Pile
- DOMAIN hits are the bulk — actual state media content regardless of narrative
- Note: a doc can be both DOMAIN and NARRATIVE (RT article about 9/11 conspiracy)

## Signal sourcing

All keyword lists derived from official US/international sources. **NOT invented.**

### Sources we cite

| Source | What | Status |
|---|---|---|
| NIST NCSTAR 1 (2005) | WTC collapse findings — defines what contradicts official record | Published, freely available |
| 9/11 Commission Report (2004) | Official investigation | Published, freely available |
| Warren Commission Report (1964) | JFK assassination findings — lone gunman conclusion | Published, National Archives |
| HSCA Final Report (1979) | JFK follow-up — acknowledged probable conspiracy but NOT CIA | Published |
| GEC "Pillars of Russia's Disinformation" (Aug 2020) | Names proxy outlets, identifies narrative pillars | **TODO: download PDF, include as supplementary** |
| EUvsDisinfo database | 16,000+ catalogued disinfo cases with narratives | **TODO: export relevant cases for 3 narratives** |
| OFAC SDN List | Sanctioned entities including media | Published, downloadable CSV |
| EU Council Regulation 2022/879 | RT/Sputnik broadcasting ban | Published in Official Journal of the EU |
| US State Dept RT designation (2017, 2022) | RT as state-controlled media | **TODO: find exact press release / Federal Register entry** |
| NATO Washington Treaty Art. 10 | Open door policy — ground truth for N2 | Published treaty text |
| Budapest Memorandum (1994) | Security assurances — ground truth for N2 | Published |

### TODO before paper submission

- [ ] Download GEC "Pillars" PDF (Aug 2020) — include as supplementary material
- [ ] Export EUvsDisinfo cases for "9/11", "NATO aggression", "JFK" — include counts
- [ ] Find US State Dept press release designating RT as state-controlled
- [ ] Add all source documents to `data/sources/` directory
- [ ] Verify every keyword in signals.rs traces back to a specific source document page number
- [ ] Manual annotation: sample ~100 hits per narrative, label TP/FP, report precision

## Decomposition experiment idea

Instead of one "decontaminated" model, train multiple filtering conditions:

| Condition | What's removed | What it tests |
|---|---|---|
| Baseline | Nothing | Published Pythia-1B checkpoint |
| Filter A | DOMAIN only (RT/Sputnik content) | Does removing the source fix it? |
| Filter B | NARRATIVE only (conspiracy keywords from any source) | Does removing the framing fix it? |
| Filter C | Both | Full decontamination |

This decomposes whether the bias comes from **the propaganda itself** or from
**the ecosystem of legitimate journalism that references it** (NYT quoting RT,
academic papers citing state media, Reddit threads debunking conspiracies).

If Filter A fixes bias but B doesn't → source matters more than content.
If Filter B fixes but A doesn't → narrative framing matters regardless of source.
If both needed → it's the combination.

Cost: 3 training runs instead of 1 (~$450-750 vs ~$150-250).

Potential second paper: "Does responsible journalism about disinformation
inadvertently train LLMs to reproduce it?"

## Scanner metadata needed for decomposition

Each hit should be tagged as:
- **DOMAIN_ONLY** — state media URL but no narrative keywords
- **NARRATIVE_ONLY** — narrative keywords but not from state media domain
- **DOMAIN+NARRATIVE** — both (strongest signal)

This tagging is already implicit in the current output (separate DOMAIN and
narrative hits) but should be made explicit with cross-referencing.

## Open questions

- Is ~7,700 N1 docs enough to produce a detectable effect after retraining?
  (Anthropic showed 250 synthetic docs suffice, but those were optimized for impact.
  Real propaganda is diluted — unclear if same threshold applies.)
- N2 at ~60 docs projected — is this too thin? Keep it as a "low-signal" test case?
- Do we need the full Pile (with copyrighted subsets) or is uncopyrighted sufficient?
  (Pythia was trained on full Pile. Uncopyrighted excludes OpenWebText2 which may
  contain propaganda amplification via Reddit links.)
- How many training runs can we actually afford? 3 conditions × 2 seeds = 6 runs.
