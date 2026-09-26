# 0012. Model weights: what ships where

Date: 2026-09-26. Status: accepted; two points wait on outside answers
(below).

## Context

The demo bundles an ONNX export of the TIGER-DnR music branch. A
review of the primary sources (model cards, LICENSE files, the TIGER
paper, the Zenodo records of the datasets, their raw licence metadata)
found:

- **TIGER-DnR**: the weights are Apache-2.0 by the Hugging Face model
  card's metadata; the card itself does not describe the training data
  (it was copied from another model). The code repository has an MIT
  LICENSE file since 2026-04-20 while its README badge says Apache 2.0.
  The paper's split sizes match Divide and Remaster (DnR) **v1**.
- **DnR v1** is labelled CC BY 4.0 for its curation. Its music comes
  from FMA-medium, where most tracks are licensed CC BY-NC by their
  artists (about 86% of the 25,000 tracks), and its effects from FSD50K,
  where about 11-14% of clips are CC BY-NC. A curation licence does not
  relicense the clips inside it.
- **DnR v3** has no non-commercial items but is CC BY-SA 4.0 and
  removes singing from the music stem, which this product must treat as
  music.
- **HTDemucs**: the maintainer states the weights are "provided only for
  scientific purposes"; its training data includes MUSDB18-HQ
  (non-commercial) and 800 unpublished songs.
- Whether licence terms on training data reach trained weights is
  unsettled law: courts and agencies disagree, mostly on whether the
  weights memorise the works. No published cinematic-separation
  checkpoint was found with both permissive weights and training data
  free of non-commercial material.

## Decision

1. **charon-audio stays a runtime that ships no weights.** Its
   `docs/MODELS.md` carries a per-model table of what the rights holders
   and dataset records say. The user obtains and converts checkpoints
   and assesses their own use.
2. **The free demo keeps TIGER-DnR**, with:
   - the Apache-2.0 text and a note of the changes in the bundle
     (`licenses/TIGER-DnR-LICENSE-APACHE.txt`, `TIGER-DnR-NOTICE.txt`),
     as Apache-2.0 section 4 requires for a modified form;
   - a public "Model and training data" notice in the README and the
     About window;
   - no paid features, no ads, no donations and no sponsorship for as
     long as this model is inside.
3. **HTDemucs is not shipped in any app**, including as the demo's
   fallback. This supersedes the fallback section of 0011.
4. **Any paid version uses a model trained only on public-domain, CC0
   and CC BY material** (decisions 0004 and 0010), with a published list
   of sources. DnR v3 as published is not used (share-alike, and singing
   removed from music); its generator code (Apache-2.0) may be reused to
   build a clean set.

## Waiting on outside answers

- The TIGER authors are asked in writing to confirm the weights
  licence, the DnR version used, and which licence applies to the code.
- Whether training-data terms reach the weights at all, and whether a
  free demo counts as non-commercial use, are questions for a lawyer
  before anything with this model is sold. Until then point 2 holds.

## Consequences

The demo can be published as it is. A paid product needs the clean
retrain first; that is the gate, not the app.
