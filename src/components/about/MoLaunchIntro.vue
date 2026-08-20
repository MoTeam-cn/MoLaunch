<script setup lang="ts">
/**
 * MoLaunch 实现原理介绍组件
 *
 * 用于「设置 → 更多 → 关于」页面，介绍 MoLaunch 的技术实现细节。
 * 默认折叠，用户点击标题栏可展开查看 200 字实现说明。
 *
 * 内容涵盖：
 * - 技术栈选型（Tauri 2 + Vue 3 + Rust）
 * - 启动器核心实现（版本管理、Java 检测、游戏启动）
 * - 联机模块（FRP 隧道集成）
 * - UI 设计理念（单列布局 / Arco Design 紧凑风格）
 * - 数据存储与安全（本地加密、设备 ID 绑定）
 */
import { defineAsyncComponent } from 'vue'
import { ChevronDownIcon, BeakerIcon } from '@heroicons/vue/24/outline'
import { useCollapseAnimation } from '@/composables/useCollapseAnimation'
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))

const { isOpen, toggle, contentClass, iconClass } = useCollapseAnimation()
</script>

<template>
  <div class="overflow-hidden rounded-lg border border-gray-200 bg-white">
    <!-- 标题栏（点击展开/折叠） -->
    <!-- 保留原生 button：折叠头（w-full justify-between + 图标旋转 + aria-expanded），
         Button.vue 的 scoped size 类与布局不适合折叠头 -->
    <button
      class="flex w-full items-center justify-between px-4 py-3 text-left transition-colors hover:bg-gray-50"
      :aria-expanded="isOpen"
      @click="toggle"
    >
      <div class="flex items-center gap-2">
        <BeakerIcon class="h-4 w-4 text-primary-500" />
        <span class="text-sm font-semibold text-gray-800">MoLaunch 实现原理</span>
        <Tag size="small" color="gray">点击展开</Tag>
      </div>
      <ChevronDownIcon
        class="h-4 w-4 flex-none text-gray-400"
        :class="iconClass"
      />
    </button>

    <!-- 内容区（grid-template-rows 0fr→1fr 平滑高度过渡） -->
    <div :class="contentClass">
      <div class="overflow-hidden">
        <div class="border-t border-gray-100 px-4 py-3">
          <p class="text-[12px] leading-relaxed text-gray-600">
            <span class="font-semibold text-gray-800">MoLaunch</span> 基于
            <span class="font-medium text-primary-600">Tauri 2</span> 框架构建，前端使用
            <span class="font-medium text-primary-600">Vue 3 + TypeScript + Tailwind CSS</span>，
            后端使用 <span class="font-medium text-primary-600">Rust</span>，前后端通过 IPC 通信。
            启动游戏时，后端解析版本清单，按需从官方源或 BMCLAPI、MoCDN 等镜像下载
            client JAR、libraries 与 assets，并补齐模组加载器；随后扫描注册表、环境变量
            与磁盘定位 Java，按兼容性匹配最优运行时，组装 JVM 参数后创建游戏进程并持续监控。
            针对中文路径等环境，内置 Java Launch Wrapper 规避 JVM 参数乱码；针对
            LWJGL 3.4.1，按需注入 lwjgl-unsafe-agent 修复性能问题。游戏异常退出时，
            崩溃分析器结合日志、崩溃报告与 hs_err 文件提取证据，给出排查建议。此外，
            联机模块内置 FRP 隧道，免去端口映射直连；账号支持离线与微软登录，凭据加密
            存于本地；整合包支持 CurseForge、Modrinth、MCBBS 等格式一键安装，并提供
            多版本隔离，各版本的游戏数据互不干扰；缺失的 Java 运行时可由启动器自动下载
            安装。UI 采用单列布局与自研组件，风格统一。
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
