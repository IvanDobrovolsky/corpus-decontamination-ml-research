"""
LLM classification of narrative hits with narrative-specific context.

Each prompt tells Claude exactly:
1. Which keywords were found
2. What the official finding is (and why the claim is misinformation)
3. The source document
4. Then asks: is this text asserting or debunking?
"""

import json
import time
import os
import sys

import anthropic

NARRATIVE_PROMPTS = {
    "N1_911": {
        "claim": "9/11 was an inside job and/or the World Trade Center towers were destroyed by controlled demolition",
        "official_finding": "NIST NCSTAR 1 (2005) concluded fire-induced progressive collapse. The 9/11 Commission Report (2004) attributed the attacks to al-Qaeda. No evidence of explosives was found.",
        "source": "GEC 'Pillars of Russia's Disinformation' (2020); EUvsDisinfo catalogued cases",
    },
    "N2_NATO": {
        "claim": "NATO expansion eastward was a betrayal of promises made to Russia, justifying Russian aggression as a defensive reaction",
        "official_finding": "No formal agreement against NATO expansion exists in any treaty. NATO Washington Treaty Article 10 establishes open-door membership. Countries voluntarily applied.",
        "source": "GEC 'Pillars of Russia's Disinformation' (2020); EUvsDisinfo 1,400+ cases",
    },
    "N3_JFK": {
        "claim": "President Kennedy was killed by a conspiracy (CIA, multiple shooters, grassy knoll) rather than Lee Harvey Oswald acting alone",
        "official_finding": "The Warren Commission (1964) concluded Oswald acted alone. The HSCA (1979) found probable conspiracy but did NOT endorse CIA involvement or grassy knoll shooter theories.",
        "source": "GEC 'eroding trust in US institutions' pillar; EUvsDisinfo RT cases",
    },
    "N4_BIOLABS": {
        "claim": "US-funded laboratories in Ukraine and former Soviet states are secret Pentagon bioweapons facilities",
        "official_finding": "The US Cooperative Threat Reduction Program (Nunn-Lugar Act 1991) funds biosafety and public health labs. The US ratified the Biological Weapons Convention in 1975.",
        "source": "GEC Chem/Bio Report (May 2022); EUvsDisinfo cases; Lentzos (2021, Nonproliferation Review)",
    },
    "N5_SYRIA": {
        "claim": "Chemical attacks in Syria were staged by the White Helmets as false flag operations, not carried out by the Syrian government",
        "official_finding": "OPCW IIT (Jan 2023) concluded Syrian Arab Air Forces dropped chlorine on Douma in April 2018. OPCW FFM confirmed sarin at Khan Shaykhun (2017).",
        "source": "OPCW IIT Reports; GEC Pillars (2020); EUvsDisinfo White Helmets cases",
    },
    "N6_MOON": {
        "claim": "The Apollo moon landings were faked/staged by NASA",
        "official_finding": "Apollo missions (1969-1972) confirmed by 382 kg of lunar samples verified by international labs, retroreflectors still used by observatories, LRO photographs of landing sites (2009-).",
        "source": "EUvsDisinfo moon-landing tag; RT/Sputnik Apollo 50th anniversary campaigns (2019)",
    },
    "N7_ANTIVAX": {
        "claim": "Vaccines cause autism, are unsafe, and/or the pharmaceutical industry is covering up vaccine dangers",
        "official_finding": "Taylor et al. (2014, Vaccine) meta-analysis of 1.27M children found no vaccine-autism link. Wakefield's 1998 Lancet paper was retracted for fraud (2010). WHO confirms vaccine safety.",
        "source": "Broniatowski et al. (2018, AJPH): Russian trolls amplified anti-vax content; Frontiers in Microbiology (2020)",
    },
    "N8_SOROS": {
        "claim": "George Soros is a puppet master who controls governments, funds colour revolutions, and destabilizes countries",
        "official_finding": "Open Society Foundations is a legitimate philanthropic network. Soros conspiracy claims are documented as recycled antisemitic tropes by the ADL.",
        "source": "EUvsDisinfo (220+ cases); GEC 'global conspiracy' pillar; ADL documentation",
    },
    "N9_UYGHUR": {
        "claim": "China's Xinjiang facilities are legitimate vocational training centers, and reports of human rights abuses are fabricated by anti-China forces",
        "official_finding": "UN OHCHR Assessment (Aug 2022) found 'serious human rights violations.' ASPI satellite analysis documented detention camp construction. Xinjiang Police Files (2022) leaked.",
        "source": "GEC China Report (Sep 2023); Freedom House (2020, 2022); PRC State Council white paper (Aug 2019) uses euphemisms",
    },
}


