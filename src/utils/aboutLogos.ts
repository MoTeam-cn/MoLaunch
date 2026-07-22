/**
 * 关于页面的共享工具：Logo 映射 + 外链打开
 */
import { open as shellOpen } from '@tauri-apps/plugin-shell'

// Logo 资源：AboutIcon 目录下所有图片，构建 文件名 -> URL 映射表
// 使用 import.meta.glob eager 模式预加载，运行时按文件名查 URL
const logoModules = import.meta.glob<{ default: string }>('@/assets/AboutIcon/*', { eager: true })
const logoMap: Record<string, string> = {}
for (const [path, mod] of Object.entries(logoModules)) {
  const filename = path.split('/').pop() || ''
  logoMap[filename] = mod.default
}

/** 根据后端返回的 logo 文件名获取前端图片 URL */
export function resolveLogo(filename: string): string {
  return logoMap[filename] || ''
}

/** 打开外部链接（静默处理错误） */
export function openLink(url: string): void {
  shellOpen(url).catch(() => {})
}
