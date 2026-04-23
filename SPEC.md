# Causal Decontamination

**Mechanistic analysis of state propaganda encoding in LLMs through contrastive retraining.**

## Research Question

How is foreign state propaganda encoded in LLM representations, why does safety training fail to correct it, and does removing propaganda from pretraining data causally eliminate these representations?

## Hypothesis

State propaganda creates persistent "framing directions" in transformer representation space that are geometrically distinct from factual knowledge and occupy network layers that RLHF cannot reach. Contrastive retraining (removing propaganda from the training corpus) eliminates these directions, while safety training only suppresses their expression in outputs.

## Why This Matters

- Anthropic/Turing/AISI (2025) proved 250 synthetic documents can backdoor any LLM (arXiv:2510.07192)
- DFRLab (2026) found 40,000 Pravda articles in Common Crawl
- Dobrovolskyi (2026) found 507,000 Russia-framing documents in C4 alone
- Roberts et al. (*Nature*, R&R) proved propaganda addition shifts outputs — but not where or how in the network
- Deep Ignorance (ICLR 2026) did contrastive retraining for bioweapons — but bioweapons are factual knowledge, propaganda is framing bias
- **Nobody has explained mechanistically why propaganda survives RLHF**

## Novel Contributions

1. **First mechanistic analysis of propaganda encoding in LLMs** — probing classifiers + causal tracing reveal where narrative framing lives in the residual stream, distinct from factual knowledge
2. **Mechanistic explanation for the RLHF gap** — framing bias is encoded in early/mid layers where safety training has no leverage; RLHF suppresses outputs but directions persist
3. **Taxonomy of propaganda encoding types** — conspiracy (false fact), framing (selective truth), and fabrication (invented claim) occupy different network components
4. **Contrastive retraining as causal proof** — removing propaganda eliminates framing directions entirely, confirming the training data → representation → behavior causal chain

## Research Arc

This paper is the third in a research program on foreign influence in AI systems:

```
kyivnotkiev (2024)        crimeaisukraine (2026)      this paper
toponym adoption          training data audit          causal mechanism

"Media naming reflects    "507K propaganda docs        "Here's WHERE it lives
 political influence"      in C4 shift LLM outputs,    in the network, WHY
                           RLHF doesn't fully fix"     RLHF can't fix it,
                                                        and PROOF of causality"
```

## Model & Dataset

### Pythia-1B (EleutherAI)

Purpose-built research model suite (ICML 2023). Chosen because:
- **154 intermediate checkpoints** publicly available — enables training dynamics analysis
- **Deterministic data order** — can reconstruct exactly which documents the model saw at any step
- **TransformerLens first-class support** — all sizes, all checkpoints, `revision="stepNNNN"`
- **Same architecture used by prior art** — Anthropic 250-doc study used GPT-NeoX/Pythia; Deep Ignorance used Pythia 6.9B architecture
- **Multi-seed variants** (PolyPythias) available for sizes 14M–410M

Architecture: 16 layers, 2048 hidden dim, 8 attention heads, ~1B parameters.

### The Pile (EleutherAI)

825 GiB English text corpus, 22 subsets. Pythia's native training data (~300B tokens).

Key subsets for propaganda scanning:
- **Pile-CC** (227 GiB) — Common Crawl filtered text. Primary location of RT, Sputnik, TASS articles
- **OpenWebText2** (63 GiB) — Reddit-linked web pages, may contain propaganda amplification
- **Wikipedia** (6.4 GiB) — editing wars on geopolitical topics
- Other subsets (PubMed, arXiv, GitHub, FreeLaw, etc.) — unlikely to contain propaganda, serve as controls

### Training Configuration

**Chinchilla-optimal**: ~20B tokens for 1B parameters.

| Model | Tokens | Data | Purpose |
|-------|--------|------|---------|
| **Baseline** | Published Pythia-1B at step ~9,500 | ~20B tokens of The Pile (with propaganda) | Free — no training required |
| **Decontaminated** | 20B tokens | The Pile minus identified propaganda | Train from scratch, 2 seeds |

Baseline uses published Pythia-1B intermediate checkpoint at the step corresponding to ~20B tokens seen. Same architecture, same data, same hyperparameters — only difference is propaganda presence. Clean causal comparison at zero cost for baseline.

## Narrative Categories

Three narratives, chosen to test distinct propaganda encoding types:

| ID | Narrative | Type | Source actors | Ground truth | Why this type matters |
|----|-----------|------|--------------|-------------|----------------------|
| N1 | 9/11 "inside job" / controlled demolition | **Conspiracy** (factually false) | RT, various | NIST reports, 9/11 Commission | Should be cleanly separable from factual knowledge — a "wrong fact" direction |
| N2 | NATO expansion as provocation / aggression | **Framing** (selectively true) | RT, TASS, MFA | NATO open-door policy, Budapest Memorandum | Hardest case — model must distinguish framing from facts. This is where the RLHF gap finding lives |
| N3 | US biolabs in Ukraine / biological weapons | **Fabrication** (invented claim with kernel of truth) | RT, MFA, Sputnik | DoD Cooperative Threat Reduction program, WHO inspections | Tests separation of real program (CTR) from fabricated claims (bioweapons) |

