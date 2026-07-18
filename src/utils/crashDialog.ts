/**
 * 崩溃弹窗服务
 *
 * 参考 PCL2 ModCrash.vb Output 方法：
 * - 弹窗标题："Minecraft 出现错误"
 * - 展示崩溃原因、建议
 * - 按钮：确定 / 查看输出 / 导出错误报告
 */

import { ref } from 'vue'

/** 崩溃类别 */
export type CrashCategory = 'Java' | 'Memory' | 'Graphics' | 'Mod' | 'Forge' | 'Fabric' | 'OptiFine' | 'Unknown'

/** 崩溃详情 */
export interface CrashInfo {
  reason: string
  category: CrashCategory
  log_lines: string[]
  suggestion: string
  problematic_mod: string | null
  crash_report_path?: string
  log_tail: string[]
}

/** CrashDialog 组件实例对外暴露的接口 */
export interface CrashDialogInstance {
  show: (info: CrashInfo) => void
}

const crashDialogRef = ref<CrashDialogInstance | null>(null)

export function setCrashDialogRef(ref: CrashDialogInstance | null) {
  crashDialogRef.value = ref
}

/**
 * 显示崩溃弹窗
 * 参考 PCL2 ModCrash.vb Output：自动触发崩溃分析后弹出
 */
export function showCrashDialog(info: CrashInfo) {
  crashDialogRef.value?.show(info)
}
