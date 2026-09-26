# Licence resolution: every yellow and red item

2026-09-25. Rule (decision 0002): every shipped component is green,
or it is replaced, or we build it ourselves. Not legal advice; two
items are marked for counsel before the paid release.

| item | was | resolution | now | effort |
|---|---|---|---|---|
| HTDemucs weights (scientific use only) | red | not in the product; stays a research path in the library | removed | none |
| Bandit v2 weights (CC BY-SA 4.0) | yellow | not shipped; used only as an evaluation reference | removed | none |
| Bandit v1, BandIt Plus, MVSEP-CDX23, umxl, Kim Mel-Band, umx/umxhq | red/yellow | not used | removed | none |
| MRX / TIGER-DnR training-data taint (NC clips in DnR v1) | yellow | **retrain our own TIGER-family model** on mixtures built from CC0 and CC BY sources only (FSD50K CC0+CC BY clips, FMA CC BY/PD tracks, Incompetech CC BY, LibriSpeech, MLS, Common Voice CC0, VCTK); attribution in the model card; singing kept inside the music stem on purpose. The published checkpoints are used only for the quality gate. | green (own weights, our licence) | L |
| DeepFilterNet weights (no licence) | red | DPDFNet (Apache-2.0 code and weights, 48 kHz ONNX) | green | S |
| Symphonia AAC decoder (AAC pool) | yellow | decode AAC through the OS: AudioToolbox (macOS), Media Foundation (Windows), the Flatpak runtime's FFmpeg (Linux); charon-audio gets a feature switch to build without its AAC decoder | green | S |
| AAC encoder for export | yellow | OS encoders on macOS and Windows; on Linux the runtime's FFmpeg, or FLAC/Opus in MKV | green | S |
| AC-3 / E-AC-3 input | red on Windows (Dolby decoder locked) | macOS: system decoder (to verify); Windows and Linux: runtime FFmpeg on Linux, and on Windows an own AC-3 decoder (AC-3 patents expired 2008-2017) or "unsupported, convert first" in the MVP | green or not supported | M (own decoder) |
| Video preview decode (H.264/HEVC pools) | yellow | OS decoders only (VideoToolbox, Media Foundation); Linux: dav1d and libvpx bundled for AV1/VP9, H.264/HEVC from the runtime codecs extension; placeholder when missing, export unaffected | green | M |
| FFmpeg | yellow (LGPL) / red (GPL) | never bundled; on Linux it comes with the Flatpak runtime | green | none |
| ONNX Runtime prebuilt by pyke (no notices, patched, no Intel Mac) | yellow | build ORT 1.28 ourselves in CI for each target, operator-reduced, ship its ThirdPartyNotices | green | M |
| DirectML.dll (proprietary redistributable) | yellow | WebGPU (Dawn, BSD-3) is the default Windows GPU path; DirectML only by explicit decision after measurement | green | S |
| CUDA/cuDNN/TensorRT (NVIDIA terms, >1 GB) | yellow | never redistributed; optional user-installed ORT via `load-dynamic` | green | S |
| Slint (attribution clause) | yellow | not used; egui (MIT/Apache) | green | none |
| egui fonts (Ubuntu Font Licence, OFL) | green with notices | notices in the acknowledgements screen | green | S |
| Mac App Store first-run model download | yellow | models bundled in the app (well under the 200 GB limit) | green | none |
| Marketing wording | red if "avoid Content ID" | wording rules in the product brief | green | none |
| Symphonia (MPL-2.0) | green with obligation | source link in acknowledgements | green | none |

For counsel before the paid release: whether calling an OS AAC codec
leaves any pool obligation with us, and the attribution form for the
CC BY training sources in the model card.
