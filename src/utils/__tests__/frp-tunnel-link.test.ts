/**
 * FRP 隧道访问链接提取单测
 */
import { describe, expect, it } from 'vitest'
import { buildTunnelLink, extractTunnelLink, resolveTunnelLink } from '../frp-tunnel-link'

/** Lolia 实际日志（域名在首个点处曾被截断成 http://qwq） */
const LOLIA_LINE =
  '[20:36:36.689] [INFO] 8:36PM INFO <client/control.go:198> LoliaFRP-CLI: ' +
  '[c527f140cda79d98] [267e905e064c4e768e85485f66c6e971] 您可通过 http://qwq.i-cdn.xyz 访问您的服务'

describe('frp-tunnel-link', () => {
  it('should keep the full domain when the vendor prints a scheme url after the hint', () => {
    expect(extractTunnelLink([LOLIA_LINE])).toBe('http://qwq.i-cdn.xyz')
  })

  it('should extract bare host:port after the hint when the vendor prints no scheme', () => {
    expect(extractTunnelLink(['您可通过 abc.def.com:25565 访问您的服务'])).toBe('abc.def.com:25565')
  })

  it('should strip ansi color codes around the link', () => {
    expect(extractTunnelLink(['\x1b[32m您可通过 http://qwq.i-cdn.xyz 访问您的服务\x1b[0m'])).toBe(
      'http://qwq.i-cdn.xyz',
    )
  })

  it('should drop trailing sentence punctuation but keep dots inside the domain', () => {
    expect(extractTunnelLink(['您可通过 http://a.b.c 访问您的服务。'])).toBe('http://a.b.c')
    expect(extractTunnelLink(['您可通过 http://a.b.c:8080 访问您的服务'])).toBe('http://a.b.c:8080')
  })

  it('should keep path and query string of the extracted link', () => {
    expect(extractTunnelLink(['您可通过 https://x.y.com:8443/play?p=1 访问您的服务'])).toBe(
      'https://x.y.com:8443/play?p=1',
    )
  })

  it('should extract tcp link from the start proxy success line', () => {
    expect(extractTunnelLink(['start proxy success, [my-tunnel] tcp://1.2.3.4:25565'])).toBe(
      'tcp://1.2.3.4:25565',
    )
  })

  it('should not treat log timestamps or go source locations as host:port', () => {
    expect(extractTunnelLink(['[20:36:36.689] [INFO] 成功启动隧道'])).toBeNull()
    expect(extractTunnelLink(['<client/control.go:198> 成功启动隧道'])).toBeNull()
  })

  it('should fall back to the last link in the log when no hint exists', () => {
    expect(extractTunnelLink(['banner http://a.b.com', '访问地址 http://c.d.com:9000'])).toBe(
      'http://c.d.com:9000',
    )
  })

  it('should build a default link from tunnel fields when the log has no link', () => {
    expect(buildTunnelLink('srv.example.com', 25565, 'tcp')).toBe('srv.example.com:25565')
    expect(buildTunnelLink('srv.example.com', 8080, 'https')).toBe('https://srv.example.com:8080')
    expect(resolveTunnelLink(['no link here'], 'srv.example.com', 25565, 'tcp')).toBe(
      'srv.example.com:25565',
    )
  })
})