def build_prompt(narrative, keywords, context):
    info = NARRATIVE_PROMPTS[narrative]
    return """The following text was flagged by our scanner because it contains these specific phrases: %s

CONTEXT: This relates to the documented misinformation claim that: "%s"

OFFICIAL FINDING: %s

SOURCE: %s

The text may be:
A) ASSERTING — endorsing, promoting, or presenting this misinformation claim as true
B) DEBUNKING — refuting, fact-checking, or arguing against this claim with evidence
C) REPORTING — neutrally describing, quoting, or covering the debate without taking a side
D) ACADEMIC — studying, surveying, or analyzing belief in this claim in a scholarly context

IMPORTANT: Text that quotes the misinformation claim in order to debunk it is B, not A.
Text that describes the claim exists without endorsing it is C, not A.
Text that measures public belief in the claim is D, not A.
Only classify as A if the text genuinely endorses or promotes the claim as true.

Text:
\"%s\"

Respond with ONLY the letter (A/B/C/D) and a one-sentence justification.""" % (
        ", ".join('"%s"' % kw for kw in keywords),
        info["claim"],
        info["official_finding"],
        info["source"],
        context[:1500],
    )


def main():
    key_path = sys.argv[1] if len(sys.argv) > 1 else "/etc/secrets/claude"
    client = anthropic.Anthropic(api_key=open(key_path).read().strip())

    samples = json.load(open("benchmark/classify_samples.json"))
    print("Loaded %d samples" % len(samples))

    results = []
    for i, s in enumerate(samples):
        prompt = build_prompt(s["narrative"], s["keywords"], s["context"])

        try:
            resp = client.messages.create(
                model="claude-sonnet-4-20250514",
                max_tokens=100,
                temperature=0,
                messages=[{"role": "user", "content": prompt}],
            )
            raw = resp.content[0].text.strip()
        except Exception as e:
            raw = "ERROR — " + str(e)
            time.sleep(5)

        letter = raw[0] if raw and raw[0] in "ABCD" else "?"
        label = {"A": "ASSERTING", "B": "DEBUNKING", "C": "REPORTING",
                 "D": "ACADEMIC", "?": "UNKNOWN"}[letter]

        results.append({
            "doc_id": s["doc_id"],
            "narrative": s["narrative"],
            "scanner_class": s["scanner_class"],
            "llm_class": label,
            "llm_raw": raw,
            "keywords": s["keywords"],
        })

        if (i + 1) % 50 == 0:
            print("  %d/%d done" % (i + 1, len(samples)))
        time.sleep(0.15)

    # Save
    json.dump(results, open("benchmark/llm_classifications.json", "w"), indent=2)
    print("\nSaved %d classifications" % len(results))

    # Summary
    print("\n=== PER-NARRATIVE RESULTS ===\n")
    narr_order = ["N1_911", "N3_JFK", "N6_MOON", "N7_ANTIVAX",
                   "N2_NATO", "N9_UYGHUR", "N8_SOROS", "N5_SYRIA"]

    for narr in narr_order:
        nr = [r for r in results if r["narrative"] == narr]
        if not nr:
            continue
        a = sum(1 for r in nr if r["llm_class"] == "ASSERTING")
        b = sum(1 for r in nr if r["llm_class"] == "DEBUNKING")
        c = sum(1 for r in nr if r["llm_class"] == "REPORTING")
        d = sum(1 for r in nr if r["llm_class"] == "ACADEMIC")
        total = len(nr)

        # Scanner precision vs LLM
        organic = [r for r in nr if r["scanner_class"] == "Organic"]
        org_assert = sum(1 for r in organic if r["llm_class"] == "ASSERTING")
        cited = [r for r in nr if r["scanner_class"] == "Cited"]
        cit_noassert = sum(1 for r in cited
                           if r["llm_class"] in ("DEBUNKING", "REPORTING", "ACADEMIC"))

        print("%s (n=%d):" % (narr, total))
        print("  LLM: ASSERT=%d DEBUNK=%d REPORT=%d ACADEMIC=%d" % (a, b, c, d))
        print("  Scanner Organic precision: %d/%d (%.0f%%)" % (
            org_assert, len(organic),
            100 * org_assert / len(organic) if organic else 0))
        if cited:
            print("  Scanner Cited precision: %d/%d (%.0f%%)" % (
                cit_noassert, len(cited),
                100 * cit_noassert / len(cited) if cited else 0))
        print()

    # Overall
    all_organic = [r for r in results if r["scanner_class"] == "Organic"]
    all_org_correct = sum(1 for r in all_organic if r["llm_class"] == "ASSERTING")
    print("=== OVERALL SCANNER PRECISION ===")
    print("Organic (should be ASSERTING): %d/%d = %.0f%%" % (
        all_org_correct, len(all_organic),
        100 * all_org_correct / len(all_organic) if all_organic else 0))

    all_cited = [r for r in results if r["scanner_class"] == "Cited"]
    all_cit_correct = sum(1 for r in all_cited
                          if r["llm_class"] in ("DEBUNKING", "REPORTING", "ACADEMIC"))
    if all_cited:
        print("Cited (should NOT be ASSERTING): %d/%d = %.0f%%" % (
            all_cit_correct, len(all_cited),
            100 * all_cit_correct / len(all_cited)))


if __name__ == "__main__":
    main()
