/**
 * 崩溃弹窗服务
 *
 * - 右侧 Drawer 抽屉展示崩溃原因、建议、崩溃报告与日志详情
 * - 按钮：查看报告 / 导出报告 / 关闭
 */

import { ref } from 'vue'
import type { CrashInfo } from '@/types/version'

export type { CrashCategory, CrashInfo } from '@/types/version'

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
 * 自动触发崩溃分析后弹出
 */
export function showCrashDialog(info: CrashInfo) {
  crashDialogRef.value?.show(info)
}
