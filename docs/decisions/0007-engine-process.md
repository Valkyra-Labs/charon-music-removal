# 0007. Inference in a child engine process

Date: 2026-09-25. Status: accepted by the owner, 2026-09-25.

## Decision

Model inference runs in a child process (the same binary with
`--engine`), speaking JSON lines over stdin/stdout, exchanging audio
through files. The UI process never loads ONNX Runtime.

## Why

- ONNX Runtime peaks at 1.5-3.5 GB during a job (charon-audio
  records); the owner's earlier video app measured that model memory freed inside a
  long-lived UI process is not returned to the OS, while an exiting
  process returns everything.
- A GPU driver crash must not close the window with unsaved state.
- The resident engine pays CoreML's 7-27 s compile once per session,
  the reason charon-audio has `charon serve`.

## Alternative rejected

In-process worker threads: simpler, but the two failure modes above.
