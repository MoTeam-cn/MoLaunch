/**
 * 自定义布局 button section 解析测试
 *
 * 覆盖四类动作的合法/非法输入，以及跳转白名单与协议校验。
 */
import { describe, expect, it } from 'vitest'
import { parseJsonLayout } from '../json-parser'

function layoutWith(action: unknown): string {
  return JSON.stringify({ sections: [{ type: 'button', label: '按钮', action }] })
}

describe('自定义布局 button section', () => {
  it('should 解析 launch 动作 when 未指定版本', () => {
    const result = parseJsonLayout(layoutWith({ type: 'launch' }))
    expect(result.error).toBeNull()
    const section = result.schema?.sections[0]
    expect(section?.type).toBe('button')
    if (section?.type !== 'button') throw new Error('类型应为 button')
    expect(section.action).toEqual({ type: 'launch' })
  })

  it('should 保留版本号 when launch 指定 version', () => {
    const result = parseJsonLayout(layoutWith({ type: 'launch', version: '1.20.1-forge' }))
    const section = result.schema?.sections[0]
    if (section?.type !== 'button') throw new Error('类型应为 button')
    expect(section.action).toEqual({ type: 'launch', version: '1.20.1-forge' })
  })

  it('should 拒绝非 http 协议 when open_url 传 file 路径', () => {
    const result = parseJsonLayout(layoutWith({ type: 'open_url', url: 'file:///C:/evil.exe' }))
    expect(result.schema).toBeNull()
    expect(result.error).toContain('http')
  })

  it('should 拒绝白名单外路径 when navigate 到任意路由', () => {
    const result = parseJsonLayout(layoutWith({ type: 'navigate', path: '/etc/passwd' }))
    expect(result.schema).toBeNull()
    expect(result.error).toContain('path')
  })

  it('should 接受白名单路径 when navigate 到工具页', () => {
    const result = parseJsonLayout(layoutWith({ type: 'navigate', path: '/apps/tools' }))
    const section = result.schema?.sections[0]
    if (section?.type !== 'button') throw new Error('类型应为 button')
    expect(section.action).toEqual({ type: 'navigate', path: '/apps/tools' })
  })

  it('should 拒绝未知动作类型 when action.type 非法', () => {
    const result = parseJsonLayout(layoutWith({ type: 'run_script', code: 'alert(1)' }))
    expect(result.schema).toBeNull()
  })

  it('should 拒绝缺少 label when button 未写文字', () => {
    const result = parseJsonLayout(JSON.stringify({ sections: [{ type: 'button', action: { type: 'launch' } }] }))
    expect(result.schema).toBeNull()
    expect(result.error).toContain('label')
  })

  it('should 保留 variant when 指定 primary', () => {
    const result = parseJsonLayout(
      JSON.stringify({ sections: [{ type: 'button', label: '启动', variant: 'primary', action: { type: 'launch' } }] }),
    )
    const section = result.schema?.sections[0]
    if (section?.type !== 'button') throw new Error('类型应为 button')
    expect(section.variant).toBe('primary')
  })
})
