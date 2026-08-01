/**
 * 教程元数据索引
 *
 * 教程内容以硬编码 HTML 存放在 src-tauri/resources/templates/ 目录，
 * 通过 picker 子窗口直接加载（无需 marked.min.js 渲染）。
 *
 * 新增教程只需：
 * 1. 在 src-tauri/resources/templates/ 创建 tutorial-xxx.html
 * 2. 在 src-tauri/src/resources.rs 的 embedded_text 注册
 * 3. 在 src/config/picker-templates.ts 注册模板配置
 * 4. 在此文件追加 TutorialMeta 项
 */

export type TutorialCategory = '基础' | 'FRP 开发'

export interface TutorialMeta {
  id: string
  title: string
  description: string
  category: TutorialCategory
  /** picker 模板名（对应 templates/tutorial-xxx.html） */
  template: string
}

export const TUTORIALS: TutorialMeta[] = [
  {
    id: 'launcher-basics',
    title: 'MoLaunch 使用基础',
    description: '从安装版本、启动游戏到联机功能的入门指南',
    category: '基础',
    template: 'tutorial-basics',
  },
  {
    id: 'frp-provider-guide',
    title: 'FRP 厂商开发指南',
    description: 'manifest.json 清单格式、认证配置（OAuth2/Device Code/API Key）、网络与进程权限',
    category: 'FRP 开发',
    template: 'tutorial-frp',
  },
]
