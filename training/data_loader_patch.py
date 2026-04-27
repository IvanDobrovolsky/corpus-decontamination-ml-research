"""
Patch for GPT-NeoX's MMapIndexedDataset to skip excluded sequences.

This is the ONLY modification to the GPT-NeoX training framework.
All other code, hyperparameters, and infrastructure remain identical
to the original Pythia training.

Usage: The exclusion manifest is a JSON file containing:
  {"total_documents": N, "excluded_count": K, "doc_ids": [sorted list of sequence indices]}

Sequences in the exclusion list are skipped during training. The effective
dataset size is reduced by K sequences (typically < 0.01% of total).
"""

import json
import os
import numpy as np


def build_exclusion_set(manifest_path):
    """Load exclusion manifest and return a set of sequence indices to skip."""
    if not manifest_path or not os.path.exists(manifest_path):
        return set()
    with open(manifest_path) as f:
        manifest = json.load(f)
    excluded = set(manifest["doc_ids"])
    print(f"[Decontamination] Loaded exclusion manifest: "
          f"{len(excluded)} sequences to skip from {manifest_path}")
    return excluded


def build_filtered_index(total_sequences, exclusion_set):
    """Build a mapping from filtered index → original sequence index.

    Returns a numpy array where filtered_index[i] = original_sequence_index.
    This allows the data loader to transparently skip excluded sequences
    while maintaining sequential access patterns.
    """
    if not exclusion_set:
        return np.arange(total_sequences, dtype=np.int64)

    filtered = np.array(
        [i for i in range(total_sequences) if i not in exclusion_set],
        dtype=np.int64
    )
    print(f"[Decontamination] Dataset: {len(filtered)}/{total_sequences} sequences "
          f"({len(exclusion_set)} excluded, {len(exclusion_set)/total_sequences:.4%})")
    return filtered
