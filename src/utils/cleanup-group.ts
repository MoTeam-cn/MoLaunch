/**
 * 清理分组展示工具：分组 key → 图标映射
 */
import { FolderIcon, CubeIcon } from '@heroicons/vue/24/outline'
import type { Component } from 'vue'

/** 全局分组用文件夹图标，版本分组用立方体图标 */
export function groupIcon(key: string): Component {
  return key === '全局' ? FolderIcon : CubeIcon
}
