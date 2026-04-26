"""
LLM-assisted annotation of scanner classification accuracy.

For a sample of Organic and Cited hits, extract full text context
around the keywords and prepare for LLM annotation + human verification.

Outputs a JSON file with full context that can be sent to an LLM
for classification, then verified by a human annotator.
"""

import json
import struct
import array
import os
import random

random.seed(42)

DATA_DIR = "/mnt/HC_Volume_105510485/pythia-data/"
HITS_FILE = os.path.join(DATA_DIR, "pile_hits_v5.jsonl")

# Load idx
with open(os.path.join(DATA_DIR, "document.idx"), "rb") as f:
    f.read(9); f.read(8); f.read(1)
    num_seq = struct.unpack("<Q", f.read(8))[0]; f.read(8)
    sizes = array.array("i"); sizes.frombytes(f.read(num_seq * 4))
    ptrs = array.array("q"); ptrs.frombytes(f.read(num_seq * 8))

bin_files = sorted(bf for bf in os.listdir(DATA_DIR) if bf.endswith(".bin"))
cum = [0]
for bf in bin_files:
    cum.append(cum[-1] + os.path.getsize(os.path.join(DATA_DIR, bf)))

from tokenizers import Tokenizer
tok = Tokenizer.from_file(os.path.join(DATA_DIR, "tokenizer.json"))


def read_and_decode(seq_idx):
    n = sizes[seq_idx]; off = ptrs[seq_idx]
    shard = 0
    while shard + 1 < len(cum) and cum[shard + 1] <= off:
        shard += 1
    local = off - cum[shard]
    with open(os.path.join(DATA_DIR, bin_files[shard]), "rb") as f:
        f.seek(local)
        data = f.read(n * 2)
    tokens = [int.from_bytes(data[i:i+2], "little") for i in range(0, len(data), 2)]
    return tok.decode(tokens, skip_special_tokens=False)


# Load hits
hits = [json.loads(l) for l in open(HITS_FILE)]
narrative_hits = [h for h in hits if h["narrative"] != "DOMAIN"]

organic = [h for h in narrative_hits if h["class"] == "Organic"]
cited = [h for h in narrative_hits if h["class"] == "Cited"]

print(f"Total narrative hits: {len(narrative_hits)}")
print(f"Organic: {len(organic)}, Cited: {len(cited)}")

# Sample 50 Organic + 50 Cited
n_sample = 50
organic_sample = random.sample(organic, min(n_sample, len(organic)))
cited_sample = random.sample(cited, min(n_sample, len(cited)))

samples = []
for h in organic_sample + cited_sample:
    text = read_and_decode(h["doc_id"])
    text_lower = text.lower()

    # Extract 500-char window around each keyword
    windows = []
    for kw in h["matched_keywords"]:
        pos = text_lower.find(kw.lower())
        if pos >= 0:
            start = max(0, pos - 250)
            end = min(len(text), pos + len(kw) + 250)
            windows.append(text[start:end].replace("\n", " "))

    # Also get attribution cues found
    attr = h.get("attribution_cues_found", [])

    samples.append({
        "doc_id": h["doc_id"],
        "narrative": h["narrative"],
        "scanner_class": h["class"],
        "matched_keywords": h["matched_keywords"],
        "attribution_cues_found": attr,
        "keyword_context": windows,
    })

# Save for LLM annotation
out_path = "/root/annotation_samples.json"
json.dump(samples, open(out_path, "w"), indent=2)
print(f"\nSaved {len(samples)} samples to {out_path}")
print(f"  {len(organic_sample)} Organic + {len(cited_sample)} Cited")
