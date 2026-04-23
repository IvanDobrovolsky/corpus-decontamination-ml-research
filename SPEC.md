# Causal Decontamination

**Measuring the causal effect of state propaganda in LLM training data through contrastive retraining.**

## Research Question

Does removing identified foreign state propaganda from an LLM's training corpus causally improve the model's alignment with factual consensus on US-targeted disinformation narratives?

## Hypothesis

If N state-propaganda documents are removed from a pretraining corpus and the model is retrained, measurable behavioral changes will occur on narrative-specific benchmarks — proving a causal link between training data contamination and model output bias that RLHF alone cannot fully correct.

## Why This Matters

- Anthropic/Turing/AISI (2025) proved 250 synthetic documents can backdoor any LLM
- DFRLab (2026) found 40,000 Pravda articles in Common Crawl
- Dobrovolskyi (2026) found 507,000 Russia-framing documents in C4 alone
- **Nobody has done the subtraction experiment**: remove real propaganda, retrain, measure the delta
- Roberts et al. (*Nature*, conditionally accepted) proved propaganda addition shifts outputs — we prove removal corrects them

## Novel Contributions

1. **First contrastive retraining experiment for real-world state propaganda** (Deep Ignorance did bioweapons; we do propaganda)
2. **First sanctions-aware training data audit** — applying OFAC/EU/UK compliance logic to AI pipelines
3. **Mechanistic analysis** — probing classifiers + causal tracing to locate where propaganda narratives are encoded in transformer layers
4. **Cross-lingual transfer measurement** — does removing English propaganda reduce bias in 50 other languages?

## Narrative Categories (US-Targeted)

| ID | Narrative | Source actors | Ground truth |
|----|-----------|--------------|-------------|
| N1 | 9/11 conspiracy theories ("inside job", "controlled demolition") | RT, various | NIST reports, 9/11 Commission |
| N2 | US election interference denial / "stolen election" | RT, Sputnik, IRA | DNI assessments, Mueller Report |
| N3 | NATO expansion as provocation | RT, TASS, MFA | NATO open-door policy, Budapest Memorandum |
| N4 | US biolabs in Ukraine | RT, MFA, Sputnik | DoD factsheets, WHO inspections |
| N5 | COVID-19 US origin / lab leak conspiracy | Xinhua, CGTN, RT | WHO reports, peer-reviewed virology |
| N6 | US imperialism / regime change framing | RT, PressTV, CGTN | Historical record, UN resolutions |

## Architecture

```
causal-decontamination/
├── SPEC.md                          # This file
├── CLAUDE.md                        # Agent instructions
├── Makefile                         # Full pipeline: make all
│
├── pipelines/
│   ├── identify/                    # Phase 1: Find propaganda in training corpus
│   │   ├── classifier.rs            # Rust classifier (extend from crimeaisukraine)
│   │   ├── signals/                 # Per-narrative regex + embedding signals
│   │   ├── validate.py              # Fine-tuned validator (Gemma 4 or similar)
│   │   └── data/
│   │       ├── dolma_census.jsonl   # Identified propaganda docs in Dolma
│   │       └── manifest.json
│   │
│   ├── retrain/                     # Phase 2: Contrastive retraining
│   │   ├── prepare_corpus.py        # Generate clean vs contaminated Dolma splits
│   │   ├── train.py                 # OLMo-1B training (Dolma → OLMo)
│   │   ├── configs/
│   │   │   ├── baseline.yaml        # Full Dolma (contaminated)
│   │   │   └── decontaminated.yaml  # Dolma minus identified propaganda
│   │   └── data/
│   │       └── manifest.json
│   │
│   ├── benchmark/                   # Phase 3: Behavioral measurement
│   │   ├── narrative_benchmark.py   # Forced-choice + free-recall (SAS-style)
│   │   ├── questions/               # Per-narrative question sets
│   │   ├── compute_scores.py        # Delta computation
│   │   └── data/
│   │       ├── baseline_scores.json
│   │       ├── decontaminated_scores.json
│   │       └── manifest.json
│   │
│   ├── interpret/                   # Phase 4: Mechanistic analysis
│   │   ├── probing.py               # Linear probes on residual stream
│   │   ├── causal_trace.py          # Activation patching per narrative
│   │   ├── influence.py             # Training document influence estimation
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
│   ├── narratives/                  # Ground-truth narrative definitions
│   └── results/                     # Final aggregated results
│
├── models/                          # Trained model checkpoints
│   ├── olmo-1b-baseline/
│   └── olmo-1b-decontaminated/
│
└── paper/
    └── docs/
```

