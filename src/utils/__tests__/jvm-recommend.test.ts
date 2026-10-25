import { describe, expect, it } from 'vitest'
import { recommendJvm } from '../jvm-recommend'

describe('jvm-recommend', () => {
  it('should fall back to 2GB when total memory is unknown', () => {
    expect(recommendJvm(0).memoryMB).toBe(2048)
    expect(recommendJvm(Number.NaN).memoryMB).toBe(2048)
  })

  it('should take 40% of physical memory rounded down to 512MB', () => {
    expect(recommendJvm(8192).memoryMB).toBe(3072)
    expect(recommendJvm(16384).memoryMB).toBe(6144)
  })

  it('should clamp to at least 2GB and at most 8GB', () => {
    expect(recommendJvm(4096).memoryMB).toBe(2048)
    expect(recommendJvm(65536).memoryMB).toBe(8192)
  })

  it('should always include AlwaysPreTouch and a readable reason', () => {
    const rec = recommendJvm(16384)
    expect(rec.extraArgs).toContain('-XX:+AlwaysPreTouch')
    expect(rec.reason).toContain('16 GB')
  })
})
