#!/bin/bash
# Training environment setup for Pythia-160M retraining
# Uses GPT-NeoX — the identical framework that trained the original Pythia models
set -e

echo "=== Setting up GPT-NeoX training environment ==="

# 1. Clone GPT-NeoX
if [ ! -d "gpt-neox" ]; then
    git clone https://github.com/EleutherAI/gpt-neox.git
    cd gpt-neox
else
    cd gpt-neox
    git pull
fi

# 2. Install dependencies
pip install -r requirements/requirements.txt
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
echo "To train:"
echo "  python deepy.py train.py configs/pythia/160M.yml configs/local_setup.yml"
echo ""
echo "configs/local_setup.yml should override:"
echo "  - data-path: path to your data"
echo "  - save/load paths"
echo "  - number of GPUs"
