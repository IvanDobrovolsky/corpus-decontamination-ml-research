"""
Retrain Pythia-160M with decontaminated data.

Matches original Pythia hyperparameters exactly:
- Architecture: 12 layers, 768 hidden, 12 heads, RoPE
- Optimizer: Adam (beta1=0.9, beta2=0.95, eps=1e-8)
- LR: 6e-4 with cosine decay to 6e-5, 1% warmup
- Batch size: 1024 sequences × 2048 tokens = 2M tokens/step
- Weight decay: 0.1, gradient clip: 1.0
- FP16 training

Source: EleutherAI/pythia models/160M/pythia-160m.yml
"""

import argparse
import json
import math
import os
import struct
import array as pyarray
import time

import numpy as np
import torch
import torch.nn.functional as F
from torch.utils.data import Dataset, DataLoader
from transformers import GPTNeoXConfig, GPTNeoXForCausalLM, AutoTokenizer

# ── Pythia-160M hyperparameters (from pythia-160m.yml) ──────────────

CONFIG = {
    "hidden_size": 768,
    "num_hidden_layers": 12,
    "num_attention_heads": 12,
    "intermediate_size": 768 * 4,  # 3072
    "max_position_embeddings": 2048,
    "rotary_pct": 0.25,
    "vocab_size": 50304,  # padded to multiple of 128
    "use_parallel_residual": True,
    "layer_norm_eps": 1e-5,
}

LR = 6e-4
MIN_LR = 6e-5
WARMUP_RATIO = 0.01
WEIGHT_DECAY = 0.1
GRAD_CLIP = 1.0
BETAS = (0.9, 0.95)
EPS = 1e-8
SEQ_LEN = 2048
TOTAL_BATCH_TOKENS = 2_097_152  # 1024 sequences × 2048
TOTAL_STEPS = 143_000
SEED = 1234  # GPT-NeoX default — must match original Pythia training


class MMapPileDataset(Dataset):
    """Reads Megatron-LM MMap format, skipping excluded sequence indices."""

    def __init__(self, data_dir, exclusion_path=None):
        idx_path = os.path.join(data_dir, "document.idx")

        with open(idx_path, "rb") as f:
            f.read(9)  # magic
            f.read(8)  # version
            f.read(1)  # dtype
            self.num_seq = struct.unpack("<Q", f.read(8))[0]
            f.read(8)  # num_doc

            self.sizes = pyarray.array("i")
            self.sizes.frombytes(f.read(self.num_seq * 4))
            self.ptrs = pyarray.array("q")
            self.ptrs.frombytes(f.read(self.num_seq * 8))

        # Bin shards
        bin_files = sorted(f for f in os.listdir(data_dir) if f.endswith(".bin"))
        self.bin_paths = [os.path.join(data_dir, f) for f in bin_files]
        self.cum = [0]
        for bp in self.bin_paths:
            self.cum.append(self.cum[-1] + os.path.getsize(bp))

        # Open mmaps
        self.mmaps = []
        for bp in self.bin_paths:
            f = open(bp, "rb")
            mm = torch.frombuffer(
                bytearray(os.path.getsize(bp)), dtype=torch.uint8
            ) if False else None
            self.mmaps.append(bp)  # lazy read

        # Build index: map logical index → physical sequence index
        excluded = set()
        if exclusion_path and os.path.exists(exclusion_path):
            manifest = json.load(open(exclusion_path))
            excluded = set(manifest["doc_ids"])
            print(f"Excluding {len(excluded)} sequences from {exclusion_path}")

        self.index = [i for i in range(self.num_seq) if i not in excluded]
        print(f"Dataset: {len(self.index)}/{self.num_seq} sequences "
              f"({len(excluded)} excluded)")

    def __len__(self):
        return len(self.index)

    def __getitem__(self, idx):
        seq_idx = self.index[idx]
        n_tokens = self.sizes[seq_idx]
        byte_offset = self.ptrs[seq_idx]
        n_bytes = n_tokens * 2

        # Find shard
        shard = 0
        while shard + 1 < len(self.cum) and self.cum[shard + 1] <= byte_offset:
            shard += 1
        local_offset = byte_offset - self.cum[shard]

        with open(self.bin_paths[shard], "rb") as f:
            f.seek(local_offset)
            data = f.read(n_bytes)

        # uint16 tokens — numpy handles unsigned correctly
        tokens = torch.from_numpy(
            np.frombuffer(data, dtype=np.uint16).astype(np.int64).copy()
        )

        # Input: tokens[:-1], labels: tokens[1:]
        input_ids = tokens[:SEQ_LEN]
        labels = tokens[1:SEQ_LEN + 1]
        return input_ids, labels


def get_lr(step, total_steps, warmup_steps, lr, min_lr):
    """Cosine decay with linear warmup."""
    if step < warmup_steps:
        return lr * step / warmup_steps
    progress = (step - warmup_steps) / (total_steps - warmup_steps)
    return min_lr + 0.5 * (lr - min_lr) * (1 + math.cos(math.pi * progress))


