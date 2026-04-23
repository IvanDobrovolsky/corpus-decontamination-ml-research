# Causal Decontamination

**Does removing state propaganda from training data causally change LLM behavior?**

## Setup

- **Model**: Pythia-1B (EleutherAI)
- **Dataset**: The Pile (~825 GiB, 30 shards)
- **Baseline**: Published Pythia-1B checkpoint at step ~9,500 (~20B tokens seen)
- **Decontaminated**: Pythia-1B trained from scratch on ~20B tokens minus propaganda
- **Narratives**: 9/11 "inside job", NATO expansion as provocation, US biolabs in Ukraine

## Phases

1. **Scan** — Rust classifier finds propaganda in The Pile (local, CPU)
2. **Retrain** — Pythia-1B on clean Pile subset (1× A100, ~$150-250/run)
3. **Measure** — Narrative Alignment Score: baseline vs decontaminated
4. **Interpret** — If delta exists: probing + causal tracing to find where propaganda lives in the network

## Prior art

- Deep Ignorance (ICLR 2026) — same method for bioweapons, Pythia 6.9B
- Roberts et al. (Nature, R&R) — propaganda addition shifts outputs
- Anthropic 250-doc (2025) — 250 docs backdoor any LLM
- Dobrovolskyi (2026) — 507K propaganda docs in C4, RLHF gap

## Open questions

- How much propaganda is actually in The Pile? (Phase 1 answers this)
- Is the effect detectable at Chinchilla-optimal scale?
- Is the published checkpoint a valid baseline or do we need to train our own?

## Author

**Ivan Dobrovolskyi** · [dobrovolsky94@gmail.com](mailto:dobrovolsky94@gmail.com)
