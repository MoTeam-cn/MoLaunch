<script setup lang="ts">
/**
 * 更新日志时间线组件
 *
 * api-server 的 `check_for_update` 返回的 `notes` 为「当前版本 → 最新版本」的
 * 多版本合并 Markdown（最新在前，每段以 `## MoLaunch <version>` 标题开头，
 * 见 `.github/workflows/release.yml` 生成格式与 `api-server/src/services/updates.rs`
 * 的 `merge_release_notes`）。本组件按版本标题切分为独立节点，
 * 左侧竖线串联成时间线展示，替代"整段 Markdown 一次性渲染"。
 *
 * 版本识别：
 * - 识别 `## MoLaunch 0.3.2`、`## v0.3.2-rc1`、`### 0.3.2` 等标题变体；
 * - 复用 `utils/version.ts` 的 `parseVersion` 判断正式版 / rc / beta / alpha / canary，
 *   在版本徽章旁展示通道标签（正式版不展示）；
 * - 无法识别任何版本标题时（如历史单段数据）退化为整段 Markdown 渲染，不套时间线。
 *
 * 日志条目：
 * - 识别 `- fix(xxx): 内容 ([hash](url))` 形式的 conventional commits，
 *   按前缀（feat/fix/docs/refactor/perf/chore 等）渲染 Arco 风格 Tag 徽章
 *   （复用 common/Tag.vue，浅色底 + 语义色）；
 * - 非提交条目（普通列表项）原样保留在时间线内。
 *
 * 折叠：
 * - 点击版本标题可折叠/展开该版本日志，默认全部展开；
 * - 折叠仅作用于当前点击的版本，不影响其他节点；
 * - 展开/收起动画由通用 `Collapse` 组件提供（grid-template-rows 0fr→1fr 过渡）。
 */
import { reactive, computed } from 'vue'
import { ChevronDownIcon } from '@heroicons/vue/24/outline'
import Collapse from '@/components/common/Collapse.vue'
import Tag from '@/components/common/Tag.vue'
import { renderMarkdown, handleMarkdownLinkClick } from '@/utils/markdown'
import { parseVersion, type VersionChannel } from '@/utils/version'

const props = defineProps<{ notes: string }>()

interface ReleaseSegment {
  /** 版本号（不含 v 前缀）；无法识别时为 null */
  version: string | null
  /** 发布通道（正式版为 stable，无法识别版本时为 null） */
  channel: VersionChannel | null
  /** 该段正文（Markdown） */
  content: string
  /** 正文渲染为已消毒 HTML（renderMarkdown 内部 DOMPurify 消毒） */
  html: string
}

interface CommitItem {
  /** conventional commits 前缀（feat/fix/docs 等），无法识别时为 null */
  prefix: string | null
  /** 作用域（`fix(skin)` 中的 skin），无则为 null */
  scope: string | null
  /** 去除 `- prefix(scope): ` 前缀后的正文（含链接），Markdown */
  body: string
  /** 渲染为已消毒 HTML */
  html: string
}

/** 版本标题行：`##`~`####` + 可选 "MoLaunch" 前缀 + 可选 v + 语义化版本号 */
const HEADER_RE = /^#{2,4}\s+(?:MoLaunch\s+)?v?(\d+\.\d+\.\d+(?:[-+][0-9A-Za-z.-]+)?)\s*$/

/** 命中这些作用域（scope）的提交条目直接省略，避免细粒度 scope 拉长行宽 */
const SKIP_SCOPES = new Set([
  'skin',
  'watcher',
  'modrinth',
  'searcher',
  'download',
  'image_cache',
  'java',
  'parse',
  'jvm_args',
  'skin_resourcepack',
  'signaling',
])

/** 识别到 `!c`（跳过 CI 等标记）的提交条目默认忽略 */
const SKIP_MARK_RE = /!c/i

/** 通道中文名（stable 不展示标签） */
const CHANNEL_LABELS: Record<Exclude<VersionChannel, 'stable'>, string> = {
  rc: '测试版',
  beta: '测试版',
  alpha: '测试版',
  canary: '测试版',
}

