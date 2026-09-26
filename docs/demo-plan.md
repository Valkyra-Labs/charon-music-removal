# Demo plan: free, local, macOS

2026-09-25. Decision 0011. Cost: zero (no training, no rented
compute, no server, no paid accounts). Estimated effort: about two
weeks of focused work.

## Status (2026-09-25, end of day)

Steps 1-8 are built. Open: real test recordings (the owner is recording
them), a GitHub release (nothing is pushed yet), region editing.

## Status (2026-09-26)

Three real gameplay recordings from the owner are measured (devlog 14):
preview playback fixed (sequential decode, 60 fps), analysis 1.6-1.7x
faster with the same regions (two model sessions), detector
recalibrated (medians over 5.5 s;
the quiet score under dialogue is found). Open: the owner's listening
check of the new regions, a GitHub release, region editing.

## Steps

| # | step | where | days |
|---|---|---|---|
| 1 | Export TIGER-DnR to ONNX (STFT outside the graph, as the LiteRT export did), parity against PyTorch; measure CPU and CoreML speed | charon-audio `tools/export/` | 1-2 |
| 2 | `Spectral` contract, mono-per-channel, model sample rate; tests; release 0.1.2 (also fixes the README on crates.io) | charon-audio | 2 |
| 3 | Small record: 10-20 synthetic mixes from CC-licensed speech, effects and music; music-stem SDR, Music Residual Ratio, speed | charon-audio `tools/parity/` | 1 |
| 4 | App skeleton: egui window, open a file (rfd, drag and drop), AVFoundation probe and audio decode to a PCM file, waveform peaks | app | 2 |
| 5 | Processing: music regions from the music stem's loudness, `remove_in_regions` on a worker thread with progress and cancel | app | 1 |
| 6 | Preview and playback: AVFoundation frames at preview size into an egui texture; cpal playback with A/B/C switch; timeline with regions and depth per region | app | 3 |
| 7 | Export: new AAC track through AVFoundation, video passed through, same container | app | 1-2 |
| 8 | README with screenshots, About window with licences, GitHub Release (ad-hoc signed DMG), devlog entry with the measured results | app | 1 |

## Out of the demo

Windows and Linux, our own model, voice clean-up, AC-3, batch queue,
localization, trial and licence, any network use.

## Success criteria

- A 10-minute vlog or gameplay clip with music is processed on an M-series
  Mac, previewed and exported to the same container without re-encoding
  the video.
- Outside the regions the exported audio equals the source audio
  decoded by AVFoundation, sample for sample (before the AAC encode).
- The measured record exists and is linked from the README.
