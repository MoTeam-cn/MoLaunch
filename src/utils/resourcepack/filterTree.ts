/**
 * 资源包文件树搜索过滤（纯函数）
 *
 * 按关键字递归过滤结构树：文件命中（文件名/相对路径，不区分大小写）则保留，
 * 目录只要存在命中的子孙节点即保留；返回过滤后的新树（不改原树）。
 */
import type { RpTreeNode } from '@/utils/api/tools'

/** 关键字归一化：去空格、转小写 */
export function normalizeKeyword(keyword: string): string {
  return keyword.trim().toLowerCase()
}

/** 文件是否命中关键字（匹配 name 或 rel_path） */
export function fileMatches(node: RpTreeNode, kw: string): boolean {
  return node.name.toLowerCase().includes(kw) || node.rel_path.toLowerCase().includes(kw)
}

/** 递归过滤：命中或含命中子孙的节点保留，返回新树；无命中返回 null */
export function filterTreeNode(node: RpTreeNode, kw: string): RpTreeNode | null {
  if (!kw) return node
  if (node.kind === 'file') {
    return fileMatches(node, kw) ? node : null
  }
  const kept: RpTreeNode[] = []
  for (const child of node.children) {
    const filtered = filterTreeNode(child, kw)
    if (filtered) kept.push(filtered)
  }
  if (kept.length === 0 && !fileMatches(node, kw)) return null
  return { ...node, children: kept }
}

/** 收集过滤结果中需要展开的目录路径（命中文件的祖先目录 + 目录自身命中） */
export function collectExpandPaths(node: RpTreeNode, kw: string, out: string[] = []): string[] {
  if (!kw) return out
  if (node.kind === 'dir') {
    const children = node.children.filter((c) => filterTreeNode(c, kw))
    if (children.length > 0) out.push(node.rel_path)
    for (const child of children) collectExpandPaths(child, kw, out)
  }
  return out
}
