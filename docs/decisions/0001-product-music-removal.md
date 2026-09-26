# 0001. Product: remove music from video

Date: 2026-09-25. Status: accepted, gated on measurement.

## Decision

The first product built on charon-audio is a local desktop app that
removes music from a video file while keeping speech and sound
effects, and writes the same container back with the video stream
copied.

## Gate

Before app development starts: a permissively licensed
dialogue/music/effects model keeps speech and effects without robotic
artefacts on real clips, routes singing to the music stem, and reaches
the fingerprint detection target on the oracle curve. If none does, the
decision is revisited.

## Alternatives rejected

Practice player, DJ stem preparation, producer batch splitter, voice
cleanup as a product. Reasons in devlog entry 09.
