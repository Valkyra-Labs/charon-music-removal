# 0005. GPU matrix: CoreML and WebGPU, ONNX Runtime built by us

Date: 2026-09-25. Status: accepted by the owner, 2026-09-25; to be confirmed by measurement.

## Decision

- macOS arm64: CoreML (MLProgram), then WebGPU on Metal, then CPU.
- macOS x86_64, Windows, Linux: WebGPU (Metal, D3D12, Vulkan through
  Dawn, BSD-3), then CPU.
- DirectML is not the default: its DLL is proprietary (redistributable)
  and in maintenance mode; it is adopted only if measured clearly
  faster and accepted by an explicit decision.
- NVIDIA CUDA is never redistributed; an opt-in path loads a
  user-installed ONNX Runtime.
- ONNX Runtime 1.28 is built from source in CI for every target
  (universal macOS, operator-reduced), with its notices shipped.
- The provider is chosen at first run by a measured micro-benchmark
  and can be overridden.

## Why

Green on every OS; the chosen model family uses no ops missing from
CoreML or WebGPU; pyke's prebuilt binaries lack notices and an Intel
Mac build.

## Open

WebGPU was slower than CPU for HTDemucs on the M4 Pro; it must be
measured for our model on each OS before the matrix is final.
