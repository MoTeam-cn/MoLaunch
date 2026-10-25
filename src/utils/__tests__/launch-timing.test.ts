import { describe, expect, it } from 'vitest'
import { formatDuration, stageLabel, timingRows } from '../launch-timing'
import type { LaunchTimings } from '@/utils/api/launch'

describe('launch-timing', () => {
  it('should map known stage names to Chinese when given enum names', () => {
    expect(stageLabel('GetJava')).toBe('获取 Java')
    expect(stageLabel('ValidateFiles')).toBe('文件检查')
    expect(stageLabel('Unknown')).toBe('Unknown')
  })

  it('should format duration in ms/s/m when given milliseconds', () => {
    expect(formatDuration(0)).toBe('0ms')
    expect(formatDuration(320)).toBe('320ms')
    expect(formatDuration(1500)).toBe('1.5s')
    expect(formatDuration(42000)).toBe('42s')
    expect(formatDuration(63000)).toBe('1m 03s')
    expect(formatDuration(Number.NaN)).toBe('-')
  })

  it('should return empty rows when given no timings', () => {
    expect(timingRows(null)).toEqual([])
    expect(timingRows({ stages: [], total_ms: 0, version_id: 'x' })).toEqual([])
  })

  it('should sort rows by duration desc and keep bar width positive when given timings', () => {
    const timings: LaunchTimings = {
      version_id: '1.20.1-forge',
      total_ms: 5000,
      stages: [
        { stage: 'GetJava', ms: 1000 },
        { stage: 'ValidateFiles', ms: 4000 },
      ],
    }
    const rows = timingRows(timings)
    expect(rows.map((row) => row.stage)).toEqual(['ValidateFiles', 'GetJava'])
    expect(rows[0].percent).toBe(100)
    expect(rows[1].percent).toBe(25)
    expect(rows[0].text).toBe('4.0s')
  })
})
