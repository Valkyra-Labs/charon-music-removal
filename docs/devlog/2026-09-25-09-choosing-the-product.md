# 09. From stems to a product: choosing one job

2026-09-25. Three research passes (creator pains, musician pains,
license audit of a product stack), after two on speech enhancement and
music detection. Reddit was unreachable for the fetch tools; evidence
comes from vendor forums, platform behaviour, reviews and issue
trackers.

## Candidates

| candidate | pain | gap for a local app | clean models | verdict |
|---|---|---|---|---|
| Remove music from video, keep speech and effects | medium-strong | large | yes | chosen |
| Practice player (mute a part, loop, tempo) | strong | small (crowded, Logic has it) | no 4-stem model near HTDemucs | later |
| DJ stem preparation | medium | medium | worst licensing | rejected |
| UVR replacement for producers | medium | small (DAWs absorb it) | model zoo licensing | rejected |
| Voice cleanup | strong | small (saturated) | yes | a feature |

## Why music removal

- The platforms validate the pain: YouTube built "Erase song", Adobe
  sells music removal, Twitch mutes VODs in 30-minute blocks, games
  ship streamer modes.
- The incumbents leave gaps a local engine fills: cloud quotas (1-2 h
  per file), audio-only output, irreversible post-claim edits, heavy
  editor features that fail on game audio.
- It is the only candidate where the permissively licensed models are
  the right tool rather than a downgrade.
- The same engine can serve the owner's other audio tools.

## Key constraint discovered

A four-stem music model puts singing in "vocals" together with speech,
so it would keep exactly what a fingerprinting system hears. The
product needs a dialogue/music/effects model, and whether singing lands
in the music stem is the first thing to measure.

## Decision

Build the app; gate it on measured quality of the clean-license models
and on a fingerprint experiment. Market it as removing music you do not
have rights to, never as a way to avoid detection.
