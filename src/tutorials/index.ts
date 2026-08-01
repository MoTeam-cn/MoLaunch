/**
 * 教程内容索引
 *
 * 教程以 Markdown 文件存储在 src/tutorials/ 目录下，
 * 通过 Vite ?raw 导入为字符串，经 openTutorialWindow 传入
 * picker 子窗口用 marked.min.js 渲染。
 *
 * 新增教程步骤：
 * 1. 在本目录创建 *.md 文件
 * 2. 下方 import 并在 TUTORIALS 数组中添加条目
 */

import frpProviderGuide from './frp-provider-guide.md?raw'
import launcherBasics from './launcher-basics.md?raw'

/** 教程分类 */
export type TutorialCategory = '基础' | 'FRP 开发'

/** 教程元数据 */
export interface TutorialMeta {
  /** 唯一标识 */
  id: string
  /** 显示标题 */
  title: string
  /** 简短描述（卡片摘要） */
  description: string
  /** 分类 */
  category: TutorialCategory
  /** Markdown 内容 */
  content: string
}

/** 所有教程列表 */
export const TUTORIALS: TutorialMeta[] = [
  {
    id: 'launcher-basics',
    title: 'MoLaunch 使用基础',
    description: '从安装版本、启动游戏到联机功能的入门指南',
    category: '基础',
    content: launcherBasics,
  },
  {
    id: 'frp-provider-guide',
    title: 'FRP 厂商开发指南',
    description: 'manifest.json 清单格式、认证配置（OAuth2/Device Code/API Key）、网络与进程权限',
    category: 'FRP 开发',
    content: frpProviderGuide,
  },
]
