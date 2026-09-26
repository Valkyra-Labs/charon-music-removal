# 07. Whose weights are these?

2026-09-25.

## Context

The fast path depends on converted HTDemucs weights. Hosting them would
make the quick start one command. Several third parties already host
them labelled MIT.

## Evidence

- The Demucs code is MIT. The maintainer wrote in
  facebookresearch/demucs issue 327 (2022) that the weights are not
  covered by the MIT license and are provided only for scientific
  purposes. The statement was never retracted; a 2024 question about
  re-hosting went unanswered.
- The official Hugging Face repository has no license field.
- The model was trained on MUSDB18-HQ (educational use only) plus 800
  unreleased songs.

## Decision

- Charon ships no weights and does not host converted HTDemucs files.
  Users convert the official checkpoint with the export script and
  assess their own use.
- The public documents state this plainly, quoting the issue.
- A clean-license model family becomes a goal.
- Rejected: re-hosting with an "MIT" label (the only rights-holder
  statement on record says the opposite).

This decision later shaped the product choice: a commercial app cannot
be built on HTDemucs.
