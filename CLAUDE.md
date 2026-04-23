# Claude Instructions

## Project Context

This is a research project proving the causal effect of foreign state propaganda in LLM training data and mechanistically explaining where propaganda is encoded in transformer representations. The core experiment: identify propaganda in The Pile, retrain Pythia-1B without it, use probing classifiers and causal tracing to show framing bias occupies different network layers than factual knowledge.

**Key principle:** This is a controlled scientific experiment. Every claim must be backed by reproducible evidence. No speculation, no exciting numbers without proof.

## Architecture

See SPEC.md for full architecture. Five phases:
1. **Identify** — Find propaganda in The Pile using Rust classifier + LLM validator
2. **Retrain** — Train Pythia-1B decontaminated (baseline is published checkpoint, free)
3. **Benchmark** — Measure behavioral delta (NAS scoring)
4. **Interpret** — Mechanistic analysis (probing, causal tracing, influence functions)
5. **Grounding** — Search contamination audit (reuse from crimeaisukraine)

## Model & Data

- **Model**: Pythia-1B (EleutherAI, GPT-NeoX architecture)
- **Dataset**: The Pile (~825 GiB, 300B tokens, 22 subsets)
- **Training**: Chinchilla-optimal ~20B tokens, 2 seeds for decontaminated
- **Baseline**: Published Pythia-1B checkpoint at step ~9,500
- **Interpretability**: TransformerLens (first-class Pythia support)

## Code Standards

- Rust for high-throughput data scanning (The Pile is ~825 GiB)
- Python for ML pipelines (PyTorch, GPT-NeoX for training, TransformerLens for interpretability)
- Each pipeline has its own directory, `data/` subdirectory, and `manifest.json`
- All results go in pipeline-specific `data/` directories
- Raw data is append-only during scans (never rename/rewrite while processes run)
- Temperature 0, seed 42 for all LLM queries

## Dependencies

This project reuses infrastructure from `../crimeaisukraine/`:
- Rust classifier framework (extend with new narrative signals for 3 US-targeted narratives)
- SAS scoring methodology (adapted as NAS)
- Grounding audit pipeline
- Sanctions CSV matching

## Three Narratives

| ID | Narrative | Encoding type |
|----|-----------|---------------|
| N1 | 9/11 "inside job" | Conspiracy (factually false) |
| N2 | NATO expansion as provocation | Framing (selectively true) |
| N3 | US biolabs in Ukraine | Fabrication (kernel of truth) |

## Working Style

- Verify every claim against actual data
- Manual annotation required for validation sets
- No Co-Authored-By Claude in git commits
- Budget-conscious: Pythia-1B Chinchilla-optimal, published checkpoint as free baseline
- Check `pgrep` before rewriting any append-only data files
