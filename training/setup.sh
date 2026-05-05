#!/bin/bash
# Training environment setup for Pythia-160M retraining
# Uses GPT-NeoX v1.0 — the EXACT version that trained the original Pythia models
#
# CRITICAL: Three things must match original Pythia exactly:
# 1. GPT-NeoX v1.0 (not latest)
# 2. Seed 1234 (GPT-NeoX default)
# 3. Data must be unsharded into single .bin file
set -e

echo "=== Setting up GPT-NeoX training environment ==="

# 1. Clone GPT-NeoX at EXACT version used for Pythia
if [ ! -d "gpt-neox" ]; then
    git clone https://github.com/EleutherAI/gpt-neox.git
    cd gpt-neox
    git checkout v1.0  # MUST match Pythia training version
else
    cd gpt-neox
fi

# 2. Install dependencies (flash attention version for v1.0)
pip install -r requirements/requirements-flashattention.txt
pip install wandb  # for training curve logging

# 3. Get the exact Pythia-160M config
mkdir -p configs/pythia
if [ ! -f "configs/pythia/160M.yml" ]; then
    curl -o configs/pythia/160M.yml \
        https://raw.githubusercontent.com/EleutherAI/pythia/main/models/160M/pythia-160m.yml
fi

# 4. Copy our custom data loader patch
cp ../data_loader_patch.py megatron/data/data_loader_patch.py

echo ""
echo "=== Setup complete ==="
echo ""
echo "BEFORE TRAINING: unshard the data if not done yet:"
echo "  python utils/unshard_memmap.py \\"
echo "    --input_file /path/to/pythia-data/document-00000-of-00020.bin \\"
echo "    --num_shards 20 \\"
echo "    --output_dir /path/to/merged/"
echo "  cp /path/to/pythia-data/document.idx /path/to/merged/document.idx"
echo ""
echo "To train:"
echo "  python deepy.py train.py configs/pythia/160M.yml configs/local_setup.yml"
