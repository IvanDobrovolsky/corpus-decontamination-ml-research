# Claude Instructions

## Project Context

This is a research project proving the causal effect of foreign state propaganda in LLM training data. The core experiment: remove identified propaganda from Dolma, retrain OLMo-1B, measure the behavioral delta.

**Key principle:** This is a controlled scientific experiment. Every claim must be backed by reproducible evidence. No speculation, no exciting numbers without proof.

## Architecture

See SPEC.md for full architecture. Five phases:
1. **Identify** — Find propaganda in Dolma using Rust classifier + LLM validator
2. **Retrain** — Train OLMo-1B baseline vs decontaminated
3. **Benchmark** — Measure behavioral delta (NAS scoring)
4. **Interpret** — Mechanistic analysis (probing, causal tracing, influence functions)
5. **Grounding** — Search contamination audit (reuse from crimeaisukraine)

## Code Standards

- Rust for high-throughput data scanning (Dolma is ~3T tokens)
- Python for ML pipelines (PyTorch, TransformerLens for interpretability)
- Each pipeline has its own `scan.py` or equivalent, `data/manifest.json`, and `README.md`
- All results go in pipeline-specific `data/` directories
- Raw data is append-only during scans (never rename/rewrite while processes run)
- Temperature 0, seed 42 for all LLM queries

## Dependencies

This project reuses infrastructure from `../crimeaisukraine/`:
- Rust classifier framework (extend with new narrative signals)
- SAS scoring methodology (adapted as NAS)
- Grounding audit pipeline
- Sanctions CSV matching
- 50-language query framework

## Working Style

- Verify every claim against actual data
- Manual annotation required for validation sets
- No Co-Authored-By Claude in git commits
- Budget-conscious: OLMo-1B, not larger models
- Check `pgrep` before rewriting any append-only data files
