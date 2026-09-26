# 05. Resident server and the head-to-head

2026-09-25. 193 s stereo track, whole runs (decode, separate, write).
Cold rows: one process each, second pass shown. Resident rows: model
loaded once, three consecutive jobs.

## What was done

`charon serve`: a Unix-socket server answering one JSON object per
line (`Separate`, `Ping`, `Stop`), keeping the session and the compiled
CoreML model resident. Then every available tool was run on the same
file on the same machine.

## Evidence

| tool | device | mode | wall | peak RSS |
|---|---|---|---|---|
| Charon `serve` | CoreML | resident | 6.15 / 6.27 / 6.39 s | 1.5-1.6 GB |
| PyTorch demucs 4.1.0 | MPS | resident | 7.32 / 7.35 / 7.59 s | |
| PyTorch demucs 4.1.0 | MPS | cold | 8.67 s | 1.96 GB |
| demucs-rs (Burn, Metal) | Metal | cold | 13.70 s | 0.88 GB, NaN in 53% of frames |
| Charon one-shot | CoreML | cold | 14.08 s | 2.6 GB |
| Charon one-shot | CPU | cold | 16.70 s | 3.5 GB |
| stem-splitter-core 1.2.0 | CPU | cold | 27.28 s | 4.9 GB |
| PyTorch demucs 4.1.0 | CPU | cold | 35.20 s | 2.6 GB |

Quality on MUSDB previews: stem-splitter-core 0.2-0.3 dB below the
reference; demucs-rs produced non-finite output on 6 of 50 tracks.

## What went wrong

The first socket path exceeded the Unix socket length limit (SUN_LEN);
the default moved to a short path under `$TMPDIR`.

## Decision

Claims are scoped to what the table supports: faster than PyTorch on
MPS with the same warm-model treatment; not faster cold (PyTorch starts
faster on the GPU); nothing about CUDA or Linux. One track and one
machine; the documents say so.
