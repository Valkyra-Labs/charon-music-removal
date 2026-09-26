# 13. Building the demo

2026-09-25. Measurements in `../measurements.md` and charon-audio's
`docs/MEASUREMENTS.md`.

## What was built

- **Model**: the music branch of TIGER-DnR exported to ONNX by a new
  script in charon-audio, with the STFT outside the graph. TIGER-DnR
  turned out to be three separate networks, one per output; removing
  music needs only one of them, a third of the work.
- **charon-audio**: a `Spectral` contract (host `torch.stft` and
  `torch.istft`, checked against PyTorch fixtures at 1e-5), mono models
  run per channel on any layout, a uniform window blend, CLI flags for
  overlap, blend and threads, and a regression test against PyTorch.
- **App**: AVFoundation for all media (decode, preview frames, AAC
  encode, remux with the video passed through), egui for the window,
  cpal for playback. Detection runs the model once over a mono downmix
  and thresholds the music stem's level; removal runs only inside the
  regions. The removal depth is applied in the audio callback from a
  stored music estimate, so changing it is instant.

## What went wrong, and what it taught

- **The reference was the slow part.** The PyTorch module needed ten
  minutes for 12 s of audio: 97% of the time was NNPACK convolving one
  group at a time. Disabling NNPACK made it 14 s. The export itself was
  never affected.
- **The legacy ONNX exporter could not express adaptive pooling** with
  uneven sizes; the dynamo exporter could, and its output matched
  PyTorch to 96-106 dB on real audio.
- **A float stride.** Charon computed the window stride in f32 and
  truncated it; for 2/3 overlap of 12 s windows that is 176399 samples
  instead of 176400. Against the authors' own pipeline the output
  agreed at only 25 dB. With the stride fixed it agreed at 84 dB. The
  model is sensitive to where a window starts, so a one-sample drift
  per window was audible in the numbers.
- **CoreML is not a path for this model.** It first refused the graph
  (a PReLU shape), then, with the PReLU written out, took 1034 s and
  28 GB for one minute of audio. The CPU does the same in 41 s.
- **Threads, batching and shorter windows did not speed it up**: time
  is linear in audio length and spread over Transpose, Conv, group
  normalization and Resize. The lever that worked was the pipeline:
  processing only music regions, one detection pass on mono, and 0.25
  window overlap for removal (same quality, 18% faster).
- **Memory**: ONNX Runtime's memory-pattern planning doubled peak
  memory (6.5 GB against 3.4 GB) at equal speed; it is off.
- **Test material**: the first real recording had the game's music
  switched off. It became a negative control (no false detection);
  the owner is recording clips with music.

## Results so far (synthetic set)

Music found in all eight clips with no false frames; median music
residual -16.7 dB; median speech improvement +8.7 dB SI-SDR; audio
outside the regions untouched bit for bit; video bit-identical after
export; about 1 x real time on the CPU; a 64 MB app, 22 MB DMG.

## Open

Real recordings with music; a quieter song under speech was only half
detected (recall 0.5 at -15 dB), so the detector threshold needs real
data; region editing (move and split edges) is not in the demo.
