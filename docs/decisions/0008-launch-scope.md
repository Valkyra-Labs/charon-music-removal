# 0008. Launch scope

Date: 2026-09-25. Status: accepted by the owner.

## Decision

- **Windows ships in the first release** together with macOS (Steam
  survey: 94% of gamers on Windows). Linux follows as a Flatpak.
- **Voice clean-up is in the first release** (DPDFNet, 48 kHz,
  Apache-2.0), with a strength control and off by default.
- **AC-3 input: our own decoder** (AC-3 patents expired 2008-2017), in
  Rust, parity-tested against a reference decoder; E-AC-3 after its
  patent status is confirmed.
- **Localization order**: English; then Spanish, Portuguese, Hindi,
  Indonesian, Japanese, German, Russian.
- **All library work the app needs goes into charon-audio 0.1.2**,
  including the dual MIT/Apache-2.0 licence. The published 0.1.1 README
  carries a wrong build command that only a new release can fix on
  crates.io, so 0.1.2 is sequenced to ship as early as its acceptance
  gate allows.
