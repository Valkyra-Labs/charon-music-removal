# 0010. Training data policy

Date: 2026-09-25. Status: proposed.

## Decision

- Only CC0, CC BY, public domain, and MIT/BSD/Apache sources; every
  item has a manifest row with its licence and author; the manifest is
  published with the model as its attribution.
- Excluded: NC, ND, share-alike, "research only", sources whose terms
  forbid ML use (Jamendo, Sonniss, ZapSplat, Pixabay, Mixkit), YouTube
  audio without a CC grant, generator outputs.
- Songs with vocals are music. The model's music target is whole
  tracks; no vocal stems are needed.
- The model keeps three outputs (speech, effects, music) for stem
  export; product quality is measured on the music estimate.
- Pilot throughput is measured before any rented compute is paid for.

## Open for the owner

Accept FMA's clean subset despite the README's research note (it is
the only large pool of commercially licensed songs with vocals); if
not, the music pool shrinks to instrumentals plus VocalSet mixes.
