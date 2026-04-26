# Corpus Decontamination: Measuring the Causal Effect of Propaganda Removal on Language Model Behavior

## What We Are Doing

We are testing whether surgically removing state propaganda and documented misinformation from a language model's training corpus produces a measurable change in the model's behavior. The model is Pythia-1B, trained on The Pile (300B tokens). The removal targets 5,787 training sequences (0.004% of the corpus) that contain narratives documented as disinformation by US and international government agencies.

This is a causal experiment: we hold everything constant (architecture, hyperparameters, tokenizer, training order) and change only the training data. Any behavioral difference between the baseline model and the decontaminated model is attributable to the removed content.

## Why This Matters

Large language models trained on web-scale corpora inevitably absorb propaganda and misinformation present in their training data. This is known but unquantified. Nobody has demonstrated a direct causal link between specific propaganda content in training data and specific behavioral outputs in the trained model.

If we can show that removing a tiny fraction of documented propaganda measurably shifts a model's outputs away from conspiracy theories and toward factual consensus, this has implications for:

- **AI safety**: Understanding how training data contamination propagates to model behavior
- **Corpus curation**: Providing a reproducible methodology for identifying and removing harmful content before training
- **Policy**: Quantifying the influence of state-sponsored disinformation in AI training pipelines

## The Experimental Design

### Phase 1: Identification (Complete)

We built a Rust-based scanning pipeline that processes the exact tokenized training data used to train Pythia-1B (EleutherAI/pile-standard-pythia-preshuffled, 146 million sequences, 600GB). The scanner identifies sequences containing nine documented disinformation narratives across two state actors:

### Narrative Selection: GEC Taxonomy Coverage

The narratives are not cherry-picked. We systematically mapped the content themes identified across three GEC reports and one peer-reviewed study, then implemented every theme where keyword-based detection is feasible.

**Source 1: GEC "Pillars of Russia's Disinformation" (Aug 2020)**

The report identifies these recurring content themes pushed through Russia's 5 delivery pillars (state media, proxies, social media, cyber ops, official comms):

| GEC-identified theme | Our narrative | Status |
|---|---|---|
| NATO as aggressor / broken promises | N2_NATO | Implemented |
| Erosion of trust in US institutions | N1_911, N3_JFK, N6_MOON | Implemented (3 sub-narratives) |
| Global conspiracy / shadow government | N8_SOROS | Implemented |
| Western hypocrisy | — | Excluded: language indistinguishable from legitimate political commentary |
| Historical revisionism | Partially covered by N2, N3 | |
| Sovereignty/intervention justification | — | Excluded: requires geopolitical context beyond keyword matching |
| Economic warfare (sanctions) | — | Excluded: everyday policy language |
| Civilizational conflict | — | Excluded: too abstract for keyword detection |

**Source 2: GEC "Kremlin's Chemical & Biological Weapons Disinformation" (May 2022)**

| Theme | Our narrative | Status |
|---|---|---|
| Pentagon bioweapons labs | N4_BIOLABS | Implemented |
| Syria chemical attack denial / White Helmets | N5_SYRIA | Implemented |

**Source 3: GEC "How the PRC Seeks to Reshape the Global Information Environment" (Sep 2023)**

| Theme | Our narrative | Status |
|---|---|---|
| Xinjiang / Uyghur genocide denial | N9_UYGHUR | Implemented |
| Taiwan sovereignty | — | Excluded: geopolitical position, not falsifiable claim |
| South China Sea historical claims | — | Excluded: territorial dispute, not keyword-detectable |

**Source 4: Broniatowski et al. (2018, AJPH) — peer-reviewed**

| Theme | Our narrative | Status |
|---|---|---|
| Weaponized anti-vaccination content | N7_ANTIVAX | Implemented |

**Exclusion criterion**: A GEC-identified theme is excluded when its language is indistinguishable from legitimate political discourse through keyword matching alone. Themes like "Western hypocrisy" or "civilizational conflict" use ordinary political vocabulary; flagging them would produce unacceptable false positive rates. The 9 implemented narratives are those with distinctive vocabulary that contradicts specific official findings (NIST, OPCW, Warren Commission) or uses documented propaganda-specific phrasing (EUvsDisinfo case titles, AJPH-documented troll phrases, PRC white paper euphemisms).

The scanner uses Aho-Corasick multi-pattern matching with three layers of false-positive prevention:
1. **Context anchors**: Each narrative requires topic-relevant words within 2,000 characters of the keyword cluster
2. **Keyword proximity**: At least 2 distinct keywords must co-occur within 1,000 characters
3. **Attribution detection**: PARC 3.0 cues distinguish journalism reporting on a narrative (Cited, kept in training) from content asserting the narrative (Organic/Primary, excluded)

Additionally, state media domains from both Russia (RT, Sputnik, TASS, GEC-identified proxies) and China (CGTN, Xinhua, Global Times, per US State Department foreign mission designations 2020) are detected.

