/**
 * Picker 子窗口模板配置
 *
 * 集中管理各 picker 模板的默认参数、白名单规则和 CSP 策略。
 * 修改白名单、CSP 或默认尺寸只需改此文件，不需动逻辑代码。
 *
 * ## 白名单规则
 *
 * - 精确匹配：'example.com' 匹配 hostname === 'example.com'
 * - 通配符前缀：'*.example.com' 匹配 example.com 的所有子域名
 *
 * ## CSP 策略
 *
 * 每个模板可配置 `csp` 字段（Content-Security-Policy），通过 IPC 传递给后端，
 * 后端在创建 picker:// 响应时注入到 HTTP 响应头中。CSP 用于限制子窗口内可加载
 * 的资源范围，防止模板被注入恶意资源。
 *
 * CSP 中的协议来源说明：
 * - 'self'：当前 picker:// 页面自身
 * - picker:：允许 picker:// 协议（用于 fetch /data 等同协议请求）
 * - res:：允许 res:// 协议（用于加载后端嵌入资源，如 marked.min.js）
 * - data: / blob:：允许 data/blob URL（图片、二维码等）
 */

/** 模板配置 */
export interface PickerTemplateConfig {
  /** 模板名称（对应后端 resources/templates/<name>.html） */
  template: string
  /** 默认窗口标题 */
  title: string
  /** 默认窗口宽度 */
  width?: number
  /** 默认窗口高度 */
  height?: number
  /** Content-Security-Policy 策略字符串（通过 IPC 传给后端应用到响应头） */
  csp?: string
  /** 重定向模板的域名白名单（支持 * 通配符前缀，如 *.example.com） */
  allowedDomains?: string[]
}

/**
 * 通用基础 CSP：所有 picker 模板的默认策略
 *
 * - default-src 'self'：默认只允许同源资源
 * - style-src 'unsafe-inline'：允许内联样式（模板中的 <style>）
 * - script-src 'unsafe-inline'：允许内联脚本（模板中的 <script>）
 * - img-src 'self' data: blob:：允许同源图片 + data/blob URL
 * - connect-src 'self' picker:：允许同源 + picker:// 协议的 fetch（如 /data 请求）
 * - font-src 'self' data:：允许同源字体 + data URL
 */
const BASE_CSP = [
  "default-src 'self'",
  "style-src 'self' 'unsafe-inline'",
  "script-src 'self' 'unsafe-inline'",
  "img-src 'self' data: blob:",
  "connect-src 'self' picker:",
  "font-src 'self' data:",
].join('; ')

/** 模板配置表 */
export const PICKER_TEMPLATES: Record<string, PickerTemplateConfig> = {
  /** 本机端口选择器 */
  'port-picker': {
    template: 'port-picker',
    title: '选择本机端口',
    width: 400,
    height: 500,
    csp: BASE_CSP,
  },

  /** URL 重定向页面 */
  'redirect': {
    template: 'redirect',
    title: '正在跳转',
    width: 400,
    height: 300,
    csp: BASE_CSP,
    allowedDomains: [
      'localhost',
      '127.0.0.1',
      'moteam.top',
      '*.moteam.top',
      '*.molaunch.moiu.cn',
    ],
  },

  /** 确认对话框（返回 'true' / 'false'） */
  'confirm': {
    template: 'confirm',
    title: '确认',
    width: 420,
    height: 240,
    csp: BASE_CSP,
  },

  /** 信息展示页面（标题 + 正文，正文支持简单 HTML） */
  'info': {
    template: 'info',
    title: '信息',
    width: 480,
    height: 360,
    csp: BASE_CSP,
  },

  /** 图片查看器（接收图片 URL，支持缩放） */
  'image-viewer': {
    template: 'image-viewer',
    title: '图片查看',
    width: 800,
    height: 600,
    // 图片查看器需要加载外部图片，img-src 扩展 https/http
    csp: [
      "default-src 'self'",
      "style-src 'self' 'unsafe-inline'",
      "script-src 'self' 'unsafe-inline'",
      "img-src 'self' data: blob: https: http:",
      "connect-src 'self' picker:",
    ].join('; '),
  },

  /** Markdown 渲染页面（通过 res:// 加载 marked.min.js） */
  'markdown': {
    template: 'markdown',
    title: '文档',
    width: 720,
    height: 560,
    // markdown 模板需要从 res:// 协议加载 marked.min.js
    // 注意：Windows 上 res:// 转为 https://res.localhost/，CSP 需同时允许 res: 和 https://res.localhost
    csp: [
      "default-src 'self'",
      "style-src 'self' 'unsafe-inline'",
      "script-src 'self' 'unsafe-inline' res: https://res.localhost",
      "img-src 'self' data: blob: https:",
      "connect-src 'self' picker: res: https://res.localhost",
    ].join('; '),
  },

  /** 教程渲染页面（亮色主题，通过 res:// 加载 marked.min.js） */
  'tutorial': {
    template: 'tutorial',
    title: '教程',
    width: 760,
    height: 600,
    // 与 markdown 模板相同的 CSP：允许 res:// 加载 marked.min.js
    // 注意：Windows 上 res:// 转为 https://res.localhost/，CSP 需同时允许 res: 和 https://res.localhost
    csp: [
      "default-src 'self'",
      "style-src 'self' 'unsafe-inline'",
      "script-src 'self' 'unsafe-inline' res: https://res.localhost",
      "img-src 'self' data: blob: https:",
      "connect-src 'self' picker: res: https://res.localhost",
    ].join('; '),
  },

  /** 二维码展示页面（接收 url/text 生成二维码） */
  'qrcode': {
    template: 'qrcode',
    title: '二维码',
    width: 360,
    height: 420,
    // qrcode 模板通过 res:// 协议加载后端嵌入的 qrcode.min.js，script-src 需要允许 res:
    // 注意：Windows 上 res:// 转为 https://res.localhost/，CSP 需同时允许 res: 和 https://res.localhost
    csp: [
      "default-src 'self'",
      "style-src 'self' 'unsafe-inline'",
      "script-src 'self' 'unsafe-inline' res: https://res.localhost",
      "img-src 'self' data: blob:",
      "connect-src 'self' picker: res: https://res.localhost",
    ].join('; '),
  },
}

/**
 * 检查 URL 是否匹配白名单
 *
 * @param url 待检查的 URL
 * @param allowedDomains 域名白名单（支持 *.example.com 通配符）
 * @returns 是否允许
 */
export function isUrlAllowed(url: string, allowedDomains: string[]): boolean {
  try {
    const hostname = new URL(url).hostname
    return allowedDomains.some(pattern => {
      if (pattern.startsWith('*.')) {
        const suffix = pattern.slice(2)
        return hostname === suffix || hostname.endsWith('.' + suffix)
      }
      return hostname === pattern
    })
  } catch {
    return false
  }
}

/**
 * 获取模板配置
 *
 * @param template 模板名称
 * @returns 模板配置，不存在返回 undefined
 */
export function getTemplateConfig(template: string): PickerTemplateConfig | undefined {
  return PICKER_TEMPLATES[template]
}