If the mechanistic analysis shows these three types are encoded differently in the network — that's a novel taxonomy. If they're encoded the same way — also informative.

## Architecture

```
causal-decontamination/
├── SPEC.md                          # This file
├── CLAUDE.md                        # Agent instructions
├── Makefile                         # Pipeline orchestration
│
├── pipelines/
│   ├── identify/                    # Phase 1: Find propaganda in The Pile
│   │   ├── classifier/              # Rust classifier (extend from crimeaisukraine)
│   │   │   ├── Cargo.toml
│   │   │   └── src/
│   │   │       ├── main.rs          # Shard-by-shard scanner
│   │   │       └── signals/         # Per-narrative signal modules
│   │   │           ├── n1_911.rs
│   │   │           ├── n2_nato.rs
│   │   │           └── n3_biolabs.rs
│   │   ├── validate.py              # LLM validator for precision (F1 target: >0.95)
│   │   └── data/
│   │       ├── pile_census.jsonl    # Identified propaganda docs with narrative labels
│   │       └── manifest.json
│   │
│   ├── retrain/                     # Phase 2: Contrastive retraining
│   │   ├── prepare_corpus.py        # Filter pre-tokenized Pile, output clean split
│   │   ├── train.py                 # Pythia-1B training (GPT-NeoX config)
│   │   ├── configs/
│   │   │   └── decontaminated.yaml  # Training config matching Pythia-1B hyperparams
│   │   └── data/
│   │       └── manifest.json
│   │
│   ├── benchmark/                   # Phase 3: Behavioral measurement
│   │   ├── narrative_benchmark.py   # Forced-choice + free-recall per narrative
│   │   ├── questions/               # Per-narrative question sets
│   │   │   ├── n1_911.json
│   │   │   ├── n2_nato.json
│   │   │   └── n3_biolabs.json
│   │   ├── compute_nas.py           # Narrative Alignment Score computation
│   │   └── data/
│   │       ├── baseline_scores.json
│   │       ├── decontaminated_scores.json
│   │       └── manifest.json
│   │
│   ├── interpret/                   # Phase 4: Mechanistic analysis (core contribution)
│   │   ├── probing.py               # Linear probes on every layer's residual stream
│   │   ├── causal_trace.py          # Activation patching per narrative per layer
│   │   ├── influence.py             # TracIn on published Pythia-1B checkpoints
│   │   ├── framing_vs_factual.py    # Compare framing directions vs factual knowledge
│   │   └── data/
│   │       └── manifest.json
│   │
│   └── grounding/                   # Phase 5: Search contamination (reuse from crimeaisukraine)
│       ├── scan.py
│       └── data/
│           └── manifest.json
│
├── data/                            # Shared data
│   ├── sanctions/                   # OFAC SDN, EU, UK OFSI CSVs
│   ├── narratives/                  # Ground-truth narrative definitions + signal words
│   └── results/                     # Final aggregated results
│
├── models/                          # Trained model checkpoints
│   └── pythia-1b-decontaminated/
│
└── paper/
    └── docs/
```

## Method

### Phase 1 — Identify (Rust classifier on The Pile)

Scan The Pile raw text (~825 GiB, 30 shards) for documents matching the 3 narrative categories:

