import { beforeEach, describe, expect, it, vi } from 'vitest'
import { createPinia, setActivePinia } from 'pinia'

// `saveStations` calls Tauri's `invoke`, which doesn't exist outside a Tauri
// webview. Mocked so store-mutation tests exercise only the logic under
// test (moveUp/moveDown/toggleFavorite), not IPC. Also lets tests assert
// persistence actually happened (code review finding #5), not just that
// in-memory state changed.
vi.mock('@tauri-apps/api', () => ({ invoke: vi.fn().mockResolvedValue(undefined) }))

import { invoke } from '@tauri-apps/api'
import { useStationsStore, swapFavoriteOrder, sortByFavoriteOrder, normalizeFavoriteOrder, type Station } from './stations'

const makeStation = (id: string, favoriteOrder: number): Station => ({
  id,
  name: id,
  url: `https://example.com/${id}`,
  isFavorite: true,
  addedAt: 0,
  favoriteOrder,
})

describe('swapFavoriteOrder', () => {
  it('swaps adjacent favoriteOrder values when moving up from an interior position', () => {
    const list = [makeStation('a', 0), makeStation('b', 1), makeStation('c', 2)]
    const result = swapFavoriteOrder(list, 'b', 'up')

    expect(result.find((s) => s.id === 'a')?.favoriteOrder).toBe(1)
    expect(result.find((s) => s.id === 'b')?.favoriteOrder).toBe(0)
    expect(result.find((s) => s.id === 'c')?.favoriteOrder).toBe(2)
  })

  it('swaps adjacent favoriteOrder values when moving down from an interior position', () => {
    const list = [makeStation('a', 0), makeStation('b', 1), makeStation('c', 2)]
    const result = swapFavoriteOrder(list, 'b', 'down')

    expect(result.find((s) => s.id === 'b')?.favoriteOrder).toBe(2)
    expect(result.find((s) => s.id === 'c')?.favoriteOrder).toBe(1)
    expect(result.find((s) => s.id === 'a')?.favoriteOrder).toBe(0)
  })

  it('is a no-op when moving the first item up (boundary)', () => {
    const list = [makeStation('a', 0), makeStation('b', 1)]
    const result = swapFavoriteOrder(list, 'a', 'up')

    expect(result.find((s) => s.id === 'a')?.favoriteOrder).toBe(0)
    expect(result.find((s) => s.id === 'b')?.favoriteOrder).toBe(1)
  })

  it('is a no-op when moving the last item down (boundary)', () => {
    const list = [makeStation('a', 0), makeStation('b', 1)]
    const result = swapFavoriteOrder(list, 'b', 'down')

    expect(result.find((s) => s.id === 'a')?.favoriteOrder).toBe(0)
    expect(result.find((s) => s.id === 'b')?.favoriteOrder).toBe(1)
  })

  it('sorts by favoriteOrder regardless of array position', () => {
    const list = [makeStation('c', 2), makeStation('a', 0), makeStation('b', 1)]
    expect(sortByFavoriteOrder(list).map((s) => s.id)).toEqual(['a', 'b', 'c'])
  })

  // Code review finding #1: a `store.json` written before `favoriteOrder`
  // existed has every station default to 0 (`#[serde(default)]`), so all
  // stations are tied. Swapping two tied *values* is a no-op — reorder
  // must backfill distinct values first.
  it('backfills distinct sequential favoriteOrder values when the list is a migration tie (all stations share one value)', () => {
    const list = [makeStation('a', 0), makeStation('b', 0), makeStation('c', 0)]

    const result = swapFavoriteOrder(list, 'b', 'up')

    const a = result.find((s) => s.id === 'a')!
    const b = result.find((s) => s.id === 'b')!
    const c = result.find((s) => s.id === 'c')!
    // Backfilled to 0,1,2 by original array order, then 'b' (index 1)
    // swaps with 'a' (index 0) moving up — a genuine change, not a no-op.
    expect(b.favoriteOrder).toBeLessThan(a.favoriteOrder)
    expect(c.favoriteOrder).toBe(2)
  })

  it('normalizeFavoriteOrder is a no-op when values are already distinct', () => {
    const list = [makeStation('a', 5), makeStation('b', 9)]
    expect(normalizeFavoriteOrder(list)).toEqual(list)
  })

  it('normalizeFavoriteOrder backfills sequential values by array order when any two collide', () => {
    const list = [makeStation('a', 0), makeStation('b', 0)]
    expect(normalizeFavoriteOrder(list).map((s) => s.favoriteOrder)).toEqual([0, 1])
  })
})