## Method

### Phase 1 — Identify (extend existing infrastructure)

Scan Dolma v1.7 (OLMo's training corpus, ~3T tokens) for documents matching the 6 narrative categories. Use:
- Rust classifier with per-narrative signal sets (extend crimeaisukraine's 91-signal approach)
- Fine-tuned Gemma 4 validator for precision (F1 target: >0.95)
- Cross-reference against OFAC/EU/UK sanctions domain lists
- Output: list of document IDs to remove, with narrative labels and confidence scores

### Phase 2 — Retrain (the core experiment)

Train two OLMo-1B models from scratch on identical hyperparameters:
- **Baseline**: full Dolma corpus
- **Decontaminated**: Dolma minus identified propaganda documents

Use AI2's OLMo training infrastructure. Estimated cost: $200-500 per run on cloud GPUs.

### Phase 3 — Benchmark (measure the delta)

Design a Narrative Alignment Score (NAS) analogous to SAS from crimeaisukraine:
- Forced-choice questions per narrative (e.g., "Was 9/11 an inside job? yes/no")
- Free-recall questions (e.g., "What happened on September 11, 2001?")
- 50 languages for cross-lingual transfer measurement
- Compute NAS for baseline vs decontaminated models
- Statistical tests: paired t-test per narrative, effect sizes (Cohen's d)

### Phase 4 — Interpret (the ML that impresses)

On both baseline and decontaminated models:
- **Probing classifiers**: train linear probes on each layer's residual stream to detect narrative-specific activations. Does decontamination remove the "9/11 inside job" direction?
- **Causal tracing**: use activation patching to identify which layers/attention heads encode each narrative. Compare baseline vs decontaminated architecture.
- **Influence estimation**: for specific model outputs, estimate which training documents contributed most (using gradient-based influence functions or TracIn)

### Phase 5 — Grounding (reuse existing pipeline)

Run the grounding audit from crimeaisukraine on US-targeted narratives:
- Do chatbots cite RT/Sputnik/TASS when answering 9/11 or election questions?
- Cross-reference against sanctions lists

## Dependencies on crimeaisukraine

| Component | Reuse | Extend |
|-----------|-------|--------|
| Rust classifier framework | Yes | Add 6 new narrative signal sets |
| SAS scoring methodology | Yes | Rename to NAS, adapt tiers |
| Grounding audit pipeline | Yes | New queries for US narratives |
| Sanctions CSV matching | Yes | Same OFAC/EU/UK lists |
| 50-language framework | Yes | Same languages |
| LLM audit infrastructure | Yes | Same dual-tier protocol |

## Compute Budget

| Phase | Estimated cost |
|-------|---------------|
| Phase 1 (identify) | $0 (Dolma is downloadable, Rust classifier runs on CPU) |
| Phase 2 (retrain OLMo-1B x2) | $400-800 |
| Phase 3 (benchmark) | $50-100 (API calls) |
| Phase 4 (interpret) | $100-200 (GPU for probing/tracing) |
| Phase 5 (grounding) | $50-100 (API calls) |
| **Total** | **$600-1,200** |

## Target Venues

1. **ACL / EMNLP** — computational linguistics, flagship venue
2. **NeurIPS / ICML** — if mechanistic interpretability findings are strong
3. **USENIX Security** — if framed as AI supply chain security
4. **Nature Machine Intelligence** — if results are clean and causal chain is proven
5. **Policy brief** → FDD, CSIS, Brookings, Atlantic Council

## Prior Art (differentiation)

| Paper | What they did | What we add |
|-------|--------------|-------------|
| Deep Ignorance (ICLR 2026) | Retrain-without for bioweapons | First for propaganda |
| Roberts et al. (*Nature*) | Propaganda addition shifts outputs | We do subtraction (stronger causal claim) |
| Anthropic 250-doc (2025) | Synthetic backdoor threshold | Real propaganda, real narratives |
| DFRLab Pravda (2026) | Counted articles in Common Crawl | We trace causal chain to behavior |
| Dobrovolskyi Crimea (2026) | Three-layer audit, RLHF gap | We add mechanistic interpretability + contrastive retraining |
| OLMo C4/4chan (ICML 2025) | Toxicity from 4chan in training data | State propaganda, not toxicity |

## Author

**Ivan Dobrovolskyi** · [dobrovolsky94@gmail.com](mailto:dobrovolsky94@gmail.com)

Builds on methodology from [crimeaisukraine](https://github.com/IvanDobrovolsky/crimeaisukraine) and [kyivnotkiev](https://github.com/IvanDobrovolsky/kyivnotkiev).
