.PHONY: all identify retrain benchmark interpret grounding

all: identify retrain benchmark interpret grounding

identify:
	cd pipelines/identify && python3 validate.py

retrain:
	cd pipelines/retrain && python3 train.py

benchmark:
	cd pipelines/benchmark && python3 narrative_benchmark.py

interpret:
	cd pipelines/interpret && python3 probing.py

grounding:
	cd pipelines/grounding && python3 scan.py
