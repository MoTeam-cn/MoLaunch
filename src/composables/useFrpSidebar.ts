/**
 * FRP 联机侧边栏分类定义 composable（从 Online.vue 抽离，避免主文件超 300 行）
 *
 * 返回 frpCategory 常量供 Online.vue 的 categories computed 追加；含「认证中心」「运行日志」子项。
 */

import type { Component } from 'vue'
import {
  CloudIcon,
  ServerStackIcon,
  ArrowPathIcon,
  ShieldCheckIcon,
  DocumentTextIcon,
  BookOpenIcon,
} from '@heroicons/vue/24/outline'
import gofrpIcon from '@/assets/Common/gofrp-icon.png'

/** NavCategory 类型（与 Online.vue 内部定义一致） */
interface NavCategory {
  id: string
  label: string
  icon: Component
  /** 可选图标图片地址（传入时 NavSidebar 优先渲染 <img>，否则渲染 icon 组件） */
  image?: string
  desc?: string
  children?: NavCategory[]
  disabled?: boolean
}

/** FRP 联机分类（厂商列表 + 穿透管理 + 认证中心 + 运行日志） */
export const frpCategory: NavCategory = {
  id: 'frp',
  label: 'FRP 联机',
  icon: CloudIcon,
  image: gofrpIcon,
  desc: '管理 FRP 内网穿透厂商、隧道与 frpc 进程',
  children: [
    {
      id: 'providers',
      label: '厂商列表',
      icon: ServerStackIcon,
      desc: '浏览、安装与管理 Frp 厂商',
    },
    {
      id: 'tunnels',
      label: '穿透管理',
      icon: ArrowPathIcon,
      desc: '创建、启动与停止内网穿透隧道',
    },
    {
      id: 'auth',
      label: '认证中心',
      icon: ShieldCheckIcon,
      desc: '管理厂商 OAuth / Device Code 认证',
    },
    {
      id: 'logs',
      label: '运行日志',
      icon: DocumentTextIcon,
      desc: '查看 frpc 实时输出与历史日志',
    },
    {
      id: 'tutorial',
      label: '教程帮助',
      icon: BookOpenIcon,
      desc: '查看 FRP 厂商开发指南与启动器基础教程',
    },
  ],
}

/** Frp 子分类 ID 类型 */
export type FrpSubCategory = 'providers' | 'tunnels' | 'auth' | 'logs'