/** conventional commits 前缀 → 展示文案 + Arco Tag 预设色 */
const PREFIX_STYLES: Record<string, { label: string; color: string }> = {
  feat: { label: '新功能', color: 'green' },
  fix: { label: '修复', color: 'red' },
  docs: { label: '文档', color: 'blue' },
  refactor: { label: '重构', color: 'purple' },
  perf: { label: '性能', color: 'orange' },
  chore: { label: '杂项', color: 'gray' },
  style: { label: '样式', color: 'cyan' },
  test: { label: '测试', color: 'magenta' },
  build: { label: '构建', color: 'arcoblue' },
  ci: { label: 'CI', color: 'gold' },
}

/** 默认前缀样式（未收录的 prefix） */
const DEFAULT_PREFIX_STYLE = { label: '其他', color: 'gray' }

/** 按版本标题切分合并后的 notes（顺序保持不变：最新在前） */
const segments = computed<ReleaseSegment[]>(() => {
  const text = (props.notes ?? '').trim()
  if (!text) return []

  const lines = text.split('\n')
  const headers: { line: number; version: string }[] = []
  lines.forEach((line, i) => {
    const m = line.match(HEADER_RE)
    if (m) headers.push({ line: i, version: m[1] })
  })

  // 无任何版本标题：整体作为一段，由调用方按整段 Markdown 渲染
  if (headers.length === 0) {
    return [{ version: null, channel: null, content: text, html: renderMarkdown(text) }]
  }

  return headers.map((h, idx) => {
    const content = lines
      .slice(h.line + 1, idx + 1 < headers.length ? headers[idx + 1].line : lines.length)
      .join('\n')
      .trim()
    const channel = parseVersion(h.version).channel
    return { version: h.version, channel, content, html: renderMarkdown(content) }
  })
})

/** 是否存在可识别的版本标题（决定走时间线还是整段渲染） */
const hasVersioned = computed(() => segments.value.some((s) => s.version !== null))

/** 各版本节点的折叠状态（按节点序号记录手动操作；默认全部展开） */
const collapsed = reactive<Record<number, boolean>>({})

/** 节点是否折叠（未手动操作时默认展开；点击仅切换当前版本，不影响其他节点） */
function isCollapsed(i: number): boolean {
  return collapsed[i] ?? false
}

/** 点击版本标题切换折叠（仅当前版本） */
function toggleCollapsed(i: number): void {
  collapsed[i] = !isCollapsed(i)
}

/** 将版本段正文切分为可识别前缀的提交条目 */
function parseItems(content: string): CommitItem[] {
  const lines = content.split('\n')
  const items: CommitItem[] = []
  for (const line of lines) {
    const trimmed = line.trim()
    if (!trimmed) continue
    // 识别到 `!c` 标记的条目默认忽略
    if (SKIP_MARK_RE.test(trimmed)) continue
    // 形如 `- fix(skin): 内容 ([hash](url))` 或 `- feat: 内容` 的条目
    const m = trimmed.match(/^[-*]\s+(?:(\w+)(?:\(([^)]*)\))?:\s+)(.*)$/)
    if (m) {
      const prefix = m[1]
      const scope = m[2]?.trim() || null
      // 命中省略 scope（skin/java/signaling 等）的条目直接省略
      if (scope && SKIP_SCOPES.has(scope.toLowerCase())) continue
      const body = m[3].trim()
      items.push({ prefix, scope, body, html: renderMarkdown(body) })
    } else {
      items.push({ prefix: null, scope: null, body: trimmed, html: renderMarkdown(trimmed) })
    }
  }
  return items
}

/** 正文是否包含列表条目（决定走条目渲染还是整段渲染） */
function hasListItems(content: string): boolean {
  return content.split('\n').some((l) => /^\s*[-*]\s+/.test(l))
}

/** 获取前缀的展示样式 */
function prefixStyle(prefix: string): { label: string; color: string } {
  const style = PREFIX_STYLES[prefix.toLowerCase()]
  return style ?? DEFAULT_PREFIX_STYLE
}
</script>

