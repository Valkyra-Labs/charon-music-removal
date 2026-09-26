# Product requirements (MVP)

2026-09-25. Derived from `research/2026-09-25-competitor-complaints.md`
(numbers in brackets refer to its design requirements).

## Must

- Open MP4, MOV and MKV of any length; every audio track, any channel
  layout up to 7.1 [1, 2].
- Export the same container with video and untouched streams copied
  bit-exact and only the processed audio replaced; never overwrite
  the original [1].
- Video preview above the timeline, synced to audio; placeholder with
  the codec name when the OS cannot decode it [3].
- Music regions detected automatically, editable (move, split,
  delete, add) [4].
- Processing only inside regions; outside them the output equals the
  input sample for sample [4].
- Removal depth per region (full, -30 dB, -20 dB, off) [5].
- A/B/C playback: original, result, removed music [3].
- Singing is removed as music; speech, laughter and effects are kept,
  and this is measured on a test set before release [6, 7].
- Batch queue [11].
- Offline: no account, no upload, offline licence file [8, 9].
- macOS (arm64 and x86_64) and Windows 10/11 at launch; Linux as a
  Flatpak shortly after [10].
- Install under 300 MB [12].
- Acknowledgements screen with every licence [decision 0002].

## Should

- Optional voice clean-up with a strength control.
- Stem export (speech, effects, music) as WAV/FLAC for editors.
- Speech-only WAV for subtitles.
- Localisation: English, then Spanish, Portuguese, Hindi,
  Indonesian, Japanese, German, Russian.

## Won't (MVP)

Replacing music with other music, editing video, cloud processing,
plugins for editors, mobile.
