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
})
