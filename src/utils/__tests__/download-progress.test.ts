import { describe, it, expect } from 'vitest'
import { resolveDisplayPercentage, toProgressSnapshot } from '../downloadProgress'
import type { RawDownloadProgress, RawDownloadStage } from '@/types/download'

function stage(over: Partial<RawDownloadStage> = {}): RawDownloadStage {
  return {
    name: 'stage',
    progress: 0,
    weight: 1,
    status: 'loading',
    bytes_downloaded: 0,
    bytes_total: 0,
    ...over,
  }
}

function raw(over: Partial<RawDownloadProgress> = {}): RawDownloadProgress {
  return {
    stages: [stage()],
    current_stage_index: 0,
    global_speed: 0,
    global_bytes_downloaded: 0,
    global_bytes_total: 0,
    is_active: true,
    is_complete: false,
    error_code: 0,
    ...over,
  }
}

describe('toProgressSnapshot', () => {
  it('should weight stages when building the global percentage', () => {
    const snapshot = toProgressSnapshot(
      raw({
        stages: [
          stage({ name: '下载整合包', progress: 1, weight: 10 }),
          stage({ name: '解析整合包', progress: 0.5, weight: 1 }),
          stage({ name: '下载 MOD', progress: 0, weight: 40 }),
        ],
        current_stage_index: 1,
      }),
    )
    // (1*10 + 0.5*1 + 0*40) / 51 ≈ 20.6%
    expect(snapshot.percentage).toBeCloseTo(20.6, 1)
  })

  it('should expose the current stage percentage separately', () => {
    const snapshot = toProgressSnapshot(
      raw({
        stages: [stage({ progress: 0.3 }), stage({ progress: 0.42 })],
        current_stage_index: 1,
      }),
    )
    expect(snapshot.stage_percentage).toBeCloseTo(42, 1)
  })

  it('should fall back to the last stage when the index overflows', () => {
    const snapshot = toProgressSnapshot(
      raw({ stages: [stage({ progress: 0.8 })], current_stage_index: 9 }),
    )
    expect(snapshot.stage_percentage).toBeCloseTo(80, 1)
  })

  it('should default optional fields and detect the paused flag', () => {
    const snapshot = toProgressSnapshot(
      raw({ stages: [stage({ is_paused: true })] }),
    )
    expect(snapshot.isPaused).toBe(true)
    expect(snapshot.stages[0].files_total).toBe(0)
    expect(snapshot.stages[0].group).toBeNull()
  })

  it('should return zero percentages when there are no stages', () => {
    const snapshot = toProgressSnapshot(raw({ stages: [] }))
    expect(snapshot.percentage).toBe(0)
    expect(snapshot.stage_percentage).toBe(0)
  })
})

describe('resolveDisplayPercentage', () => {
  it('should prefer the weighted value once bytes are known', () => {
    expect(
      resolveDisplayPercentage({ weighted: 37.5, globalBytesTotal: 1024, stage: 90 }),
    ).toBe(37.5)
  })

  it('should fall back to the stage percentage while no bytes are known', () => {
    expect(
      resolveDisplayPercentage({ weighted: 1.8, globalBytesTotal: 0, stage: 42 }),
    ).toBe(42)
  })

  it('should never claim 100 percent while a stage is still running', () => {
    expect(
      resolveDisplayPercentage({ weighted: 0, globalBytesTotal: 0, stage: 100 }),
    ).toBe(99.9)
  })

  it('should clamp out-of-range inputs', () => {
    expect(resolveDisplayPercentage({ weighted: 130, globalBytesTotal: 10, stage: 0 })).toBe(100)
    expect(resolveDisplayPercentage({ weighted: -5, globalBytesTotal: 10, stage: 0 })).toBe(0)
    expect(resolveDisplayPercentage({ weighted: 0, globalBytesTotal: 0, stage: -20 })).toBe(0)
  })
})
