#!/usr/bin/env python3
"""Run the app's headless pipeline on the synthetic clips and score it.

Metrics per clip (music region = the truth interval):
- detection: precision and recall of 0.5 s frames marked as music;
- music residual: 10*log10(|alpha*m|^2 / |m|^2) where alpha is the least-
  squares projection of (result - speech) on the placed music m, i.e. how
  much of the original music is still linearly present (lower is better;
  -inf would be perfect removal);
- speech SI-SDR inside the music interval, of the mix and of the result,
  against the clean speech (higher is better): how much the speech was
  helped by removing the music, and how little it was damaged;
- outside: max |result - input| outside the detected regions (must be 0).

Usage: evaluate.py CLIPS_DIR APP_BINARY MODEL
"""
import json
import subprocess
import sys
import time
from pathlib import Path

import numpy as np
import soundfile as sf

FRAME = 0.5
SR = 44100


def si_sdr(est, ref):
    est, ref = est.ravel(), ref.ravel()
    alpha = np.dot(est, ref) / (np.dot(ref, ref) + 1e-12)
    t = alpha * ref
    return 10 * np.log10(np.sum(t ** 2) / (np.sum((est - t) ** 2) + 1e-12))


def main():
    clips, app, model = Path(sys.argv[1]), sys.argv[2], sys.argv[3]
    rows = []
    for d in sorted(p for p in clips.iterdir() if (p / "mix.wav").exists()):
        out = d / "out"
        t = time.time()
        log = subprocess.run([app, "run-audio", str(d / "mix.wav"), str(out), model],
                             capture_output=True, text=True, check=True).stdout.strip()
        wall = time.time() - t
        speech, _ = sf.read(str(d / "speech.wav"), dtype="float32")
        music, _ = sf.read(str(d / "music.wav"), dtype="float32")
        result, _ = sf.read(str(out / "result.wav"), dtype="float32")
        inp, _ = sf.read(str(out / "input.wav"), dtype="float32")
        n = min(len(speech), len(result))
        a, b = [float(x) for x in (d / "truth.txt").read_text().split()]
        ia, ib = int(a * SR), int(b * SR)
        regions = [tuple(map(float, l.split())) for l in (out / "regions.txt").read_text().split("\n") if l.strip()]

        frames = int(n / SR / FRAME)
        truth = np.array([a <= (i + 0.5) * FRAME < b for i in range(frames)])
        pred = np.array([any(s <= (i + 0.5) * FRAME < e for s, e in regions) for i in range(frames)])
        tp = int((truth & pred).sum())
        precision = tp / max(pred.sum(), 1)
        recall = tp / max(truth.sum(), 1)

        m = music[ia:ib]
        r = result[ia:ib] - speech[ia:ib]
        alpha = np.sum(r * m) / (np.sum(m * m) + 1e-12)
        residual_db = 10 * np.log10(max(alpha ** 2, 1e-12))
        before = si_sdr(inp[ia:ib], speech[ia:ib])
        after = si_sdr(result[ia:ib], speech[ia:ib])

        outside = np.ones(n, dtype=bool)
        for s, e in regions:
            outside[int(s * SR):int(e * SR)] = False
        outside_diff = float(np.abs(result[:n][outside] - inp[:n][outside]).max()) if outside.any() else 0.0

        row = dict(clip=d.name, about=(d / "about.txt").read_text().strip(),
                   regions=regions, precision=round(float(precision), 3),
                   recall=round(float(recall), 3),
                   music_residual_db=round(float(residual_db), 1),
                   speech_si_sdr_mix=round(float(before), 1),
                   speech_si_sdr_result=round(float(after), 1), outside_max_diff=float(outside_diff),
                   wall_s=round(wall, 1), log=log)
        rows.append(row)
        print(f"{d.name}: P {precision:.2f} R {recall:.2f} residual {residual_db:6.1f} dB "
              f"speech SI-SDR {before:5.1f} -> {after:5.1f} dB outside {outside_diff:.1e} "
              f"({wall:.0f} s) {row['about']}", flush=True)
    (clips / "results.json").write_text(json.dumps(rows, indent=2))


if __name__ == "__main__":
    main()