def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--data_dir", required=True)
    parser.add_argument("--exclusion", default=None,
                        help="Path to exclusion manifest JSON")
    parser.add_argument("--output_dir", default="./checkpoints")
    parser.add_argument("--steps", type=int, default=TOTAL_STEPS)
    parser.add_argument("--micro_batch", type=int, default=16)
    parser.add_argument("--save_every", type=int, default=5000)
    parser.add_argument("--log_every", type=int, default=10)
    parser.add_argument("--resume", default=None, help="Checkpoint to resume from")
    args = parser.parse_args()

    torch.manual_seed(SEED)
    os.makedirs(args.output_dir, exist_ok=True)

    # Effective batch size
    seqs_per_step = TOTAL_BATCH_TOKENS // SEQ_LEN  # 1024
    grad_accum = seqs_per_step // args.micro_batch
    warmup_steps = int(args.steps * WARMUP_RATIO)

    print(f"=== Pythia-160M Training ===")
    print(f"Steps: {args.steps}, micro_batch: {args.micro_batch}, "
          f"grad_accum: {grad_accum}, effective_batch: {seqs_per_step}")
    print(f"LR: {LR} → {MIN_LR}, warmup: {warmup_steps} steps")
    print(f"Exclusion: {args.exclusion or 'None (baseline)'}")

    # Model
    config = GPTNeoXConfig(**CONFIG)
    if args.resume:
        print(f"Resuming from {args.resume}")
        model = GPTNeoXForCausalLM.from_pretrained(args.resume)
    else:
        print("Initializing model from scratch")
        model = GPTNeoXForCausalLM(config)

    param_count = sum(p.numel() for p in model.parameters())
    print(f"Parameters: {param_count:,}")

    model = model.cuda().train()
    scaler = torch.amp.GradScaler("cuda")

    # Optimizer (no weight decay on bias and layernorm)
    no_decay = {"bias", "layer_norm.weight", "layernorm.weight"}
    param_groups = [
        {"params": [p for n, p in model.named_parameters()
                     if not any(nd in n for nd in no_decay)],
         "weight_decay": WEIGHT_DECAY},
        {"params": [p for n, p in model.named_parameters()
                     if any(nd in n for nd in no_decay)],
         "weight_decay": 0.0},
    ]
    optimizer = torch.optim.AdamW(param_groups, lr=LR, betas=BETAS, eps=EPS)

    # Dataset
    dataset = MMapPileDataset(args.data_dir, args.exclusion)
    loader = DataLoader(
        dataset, batch_size=args.micro_batch, shuffle=True,
        num_workers=4, pin_memory=True, drop_last=True,
    )
    loader_iter = iter(loader)

    # Training loop
    step = 0
    total_loss = 0.0
    total_tokens = 0
    start_time = time.time()

    print(f"\nStarting training...")
    while step < args.steps:
        optimizer.zero_grad()
        step_loss = 0.0

        for micro_step in range(grad_accum):
            try:
                input_ids, labels = next(loader_iter)
            except StopIteration:
                loader_iter = iter(loader)
                input_ids, labels = next(loader_iter)

            input_ids = input_ids.cuda()
            labels = labels.cuda()

            with torch.amp.autocast("cuda", dtype=torch.float16):
                outputs = model(input_ids=input_ids, labels=labels)
                loss = outputs.loss / grad_accum

            scaler.scale(loss).backward()
            step_loss += loss.item()

        # Gradient clipping
        scaler.unscale_(optimizer)
        torch.nn.utils.clip_grad_norm_(model.parameters(), GRAD_CLIP)

        # LR schedule
        lr = get_lr(step, args.steps, warmup_steps, LR, MIN_LR)
        for pg in optimizer.param_groups:
            pg["lr"] = lr

        scaler.step(optimizer)
        scaler.update()

        step += 1
        total_loss += step_loss
        total_tokens += seqs_per_step * SEQ_LEN

        if step % args.log_every == 0:
            avg_loss = total_loss / args.log_every
            elapsed = time.time() - start_time
            tokens_per_sec = total_tokens / elapsed
            eta_hours = (args.steps - step) / (step / elapsed) / 3600

            print(f"step {step}/{args.steps} | loss {avg_loss:.4f} | "
                  f"lr {lr:.2e} | {tokens_per_sec:.0f} tok/s | "
                  f"ETA {eta_hours:.1f}h")
            total_loss = 0.0

        if step % args.save_every == 0:
            ckpt_path = os.path.join(args.output_dir, f"step_{step}")
            model.save_pretrained(ckpt_path)
            print(f"Saved checkpoint: {ckpt_path}")

    # Final save
    final_path = os.path.join(args.output_dir, f"step_{step}")
    model.save_pretrained(final_path)
    elapsed = time.time() - start_time
    print(f"\nDone. {step} steps in {elapsed/3600:.1f}h. "
          f"Final checkpoint: {final_path}")


if __name__ == "__main__":
    main()
