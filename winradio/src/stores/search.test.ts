import { describe, expect, it, vi } from 'vitest'

// `toPlayableStation` is a pure function, but `search.ts` also imports
// `invoke` at module scope (used elsewhere in the store), which doesn't
// exist outside a Tauri webview — mocked so this file can import the module
// at all, same as `stations.test.ts`/`playback.test.ts`.
vi.mock('@tauri-apps/api', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))

import { toPlayableStation } from './search'

describe('toPlayableStation', () => {
  it('carries codec/bitrate through onto the playable station (spec-2-4)', () => {
    const station = toPlayableStation({
      id: 'a',
      name: 'Station A',
      url: 'https://a.example/stream',
      codec: 'MP3',
      bitrate: 128,
    })

    expect(station.codec).toBe('MP3')
    expect(station.bitrate).toBe(128)
  })

  it('leaves codec/bitrate undefined when the search result has none', () => {
    const station = toPlayableStation({
      id: 'a',
      name: 'Station A',
      url: 'https://a.example/stream',
    })

    expect(station.codec).toBeUndefined()
    expect(station.bitrate).toBeUndefined()
  })

  // Structural coverage guard (epic-1-retro item 1): every DirectoryStation
  // field must have a Station counterpart threaded through here — a field
  // populated on the input but absent on the output means it was silently
  // dropped, the same category of bug Story 2.2's review caught for
  // country/geoLat/geoLong.
  it('carries every populated DirectoryStation field through onto the playable station', () => {
    const fullyPopulated = {
      id: 'a',
      name: 'Station A',
      url: 'https://a.example/stream',
      favicon: 'https://a.example/icon.png',
      tags: 'jazz,live',
      country: 'Belgium',
      language: 'English',
      codec: 'MP3',
      bitrate: 128,
      geoLat: 50.85,
      geoLong: 4.35,
    }

    const station = toPlayableStation(fullyPopulated)

    expect(station.id).toBe(fullyPopulated.id)
    expect(station.name).toBe(fullyPopulated.name)
    expect(station.url).toBe(fullyPopulated.url)
    expect(station.faviconUrl).toBe(fullyPopulated.favicon)
    expect(station.country).toBe(fullyPopulated.country)
    expect(station.codec).toBe(fullyPopulated.codec)
    expect(station.bitrate).toBe(fullyPopulated.bitrate)
    expect(station.geoLat).toBe(fullyPopulated.geoLat)
    expect(station.geoLong).toBe(fullyPopulated.geoLong)
    // `language` has no Station counterpart by design (never persisted);
    // `category` is derived from `tags`/`country`, not a direct carry-through.
    expect(station.category).toBe('jazz')
  })
})
