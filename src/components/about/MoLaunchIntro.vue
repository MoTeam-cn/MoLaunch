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
import { ref } from 'vue'
import { ChevronDownIcon, BeakerIcon } from '@heroicons/vue/24/outline'
import Tag from '@/components/common/Tag.vue'

const isOpen = ref(false)

function toggle() {
  isOpen.value = !isOpen.value
}
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
        class="h-4 w-4 flex-none text-gray-400 transition-transform duration-300 ease-in-out"
        :class="isOpen ? 'rotate-180' : ''"
      />
    </button>

    <!-- 内容区（grid-template-rows 0fr→1fr 平滑高度过渡） -->
    <div
      class="grid transition-all duration-300 ease-in-out"
      :class="isOpen ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
    >
      <div class="overflow-hidden">
        <div class="border-t border-gray-100 px-4 py-3">
          <p class="text-[12px] leading-relaxed text-gray-600">
            <span class="font-semibold text-gray-800">MoLaunch</span> 基于
            <span class="font-medium text-primary-600">Tauri 2</span> 框架构建，前端使用
            <span class="font-medium text-primary-600">Vue 3 + TypeScript + Tailwind CSS</span>，
            后端使用 <span class="font-medium text-primary-600">Rust</span> 实现。启动器通过 Rust
            原生调用系统进程启动 Minecraft，解析版本 JSON 并自动下载缺失依赖（client JAR、
            libraries、assets、模组加载器）。Java 检测模块扫描注册表、环境变量与本地磁盘，
            按版本号自动匹配最优运行时。联机功能内置 FRP 隧道客户端，通过 SDK 动态库
            （Windows dll / macOS dylib / Linux so，编译时嵌入二进制并释放到临时目录加载）
            创建隧道，无需用户手动配置端口映射。账号系统支持离线登录与微软登录，
            凭据通过设备 ID 派生密钥加密后存储在本地。UI 设计采用单列布局与
            Arco Design 的紧凑组件风格，自研 Select / Button / Tooltip 等组件以保持视觉一致性。
          </p>
        </div>
      </div>
    </div>
  </div>
</template>
