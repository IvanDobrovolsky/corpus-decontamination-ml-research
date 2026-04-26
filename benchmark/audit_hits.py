"""Full-text audit of all small-narrative hits."""
import json, struct, os, sys, array
from collections import defaultdict
from tokenizers import Tokenizer

DATA_DIR = "/mnt/HC_Volume_105510485/pythia-data/"
HITS_FILE = os.path.join(DATA_DIR, "pile_hits_v5.jsonl")

# Load tokenizer
tok = Tokenizer.from_file(os.path.join(DATA_DIR, "tokenizer.json"))

# Load idx header + arrays
with open(os.path.join(DATA_DIR, "document.idx"), "rb") as f:
    f.read(9)  # magic
    f.read(8)  # version
    f.read(1)  # dtype
    num_seq = struct.unpack("<Q", f.read(8))[0]
    f.read(8)  # num_doc
    sizes = array.array("i")
    sizes.frombytes(f.read(num_seq * 4))
    ptrs = array.array("q")
    ptrs.frombytes(f.read(num_seq * 8))

# Bin shard cumulative offsets
bin_files = sorted(f for f in os.listdir(DATA_DIR) if f.endswith(".bin"))
cum = [0]
for bf in bin_files:
    cum.append(cum[-1] + os.path.getsize(os.path.join(DATA_DIR, bf)))

def read_tokens(seq_idx):
    n = sizes[seq_idx]
    off = ptrs[seq_idx]
    shard = 0
    while shard + 1 < len(cum) and cum[shard + 1] <= off:
        shard += 1
    local = off - cum[shard]
    with open(os.path.join(DATA_DIR, bin_files[shard]), "rb") as f:
        f.seek(local)
        data = f.read(n * 2)
    return [int.from_bytes(data[i:i+2], "little") for i in range(0, len(data), 2)]

# Load hits
hits = [json.loads(l) for l in open(HITS_FILE)]
narrative_hits = [h for h in hits if h["narrative"] not in ("DOMAIN", "N1_911", "N3_JFK")]
by_doc = defaultdict(list)
for h in narrative_hits:
    by_doc[h["doc_id"]].append(h)

print(f"Auditing {len(narrative_hits)} hits across {len(by_doc)} sequences")
print()

not_found_total = 0
fp_suspects = []

for doc_id in sorted(by_doc.keys()):
    tokens = read_tokens(doc_id)
    text = tok.decode(tokens, skip_special_tokens=False)
    text_lower = text.lower()

    for h in by_doc[doc_id]:
        narr = h["narrative"]
        cls = h["class"]
        kws = h["matched_keywords"]

        print(f"--- doc={doc_id} {narr} {cls} len={len(text)} ---")

        all_found = True
        for kw in kws:
            pos = text_lower.find(kw.lower())
            if pos >= 0:
                start = max(0, pos - 150)
                end = min(len(text), pos + len(kw) + 150)
                snippet = text[start:end].replace("\n", " ").replace("\r", "")
                marker = ">>>" + kw.upper() + "<<<"
                # highlight keyword in snippet
                kw_start = pos - start
                kw_end = kw_start + len(kw)
                highlighted = snippet[:kw_start] + marker + snippet[kw_end:]
                print(f"  FOUND \"{kw}\" at char {pos}:")
                print(f"    {highlighted}")
            else:
                print(f"  NOT FOUND: \"{kw}\"")
                not_found_total += 1
                all_found = False

        if not all_found:
            fp_suspects.append(doc_id)
        print()

print(f"=== SUMMARY ===")
print(f"Total hits audited: {len(narrative_hits)}")
print(f"Keywords NOT FOUND in full text: {not_found_total}")
print(f"Sequences with missing keywords: {len(fp_suspects)}")
if fp_suspects:
    print(f"Suspect doc_ids: {fp_suspects}")
