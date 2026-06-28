<script setup lang="ts">
import { ref, onMounted } from 'vue'
import { useSdkStore } from '@/stores/sdk'
import Select from '@/components/common/Select.vue'
import * as tauri from '@/utils/tauri'

const sdkStore = useSdkStore()
const logLevel = ref(3)
const configPath = ref('')

onMounted(async () => {
  try {
    configPath.value = await tauri.getConfigPath()
  } catch (e) {
    console.error('Failed to get config path:', e)
    configPath.value = '获取失败'
  }
})
</script>

<template>
  <div class="space-y-6">
    <!-- 系统设置 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">系统</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">日志级别</p>
            <p class="text-xs text-gray-500 mt-0.5">控制日志输出的详细程度</p>
          </div>
          <Select
            :model-value="logLevel"
            :options="[
              { label: '关闭', value: 0 },
              { label: '错误', value: 1 },
              { label: '警告', value: 2 },
              { label: '信息', value: 3 },
              { label: '调试', value: 4 },
              { label: '跟踪', value: 5 },
            ]"
            style="min-width: 100px"
            @update:model-value="logLevel = Number($event)"
          />
        </div>
      </div>
    </div>

    <!-- 配置信息 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">配置信息</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-3">
          <p class="text-sm text-gray-500 mb-1">配置文件路径</p>
          <p class="text-xs text-gray-900 font-mono bg-gray-50 px-3 py-2 rounded break-all">{{ configPath || '加载中...' }}</p>
        </div>
      </div>
    </div>

    <!-- SDK 信息 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">SDK 信息</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-3 flex items-center justify-between">
          <span class="text-sm text-gray-500">平台</span>
          <span class="text-sm text-gray-900">{{ sdkStore.status?.platform || '未知' }}</span>
        </div>
        <div class="px-5 py-3 flex items-center justify-between">
          <span class="text-sm text-gray-500">版本</span>
          <span class="text-sm text-gray-900">{{ sdkStore.version || '未加载' }}</span>
        </div>
        <div class="px-5 py-3 flex items-center justify-between">
          <span class="text-sm text-gray-500">状态</span>
          <span class="text-sm" :class="sdkStore.isReady ? 'text-green-600' : 'text-yellow-600'">
            {{ sdkStore.isReady ? '就绪' : '加载中' }}
          </span>
        </div>
        <div class="px-5 py-3 flex items-center justify-between">
          <span class="text-sm text-gray-500">设备 ID</span>
          <span class="text-sm text-gray-900 font-mono">{{ sdkStore.deviceId || '未获取' }}</span>
        </div>
        <div class="px-5 py-3 flex items-center justify-between">
          <span class="text-sm text-gray-500">库路径</span>
          <span class="text-sm text-gray-900 truncate ml-4 max-w-xs">{{ sdkStore.status?.library_path || '未知' }}</span>
        </div>
      </div>
    </div>
  </div>
</template>
