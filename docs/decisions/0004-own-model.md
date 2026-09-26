# 0004. Train our own separation model on clean data

Date: 2026-09-25. Status: accepted by the owner, 2026-09-25; provider and dataset research in progress.

## Decision

The shipped dialogue/music/effects model is our own, trained on
mixtures built only from CC0 and CC BY sources, with sung vocals
deliberately kept in the music stem. Architecture: the TIGER family
(convolutions and attention, 1.4M parameters, runs on CoreML, WebGPU
and CPU; no recurrent layers that CoreML and WebGPU 1.28 lack). The
published TIGER-DnR and MRX checkpoints are used only to pass the
quality gate and as baselines.

## Why

- No permissive DME model was trained on clean data; the two
  permissive ones (MRX, TIGER-DnR) used DnR v1 with non-commercial
  clips.
- Owning the training data lets us fix the product's specific
  requirement: singing must count as music. Public DME datasets
  removed vocals from music (DnR v3) or did not control for it.
- Our own weights can be licensed as we choose.

## Cost and risk

Compute is modest for a 1.4M-parameter model (Bandit v1 at 37M took
about 1.5 days on 8 A10G GPUs); data building is the larger job. Risk:
the clean data pool is smaller (about 8.3k music tracks), which may
cost quality; measured against the published checkpoints on the same
test set.