describe('useStationsStore', () => {
  beforeEach(() => {
    setActivePinia(createPinia())
    vi.clearAllMocks()
  })

  it('toggleFavorite adds a not-yet-favorited station, appended with the next favoriteOrder, and persists it', async () => {
    const store = useStationsStore()
    store.stations = [makeStation('a', 0), makeStation('b', 1)]

    await store.toggleFavorite({ id: 'new', name: 'New Station', url: 'https://example.com/new' })

    expect(store.isFavorite('new')).toBe(true)
    expect(store.stations.find((s) => s.id === 'new')?.favoriteOrder).toBe(2)
    expect(invoke).toHaveBeenCalledWith('save_stations', { stations: store.stations })
  })

  it('toggleFavorite carries country/geoLat/geoLong through onto the added station (spec-2-2)', async () => {
    const store = useStationsStore()
    store.stations = [makeStation('a', 0)]

    await store.toggleFavorite({
      id: 'new',
      name: 'New Station',
      url: 'https://example.com/new',
      country: 'Belgium',
      geoLat: 50.8503,
      geoLong: 4.3517,
    })

    const added = store.stations.find((s) => s.id === 'new')
    expect(added?.country).toBe('Belgium')
    expect(added?.geoLat).toBe(50.8503)
    expect(added?.geoLong).toBe(4.3517)
  })

  it('toggleFavorite carries codec/bitrate through onto the added station (spec-2-4)', async () => {
    const store = useStationsStore()
    store.stations = [makeStation('a', 0)]

    await store.toggleFavorite({
      id: 'new',
      name: 'New Station',
      url: 'https://example.com/new',
      codec: 'MP3',
      bitrate: 128,
    })

    const added = store.stations.find((s) => s.id === 'new')
    expect(added?.codec).toBe('MP3')
    expect(added?.bitrate).toBe(128)
  })

  it('toggleFavorite removes an already-favorited station and persists the removal', async () => {
    const store = useStationsStore()
    store.stations = [makeStation('a', 0), makeStation('b', 1)]

    await store.toggleFavorite(makeStation('a', 0))

    expect(store.isFavorite('a')).toBe(false)
    expect(store.stations.map((s) => s.id)).toEqual(['b'])
    expect(invoke).toHaveBeenCalledWith('save_stations', { stations: store.stations })
  })

  it('moveUp/moveDown persist via the store and update sortedStations order', async () => {
    const store = useStationsStore()
    store.stations = [makeStation('a', 0), makeStation('b', 1), makeStation('c', 2)]

    await store.moveDown('a')

    expect(store.sortedStations.map((s) => s.id)).toEqual(['b', 'a', 'c'])
    expect(invoke).toHaveBeenCalledWith('save_stations', { stations: store.stations })

    vi.mocked(invoke).mockClear()
    await store.moveUp('a')

    expect(store.sortedStations.map((s) => s.id)).toEqual(['a', 'b', 'c'])
    expect(invoke).toHaveBeenCalledWith('save_stations', { stations: store.stations })
  })

  // Code review finding #1: the exact migration shape (all favoriteOrder
  // tied at 0, as every station loaded from a pre-spec-1-4 store.json would
  // be) must still let moveUp/moveDown produce a real, persisted reorder —
  // not a silent no-op.
  it('moveUp/moveDown actually reorders a migrated list where all favoriteOrder values are tied at 0', async () => {
    const store = useStationsStore()
    store.stations = [makeStation('a', 0), makeStation('b', 0), makeStation('c', 0)]

    await store.moveDown('a')

    expect(store.sortedStations.map((s) => s.id)).toEqual(['b', 'a', 'c'])
    expect(invoke).toHaveBeenCalledWith('save_stations', { stations: store.stations })
  })
})
