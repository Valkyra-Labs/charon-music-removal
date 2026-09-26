# Competitor complaints and what they mean for the design

2026-09-25. Five research passes over Trustpilot, vendor forums (Adobe,
Apple, Blackmagic, Native Instruments, OBS), Hacker News, GitHub issue
trackers (UVR, StemRoller), vendor docs and pricing pages. Reddit, G2,
Capterra, App Store reviews and the Blackmagic forum were blocked for
the fetch tools; the search budget ran out in three passes. Counts are
distinct sources read, a floor rather than a population estimate.
Items only seen in search snippets are excluded here.

## The five findings that shape the product

1. **Nobody returns the original video with only the music removed.**
   ElevenLabs returns audio only; UVR has an open request for video
   output; AudioShake processes the audio track only; LALAL.AI claims
   video output and a user reported an MP4 returned as MP3 (Aug 2026).
   Every user re-muxes by hand.
2. **Long files and surround break the tools.** UVR cannot export
   3-hour files and fails on 6-channel audio; AudioShake excludes 5.1;
   cloud tools cap files at 30 min to 2 h (Adobe, ElevenLabs 1 h /
   500 MB). OBS recordings are MKV with up to six audio tracks.
3. **Quality complaints are about damage, not about residue.**
   Robotic or muffled voice (Adobe Podcast v2, Premiere Enhance
   Speech, Resolve at high amounts, Descript at 100%), invented or
   repeated words (Premiere Enhance Speech, 2024-2025), laughter and
   surprise removed (Adobe Podcast v2), speech silenced during
   dialogue (LALAL), sibilants removed (Krisp).
4. **Billing is the loudest complaint everywhere.** Credits that drain
   on failed jobs, auto-renewal after cancelling, refunds refused,
   features moved behind paywalls (Adobe v1, CapCut, Filmora
   "perpetual" licences becoming version-specific), per-minute
   metering that makes a 3-hour VOD cost $90 (SoundScrub) or 180,000
   credits (ElevenLabs).
5. **The platform tool is irreversible and after the fact.** YouTube
   Erase song works only on an existing claim, may fail "if the song
   is hard to remove", keeps the claim if any claimed audio remains,
   and since June 2025 cannot be reverted after saving.

## Inventory by category

| category | distinct sources | products | latest |
|---|---|---|---|
| subscription, billing, refunds, auto-renew | 30+ | LALAL, Moises, Descript, Krisp, CapCut, Media.io, ElevenLabs, Filmora, VEED, Kapwing | 2026-09 |
| credits and quotas | 12+ | Descript, CapCut, ElevenLabs, LALAL, Media.io, Adobe (4 h/day), Kapwing | 2026-09 |
| robotic, muffled, garbled voice | 10 | Adobe Podcast, Premiere, Resolve, Descript, LALAL, ElevenLabs | 2026-06 |
| invented or changed words | 3 | Premiere Enhance Speech, Adobe Podcast v2 | 2025-08 |
| laughter or non-verbal removed | 2 | Adobe Podcast v2 | 2024-12 |
| no video output / re-mux by hand | 5 | ElevenLabs, UVR, AudioShake, LALAL | 2026-08 |
| long files, surround, MP4 errors | 8 | UVR (3 h, 6-channel, MP4 errors), AudioShake (no 5.1) | 2026-07 |
| crashes and bugs | 12+ | Resolve, Descript, Moises, UVR, StemRoller, Krisp, Kapwing | 2026-09 |
| GPU requirements and new-GPU breakage | 8 | NVIDIA Broadcast (RTX only), UVR (VRAM, RTX 50 hangs), StemRoller (bundled PyTorch lacks Blackwell), Resolve | 2026-09 |
| OS support | 6 | Final Cut and Logic (Apple Silicon only), NVIDIA (Windows only), UVR (Mac crashes) | 2025-05 |
| upload, privacy, terms | 4 | CapCut (perpetual licence to uploads), HN on voice uploads | 2025-08 |
| out of sync, echo after edits | 3 | Premiere (echo after trim), Descript (drift) | 2026-07 |
| install size | 1 | StemRoller 1.85 GB for 350 MB of weights | 2025 |
| preview limits | 2 | LALAL (1-minute preview from file start), Adobe (slider has no audible effect) | 2025 |

