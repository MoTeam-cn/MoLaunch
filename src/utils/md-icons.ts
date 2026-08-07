/** Markdown 图标占位符解析与挂载：将已知名称替换为 Heroicons Vue 组件。 */

import type { Component } from 'vue'
import { h, render } from 'vue'
import {
  ArrowDownTrayIcon,
  ArrowPathIcon,
  BugAntIcon,
  ChatBubbleLeftRightIcon,
  CheckIcon,
  ClipboardDocumentListIcon,
  Cog6ToothIcon,
  CpuChipIcon,
  CubeIcon,
  DocumentTextIcon,
  ExclamationTriangleIcon,
  FireIcon,
  FolderIcon,
  InformationCircleIcon,
  LightBulbIcon,
  LinkIcon,
  MagnifyingGlassIcon,
  PaperClipIcon,
  QuestionMarkCircleIcon,
  ServerStackIcon,
  ShieldCheckIcon,
  SparklesIcon,
  Squares2X2Icon,
  StarIcon,
  WrenchScrewdriverIcon,
  XCircleIcon,
} from '@heroicons/vue/24/outline'

/** 图标名 → heroicons 组件（24px outline 变体） */
const ICON_COMPONENTS: Record<string, Component> = {
  check: CheckIcon,
  warn: ExclamationTriangleIcon,
  error: XCircleIcon,
  info: InformationCircleIcon,
  game: CubeIcon,
  doc: DocumentTextIcon,
  folder: FolderIcon,
  mod: Squares2X2Icon,
  log: ClipboardDocumentListIcon,
  bug: BugAntIcon,
  search: MagnifyingGlassIcon,
  ai: SparklesIcon,
  download: ArrowDownTrayIcon,
  server: ServerStackIcon,
  shield: ShieldCheckIcon,
  wrench: WrenchScrewdriverIcon,
  question: QuestionMarkCircleIcon,
  settings: Cog6ToothIcon,
  refresh: ArrowPathIcon,
  link: LinkIcon,
  star: StarIcon,
  fire: FireIcon,
  tip: LightBulbIcon,
  chat: ChatBubbleLeftRightIcon,
  attach: PaperClipIcon,
  cpu: CpuChipIcon,
}

/** 图标别名 → 标准图标名 */
const ICON_ALIASES: Record<string, string> = {
  warning: 'warn',
  err: 'error',
  document: 'doc',
  mods: 'mod',
  logs: 'log',
  gear: 'settings',
  cog: 'settings',
  light: 'tip',
  spark: 'ai',
  paperclip: 'attach',
  clip: 'attach',
  chip: 'cpu',
}

function resolveName(name: string): string | null {
  const key = (name ?? '').toLowerCase()
  const normalized = ICON_ALIASES[key] ?? key
  return ICON_COMPONENTS[normalized] ? normalized : null
}

/** 图标名 → heroicons Vue 组件（未知返回 null） */
export function mdIconComponent(name: string): Component | null {
  const iconName = resolveName(name)
  return iconName ? ICON_COMPONENTS[iconName] : null
}

/**
 * 将 v-html 渲染出的图标占位符（`.md-icon.md-icon-名称`）替换为 Vue 图标组件。
 * 在包含 `renderMarkdown` 结果的容器上调用（ChatMessageItem 等）。
 * 名称通过 class 携带（markdown.ts 渲染器输出 `md-icon-${name}`），
 * 不依赖 data-* 属性，避免被 DOMPurify 剥离；已挂载的跳过，防止流式重复挂载。
 */
export function mountMdIcons(root: HTMLElement): void {
  root.querySelectorAll<HTMLElement>('.md-icon').forEach((el) => {
    if (el.querySelector('svg')) return
    const name =
      Array.from(el.classList)
        .find((c) => c.startsWith('md-icon-') && c.length > 8)
        ?.slice(8) ?? ''
    const comp = mdIconComponent(name)
    if (comp) render(h(comp, { class: 'h-full w-full' }), el)
  })
}
