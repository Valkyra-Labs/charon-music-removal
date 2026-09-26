# 08. Mapping the field

2026-09-25. Seven research passes over primary sources (repositories,
licenses, model cards, papers), about 250 sources.

## What was found

- **In Rust**, Charon is the only engine with a published parity record,
  a published quality record, a measured macOS GPU path and a resident
  server. Newer hand-written ports (September 2026) publish tighter
  parity but have no ecosystem.
- **The real competition is outside Rust**: python-audio-separator
  (monthly releases, 84 RoFormer checkpoints), ZFTurbo's training
  framework and model zoo, MLX ports claiming 73x real time on Apple
  Silicon (unreplicated), and audio.cpp (3,000 stars in three months).
- **Weights licensing is the industry's blind spot**: most community
  checkpoints carry no license; several popular re-hosts label
  restricted weights MIT.
- **Published quality numbers are not comparable**: three metrics on
  three test sets (museval on MUSDB, global SDR on a private set,
  SiSEC medians).

## Decision

Do not compete on model count. Compete on measured speed, parity, a
service mode, and clean licensing. Replicate the strongest external
claims (MLX speed) before planning around them.
