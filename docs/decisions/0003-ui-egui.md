# 0003. UI: egui on wgpu, one codebase

Date: 2026-09-25. Status: accepted by the owner, 2026-09-25.

## Decision

The app UI is egui/eframe (0.36 line) on the wgpu renderer, one
codebase for macOS, Windows and Linux. Video preview frames come from
the OS decoder at preview size and are uploaded as NV12 planes with a
YUV shader; zero-copy import through wgpu 30's per-backend APIs is a
measured optimisation for later.

## Alternatives rejected

- SwiftUI: best on macOS, but Windows is the main market (Steam 94%)
  and it would mean three frontends.
- Tauri/Dioxus: HEVC in webviews is unreliable on Windows and Linux;
  raw frames over IPC are too slow.
- gpui: the zero-copy path panics on 10-bit and video-range NV12 and
  exists only on macOS; users pin forks.
- Slint (attribution clause, unstable GPU API), iced (stale, wgpu 27,
  no accessibility), Qt (LGPL duties, C++), Flutter (third language).

## Reuse

Patterns and code from the owner's earlier video app, ported from egui 0.28:
latest-frame slot, in-place texture with generation counter, peak
envelope, audio callback with atomic cursor and SPSC ring, muda menus,
rfd dialogs. Not its OpenCV and ffmpeg decode layer.

## Would be reversed by

A measured failure to reach smooth preview plus audio playback on a
mid-range Windows laptop.
