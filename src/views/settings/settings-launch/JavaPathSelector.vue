<script setup lang="ts">
/**
 * Java 路径选择器
 *
 * - 基于项目自研 Select 组件实现（customOption + 自定义 #option slot）
 * - 显示当前 Java 路径（含版本号徽章）
 * - 自动检测已安装 Java / 手动导入 javaw.exe
 * - 下拉列表展示已检测到的 Java（带版本信息）
 */
import { ref, computed } from 'vue'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import { toastInfo, toastSuccess } from '@/utils/toast'
import { showError } from '@/utils/modal'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import { ArrowPathIcon, DocumentPlusIcon } from '@heroicons/vue/24/outline'
import { safeCall } from '@/utils/async'

const javaStore = useJavaStore()
const detectingJava = ref(false)

// 当前选中项的版本号（用于触发器徽章显示，空路径显示"自动"）
const currentMajorVersion = computed(() => {
  if (!javaStore.javaPath) return null
  return javaStore.javaList.find(j => j.executable === javaStore.javaPath)?.major_version ?? null
})

// Select 选项：自动 + 已检测的 Java 列表
const selectOptions = computed(() => [
  { label: '启动时自动查找最佳 Java', value: '' },
  ...javaStore.javaList.map(j => ({
    label: j.executable,
    value: j.executable,
    majorVersion: j.major_version,
    version: j.version,
  })),
])

function onSelectChange(value: string | number) {
  javaStore.setJavaPath(String(value))
}

async function handleAutoDetectJava() {
  if (detectingJava.value) return
  detectingJava.value = true
  try {
    toastInfo('正在尝试搜索系统中存在的 Java...')
    await javaStore.refreshJava()
    if (javaStore.javaList.length > 0) {
      toastSuccess(`已找到 ${javaStore.javaList.length} 个可用 Java，请自行展开下拉框选择`)
    } else {
      toastInfo('未检测到已安装的 Java')
    }
  } finally {
    detectingJava.value = false
  }
}

async function handleManualImportJava() {
  const selected = await safeCall(() => tauri.selectFile('选择 javaw.exe', [
    { name: 'Java 可执行文件 (javaw.exe)', extensions: ['exe'] },
  ]), 'select Java')
  if (selected) {
    // 验证必须是 javaw.exe
    const fileName = selected.split('\\').pop()?.split('/').pop()?.toLowerCase()
    if (fileName !== 'javaw.exe') {
      showError('提示', '请选择 javaw.exe，而不是 java.exe')
      return
    }
    javaStore.setJavaPath(selected)
  }
}
</script>

<template>
  <div class="px-5 py-4">
    <div class="flex items-center justify-between mb-2">
      <div>
        <p class="text-sm font-medium text-gray-900">Java 路径</p>
        <p class="text-xs text-gray-500 mt-0.5">选择用于启动游戏的 Java 运行时</p>
      </div>
      <div class="flex items-center gap-1.5">
        <Button
          type="secondary"
          size="mini"
          :disabled="detectingJava"
          @click="handleAutoDetectJava"
        >
          <template #icon><ArrowPathIcon class="w-3.5 h-3.5" :class="{ 'animate-spin': detectingJava }" /></template>
          {{ detectingJava ? '检测中...' : '自动检测' }}
        </Button>
        <Button
          type="secondary"
          size="mini"
          @click="handleManualImportJava"
        >
          <template #icon><DocumentPlusIcon class="w-3.5 h-3.5" /></template>
          手动导入
        </Button>
      </div>
    </div>

    <!-- Java 路径下拉选择器 -->
    <Select
      :model-value="javaStore.javaPath"
      :options="selectOptions"
      custom-option
      placeholder="启动时自动查找最佳 Java"
      @update:model-value="onSelectChange"
    >

      <!-- 触发器：版本徽章 + 路径 -->
      <template #selected>
        <div class="flex items-center min-w-0 gap-2">
          <span
            class="text-xs px-1.5 py-0.5 rounded shrink-0"
            :class="javaStore.javaPath
              ? 'bg-primary-100 text-primary-700'
              : 'bg-gray-100 text-gray-600'"
          >
            {{ javaStore.javaPath ? `Java ${currentMajorVersion ?? '?'}` : '自动' }}
          </span>
          <span class="text-sm text-gray-900 truncate">
            {{ javaStore.javaPath || '启动时自动查找最佳 Java' }}
          </span>
        </div>
      </template>

      <!-- 选项：自动项 + 已检测 Java 项 -->
      <template #option="{ option, selected }">
        <div v-if="option.value === ''" class="flex items-center min-w-0">
          <span class="text-xs px-1.5 py-0.5 rounded bg-gray-100 text-gray-600 mr-2 shrink-0">自动</span>
          <span class="text-sm text-gray-700 truncate">启动时自动查找最佳 Java</span>
        </div>
        <div v-else class="flex items-center min-w-0 w-full">
          <span class="text-xs px-1.5 py-0.5 rounded bg-blue-100 text-blue-700 mr-2 shrink-0">
            {{ option.majorVersion }}
          </span>
          <div class="min-w-0 flex-1">
            <div class="text-sm text-gray-900 truncate">{{ option.value }}</div>
            <div class="text-xs text-gray-500 truncate">{{ option.version }}</div>
          </div>
        </div>
      </template>

      <!-- 空状态 -->
      <template #empty>
        <div class="px-3 py-2.5 text-xs text-gray-400">
          未检测到已安装的 Java，请点击「手动导入」
        </div>
      </template>
    </Select>
  </div>
</template>