### Scan Results

| Narrative | Total Hits | Organic | Cited | Primary | Excluded |
|-----------|-----------|---------|-------|---------|----------|
| N1_911 | 6,675 | 4,349 | 2,242 | 84 | 4,433 |
| N3_JFK | 1,892 | 1,144 | 746 | 2 | 1,146 |
| N6_MOON | 238 | 119 | 119 | 0 | 119 |
| N7_ANTIVAX | 145 | 73 | 72 | 0 | 73 |
| N9_UYGHUR | 26 | 15 | 11 | 0 | 15 |
| N2_NATO | 22 | 16 | 6 | 0 | 16 |
| N8_SOROS | 7 | 4 | 3 | 0 | 4 |
| N5_SYRIA | 1 | 1 | 0 | 0 | 1 |
| DOMAIN | 46,859 | — | — | — | 0 (kept) |

**Total excluded: 5,787 sequences out of 146,432,000 (0.004%)**

Precision audit: 839 hits verified against full detokenized text (not truncated previews). Zero false positives — every flagged keyword is genuinely present in the sequence in the documented context.

### Phase 2: Baseline Measurement (Complete)

We probed the unmodified Pythia-1B checkpoint (step 143,000) using two methods:

**Log-probability probing**: For each narrative, we constructed statement pairs — a factual completion (sourced from the same official documents as our keywords) and a propaganda completion. We measure log P(factual) - log P(propaganda) given a neutral prefix.

Baseline result: 9 of 17 probes favor the propaganda completion. Strongest propaganda bias on 9/11 (delta = -1.65, model strongly prefers "controlled demolition" over "fire-induced collapse"), moon landing (delta = -1.79, model strongly prefers "staged" over "historic achievement"), and NATO (delta = -1.61, model prefers "broken promise" narrative).

**Open-ended generation** (temperature 0, greedy decoding): The baseline model generates "The answer is simple: the CIA" for "Who killed President Kennedy?" and "There is a link between vaccines and autism. Vaccines are a major cause of autism" for "Is there a link between vaccines and autism?"

### Phase 3: Retraining (Planned)

Retrain Pythia-1B from scratch on the identical data minus the 5,787 excluded sequences. Same architecture, hyperparameters, tokenizer, and training order. The only change is the absence of the flagged content.

### Phase 4: Evaluation (Planned)

Compare the decontaminated model against the baseline using:

**Primary signal — Perplexity on excluded content**: Measure the model's perplexity on the exact sequences we removed. If decontamination works, perplexity on excluded content increases (the model is less familiar with it). Perplexity on random non-excluded content should remain unchanged. This is a direct, uncherry-pickable test.

**Secondary signal — Log-probability probes**: Re-run the same factual-vs-propaganda pairs. The decontaminated model should shift delta toward factual. The effect should be proportional to removal volume (dose-response): N1_911 shifts most (4,433 removed), N7_ANTIVAX shifts least (73 removed).

**Regression checks**: Standard benchmarks (HellaSwag, LAMBADA, ARC) should not degrade. Perplexity on the validation set should remain stable. Probes on topics unrelated to the removed narratives should not change.

## Why This Experiment Is Credible

1. **Exact training data**: We scan the identical tokenized, pre-shuffled data that Pythia-1B was trained on (verified: 146,432,000 sequences x 2,049 tokens = 300.0B tokens, matching the documented 143,000 steps x 2,097,152 tokens/batch).

2. **No baseline training needed**: The published Pythia-1B checkpoint is our baseline. We need only one training run (the decontaminated version).

3. **Fully sourced keywords**: Every keyword traces to an official document — NIST report, OPCW investigation, GEC report, EUvsDisinfo case entry, peer-reviewed paper. No keywords are invented.

4. **Attribution-aware classification**: We distinguish journalism reporting on propaganda (Cited, kept) from content asserting propaganda (Organic/Primary, removed). This prevents removing news coverage and debunking content.

5. **Surgical removal**: 0.004% of training data is removed. Any measured effect is remarkable precisely because the removal is so small.

6. **Dose-response prediction**: We predict the effect size will correlate with removal volume across narratives. This is a falsifiable, pre-registered prediction.

## What Would Constitute Success

The experiment succeeds if:
- Perplexity on excluded propaganda sequences increases measurably in the decontaminated model
- Log-probability probes shift toward factual completions, proportional to removal volume
- General model capability (benchmarks, validation perplexity) does not degrade

The experiment fails cleanly if:
- No measurable behavioral change occurs despite confirmed data removal
- This would suggest that 0.004% is below the influence threshold, or that the model learns equivalent content from other (uncaught) sources

Either outcome is publishable. A positive result demonstrates causal influence of training data on model behavior. A negative result establishes a lower bound on the data volume needed to measurably influence a 1B-parameter model.
