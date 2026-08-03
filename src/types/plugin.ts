/**
 * 插件系统类型定义
 *
 * 允许扩展启动器：注册主页右侧组件、经 PluginSdk 访问有限后端 API、响应启动器事件。
 * 当前仅支持内置插件（随启动器编译），框架预留外部动态加载扩展点。
 */

import type { Component } from 'vue'

/**
 * 插件能力声明
 *
 * 插件通过实现对应接口声明自己提供的能力，启动器会按需调用。
 * 未实现的能力会被跳过，不会报错。
 */
export interface PluginCapabilities {
  /**
   * 主页右侧内容区组件
   *
   * 提供此组件后，用户可在「个性化 → 主页右侧内容区」选择此插件作为主页右侧显示内容。
   * 组件应自包含，不依赖外部 props。
   */
  homePanel?: Component

  /**
   * 插件设置页组件（可选）
   *
   * 提供此组件后，插件管理页面会显示「设置」按钮，点击展开此组件。
   * 未提供时插件管理页只显示开关。
   */
  settingsPanel?: Component
}

/**
 * 插件事件钩子
 *
 * 启动器在关键事件发生时会调用插件对应的钩子（如果实现了）。
 * 钩子是异步的，但不会阻塞主流程（失败仅记日志）。
 */
export interface PluginLifecycleHooks {
  /** 插件被启用时调用 */
  onEnable?: () => void | Promise<void>
  /** 插件被禁用时调用 */
  onDisable?: () => void | Promise<void>
  /** 游戏启动前调用（可读取即将启动的版本 ID） */
  onGameLaunch?: (versionId: string) => void | Promise<void>
  /** 游戏退出时调用（exitCode 为 null 表示异常终止） */
  onGameExit?: (versionId: string, exitCode: number | null) => void | Promise<void>
  /** 下载任务完成时调用 */
  onDownloadComplete?: (taskId: string, success: boolean) => void | Promise<void>
}

/**
 * 插件清单
 *
 * 每个内置插件在 `src/plugins/index.ts` 中通过此结构注册。
 */
export interface PluginManifest {
  /** 插件唯一 ID（kebab-case，如 "quick-stats"） */
  id: string
  /** 插件名称（显示用，中文） */
  name: string
  /** 插件描述（一句话说明功能） */
  description: string
  /** 插件版本（语义化版本，如 "1.0.0"） */
  version: string
  /** 插件作者 */
  author: string
  /**
   * 插件能力声明
   *
   * 通过返回对象提供 homePanel / settingsPanel 等能力。
   * 使用函数形式以便延迟加载组件依赖。
   *
   * 外部插件不实现此字段（其 homePanel 由 PluginSandbox 组件代理）。
   */
  capabilities?: () => PluginCapabilities
  /** 生命周期钩子（可选） */
  hooks?: PluginLifecycleHooks
  /** 是否为内置插件（内置=true，外部=false） */
  builtin: boolean
  /**
   * 已声明的权限列表
   *
   * - 内置插件：null/undefined（不受沙箱权限限制，可调用全部 SDK 方法）
   * - 外部插件：字符串数组，对应 manifest.json 的 permissions 字段
   *
   * 用于插件管理页面展示每个插件可调用的 SDK 方法。
   */
  permissions?: string[] | null
}

/**
 * 外部插件清单（manifest.json）
 *
 * 由外部插件作者编写，存放在 `<userData>/plugins/<plugin_id>/manifest.json`。
 * 后端 `list_external_plugins` 命令扫描后返回此结构。
 *
 * 与 PluginManifest 区别：
 * - 没有 capabilities（由沙箱代理）
 * - 没有 hooks（沙箱插件不支持生命周期钩子）
 * - 新增 entry（HTML 入口文件相对路径）
 * - 新增 permissions（声明所需的 SDK 方法白名单）
 */
export interface ExternalPluginManifest {
  /** 插件唯一 ID（kebab-case，必须与目录名一致） */
  id: string
  /** 插件名称 */
  name: string
  /** 插件描述 */
  description: string
  /** 插件版本 */
  version: string
  /** 插件作者 */
  author: string
  /** HTML 入口文件相对路径（相对插件目录，如 "index.html"） */
  entry: string
  /**
   * 权限白名单
   *
   * 声明此插件可调用的 SDK 方法名。
   * 未列出的方法在沙箱内调用会被拒绝。
   * 可选值：getConfig / listInstalledVersions / listInstalledVersionsWithType /
   *         listLaunchHistory / getSystemMemory / getRunningGamePid
   * emit / log 始终允许（无敏感数据）。
   */
  permissions?: string[]
}

/**
 * 插件运行时状态
 *
 * 由 PluginStore 管理，反映插件当前在启动器中的状态。
 */
export interface PluginRuntimeState {
  /** 插件 ID（与 manifest.id 对应） */
  id: string
  /** 是否已启用（用户在设置页切换） */
  enabled: boolean
  /** 是否为内置插件（true 表示随启动器编译，不可卸载） */
  builtin: boolean
  /** 最近一次错误信息（启用/运行失败时记录，正常为 null） */
  lastError: string | null
}

/**
 * 主页右侧内容区显示模式
 *
 * - "default"：启动器默认组件（LaunchLog 启动日志）
 * - "plugin:<id>"：使用指定插件提供的 homePanel 组件
 * - "custom"：用户自定义布局（JSON / HTML / XML，内联或 URL 加载）
 */
export type HomePanelMode = 'default' | `plugin:${string}` | 'custom'

/**
 * 自定义布局格式
 *
 * - json：结构化布局，启动器提供组件库（stat-grid / list / progress / text / divider），
 *         用户在 JSON 中声明使用哪些组件和数据源
 * - html：直接渲染 HTML 内容，通过 window.molaunch SDK 访问数据（iframe sandbox 隔离）
 * - xml：与 json 相同的结构化布局，XML 语法，前端解析后转为同一套 LayoutSchema
 */
export type LayoutFormat = 'json' | 'html' | 'xml'

/**
 * 自定义布局来源
 *
 * - inline：内联内容，直接在设置页编辑
 * - url：从 URL 加载，缓存到本地 .Molaunch/cache/layouts/ 目录
 */
export type LayoutSource = 'inline' | 'url'

/**
 * 自定义布局配置
 *
 * 存储在前端 localStorage + 后端 INI [Plugin] 节（customLayoutConfig 键存 JSON 字符串）。
 */
export interface CustomLayoutConfig {
  /** 布局格式 */
  format: LayoutFormat
  /** 布局来源 */
  source: LayoutSource
  /** 内联内容（source=inline 时有效） */
  inlineContent: string
  /** URL 地址（source=url 时有效） */
  url: string
  /** URL 加载的内容缓存（source=url 时有效，从本地缓存文件读取） */
  cachedContent: string
  /** 缓存时间戳（毫秒，0 表示未缓存） */
  cachedAt: number
}
