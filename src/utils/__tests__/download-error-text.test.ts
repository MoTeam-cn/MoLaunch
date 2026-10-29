import { describe, it, expect } from 'vitest'
import { describeDownloadError } from '../downloadErrorText'

describe('describeDownloadError', () => {
  it('should return null when the code is zero', () => {
    expect(describeDownloadError(0)).toBeNull()
  })

  it('should explain known codes', () => {
    expect(describeDownloadError(1)).toContain('下载源')
    expect(describeDownloadError(2)).toContain('校验')
    expect(describeDownloadError(3)).toContain('磁盘')
    expect(describeDownloadError(4)).toContain('取消')
    expect(describeDownloadError(5)).toContain('安装')
  })

  it('should return null for unknown codes so callers can fall back', () => {
    expect(describeDownloadError(99)).toBeNull()
  })
})
