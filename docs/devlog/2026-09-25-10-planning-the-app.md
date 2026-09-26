# 10. Planning the app

2026-09-25.

## Context

The owner set the constraints: library work for the app goes into
charon-audio 0.1.2; every yellow or red licence item is resolved,
replaced, or built by us; cross-platform with a green GPU path from
the start; a video preview above the timeline; design against the
competitors' main complaints; decide the UI stack, the architecture,
the licence, publishing and monetization; document everything.

## What was done

Four research directions, fourteen passes in total: licence
replacements (models and datasets, audio codecs, video decode, app
stores and the ONNX Runtime supply chain, GPU runtimes), UI stacks
(Rust toolkits, webview/native/Qt/Flutter, the owner's egui app),
competitor complaints (Erase song and Content ID, editors and pro
tools, speech enhancers, stem separators, segments and prices), and
distribution and monetization. Summaries in `docs/research/`.

## What changed our minds

- **Windows moved from phase 3 to launch.** Steam's survey puts
  Windows at 94% of gamers; the streamer segment is Windows.
- **The model has to be ours.** The only two permissive
  dialogue/music/effects models were trained on data with
  non-commercial clips. Retraining on CC0/CC BY sources removes the
  taint and lets us make singing count as music, which no public
  dataset controls for.
- **Recurrent models are a GPU dead end on this stack.** CoreML has
  no LSTM or GRU; WebGPU in ORT 1.28 has no GRU. The TIGER family
  (convolutions and attention) fits every GPU path.
- **Nobody returns the video.** Every competitor gives audio back and
  leaves re-muxing to the user; two break on long files or 5.1. That
  is the product's centre, not a feature.
- **Complaints are about damage and billing.** Robotic voice,
  invented words, removed laughter; credits, auto-renewal, refusals.
  Hence: process only where music is, no generative models, one-time
  price, no account.
- **The preview does not need 4K.** Decoding to preview size and
  uploading NV12 planes is cheap; zero-copy is a later optimisation.

## Decisions

Records 0003-0007: egui on wgpu; our own TIGER-family model; CoreML
and WebGPU with ONNX Runtime built by us; FSL for the app and
MIT/Apache for the library, one-time pricing, offline licences; an
engine child process. All proposed, awaiting the owner. The licence
resolution table is `docs/licence-resolution.md`; the architecture is
`docs/architecture.md`; requirements are `docs/requirements.md`.

## Open

- Measurements that can overturn decisions: WebGPU speed for our
  model on each OS; quality of the clean-trained model against the
  published checkpoints; the fingerprint curve.
