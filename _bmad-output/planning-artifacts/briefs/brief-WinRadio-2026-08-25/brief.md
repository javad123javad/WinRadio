---
title: WinRadio Product Brief
status: draft
created: 2026-08-25
updated: 2026-08-25
---

# Product Brief: WinRadio

## Executive Summary

WinRadio is a personal Windows desktop app for listening to internet radio: search a global directory of stations, save favorites, organize channels, and control playback without living in a browser tab. It's a single-user passion project for Javad — no roadmap pressure, no audience to please but him.

It already has a start: a Tauri + Vue 3 + Rust scaffold with pieces for audio playback, a sleep timer, system tray, and dark mode. That scaffold doesn't currently run — a string of coding-agent attempts left it half-wired and looping without ever converging. This brief exists to give the next attempt (agent or otherwise) a fixed target to build against, so it stops circling and starts shipping.

## The Problem

Javad wants a proper desktop internet radio player on Windows — something that lives in the tray, remembers his stations, and plays on command — rather than juggling browser tabs or a bloated general media app. That's a reasonable itch; a lot of people solve it with things like TuneIn Desktop, VLC's internet stream bookmarks, or a browser tab pinned to a station.

The actual problem this brief is solving, though, is process, not product category: previous attempts at building this with free coding agents went in circles. `[ASSUMPTION: without a fixed, written spec to build against, the agents likely kept re-deciding scope and architecture mid-flight rather than converging]` — the current repo is evidence of that (a `commands` module that exists but is never wired into `lib.rs`, a `router-view` with no router, components referenced that only partially exist). The cost of the status quo is a stalled, non-compiling app and burned attempts.

## The Solution

A Windows desktop app (Tauri shell, Vue frontend, Rust backend — already the chosen stack) that lets Javad:

- Search and browse a large catalog of internet radio stations
- Standard playback controls (play/pause/volume/mute)
- See what's currently playing (station name, and live stream metadata where the station provides it)
- Save and manage favorites, and organize stations beyond just a flat favorites list
- Run quietly in the system tray with minimize/tray controls
- Use a sleep timer to auto-stop playback after a set time
- Keep preferences and station lists persisted locally between sessions

## Who This Serves

One user: Javad. Success is defined entirely by whether he actually uses it day to day instead of falling back to a browser tab or another app. No secondary personas, no growth considerations, no onboarding flow needed — it opens straight into "search or pick a favorite and hit play."

## Scope

**In scope for v1 (MVP):**
- Search/browse online radio stations by name, genre, country, or language
- Play / pause / volume / mute controls
- Favorites: add, remove, reorder
- Now-playing display: station name + live stream metadata (track/title) when available
- Sleep timer: auto-stop playback after N minutes `[ASSUMPTION: auto-stop only, not a wake/alarm feature — not specified, defaulting to the simpler common case]`
- Minimize to system tray with tray-based play/pause
- Dark/light theme toggle
- Local persistence of stations and settings across restarts
- Station directory: `[ASSUMPTION: Radio-Browser API — free, open, community-standard for this exact use case; no preference stated]`

**In scope for later (post-v1, not blocking a working v1):**
- Recently-played history
- Custom station groups/tags beyond favorites
- Start minimized / auto-start with Windows
- Media key support (play/pause from keyboard)
- Manually add a custom stream URL not in the directory
- Per-station volume memory

**Explicitly out of scope:**
- Recording streams to file
- Equalizer / audio effects
- Casting to other devices (Chromecast etc.)
- Accounts, sync across machines, mobile companion app

**Open question, deliberately not decided here:** whether to repair the existing broken scaffold or start the codebase fresh. `[ASSUMPTION: left open — this is an implementation-strategy call, not a product-scope call, and belongs to the architecture phase, which is set up to ratify or restart a brownfield codebase]`

## Success Criteria

This is working when:
- The app builds and runs cleanly — none of the current wiring gaps (see The Problem) remain
- Javad can search for a station, play it, and favorite it in under a few clicks
- The app survives a restart with favorites and settings intact
- Javad reaches for WinRadio instead of a browser tab when he wants internet radio

No business metrics, no growth targets — this is the whole bar.

## Vision

If it works well, WinRadio quietly becomes Javad's default way to listen to internet radio on Windows — invisible in the tray until he wants it, reliable enough that he never thinks about whether it'll still be broken tomorrow. No ambitions beyond that.