<template>
  <div v-if="segments.length">
    <!-- 无版本标题：退化为整段 Markdown 渲染（历史单段数据） -->
    <!-- eslint-disable-next-line vue/no-v-html -- renderMarkdown 已用 DOMPurify 消毒；链接点击由 handleMarkdownLinkClick 走系统浏览器 -->
    <div v-if="!hasVersioned" class="markdown-body text-xs text-gray-600 leading-relaxed" @click="handleMarkdownLinkClick" v-html="segments[0].html" />

    <!-- 时间线：左侧竖线串起各版本节点 -->
    <ol v-else class="release-timeline">
      <li
        v-for="(seg, i) in segments"
        :key="`${seg.version ?? 'raw'}-${i}`"
        class="timeline-item"
      >
        <span class="timeline-dot" :class="{ 'is-latest': i === 0 }" />
        <div
          class="timeline-head"
          role="button"
          tabindex="0"
          @click="toggleCollapsed(i)"
          @keydown.enter.prevent="toggleCollapsed(i)"
          @keydown.space.prevent="toggleCollapsed(i)"
        >
          <span class="timeline-chevron">
            <ChevronDownIcon
              class="h-3.5 w-3.5 transition-transform duration-300 ease-in-out"
              :class="isCollapsed(i) ? '-rotate-90' : ''"
            />
          </span>
          <Tag
            size="small"
            :color="i === 0 ? 'primary' : 'gray'"
            class="timeline-version"
          >
            v{{ seg.version }}
          </Tag>
          <!-- 通道标签：正式版（stable）不展示，测试版/rc/beta/alpha/canary 展示为黄色 -->
          <Tag
            v-if="seg.channel && seg.channel !== 'stable'"
            size="small"
            color="gold"
          >
            {{ CHANNEL_LABELS[seg.channel] }}
          </Tag>
          <Tag v-if="i === 0" size="small" color="primary">最新</Tag>
        </div>
        <!-- 该版本日志：Collapse 带动画展开/收起（默认展开） -->
        <Collapse :open="!isCollapsed(i)">
          <!-- 提交条目：类型 tag 徽章在内容前面，正文已剥离 fix: 等前缀 -->
          <ul v-if="hasListItems(seg.content) && parseItems(seg.content).length" class="commit-list">
            <li
              v-for="(item, j) in parseItems(seg.content)"
              :key="j"
              class="commit-item"
            >
              <Tag
                v-if="item.prefix"
                size="small"
                :color="prefixStyle(item.prefix).color"
              >
                {{ prefixStyle(item.prefix).label }}
              </Tag>
              <!-- eslint-disable-next-line vue/no-v-html -- renderMarkdown 已用 DOMPurify 消毒；链接点击由 handleMarkdownLinkClick 走系统浏览器 -->
              <div class="markdown-body text-xs text-gray-600 leading-relaxed" @click="handleMarkdownLinkClick" v-html="item.html" />
            </li>
          </ul>
          <!-- 非条目正文（纯文本/非列表内容）走整段渲染 -->
          <!-- eslint-disable-next-line vue/no-v-html -- renderMarkdown 已用 DOMPurify 消毒；链接点击由 handleMarkdownLinkClick 走系统浏览器 -->
          <div v-else-if="!hasListItems(seg.content)" class="markdown-body text-xs text-gray-600 leading-relaxed" @click="handleMarkdownLinkClick" v-html="seg.html" />
        </Collapse>
      </li>
    </ol>
  </div>
</template>

<style scoped>
/* ===== 时间线布局 ===== */
.release-timeline {
  list-style: none;
  margin: 0;
  padding: 0;
}

.timeline-item {
  position: relative;
  padding-left: 1.375rem;
  padding-bottom: 0.875rem;
}

/* 左侧竖线：贯穿所有节点，最后一项也一直延伸到最底部 */
.timeline-item::before {
  content: '';
  position: absolute;
  left: 0.25rem;
  top: 1rem;
  bottom: 0;
  width: 1px;
  background-color: #e5e6eb;
}

