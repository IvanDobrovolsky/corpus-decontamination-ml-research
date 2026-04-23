.PHONY: scan retrain benchmark

scan:
	cd pipelines/identify && cargo run --release

retrain:
	cd pipelines/retrain && python3 train.py

benchmark:
	cd pipelines/benchmark && python3 compute_nas.py
