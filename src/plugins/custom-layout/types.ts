/**
 * 自定义布局 Schema 类型定义
 *
 * JSON 和 XML 格式解析后统一转为此结构，复用同一套渲染组件。
 *
 * 支持的 section 类型：
 * - stat-grid：统计网格（卡片形式展示多个指标）
 * - list：数据列表（从数据源加载条目并按字段渲染）
 * - progress：进度条
 * - text：文本块
 * - divider：分割线
 */

/** 值格式化方式 */
export type ValueFormat = 'bytes' | 'number' | 'percent' | 'text'

/** 统计网格项 */
export interface StatItem {
  /** 标签 */
  label: string
  /** 值表达式（支持 {{dataSource.field}} 插值） */
  value: string
  /** 格式化方式 */
  format?: ValueFormat
  /** 颜色主题（primary / green / yellow / red / gray） */
  color?: string
}

/** 列表字段 */
export interface ListField {
  /** 数据条目中的字段名 */
  key: string
  /** 显示标签（可选，不填则不显示标签） */
  label?: string
  /** 格式化方式 */
  format?: ValueFormat
}

/** 布局 section 联合类型 */
export type LayoutSection =
  | { type: 'stat-grid'; columns?: number; items: StatItem[] }
  | { type: 'list'; title?: string; source: string; fields: ListField[] }
  | { type: 'progress'; label?: string; value: string; max?: string; color?: string; format?: ValueFormat }
  | { type: 'text'; content: string; variant?: 'default' | 'muted' | 'warning' }
  | { type: 'divider' }
  | {
      type: 'html'
      /** HTML 内容（内联字符串） */
      content: string
      /** 内联 JS 代码（可选，注入到 iframe 内执行） */
      script?: string
      /** 内联 CSS 样式（可选，注入到 iframe 内） */
      style?: string
      /** iframe 高度（px），默认 200 */
      height?: number
    }

/** 布局 Schema（JSON/XML 解析后的内部结构） */
export interface LayoutSchema {
  /** 面板标题（可选，不显示则不渲染标题栏） */
  title?: string
  /** 图标名（可选，对应 heroicons outline 图标名，如 "chart-bar"） */
  icon?: string
  /** section 列表 */
  sections: LayoutSection[]
}

/** 解析结果 */
export interface ParseResult {
  /** 解析成功后的 schema（失败时为 null） */
  schema: LayoutSchema | null
  /** 错误信息（成功时为 null） */
  error: string | null
}
