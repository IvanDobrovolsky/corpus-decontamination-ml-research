# Claude Instructions

## Project Context

Research project testing whether removing state propaganda from The Pile and retraining Pythia-1B produces a measurable behavioral change. Early stage — proving feasibility before committing to full interpretability analysis.

## Key Details

- **Model**: Pythia-1B on The Pile (~20B tokens, Chinchilla-optimal)
- **Narratives**: 9/11 "inside job", NATO expansion, US biolabs
- **Rust** for scanning The Pile, **Python** for ML
- Temperature 0, seed 42 for all LLM queries
- No Co-Authored-By Claude in git commits
- Extends prior work in `../crimeaisukraine/`
