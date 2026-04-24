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
- Replaced N3 with "US created ISIS" narrative
- Added N2 context filtering (must mention Russia/Moscow/Kremlin)

## Scan v2 — Shard 00 (Aho-Corasick + sourced signals)

Pending — running now.

## Signal sourcing

All keyword lists derived from official US/international sources. **NOT invented.**

### Sources we cite

| Source | What | Status |
|---|---|---|
| NIST NCSTAR 1 (2005) | WTC collapse findings — defines what contradicts official record | Published, freely available |
| 9/11 Commission Report (2004) | Official investigation — same | Published, freely available |
| GEC "Pillars of Russia's Disinformation" (Aug 2020) | Names proxy outlets, identifies narrative pillars | **TODO: download PDF, include as supplementary** |
| EUvsDisinfo database | 16,000+ catalogued disinfo cases with narratives | **TODO: export relevant cases for 3 narratives** |
| OFAC SDN List | Sanctioned entities including media | Published, downloadable CSV |
| EU Council Regulation 2022/879 | RT/Sputnik broadcasting ban | Published in Official Journal of the EU |
| US State Dept RT designation (2017, 2022) | RT as state-controlled media | **TODO: find exact press release / Federal Register entry** |
| NATO Washington Treaty Art. 10 | Open door policy — ground truth for N2 | Published treaty text |
| Budapest Memorandum (1994) | Security assurances — ground truth for N2 | Published |
| US DNI assessments on ISIS origins | Ground truth for N3 | **TODO: find specific declassified assessment** |

### TODO before paper submission

- [ ] Download GEC "Pillars" PDF (Aug 2020) — include as supplementary material
- [ ] Export EUvsDisinfo cases for "9/11", "NATO aggression", "ISIS" — include counts
- [ ] Find US State Dept press release designating RT as state-controlled
- [ ] Find specific DNI/DoD assessment on ISIS origins (declassified)
- [ ] Add all source documents to `data/sources/` directory
- [ ] Verify every keyword in signals.rs traces back to a specific source document page number

## Extrapolation from shard 00

If signal rates hold across all 30 shards:

| Category | Per shard | × 30 shards |
|---|---|---|
| DOMAIN | ~5,200 | ~156,000 |
| N1 (9/11) | ~220 | ~6,600 |
| N2 (NATO) | TBD after v2 | TBD |
| N3 (ISIS) | TBD after v2 | TBD |

This is 1/30 of the data. Numbers will be refined as more shards are scanned.

## Open questions

- Is ~6,600 N1 docs enough to produce a detectable effect after retraining?
  (Anthropic showed 250 synthetic docs suffice, but those were optimized for impact.
  Real propaganda is diluted — unclear if same threshold applies.)
- Should we also count docs that match BOTH domain + narrative keyword?
  (e.g., RT article containing "controlled demolition" — strongest signal)
- Do we need the full Pile (with copyrighted subsets) or is uncopyrighted sufficient?
  (Pythia was trained on full Pile. Uncopyrighted excludes OpenWebText2 which may
  contain propaganda amplification via Reddit links.)
