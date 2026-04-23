<h1 align="center">Causal Decontamination</h1>

<p align="center"><strong>Measuring the causal effect of state propaganda in LLM training data through contrastive retraining.</strong></p>

Anthropic/Turing/AISI proved [250 synthetic documents can backdoor any LLM](https://arxiv.org/abs/2510.07192). DFRLab found [40,000 Pravda articles in Common Crawl](https://dfrlab.org/2026/04/08/pravda-in-the-pipeline/). We found [507,000 Russia-framing documents in C4](https://crimeaisukraine.org).

**Nobody has done the subtraction experiment.** We remove identified state propaganda from [Dolma](https://allenai.org/dolma), retrain [OLMo-1B](https://allenai.org/olmo), and measure the causal effect on model behavior across 6 US-targeted disinformation narratives and 50 languages.

## The Experiment

```
Dolma (full)  ──→  OLMo-1B (baseline)         ──→  NAS = X
Dolma - propaganda ──→  OLMo-1B (decontaminated) ──→  NAS = Y
                                                      Δ = Y - X  ← causal effect
```

## Narratives

| # | Narrative | Why it matters |
|---|-----------|---------------|
| 1 | 9/11 conspiracy theories | Most persistent anti-US disinformation, 20+ years |
| 2 | US election interference denial | Active threat to democratic institutions |
| 3 | NATO expansion as provocation | Core Russian strategic narrative |
| 4 | US biolabs in Ukraine | Active IO campaign since 2022 |
| 5 | COVID-19 US origin conspiracy | Cross-actor (Russia + China) |
| 6 | US imperialism / regime change | Foundational anti-US framing |

## Pipelines

| # | Pipeline | Method |
|---|----------|--------|
| 1 | [identify](pipelines/identify/) | Rust classifier + LLM validator on Dolma |
| 2 | [retrain](pipelines/retrain/) | OLMo-1B contrastive training |
| 3 | [benchmark](pipelines/benchmark/) | Narrative Alignment Score (NAS), 50 languages |
| 4 | [interpret](pipelines/interpret/) | Probing classifiers, causal tracing, influence functions |
| 5 | [grounding](pipelines/grounding/) | Chatbot web search audit against sanctions lists |

## Prior Art

- [Deep Ignorance](https://arxiv.org/abs/2508.06601) (ICLR 2026) — retrain-without for bioweapons (we do propaganda)
- [Roberts et al.](https://csss.uw.edu/seminars/propaganda-already-influencing-large-language-models-evidence-training-data-audits-and) (*Nature*) — propaganda addition shifts outputs (we do subtraction)
- [Dobrovolskyi](https://crimeaisukraine.org) (2026) — three-layer audit of Crimea sovereignty framing (we add mechanistic interpretability)

## Author

**Ivan Dobrovolskyi** · [dobrovolsky94@gmail.com](mailto:dobrovolsky94@gmail.com)