- **Rust classifier** with per-narrative signal sets (extend crimeaisukraine's signal approach)
  - Domain matching: RT.com, sputniknews.com, tass.com, mid.ru, mfa.ru URLs in Pile-CC metadata
  - Keyword signals: narrative-specific terms and phrases (per-narrative signal modules)
  - Framing signals: co-occurrence patterns that distinguish propaganda framing from neutral reporting
- **LLM validator** for precision on borderline cases (F1 target: >0.95)
- **Cross-reference** against OFAC/EU/UK sanctions domain lists
- Output: document IDs to remove, with narrative labels and confidence scores

Processing: shard-by-shard on local machine (2TB SSD). Each shard ~12 GB compressed, ~27 GB uncompressed. Scan one at a time, accumulate results.

### Phase 2 — Retrain (contrastive pretraining)

Train Pythia-1B from scratch on decontaminated Pile:

- **Baseline**: published Pythia-1B checkpoint at step ~9,500 (~20B tokens seen). Free.
- **Decontaminated**: Pythia-1B architecture, identical hyperparameters, ~20B tokens of The Pile minus identified propaganda documents. Removed documents replaced with random non-propaganda documents to maintain token count (following Deep Ignorance methodology).
- **Seeds**: 2 independent training runs for the decontaminated version.
- **Infrastructure**: GPT-NeoX library, 1× A100 80GB rented instance.

### Phase 3 — Benchmark (Narrative Alignment Score)

Design a Narrative Alignment Score (NAS) analogous to SAS from crimeaisukraine:

- **Forced-choice questions** per narrative (e.g., "Was 9/11 an inside job?" → measure log-probability of yes vs no)
- **Free-recall questions** (e.g., "What happened on September 11, 2001?" → score response for propaganda framing markers)
- **General capability benchmarks** (HellaSwag, PIQA, LAMBADA) to confirm no degradation
- **Statistical tests**: paired comparisons per narrative, effect sizes (Cohen's d), bootstrap confidence intervals across seeds

### Phase 4 — Interpret (core contribution)

Mechanistic analysis on both baseline and decontaminated models using TransformerLens:

**4a. Probing classifiers (every layer)**
- Train linear probes on each layer's residual stream activations
- Binary classification: propaganda-framing vs. neutral-factual for each narrative
- Key question: at which layers is propaganda framing linearly separable? Does decontamination remove this separability?

**4b. Framing vs. factual directions**
- Extract "framing directions" (e.g., "NATO expansion is provocation" vs "NATO has open-door policy") using difference-in-means on residual stream activations
- Extract "factual directions" (e.g., "NATO exists" vs "NATO doesn't exist")
- Measure cosine similarity, compute principal angles between framing and factual subspaces
- Key prediction: framing directions should be geometrically distinct from factual knowledge and localized to different layers

**4c. Causal tracing (activation patching)**
- For each narrative: run prompts through both baseline and decontaminated models
- Patch activations at specific layers/heads from decontaminated model into baseline
- Identify which layers/heads, when patched, flip the model's framing without changing factual content
- Key finding: the layers that encode framing should be early/mid (where RLHF has limited leverage)

**4d. Cross-narrative comparison**
- Do conspiracy (N1), framing (N2), and fabrication (N3) occupy different network components?
- If yes → novel taxonomy of propaganda encoding types in transformers
- If no → propaganda encoding is uniform regardless of type (also informative)

**4e. Influence estimation (supplementary)**
- TracIn on published Pythia-1B intermediate checkpoints (154 available, no training needed)
- For specific propaganda-aligned model outputs, estimate which training documents contributed most
- Validates that identified propaganda documents are actually causally responsible

### Phase 5 — Grounding (reuse from crimeaisukraine)

Run grounding audit on US-targeted narratives as supporting evidence:
- Do commercial chatbots cite RT/Sputnik/TASS when answering questions about the 3 narratives?
- Cross-reference against sanctions lists
- This is not a main contribution — included for completeness and continuity with prior work

## Compute Budget

| Phase | Resource | Estimated cost |
|-------|----------|---------------|
| Phase 1 (identify) | Local Mac + 2TB SSD | ~$110 (SSD, one-time) |
| Phase 2 (retrain, 2 seeds) | 1× A100 80GB, ~8-10 days | ~$300-500 |
| Phase 3 (benchmark) | Same GPU instance | ~$10-20 |
| Phase 4 (interpret) | Same GPU instance + local | ~$20-50 |
| Phase 5 (grounding) | API calls | ~$50-100 |
| **Total compute** | | **$380-670** |

Baseline model is free (published Pythia-1B checkpoint).

## Prior Art (differentiation)

| Paper | What they did | What we add |
|-------|--------------|-------------|
| Deep Ignorance (ICLR 2026) | Contrastive retraining for bioweapons (6.9B, 550B tokens, DCLM) | Propaganda is framing, not factual knowledge — mechanistically different. We show where framing lives vs. facts |
| Roberts et al. (*Nature*, R&R) | Propaganda addition shifts outputs (continued pretraining) | We do subtraction (stronger causal claim) AND explain the mechanism |
| Anthropic 250-doc (2025) | 250 synthetic docs backdoor any LLM (600M–13B, The Pile) | Real propaganda, real narratives, mechanistic analysis of encoding |
| DFRLab Pravda (2026) | Counted 40K Pravda articles in Common Crawl | We trace the causal chain from training data → representations → behavior |
| Dobrovolskyi Crimea (2026) | Three-layer audit, found RLHF gap, 507K docs in C4 | We explain WHY the RLHF gap exists mechanistically + causal proof via retraining |

## Key Risk

The mechanistic prediction (framing in early/mid layers, factual in late layers) might be wrong. Framing and factual knowledge might not be cleanly separable. A null result ("propaganda is encoded identically to factual knowledge") is still publishable — it would mean Deep Ignorance's approach transfers directly to propaganda, which is useful to know.

## Target Venues

| Venue | Type | Why it fits |
|-------|------|------------|
| **TACL** | Q1 journal | Computational linguistics, natural home for the full arc |
| **FAccT** (ACM) | Q1 conference | Fairness/accountability in AI — propaganda in training data is core |
| **EMNLP** | Q1 conference | Strong interpretability track, flagship NLP |
| **EPJ Data Science** | Q1 journal | Interdisciplinary, data + society |
| **PNAS** | Q1 journal | Roberts et al. venue; our work directly extends theirs |
| Policy brief | — | FDD, CSIS, Atlantic Council — for policy impact |

## Author

**Ivan Dobrovolskyi** · [dobrovolsky94@gmail.com](mailto:dobrovolsky94@gmail.com)

Builds on methodology from [crimeaisukraine](https://crimeaisukraine.org) and [kyivnotkiev](https://github.com/IvanDobrovolsky/kyivnotkiev).
