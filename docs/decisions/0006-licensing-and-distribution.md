# 0006. Licences, publishing, channels

Date: 2026-09-25. Status: licences and publishing accepted by the owner 2026-09-25. Commercial terms are kept outside this repository; the public part of that decision is 0009.

## Library

charon-audio relicenses to `MIT OR Apache-2.0` from 0.1.2 (0.1.0 and
0.1.1 stay MIT). Stays on crates.io and public GitHub.

## App

- Source-available under FSL-1.1-MIT at the first public beta: code
  readable (supports the "local, private" claim),
  commercial clones barred, each version becomes MIT after two years.
- The repository stays private until the quality gate (decision 0001)
  passes, so a cancelled product is not published half-built. The
  devlog and decision records are published from the start.
- Contributions under a DCO plus licence grant, so the licence and
  store options stay open.
- Model weights ship under their own licence with a model card.

## Principles for any paid version

No subscription, no credits, no account; an offline licence file;
refunds without argument. Prices and the sales route are not public.

## Channels, in order

1. Notarized DMG from the site + Homebrew cask (macOS).
2. Windows: Microsoft Store (free MSIX signing, own commerce allowed)
   and winget; direct signed installer if a signing route is available.
3. Linux: Flatpak on Flathub (uses the runtime's codecs).
4. Mac App Store later (sandbox, bundled models).
