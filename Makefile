.PHONY: build scan

build:
	cd scanner && cargo build --release

scan: build
	./scanner/target/release/pile-scanner data/pile/ -o data/results/pile_census.jsonl
