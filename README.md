<h1 align="center">Causal Decontamination</h1>

<p align="center"><strong>Mechanistic analysis of state propaganda encoding in LLMs through contrastive retraining.</strong></p>

Anthropic/Turing/AISI proved [250 synthetic documents can backdoor any LLM](https://arxiv.org/abs/2510.07192). DFRLab found [40,000 Pravda articles in Common Crawl](https://dfrlab.org/2026/04/08/pravda-in-the-pipeline/). We found [507,000 Russia-framing documents in C4](https://crimeaisukraine.org).

**Nobody has explained where propaganda lives in the network or why RLHF can't fix it.** We scan [The Pile](https://pile.eleuther.ai/) for state propaganda, retrain [Pythia-1B](https://github.com/EleutherAI/pythia), and use mechanistic interpretability to show that propaganda creates persistent framing directions in representation space — geometrically distinct from factual knowledge and localized to layers that safety training cannot reach.

## The Experiment

```
Pythia-1B step ~9500       →  NAS = X   (baseline, with propaganda)
Pythia-1B from scratch     →  NAS = Y   (decontaminated, without propaganda)
(~20B tokens, Chinchilla-optimal)
                               Δ = Y - X  ← causal effect
```

Then: probing classifiers + causal tracing on both models to explain the delta mechanistically.

## Narratives

| # | Narrative | Encoding type | Why it matters |
|---|-----------|---------------|---------------|
| 1 | 9/11 "inside job" | Conspiracy (false fact) | Most persistent anti-US disinformation, 20+ years |
| 2 | NATO expansion as provocation | Framing (selective truth) | Core Russian strategic narrative, hardest case for detection |
| 3 | US biolabs in Ukraine | Fabrication (invented claim) | Active IO campaign, kernel of truth makes it subtle |

## Pipelines

| # | Pipeline | Method |
|---|----------|--------|
| 1 | [identify](pipelines/identify/) | Rust classifier + LLM validator on The Pile |
| 2 | [retrain](pipelines/retrain/) | Pythia-1B contrastive pretraining (Chinchilla-optimal) |
| 3 | [benchmark](pipelines/benchmark/) | Narrative Alignment Score (NAS) |
| 4 | [interpret](pipelines/interpret/) | Probing classifiers, causal tracing, influence estimation |
| 5 | [grounding](pipelines/grounding/) | Chatbot web search audit against sanctions lists |

## Core Finding (predicted)

Propaganda encoding in LLMs is **not uniform**:
- **Factual knowledge** (e.g., "what happened on 9/11") is encoded in late MLP layers where RLHF has leverage
- **Framing bias** (e.g., "NATO expansion is aggression") is encoded in early/mid attention heads where RLHF cannot reach
- This is why safety training suppresses propaganda in outputs but the representations persist
- Contrastive retraining eliminates framing directions entirely

## Prior Art

- [Deep Ignorance](https://arxiv.org/abs/2508.06601) (ICLR 2026) — contrastive retraining for bioweapons (we do propaganda, which is framing, not factual knowledge)
- [Roberts et al.](https://csss.uw.edu/seminars/propaganda-already-influencing-large-language-models-evidence-training-data-audits-and) (*Nature*, R&R) — propaganda addition shifts outputs (we do subtraction + explain the mechanism)
- [Anthropic 250-doc](https://arxiv.org/abs/2510.07192) (2025) — 250 synthetic docs backdoor any LLM (we study real propaganda, real narratives)
- [Dobrovolskyi](https://crimeaisukraine.org) (2026) — training data audit, RLHF gap (we explain WHY the gap exists + causal proof)

## Research Arc

| Project | Year | Contribution |
|---------|------|-------------|
| [kyivnotkiev](https://github.com/IvanDobrovolsky/kyivnotkiev) | 2024 | Toponym adoption reflects political influence in media |
| [crimeaisukraine](https://crimeaisukraine.org) | 2026 | 507K propaganda docs in training data shift LLM outputs, RLHF gap |
| **this paper** | 2026 | Mechanistic explanation of propaganda encoding + causal proof via retraining |

## Author

**Ivan Dobrovolskyi** · [dobrovolsky94@gmail.com](mailto:dobrovolsky94@gmail.com)
