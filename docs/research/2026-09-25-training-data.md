# Training data for our own separation model

2026-09-25. Four research passes over licence texts on the primary
pages (Zenodo, OpenSLR, Hugging Face cards, repository licences, site
terms). Not legal advice. Every source below was checked for commercial
use and derivative works; share-alike (SA), non-commercial (NC),
no-derivatives (ND) and "research only" sources are excluded.

## The simplification that matters

The product subtracts only the music estimate from the mix. So the
model needs one precise boundary: **music (including singing) versus
everything else**. Whether a laugh counts as speech or as an effect
does not change the output. The model can still have three outputs
(speech, effects, music) for the optional stem export, but only the
music stem has to be right.

It also means music does not need stems: a whole song, vocals
included, is exactly one music-stem example. The shortage of
commercially usable vocal stems stops mattering.

## Music (target stem, singing included)

| source | licence | size / format | use |
|---|---|---|---|
| FMA full, filtered to CC BY, CC0 and public domain on the per-track `license` column; the "Spoken" genre excluded | CC BY / CC0 / PD per track; the dataset README says "meant for research purposes" and the site forbids data mining (the dataset zip is not scraping) | 106,574 tracks in total, 879 GiB; the clean subset is at least 13,874 tracks (Stable Audio Open's count incl. Sampling+); MP3 ~263 kbps, 44.1 kHz stereo | main pool, with a legal flag on the README sentence |
| Musopen Kickstarter recordings (archive.org) | CC0 / Public Domain Mark on the recordings | 7.5 GB lossless | classical, instrumental |
| Incompetech (Kevin MacLeod) | CC BY 4.0, fixed credit line | mostly instrumental | game-like production music |
| OpenGameArt music | CC0, CC BY, OGA-BY per asset (CC BY-SA and GPL excluded) | small | real game music |
| Kenney music jingles | CC0 | tiny | stingers |
| VocalSet | CC BY 4.0 | 10.1 h solo singing | clean singing inside music mixtures |
| ccMixter a cappellas with `lic=by` or `lic=pd` | per upload | small | per-item check (parent samples, Jamendo cross-posts) |
| Wikimedia Commons song audio | CC BY / CC0 / PD per file (CC BY-SA excluded) | ~400 files | small |

Excluded: MUSDB18(-HQ) and MoisesDB (NC), MTG-Jamendo and anything from
Jamendo (its terms ban commercial ML use, and it litigates), MedleyDB,
RWC, DAMP, DALI, GTSinger, M4Singer, CSD, Opencpop, JVS-MuSiC (NC or
research-only), the Great 78 Project (no rights statement), Slakh2100
(rendered with Native Instruments samples; provenance unclear), Lakh
MIDI (transcriptions of copyrighted songs), music generator outputs
(undisclosed training data; Stable Audio Open forbids training models
on its outputs).

## Speech (and non-verbal voice)

| source | licence | size / rate | note |
|---|---|---|---|
| VCTK 0.92 | CC BY 4.0 | 110 speakers, 48 kHz WAV | clean read speech |
| HiFiTTS (OpenSLR 109) | CC BY 4.0 | 292 h, 44.1 kHz | LibriVox origin (MP3) |
| LibriSpeech, MLS, LibriTTS-R | CC BY 4.0 | 16-24 kHz | volume, multilingual (MLS: 8 languages) |
| Emilia-YODAS (only the `Emilia-YODAS/` folder) | CC BY 4.0 (from CC BY 3.0 YouTube uploads) | ~114k h, 24 kHz, 6 languages | spontaneous talk-show and commentary speech, the closest to streamers; per-video attribution via `video_id` |
| FLEURS | CC BY 4.0 | 102 languages, 16 kHz | language coverage |
| ICSI Meeting Corpus | CC BY 4.0 | ~72 h | natural laughter |
| ECHO corpus | CC BY 4.0, files by request | 15,000 vocalizations, 22 languages | shouts, alarm, amusement |
| FSD50K human-voice classes, CC0/CC BY clips only | per clip | 44.1 kHz WAV | laughter, screams |

Excluded: EARS, Expresso, RAVDESS, ESD, EmoV-DB (NC), plain Emilia (NC),
GigaSpeech (NC terms), VoxCeleb and AudioSet (no audio rights), Common
Voice (no-rehost terms via Mozilla Data Collective; usable only after a
terms review), SA corpora (VocalSound, JVNV, JNV, People's Speech SA
split) unless we accept SA.

## Effects

| source | licence | size | note |
|---|---|---|---|
| FSD50K, CC0 and CC BY clips, music and human-voice branches excluded | per clip; filter the `license` field in `*_clips_info_FSD50K.json` | 43,379 clips after DnR v3's filter, 44.1 kHz WAV | main pool |
| Kenney audio packs (not Voiceover or Music) | CC0 | small | game UI and impact sounds |
| OpenGameArt SFX (e.g. 512 8-bit effects, CC0) | CC0 / CC BY | small | game-style |
| Freedoom | BSD-3 | hundreds of sounds | FPS game effects |

Excluded: Freesound API harvesting (commercial API use needs a UPF
agreement; new AI-preference flags), Sonniss and ZapSplat (AI training
banned), Pixabay and Mixkit (ML scraping banned), BBC (non-commercial),
ESC-50 and UrbanSound8K (NC), WavCaps (academic).

## Rooms and noise for augmentation

BUT ReverbDB, Arni, Motus, dEchorate (CC BY 4.0), OpenSLR 26/28
(Apache-2.0), pyroomacoustics simulation (MIT), MUSAN noise subset,
Freesound CC0 noise through FSD50K. Excluded: WHAM! (NC), ACE (ND),
Treble10 and Matterport IRs (NC); DEMAND is CC BY-SA 3.0.

## Building the mixtures

1. Download each pool; build one manifest row per file: source,
   item id, URL, author, licence name and URL, SHA-256, duration.
2. Filter by the licence field; drop anything not CC0, CC BY, PD,
   MIT/BSD/Apache.
3. Resample to 44.1 kHz mono; drop items whose true bandwidth is below
   a threshold (HiFiTTS-2 style bandwidth measure) for the high-rate
   pools; keep 16-24 kHz speech as a minority.
4. Mix on the fly during training (scaper, BSD-3; torch-audiomentations,
   MIT): gameplay levels from the evaluation-set recipe (voice -18 to
   -24 LUFS, effects -20 to -28, music -26 to -36), rooms, codec
   simulation (AAC/Opus), music present in some examples and absent in
   others, sung vocals always labelled music.
5. Publish the manifest with the model: attribution for every CC BY
   item, as Stable Audio Open and FSD50K do.

## Still open

- Legal reading of FMA's "meant for research purposes" sentence
  against the per-track CC licences.
- Emilia-YODAS inherits whether YouTube uploaders applied CC correctly.
- Hours per pool after filtering (FMA clean-subset count not
  published; counted when downloaded).
