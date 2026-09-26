# Licensing, codecs and GPU paths: findings

2026-09-25. Primary sources: licence files, model cards, Zenodo
records, patent pool pages, Apple/Microsoft/NVIDIA licence texts,
ONNX Runtime source at the v1.28.0 tag, the local ort 2.0.0-rc.13 and
ort-sys sources. Not legal advice.

## Models

- Only two dialogue/music/effects models have explicitly permissive
  weights: **MRX** (MIT, REUSE-declared checkpoints, MERL) and
  **TIGER-DnR** (weights Apache-2.0 on Hugging Face, code MIT; the
  checkpoint is the 3-stem DnR model, 1.4M parameters).
- Both were trained on DnR v1 (TIGER: inferred from split sizes),
  whose music (FMA-medium) and effects (FSD50K) contain
  non-commercial clips: about 88% of FMA tracks are NC; 11-14% of
  FSD50K clips are CC BY-NC. Neither licence mentions it.
- Bandit v2 is CC BY-SA 4.0; Bandit v1 CC BY-NC; BandIt Plus and
  MVSEP-CDX23 unclear. No other permissive DME model was found.
- **A clean retrain is feasible**: DnR v3's generator is Apache-2.0
  and documents the filtering; clean pools are about 8.3k FMA tracks
  (CC BY and PD), about 43k FSD50K clips (CC0 and CC BY), LibriSpeech,
  MLS and Common Voice (CC0) for speech. TIGER is 1.4M parameters;
  Bandit v1 (37M) trained in about 1.5 days on 8 A10G GPUs.
- Creative Commons' 2025 guidance: CC licences apply only where
  copyright permission is required; whether trained weights are
  adapted material is not settled. Training on CC0 and CC BY sources
  with attribution in the model card is the conservative route.

## Audio codecs

- AAC patents: Via LA pool active in 2026, 16 licensors, $0.98 per
  unit for the first 500k units plus a $15,000 initial fee ($1,000 for
  small companies), no free threshold, per-unit for "end-user encoder
  and/or decoder products". Encoded content is free. No statement on
  apps that call an OS codec. Distributors (Fedora, Freedesktop SDK)
  ship LC-AAC openly on the view that LC patents expired; unverified.
- macOS: AudioToolbox/AVFoundation decode and encode AAC. Apple's
  macOS licence restricts only H.264/MPEG-4 Visual, not AAC.
- Windows: Media Foundation AAC decoder (LC, HE v1/v2, up to 6
  channels, 48 kHz) and encoder (LC, fixed bitrates). N editions need
  the Media Feature Pack.
- Linux: the Freedesktop SDK 25.08 runtime ships an LGPL FFmpeg with
  AAC, AC-3, Opus and FLAC; H.264/HEVC come from the auto-downloaded
  codecs-extra extension. A Flatpak app uses codecs the runtime
  publisher distributes.
- AC-3 patents expired 2008-2017 (Dolby's 2004 S-1); E-AC-3 2019-2020
  per the same filing, later patents unverified. Windows' Dolby
  decoder is locked to Microsoft-approved apps. Symphonia has no AC-3,
  E-AC-3 or Opus decoder.
- Opus (BSD + royalty-free grants) and FLAC (BSD, no known patents)
  are clean. ALAC reference code is Apache-2.0.

## Video preview decode

- Pools: AVC 100k units a year free then $0.20; HEVC now under Access
  Advance; AV1 royalty-free (AOMedia); VP9 royalty-free for Google's
  implementation.
- macOS: VideoToolbox hardware decode of H.264, HEVC, VP9 (after
  registering the supplemental decoder) and AV1 confirmed on the M4
  Pro test machine.
- Windows: Media Foundation H.264 (Baseline/Main/High) built in; HEVC
  needs the $0.99 Store extension or the OEM one; AV1 and VP9
  extensions free.
- Linux: dav1d (BSD-2) and libvpx (BSD-3) can be bundled; H.264 and
  HEVC must come from the system (GStreamer, runtime FFmpeg). Cisco
  OpenH264 covers only Constrained Baseline, not camera footage.
- Every path needs a placeholder when a codec is missing: preview is
  optional, export never decodes video.

## Containers

Pure-Rust: `shiguredo_mp4` (Apache-2.0, demux and mux, fragmented MP4,
AAC/Opus/FLAC/H.264/H.265/AV1; MOV untested), `matroska-demuxer` and
`mkv-element` for MKV. On macOS AVFoundation passthrough also works
for MP4/MOV.

## ONNX Runtime binaries

- pyke's prebuilt archives are patched builds with no licence or
  notice files; no Intel Mac build; x86_64 builds require AVX2.
- Building ORT ourselves is supported by ort rc.13 (`ORT_LIB_PATH`),
  allows a universal macOS build and an operator-reduced build, and
  lets us ship `ThirdPartyNotices.txt` for exactly what we build.
- ORT's notices contain no GPL or LGPL component; Eigen is MPL-2.0.

## GPU inference

| op | CPU | DirectML | WebGPU (1.28) | CoreML (1.28) |
|---|---|---|---|---|
| LSTM | yes | yes | yes | **no** |
| GRU | yes | yes | **no** | **no** |
| Einsum | yes | yes | yes | no |
| LayerNormalization | yes | yes | yes | yes |

- CoreML cannot run recurrent layers, so MRX (BLSTM) and Bandit v2
  (GRU) split into CPU pieces on macOS. TIGER-DnR (convolutions and
  attention) and HTDemucs are the CoreML-friendly shapes.
- DirectML covers all ops; its DLL is redistributable under
  Microsoft's terms (proprietary, Windows only) and is in
  maintenance mode; new work moved to Windows ML, which ships its own
  ORT (1.27.1 in the stable release) and downloads vendor execution
  providers on demand.
- WebGPU (Dawn, BSD-3) runs on Metal, D3D12 and Vulkan; pyke ships
  builds for macOS, Windows and Linux.
- CUDA/cuDNN/TensorRT are redistributable only under NVIDIA terms and
  weigh over 1 GB; ROCm EP was removed in ORT 1.23.
- Non-ORT fallback: burn (MIT/Apache) with wgpu imports LSTM and GRU
  and runs on Vulkan, Metal and DX12.

## App stores

- Mac App Store allows bundling models (200 GB limit) and allows
  downloading models the reviewed app already uses (Core ML
  documentation recommends it); Background Assets managed packs need
  macOS 26. The sandbox needs user-selected read-write and
  security-scoped bookmarks.
- Microsoft Store: data downloads allowed; non-game apps may use their
  own commerce; MSIX up to 25 GB; free signing for MSIX.
