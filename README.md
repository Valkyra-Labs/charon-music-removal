# Charon Music Removal (demo)

Remove the music from a video and keep the speech and sound effects,
on your Mac, with nothing uploaded.

Drop in an MP4 or MOV. The app finds where music plays, removes it
there, lets you listen to the original, the result and the removed
part, and writes the same video back with the picture copied bit for
bit and only the sound replaced.

This is a free demo, built in the open. The product decisions, research
and measurements behind it are in [docs/](docs/): start with the
[devlog](docs/devlog/README.md).

## What it does

- Finds music regions automatically on a timeline under a video preview.
- Separates the music with the music branch of TIGER-DnR through
  [charon-audio](https://github.com/Valkyra-Labs/charon-audio) and
  subtracts it from the mix only inside those regions; everywhere else
  the audio is left exactly as it was.
- Removal depth per region: remove, -30 dB, -20 dB, keep. Changes are
  heard immediately.
- Listen: original, result, and "removed" (the part taken out, to hear
  whether any speech or effects went with the music).
- Exports MP4 or MOV: video stream copied without re-encoding, new AAC
  audio from the system encoder. The original file is never modified.
- No network connections, no account, no licence.

## Measured

On a synthetic set (speech with songs and cinematic music at -3 to -15
dB): music detected in all eight clips with no false frames, median
music residual -16.7 dB, median speech improvement +8.7 dB SI-SDR, the
rest of the audio untouched bit for bit. On three 1080p60 gameplay
recordings (2-5 minutes, music almost throughout), the whole analysis
takes 0.81-0.85 x the video's length on an Apple M4 Pro CPU; no music
is found in a recording with the game's music switched off. Details and
limits in [docs/measurements.md](docs/measurements.md).

## Requirements

macOS 13 or later on Apple Silicon.

## Build

The app depends on charon-audio by path (`../charon-audio`), and needs
the model file:

```bash
python ../charon-audio/tools/export/export_tiger.py --repo TIGER --weights TIGER-DnR --out tiger_music.onnx --dynamo --opset 18
```

(see charon-audio's `docs/MODELS.md`), then:

```bash
tools/bundle.sh tiger_music.onnx
```

The app is ad-hoc signed, not notarized: the first time, open it with
right click, Open.

## Model and training data

Charon Music Removal uses the music branch of TIGER-DnR (Xu, Li, Chen,
Hu; ICLR 2025), converted to ONNX by this project. Only the music
output is kept, and no weight values were changed. The original weights
are published at huggingface.co/JusperLee/TIGER-DnR under the Apache
License 2.0, which is included with the app together with a note of
the changes.

According to the TIGER paper, the model was trained on version 1 of the
Divide and Remaster (DnR) dataset. DnR mixes speech from LibriSpeech
(public-domain LibriVox recordings), music from the Free Music Archive
and sound effects from FSD50K. Many of those music tracks, and some of
the sound effects, are published by their authors under Creative
Commons licences that forbid commercial use.

For that reason this app is free, has no paid features, no ads and no
donations, and is offered as a demonstration. Whether this model may be
used in a commercial product is an open question, and it is not used in
one. Any paid version will use a model trained only on public-domain,
CC0 and CC BY material, with a published list of sources (decision
0012).

This describes the facts as understood; it is not legal advice.

## Licences

The app is source-available under the Functional Source License 1.1
with the MIT future licence ([LICENSE.md](LICENSE.md), decision 0006):
free to use, read and modify for anything except a competing product,
and each version becomes MIT two years after its release. The
third-party licences are in [THIRD_PARTY.md](THIRD_PARTY.md) and
`licenses/`. The model: see "Model and training data" above.

The app removes music you do not have the rights to use. It makes no
promise about how any platform's content-matching system will treat
the result.
