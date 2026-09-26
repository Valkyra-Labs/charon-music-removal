# 12. Cutting to a demo

2026-09-25.

## Context

After a day of planning, the plan had grown to our own model, three
operating systems, a payment route and a server for trials. The owner
pointed out that the immediate need is something that works and can
be shown, and that the plan now cost money (rented compute for
training) and weeks before anything could be shown.

## Decision

A free macOS demo (record 0011): the published TIGER-DnR checkpoint
exported to ONNX, AVFoundation for all media, egui for the UI, no
network at all. Everything else is postponed, not cancelled; the
research stays valid for the product if it continues.

## Why this is the right cut

- Every postponed item was a cost without a demo to show for it.
- The engineering story is intact: measured parity, streaming with
  bounded memory, bit-exact behaviour outside regions, a native media
  path, a clean licence audit.
- The product story is intact and sharper: evidence for the pain, a
  scoped decision, and an explicit list of what was deliberately left
  out and why.

## The server question

The earlier design used a server only for update checks, beta and
trial tokens and optional usage counts; no user media was ever meant to
leave the computer. The owner's rule now is that the app works with no
external server, so all three went: the demo makes no network
connections, and a later paid version would use a local trial counter
and a signed licence file (record 0009, revised).

## Library consequence

charon-audio 0.1.2 is cut to what the demo needs; the work already done
(streaming, regions, progress and cancel, optional decoders, dual
licence) stays, and the TIGER contract and exporter are the remaining
items.