/* 最后一项延长一点底部空间，让竖线完整收尾 */
.timeline-item:last-child {
  padding-bottom: 1.25rem;
}

.timeline-dot {
  position: absolute;
  left: 0;
  top: 0.3125rem;
  width: 0.5625rem;
  height: 0.5625rem;
  border-radius: 9999px;
  background-color: #ffffff;
  border: 2px solid #d0d5dd;
  box-sizing: border-box;
}

/* 最新节点圆点高亮 */
.timeline-dot.is-latest {
  background-color: var(--color-primary-500, #4f6ef2);
  border-color: var(--color-primary-500, #4f6ef2);
}

.timeline-head {
  display: flex;
  align-items: center;
  gap: 0.375rem;
  margin-bottom: 0.375rem;
  cursor: pointer;
  user-select: none;
  border-radius: 0.375rem;
  padding: 0.125rem 0;
  transition: background-color 0.15s;
}

.timeline-head:hover {
  background-color: #f5f6f8;
}

.timeline-head:focus-visible {
  outline: 2px solid var(--color-primary-500, #165dff);
  outline-offset: 1px;
}

/* 折叠/展开箭头 */
.timeline-chevron {
  display: inline-flex;
  flex: none;
  color: #c0c4cc;
}

/* 版本号 Tag：最新版为 arcoblue 高亮，历史版默认 gray */
.timeline-version {
  font-weight: 600;
}

/* ===== 提交条目列表（每行：tag + 正文） ===== */
.commit-list {
  list-style: none;
  margin: 0;
  padding: 0;
}

.commit-item {
  display: flex;
  align-items: flex-start;
  gap: 0.375rem;
  margin-bottom: 0.25rem;
}

/* 项目符号圆点：保持 markdown 列表视觉，位于 tag 徽章之前 */
.commit-item::before {
  content: '';
  flex: none;
  width: 0.25rem;
  height: 0.25rem;
  margin-top: 0.4375rem;
  border-radius: 9999px;
  background-color: #c0c4cc;
}

.commit-item:last-child {
  margin-bottom: 0;
}

.commit-item .markdown-body {
  flex: 1;
  min-width: 0;
}

/* 前缀 Tag（Arco 风格，复用 common/Tag.vue）：固定宽度、与首行文字对齐 */
.commit-item :deep(.tag) {
  flex: none;
  margin-top: 0.125rem;
}

/* ===== Markdown 正文样式（作用于 v-html 渲染的节点） ===== */
.markdown-body :deep(p) {
  margin: 0 0 0.375rem;
}
.markdown-body :deep(p:last-child) {
  margin-bottom: 0;
}
.markdown-body :deep(h1),
.markdown-body :deep(h2),
.markdown-body :deep(h3),
.markdown-body :deep(h4) {
  margin: 0.5rem 0 0.25rem;
  font-size: 0.8125rem;
  font-weight: 600;
  color: #1d2129;
}
.markdown-body :deep(ul),
.markdown-body :deep(ol) {
  margin: 0.125rem 0 0.375rem;
  padding-left: 1.125rem;
  list-style: disc;
}
.markdown-body :deep(ol) {
  list-style: decimal;
}
.markdown-body :deep(li) {
  margin: 0.125rem 0;
}
.markdown-body :deep(code) {
  padding: 0.0625rem 0.25rem;
  border-radius: 0.25rem;
  background-color: #e5e6eb;
  font-family: inherit;
}
.markdown-body :deep(pre) {
  margin: 0.375rem 0;
  padding: 0.5rem 0.625rem;
  overflow-x: auto;
  border-radius: 0.375rem;
  background-color: #f2f3f5;
}
.markdown-body :deep(pre code) {
  padding: 0;
  background-color: transparent;
}
.markdown-body :deep(a) {
  color: var(--color-primary-500, #4f6ef2);
  text-decoration: underline;
}
</style>