Categories with no evidence found (coverage gap, not absence): music
bleed complaints specifically, sync drift after re-mux of separated
audio, re-encode quality loss.

## Content ID evidence

- No source anywhere measures what residual music level Content ID
  tolerates. Our fingerprint experiment would be new information.
- False claims on game and ambient music are real and recurring:
  Twitch muted games' own soundtracks (Age of Mythology: Retold, The
  Planet Crafter, 2024); a game developer reports 17 of 45 licensed
  tracks claimed; a live orchestra recording matched a 1954 work.
- Workarounds today: OBS second audio track without music, muting,
  disputes.

## Segments and platforms

| segment | pain | evidence |
|---|---|---|
| streamers re-uploading VODs | game music muted in 30-minute blocks with the voice; multi-hour MKV | Twitch 2014 blog, Steam threads 2024, OBS KB |
| YouTubers and vloggers | claims on background music; Erase song irreversible | YouTube Help |
| church streamers | claims despite CCLI licences, weekly disputes | Church Production |
| indie filmmakers | need a filled M&E track; Foley costs thousands | post-house blog |
| event videographers | sync licences needed even for private delivery | Musicbed |

Platform shares: Steam survey Aug 2026 Windows 93.95%, macOS 2.14%,
Linux 3.90% (gamers, a proxy for streamers). StatCounter desktop:
Windows 62.7%. A 2021 pro-audio poll: 70% macOS. No measured split for
video creators. Conclusion: **Windows is required at launch**, not in
a later phase.

YouTube's largest audiences (Jan 2025 ad reach): India, US, Brazil,
Indonesia, Mexico, Japan, Germany. Localization order after English:
Spanish, Portuguese, Hindi, Indonesian, Japanese, German.

## Price anchors

| product | price |
|---|---|
| Soundproof (noise, local, Mac) | $39.99-49.99 one-time |
| NUO-STEMS | $33.99 one-time |
| Song Master | $59 / $99 one-time |
| DaVinci Resolve Studio | $295 one-time |
| Final Cut Pro | $299.99 one-time |
| iZotope RX 12 | $99 / $399 / $1,399 |
| SoundScrub (music removal) | $0.25 per 30 s |
| LALAL.AI | EUR 6.75-13.50 per month + packs |
| Descript | $16-65 per month |
| Adobe Firefly Pro | $19.99 per month |
| UVR, YouTube Erase song | free |

## Design requirements derived from the evidence

Each is tied to the finding above that motivates it.

1. Video in, the same video out: video stream copied bit-exact,
   original never modified (findings 1, 5).
2. Any length and any track layout: multi-hour files, MKV with
   several audio tracks, 5.1 (finding 2).
3. Preview before export: A/B, solo of the removed music, jump to
   regions; export is always a new file (finding 5, preview limits).
4. Process only where music is; leave clean segments bit-identical
   (finding 3: every processed second is a chance to damage speech).
5. Conservative separation with a strength control per region; no
   generative model anywhere in the chain (finding 3: invented words).
6. Keep laughter, shouts and effects; measure it (finding 3).
7. Singing counts as music (music-removal job; UVR issue 968).
8. One-time price, no credits, no subscription, no account, refunds
   without argument (finding 4).
9. Works fully offline, including licence activation (privacy
   complaints; the product's own claim).
10. Windows at launch; runs without a special GPU and uses one when
    present; no bundled PyTorch (platforms; GPU breakage).
11. Batch queue for backlogs (UVR, StemRoller requests).
12. Small install (StemRoller 1.85 GB): target under 300 MB.
13. Sample-accurate audio length and timestamps after re-mux (sync
    complaints).
14. Honest claims: no "avoid Content ID"; show what was removed and
    where (finding on Content ID evidence).

## Anti-requirements

Never: credits or metered pricing; auto-renewal; a watermark; an
account to process a file; uploading anything; changing words or
voice timbre; exporting over the original; hiding the removed audio
from the user; claiming a result on a platform we do not control.
