/**
 * 模型图标显示模式（彩色 / 黑白）
 *
 * 全局响应式单例：设置页「AI 配置 → 模型图标」切换后立即生效，
 * 聊天页/头部模型图标组件（ModelIcon）统一读取本模块，避免逐层传递 props。
 * 初始默认彩色（color），与后端 AiConfig.icon_color_mode 保持一致；
 * 应用启动 / 设置页保存时通过 setIconColorMode 同步。
 */
import { ref } from 'vue'

export type IconColorMode = 'color' | 'mono'

export const iconColorMode = ref<IconColorMode>('color')

/** 同步图标显示模式（非法值回退彩色） */
export function setIconColorMode(mode: string | undefined | null): void {
  iconColorMode.value = mode === 'mono' ? 'mono' : 'color'
}
