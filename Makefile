.PHONY: all identify retrain benchmark interpret grounding

all: identify retrain benchmark interpret grounding

# Phase 1: Scan The Pile for propaganda (local, CPU)
identify:
	cd pipelines/identify/classifier && cargo run --release -- \
		--pile-dir $(PILE_DIR) \
		--output ../data/pile_census.jsonl

# Phase 1b: Validate borderline cases with LLM
validate:
	cd pipelines/identify && python3 validate.py

# Phase 2: Train decontaminated Pythia-1B (GPU instance)
retrain:
	cd pipelines/retrain && python3 train.py \
		--config configs/decontaminated.yaml

# Phase 3: Run NAS benchmark on baseline vs decontaminated
benchmark:
	cd pipelines/benchmark && python3 narrative_benchmark.py
	cd pipelines/benchmark && python3 compute_nas.py

# Phase 4: Mechanistic interpretability
interpret: probing causal-trace influence

probing:
	cd pipelines/interpret && python3 probing.py

causal-trace:
	cd pipelines/interpret && python3 causal_trace.py

influence:
	cd pipelines/interpret && python3 influence.py

framing-vs-factual:
	cd pipelines/interpret && python3 framing_vs_factual.py

# Phase 5: Grounding audit (reuse from crimeaisukraine)
grounding:
	cd pipelines/grounding && python3 scan.py
