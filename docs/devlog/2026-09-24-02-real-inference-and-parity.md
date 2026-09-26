# 02. Real inference and parity with PyTorch

2026-09-24. Reference: demucs 4.1.0, torch 2.14.0. Charon: ort
`=2.0.0-rc.13` (ONNX Runtime 1.28.0), release build. Host as in 01.

## Context

The pipeline had to run a real model and produce the same stems as the
reference implementation before any other claim could be made.

## What was done

- HTDemucs as an ONNX graph with the STFT inside (a third-party export,
  hash-pinned), wired into a pipeline rewritten to follow Demucs's
  `apply_model`: mono-reference normalization, 7.8 s segments with 25%
  overlap, triangular overlap-add weights, context-padded last segment,
  deterministic time shifts.
- A parity harness: the same input to Charon and to
  `demucs.api.Separator("htdemucs", shifts=0, overlap=0.25)`, max
  absolute sample difference and agreement SDR per stem.
- A quality check on the 50 test tracks of the MUSDB18 7-second
  previews, whole-signal SDR, median over tracks.
- Pipeline tests with tiny identity ONNX models, so CI checks the whole
  path without the 300 MB model.

## Evidence

| input | max diff (worst stem) |
|---|---|
| 3 synthetic clips (5-31 s) | 1.2e-4 |
| 3 real tracks (110-193 s) | 9.3e-4 |

MUSDB previews, median SDR drums / bass / other / vocals: Charon 9.50 /
9.04 / 5.19 / 8.88, PyTorch 9.50 / 9.04 / 5.19 / 8.88.

Acceptance criterion set before the run: max difference at most 1e-3.

## What went wrong

A profiling sweep was invalidated by a shell bug: an unquoted variable
in zsh does not word-split, so multi-key configuration cells ran with
one key. The cells were re-run with `${=cell}`. The disk filled during
a build of the reference environment; 7 GB of scratch were cleaned.

## Decision

Agreement SDR is reported as agreement, not quality; quality is only
claimed where a reference stem exists (MUSDB). The MUSDB previews are
7 s AAC excerpts, so the numbers are not comparable with published
museval tables, and the documents say so.

## Next

The plumbing around the model: decode, resample, write.
