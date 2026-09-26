# 0009. Free public beta, then a time-limited trial

Date: 2026-09-25. Status: accepted; revised the same day (no server,
see the revision section).

## Context

A route for paid sales is not available at launch (details kept
outside this repository). An export-length limit (decision 0006)
would lock in a pricing shape before the sales route exists.

## Decision

- Launch as a **free public beta**, full features.
- When a sales route exists, move to a **time-limited trial** instead
  of an export limit, which leaves room to choose the price and the
  channel later.
- **Track installs and trial licences** with the minimum data needed,
  compatible with the promise that media never leaves the computer:
  no media, no file names, no hardware fingerprints; the exact events,
  consent text and hosting follow the research.
- Beta users are treated fairly at the transition (the concrete offer
  is decided with the design).

## Supersedes

The monetization section of 0006.

## Revision (2026-09-25, owner): no server, fully local

The app must work locally without any external server. The earlier
design used a server for three things only: counting installs through
update checks, issuing beta and trial tokens, and optional usage
counts. No customer media or files were ever meant to leave the
computer. All three are dropped.

### Demo phase (now, decision 0011)

- Free, no licence, no trial, no network connections at all.
- Adoption is measured outside the app: GitHub Releases download
  counts, stars, issues, and direct feedback.
- The README and the About window say plainly: "Everything runs on
  this computer. The app makes no network connections."

### If a paid version comes later

- **Trial without a server**: a local counter of days of use (14 days,
  60-day cap from the first run), stored in the app data and the macOS
  keychain; a date earlier than the last seen one counts as a used day.
  It can be reset by a determined user; that is accepted.
- **Licence without a server**: a licence file signed with Ed25519 at
  purchase time (issued manually or by the payment platform's own
  delivery), verified in the app with an embedded public key.
- **Updates**: the user checks the releases page; an in-app check only
  if the user enables it.
- **Beta users**: announced in the release notes and on the site; a
  launch discount through a code, not a token issued by a server.
