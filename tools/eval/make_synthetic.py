#!/usr/bin/env python3
"""Synthetic evaluation clips with known stems.

Each clip: 40 s of continuous speech (macOS `say`, several voices) with
music added from 12 s to 30 s at a set level relative to the speech.
Half the clips use MUSDB18 preview mixtures (songs with vocals: singing
must count as music), half use instrumental cinematic tracks. Writes per
clip: speech.wav, music.wav (already placed and scaled), mix.wav and
truth.txt (music interval in seconds). 44.1 kHz stereo float.

For internal measurement only: MUSDB18 is licensed for research use and
the clips are not redistributed.

Usage: make_synthetic.py OUT_DIR --musdb DIR --music FILE [FILE ...]
"""
import argparse
import subprocess
import tempfile
from pathlib import Path

import numpy as np
import soundfile as sf

SR = 44100
LENGTH = 40.0
MUSIC = (12.0, 30.0)
TEXT = (
    "Okay, so we just got into the old mansion, and honestly this place is creepy. "
    "Let me check the map real quick. There should be a key somewhere upstairs. "
    "Wait, did you hear that? Something just moved behind the door. "
    "Alright, I am going in. If this goes wrong, remember I told you so. "
    "Oh wow, look at this room, the lighting is incredible. "
    "Let us grab the ammo and get out of here before it comes back. "
)
VOICES = ["Daniel", "Samantha", "Karen", "Moira", "Rishi", "Tessa", "Fred", "Albert"]


def say(text, voice):
    with tempfile.TemporaryDirectory() as tmp:
        path = Path(tmp) / "s.wav"
        subprocess.run(["say", "-v", voice, "--file-format=WAVE",
                        "--data-format=LEF32@44100", "-o", str(path), text], check=True)
        audio, rate = sf.read(str(path), dtype="float32")
    assert rate == SR, rate
    return audio


def speech_track(voice):
    n = int(LENGTH * SR)
    out = np.zeros(n, dtype=np.float32)
    pos = int(0.5 * SR)
    chunk = say(TEXT, voice)
    while pos < n:
        take = min(len(chunk), n - pos)
        out[pos:pos + take] = chunk[:take]
        pos += take + int(0.4 * SR)
    return np.stack([out, out], 1)


def read_stem_mix(path):
    # STEMS mp4: stream 0 is the mixture
    with tempfile.TemporaryDirectory() as tmp:
        wav = Path(tmp) / "m.wav"
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", str(path), "-map", "0:0",
                        "-ar", str(SR), "-ac", "2", "-c:a", "pcm_f32le", str(wav)], check=True)
        audio, _ = sf.read(str(wav), dtype="float32", always_2d=True)
    return audio


def read_any(path):
    with tempfile.TemporaryDirectory() as tmp:
        wav = Path(tmp) / "m.wav"
        subprocess.run(["ffmpeg", "-v", "error", "-y", "-i", str(path), "-ar", str(SR),
                        "-ac", "2", "-c:a", "pcm_f32le", str(wav)], check=True)
        audio, _ = sf.read(str(wav), dtype="float32", always_2d=True)
    return audio


def rms(x):
    return float(np.sqrt(np.mean(x ** 2) + 1e-12))


def main():
    ap = argparse.ArgumentParser()
    ap.add_argument("out", type=Path)
    ap.add_argument("--musdb", type=Path, required=True, help="folder of MUSDB18 .stem.mp4 previews")
    ap.add_argument("--music", type=Path, nargs="+", required=True, help="instrumental tracks")
    args = ap.parse_args()
    rng = np.random.default_rng(7)
    previews = sorted(args.musdb.glob("*.stem.mp4"))
    span = int((MUSIC[1] - MUSIC[0]) * SR)
    levels = [-3.0, -9.0, -15.0, -9.0]
    for i in range(8):
        voice = VOICES[i % len(VOICES)]
        speech = speech_track(voice)
        if i % 2 == 0:
            picks = rng.choice(len(previews), 3, replace=False)
            song = np.concatenate([read_stem_mix(previews[p]) for p in picks])
            source = "MUSDB " + " + ".join(previews[p].name.split(".stem")[0] for p in picks)
        else:
            track = args.music[(i // 2) % len(args.music)]
            song = read_any(track)
            start = len(song) // 3
            song = song[start:]
            source = track.name
        song = song[:span]
        if len(song) < span:
            song = np.pad(song, ((0, span - len(song)), (0, 0)))
        fade = int(0.05 * SR)
        ramp = np.linspace(0, 1, fade, dtype=np.float32)[:, None]
        song[:fade] *= ramp
        song[-fade:] *= ramp[::-1]
        level = levels[(i // 2) % len(levels)]
        a = int(MUSIC[0] * SR)
        speech_rms = rms(speech[a:a + span])
        gain = speech_rms * 10 ** (level / 20) / rms(song)
        music = np.zeros_like(speech)
        music[a:a + span] = song * gain
        mix = speech + music
        peak = np.abs(mix).max()
        if peak > 0.99:
            scale = 0.99 / peak
            speech, music, mix = speech * scale, music * scale, mix * scale
        d = args.out / f"clip{i:02d}"
        d.mkdir(parents=True, exist_ok=True)
        for name, x in (("speech", speech), ("music", music), ("mix", mix)):
            sf.write(str(d / f"{name}.wav"), x, SR, subtype="FLOAT")
        (d / "truth.txt").write_text(f"{MUSIC[0]} {MUSIC[1]}\n")
        (d / "about.txt").write_text(f"voice {voice}; music {source}; level {level} dB re speech\n")
        print(d.name, voice, f"{level:+.0f} dB", source)


if __name__ == "__main__":
    main()
