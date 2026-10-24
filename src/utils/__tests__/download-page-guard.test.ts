import { describe, expect, it } from 'vitest'
import { decideIdleAction } from '../downloadPageGuard'

describe('下载管理页空闲判定', () => {
  it('should 留在页面 when 前端仍认为在下载', () => {
    expect(decideIdleAction({ backendActive: false, storeActive: true })).toBe('stay')
  })

  it('should 留在页面 when 后端仍在跑但前端状态已熄灭', () => {
    expect(decideIdleAction({ backendActive: true, storeActive: false })).toBe('stay')
  })

  it('should 留在页面 when 后端查询失败', () => {
    expect(decideIdleAction({ backendActive: undefined, storeActive: false })).toBe('stay')
  })

  it('should 退回上一页 when 前后端都确认无任务', () => {
    expect(decideIdleAction({ backendActive: false, storeActive: false })).toBe('back')
  })
})
