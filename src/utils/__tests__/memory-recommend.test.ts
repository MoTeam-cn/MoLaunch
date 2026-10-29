import { describe, it, expect } from 'vitest'
import { recommendMemory } from '../memory-recommend'

describe('recommendMemory', () => {
  it('should scale the base with total memory instead of a fixed 8GB cap', () => {
    const small = recommendMemory({ totalMB: 8192, availableMB: 8192 })
    const large = recommendMemory({ totalMB: 65536, availableMB: 65536 })
    expect(small.maxMB).toBeLessThan(large.maxMB)
    expect(large.maxMB).toBeGreaterThan(8192)
  })

  it('should add memory for heavy modpacks', () => {
    const vanilla = recommendMemory({ totalMB: 32768, availableMB: 32768, modCount: 0 })
    const heavy = recommendMemory({ totalMB: 32768, availableMB: 32768, modCount: 200 })
    expect(heavy.maxMB).toBeGreaterThan(vanilla.maxMB)
  })

  it('should cap the mod bonus so a broken pack cannot ask for everything', () => {
    const a = recommendMemory({ totalMB: 65536, availableMB: 65536, modCount: 500 })
    const b = recommendMemory({ totalMB: 65536, availableMB: 65536, modCount: 5000 })
    expect(a.maxMB).toBe(b.maxMB)
  })

  it('should respect what is actually available right now', () => {
    const crowded = recommendMemory({ totalMB: 32768, availableMB: 3000 })
    expect(crowded.maxMB).toBeLessThanOrEqual(Math.floor(3000 * 0.7))
  })

  it('should never go below the hard floor even when memory is tight', () => {
    const tight = recommendMemory({ totalMB: 4096, availableMB: 200 })
    expect(tight.maxMB).toBeGreaterThanOrEqual(1024)
    expect(tight.minMB).toBeLessThanOrEqual(tight.maxMB)
  })

  it('should explain why the number was chosen', () => {
    const rec = recommendMemory({ totalMB: 32768, availableMB: 32768, modCount: 100 })
    expect(rec.reason).toContain('32768')
    expect(rec.reason).toContain('模组 100 个')
  })
})
