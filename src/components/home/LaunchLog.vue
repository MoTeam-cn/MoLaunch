<script setup lang="ts">
/**
 * 自定义内容区域（右侧）
 * 参考 PCL2 的 PanCustom 空栈设计，默认为空，后续可由用户/插件自定义内容
 * 目前仅保留启动进度步骤显示（启动中时临时展示）
 */

import { computed } from 'vue'
import { useVersionStore } from '@/stores/version'

const versionStore = useVersionStore()

// 启动阶段列表（仅启动中显示）
const stages = computed(() => {
  if (!versionStore.launchProgress) return []
  const names: Record<string, string> = {
    'Init': '初始化',
    'GetJava': '获取 Java',
    'Login': '登录验证',
    'ValidateFiles': '文件检查',
    'BuildArgs': '构建参数',
    'ExtractNatives': '解压原生库',
    'LaunchProcess': '启动进程',
    'WaitWindow': '等待窗口',
    'Finished': '完成',
  }
  const order = ['Init', 'GetJava', 'Login', 'ValidateFiles', 'BuildArgs', 'ExtractNatives', 'LaunchProcess', 'WaitWindow', 'Finished']
  const currentIdx = order.indexOf(versionStore.launchProgress.stage)
  return order.map((key, idx) => ({
    key,
    name: names[key] || key,
    status: idx < currentIdx ? 'done' : idx === currentIdx ? 'current' : 'pending',
  }))
})
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 启动中：显示进度步骤（临时浮层，启动完成后消失） -->
    <Transition name="fade">
      <div v-if="versionStore.launching && stages.length > 0" class="border-b border-primary-100 bg-primary-50/30 p-4">
        <div class="mb-3 flex items-center gap-2">
          <svg class="h-5 w-5 animate-spin text-primary-600" viewBox="0 0 24 24" fill="none">
            <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
            <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
          </svg>
          <span class="font-medium text-primary-700">正在启动游戏</span>
        </div>
        <div class="flex flex-wrap gap-x-6 gap-y-2">
          <div
            v-for="stage in stages"
            :key="stage.key"
            class="flex items-center gap-1.5 text-sm"
          >
            <div class="flex h-4 w-4 flex-none items-center justify-center">
              <svg v-if="stage.status === 'done'" class="h-3.5 w-3.5 text-green-500" viewBox="0 0 20 20" fill="currentColor">
                <path fill-rule="evenodd" d="M16.7 5.3a1 1 0 010 1.4l-8 8a1 1 0 01-1.4 0l-4-4a1 1 0 011.4-1.4L8 12.6l7.3-7.3a1 1 0 011.4 0z" clip-rule="evenodd" />
              </svg>
              <svg v-else-if="stage.status === 'current'" class="h-3.5 w-3.5 animate-spin text-primary-600" viewBox="0 0 24 24" fill="none">
                <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
              </svg>
              <div v-else class="h-1.5 w-1.5 rounded-full bg-gray-300" />
            </div>
            <span :class="{
              'text-gray-400': stage.status === 'pending',
              'font-medium text-primary-700': stage.status === 'current',
              'text-gray-500': stage.status === 'done'
            }">{{ stage.name }}</span>
          </div>
        </div>
      </div>
    </Transition>

    <!-- 自定义内容区（默认空，参考 PCL2 PanCustom） -->
    <div class="flex flex-1 items-center justify-center p-8">
      <div class="text-center text-gray-300">
        <svg class="mx-auto mb-3 h-16 w-16 opacity-40" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="1">
          <path d="M4 5a1 1 0 011-1h14a1 1 0 011 1v14a1 1 0 01-1 1H5a1 1 0 01-1-1V5z" stroke-linecap="round" stroke-linejoin="round" />
          <path d="M9 9h6M9 13h6M9 17h3" stroke-linecap="round" stroke-linejoin="round" />
        </svg>
        <p class="text-sm">自定义内容区</p>
        <p class="mt-1 text-xs text-gray-300">此处可由后续插件或用户配置填充</p>
      </div>
    </div>
  </div>
</template>

<style scoped>
.fade-enter-active, .fade-leave-active { transition: opacity 0.2s ease; }
.fade-enter-from, .fade-leave-to { opacity: 0; }
</style>
