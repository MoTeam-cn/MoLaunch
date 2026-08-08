import { renderMarkdown } from '@/utils/markdown'
import { parseVersion, type VersionChannel } from '@/utils/version'

export interface CommitItem {
  prefix: string | null
  scope: string | null
  body: string
  html: string
}

export interface ReleaseSegment {
  version: string | null
  channel: VersionChannel | null
  content: string
  html: string
  items: CommitItem[]
  hasListItems: boolean
}

const HEADER_RE = /^#{2,4}\s+(?:MoLaunch\s+)?v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)\s*$/
const SKIP_SCOPES = new Set([
  'skin', 'watcher', 'modrinth', 'searcher', 'download', 'image_cache',
  'java', 'parse', 'jvm_args', 'skin_resourcepack', 'signaling',
])
const SKIP_MARK_RE = /\s*!c\s*/gi

export const CHANNEL_LABELS: Record<Exclude<VersionChannel, 'stable'>, string> = {
  rc: '测试版', beta: '测试版', alpha: '测试版', canary: '测试版',
}

export const PREFIX_STYLES: Record<string, { label: string; color: string }> = {
  feat: { label: '新功能', color: 'green' }, fix: { label: '修复', color: 'red' },
  docs: { label: '文档', color: 'blue' }, refactor: { label: '重构', color: 'purple' },
  perf: { label: '性能', color: 'orange' }, chore: { label: '杂项', color: 'gray' },
  style: { label: '样式', color: 'cyan' }, test: { label: '测试', color: 'magenta' },
  build: { label: '构建', color: 'arcoblue' }, ci: { label: 'CI', color: 'gold' },
}

export function prefixStyle(prefix: string): { label: string; color: string } {
  return PREFIX_STYLES[prefix.toLowerCase()] ?? { label: '其他', color: 'gray' }
}

function parseItems(content: string): CommitItem[] {
  const items: CommitItem[] = []
  for (const line of content.split('\n')) {
    const trimmed = line.replace(SKIP_MARK_RE, '').trim()
    if (!trimmed) continue
    const match = trimmed.match(/^[-*]\s+(?:(\w+)(?:\(([^)]*)\))?:\s+)(.*)$/)
    if (match) {
      const prefix = match[1]
      const scope = match[2]?.trim() || null
      if (scope && SKIP_SCOPES.has(scope.toLowerCase())) continue
      const body = match[3].trim()
      items.push({ prefix, scope, body, html: renderMarkdown(body) })
    } else {
      items.push({ prefix: null, scope: null, body: trimmed, html: renderMarkdown(trimmed) })
    }
  }
  return items
}

function createSegment(version: string | null, content: string): ReleaseSegment {
  return {
    version,
    channel: version ? parseVersion(version).channel : null,
    content,
    html: renderMarkdown(content),
    items: parseItems(content),
    hasListItems: content.split('\n').some((line) => /^\s*[-*]\s+/.test(line)),
  }
}

export function parseReleaseNotes(notes: string): ReleaseSegment[] {
  const text = (notes ?? '').trim()
  if (!text) return []
  const lines = text.split('\n')
  const headers: { line: number; version: string }[] = []
  lines.forEach((line, index) => {
    const match = line.match(HEADER_RE)
    if (match) headers.push({ line: index, version: match[1] })
  })
  if (!headers.length) return [createSegment(null, text)]
  return headers.map((header, index) => createSegment(
    header.version,
    lines.slice(header.line + 1, headers[index + 1]?.line ?? lines.length).join('\n').trim(),
  ))
}
