# Devlog: Charon and Charon Music Removal

A running record of how two products are built: **charon-audio**, a Rust
library that runs audio source-separation models locally, and **Charon
Music Removal**, a desktop app built on it that removes music from video
while keeping speech and sound effects.

The devlog is written for a reader who was not there: a reviewer, a
future contributor, anyone evaluating the work. Every entry
stands on its own.

## Conventions

- One file per entry: `YYYY-MM-DD-NN-slug.md`, in order of events.
- Each entry has the same sections:
  - **Context**: where things stood and why this work started.
  - **What was done**: the work, briefly.
  - **Evidence**: numbers with their stamp (build hash, model hash,
    host, method). A number without a stamp is not reported.
  - **What went wrong**: failures, dead ends, incidents. Kept, not
    edited out.
  - **Decision**: what was decided and what was rejected, with reasons.
  - **Next**.
- External claims (vendor benchmarks, model cards, papers) are quoted as
  hypotheses until reproduced on our own load shape.
- Architecture and product decisions with lasting effect also get a
  decision record in `docs/decisions/` (short, numbered, never
  rewritten; superseded by a new record).
- Written in English; no emojis.

## Entries

| # | date | title |
|---|---|---|
| 01 | 2026-09-24 | [Audit: the library did not separate anything](2026-09-24-01-audit.md) |
| 02 | 2026-09-24 | [Real inference and parity with PyTorch](2026-09-24-02-real-inference-and-parity.md) |
| 03 | 2026-09-24 | [Plumbing that was quietly wrong](2026-09-24-03-plumbing.md) |
| 04 | 2026-09-25 | [The GPU path: own export, STFT in Rust, one CoreML partition](2026-09-25-04-gpu-path.md) |
| 05 | 2026-09-25 | [Resident server and the head-to-head](2026-09-25-05-resident-and-head-to-head.md) |
| 06 | 2026-09-25 | [Shipping 0.1.1](2026-09-25-06-shipping-0.1.1.md) |
| 07 | 2026-09-25 | [Whose weights are these?](2026-09-25-07-weights-license.md) |
| 08 | 2026-09-25 | [Mapping the field](2026-09-25-08-mapping-the-field.md) |
| 09 | 2026-09-25 | [From stems to a product: choosing one job](2026-09-25-09-choosing-the-product.md) |
| 10 | 2026-09-25 | [Planning the app](2026-09-25-10-planning-the-app.md) |
| 11 | 2026-09-25 | [Decisions taken, and the first library work](2026-09-25-11-decisions-and-first-library-work.md) |
| 12 | 2026-09-25 | [Cutting to a demo](2026-09-25-12-cutting-to-a-demo.md) |
| 13 | 2026-09-25 | [Building the demo](2026-09-25-13-building-the-demo.md) |
| 14 | 2026-09-26 | [First real recordings: preview, speed, a quiet score](2026-09-26-14-first-real-recordings.md) |

## Other documents

- `../decisions/`: decision records.
- `../research/`: research summaries with sources.
- `../demo-plan.md`: the current deliverable; `../measurements.md`: its numbers.
- `../architecture.md`, `../requirements.md`, `../licence-resolution.md`: the full product (postponed parts marked in the decision records).
- `../design/ui-prototype.html`: clickable UI mock with fake data.
