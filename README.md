<h1 align="center">Corpus Decontamination</h1>

<p align="center"><strong>Measuring the Causal Effect of Propaganda Removal on Language Model Behavior</strong></p>

<p align="center">
  <a href="docs/METHODOLOGY.md">Methodology</a> ·
  <a href="OVERVIEW.md">Research Overview</a> ·
  <a href="benchmark/">Benchmarks</a> ·
  <a href="pipeline/">Scanner</a> ·
  <a href="training/">Training</a>
</p>

---

## Research Question

Does removing documented state propaganda from a language model's training corpus produce a measurable, causal change in the model's behavior — even when the removed content is just 0.004% of the total training data?

## Key Findings (Pre-Training)

**Inverse scaling of propaganda absorption.** Across all 8 Pythia model sizes (70M–12B), 9/11 conspiracy preference scales from 39% → 61% with model size (r = +0.883). Larger models memorize more propaganda from the same training data, not less.

**180-probe benchmark.** Log-probability probes covering 9 documented disinformation narratives show Pythia models systematically prefer propaganda completions over factual ones on several narratives, with the signal robust across model sizes.

## Methodology

1. **Scan** — Rust pipeline processes 600GB of tokenized Pythia training data (146M sequences). Aho-Corasick multi-pattern matching with 3-layer false-positive prevention: context proximity, keyword co-occurrence, and counter-factive attribution detection.

2. **Classify** — LLM-assisted classification distinguishes asserting propaganda from journalism, debunking, and academic content. Human annotation by two independent annotators validates precision with Cohen's kappa.

3. **Ablate** — Retrain Pythia-160M using GPT-NeoX (the identical framework and hyperparameters). Only modification: data loader skips flagged sequence indices.

4. **Measure** — 180 log-probability probes + perplexity on excluded sequences + multi-temperature generation across all 8 model sizes. Pre-registered dose-response prediction: effect size proportional to removal volume.

## Narratives

9 narratives sourced systematically from US government disinformation reports:

| Narrative | Official Source | Training Data Hits |
|-----------|----------------|-------------------|
| N1: 9/11 conspiracy | NIST NCSTAR 1 (2005), 9/11 Commission (2004) | 6,675 |
| N3: JFK conspiracy | Warren Commission (1964), HSCA (1979) | 1,892 |
| N6: Moon landing hoax | NASA, EUvsDisinfo | 238 |
| N7: Anti-vaccination | Broniatowski et al. (2018, AJPH), WHO | 145 |
| N2: NATO provocation | GEC Pillars of Russia's Disinformation (2020) | 22 |
| N9: Uyghur denial | UN OHCHR (2022), GEC China Report (2023) | 26 |
| N4: US biolabs | GEC Chem/Bio Report (2022), Nunn-Lugar Act | 7 |
| N5: Syria / White Helmets | OPCW IIT Reports (2020-2023) | 1 |
| N8: Soros conspiracy | EUvsDisinfo (220+ cases), ADL | 7 |

25 state media domains detected (Russian + Chinese) per US State Department designations.

Selection criterion: complete coverage of GEC-identified disinformation themes where keyword detection is feasible. [Full mapping →](docs/METHODOLOGY.md#1-narrative-selection)

## Validation

- **Keyword precision:** 839/839 hits verified against full detokenized text (100%)
- **Zero exact duplicates** across 9,006 narrative hits; near-dupe rate < 3% on primary narratives
- **LLM classification** with narrative-specific prompts citing official findings
- **Human annotation** by 2 independent annotators with inter-annotator agreement (Cohen's kappa)
- **Statistical framework:** Wilcoxon signed-rank test, Cohen's d effect sizes, Bonferroni correction

## Project Structure

```
pipeline/        Rust scanner — Aho-Corasick, 16-thread parallel, MMap dataset reader
benchmark/       180-probe benchmark suite, sweep across all 8 Pythia sizes
training/        GPT-NeoX config (byte-for-byte identical to original Pythia)
docs/            Full methodology — signal definitions, probe justifications, statistical plan
```

## Technology

| Component | Stack |
|-----------|-------|
| Scanner | Rust, aho-corasick, memmap2, rayon (600GB scanned in 30 min) |
| Models | EleutherAI Pythia 70M–12B, all trained on identical data in identical order |
| Training | GPT-NeoX + DeepSpeed ZeRO-1 + Flash Attention + FP16 |
| Evaluation | PyTorch, HuggingFace Transformers |
| Classification | Anthropic Claude API with narrative-specific prompts |
| Dataset | EleutherAI/pile-standard-pythia-preshuffled (146M sequences, 300B tokens) |

## Reproducibility

- All inference: temperature 0, seed 42
- Training: identical GPT-NeoX config, only data loader patched
- Scanner: deterministic keyword matching with documented sources
- Dataset verified: 146,432,000 sequences × 2,049 tokens = 300.0B tokens (matches Pythia's 143K steps × 2M tokens/batch)

## Prior Work

Extends [crimeaisukraine](https://crimeaisukraine.org) (507K propaganda documents identified in C4 corpus) and [kyivnotkiev](https://github.com/IvanDobrovolsky/kyivnotkiev) (tracking adoption of Ukrainian toponyms in global media).

## Author

**Ivan Dobrovolskyi** · Staff Software Engineer · IEEE Senior Member
