/**
 * Frp 侧边栏分类定义
 *
 * 从 Online.vue 抽离 frp 分类逻辑，避免 Online.vue 超 300 行约束。
 * 返回 frpCategory 常量，供 Online.vue 的 categories computed 追加。
 *
 * 阶段二追加「认证中心」（占位）和「运行日志」子项。
 * 认证中心在阶段三完整实现，阶段二可点击但仅显示"开发中"占位 UI。
 */

import type { Component } from 'vue'
import {
  CloudIcon,
  ServerStackIcon,
  ArrowPathIcon,
  ShieldCheckIcon,
  DocumentTextIcon,
} from '@heroicons/vue/24/outline'

/** NavCategory 类型（与 Online.vue 内部定义一致） */
interface NavCategory {
  id: string
  label: string
  icon: Component
  desc?: string
  children?: NavCategory[]
  disabled?: boolean
}

/** Frp 管理分类（厂商列表 + 穿透管理 + 认证中心 + 运行日志） */
export const frpCategory: NavCategory = {
  id: 'frp',
  label: 'Frp 管理',
  icon: CloudIcon,
  desc: '管理 Frp 内网穿透厂商、隧道与 frpc 进程',
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
  ],
}

/** Frp 子分类 ID 类型 */
export type FrpSubCategory = 'providers' | 'tunnels' | 'auth' | 'logs'
