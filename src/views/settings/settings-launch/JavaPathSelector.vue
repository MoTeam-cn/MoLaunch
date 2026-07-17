<script setup lang="ts">
/**
 * Java 路径选择器
 *
 * - 显示当前 Java 路径（含版本号徽章）
 * - 自动检测已安装 Java / 手动导入 javaw.exe
 * - 下拉列表展示已检测到的 Java（带版本/位数/JRE-JDK 信息）
 * - 点击外部自动收起下拉
 */
import { ref, watch } from 'vue'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import { showInfo, showSuccess } from '@/utils/toast'
import { showError } from '@/utils/modal'
import { ArrowPathIcon, DocumentPlusIcon } from '@heroicons/vue/24/outline'

const javaStore = useJavaStore()

const showJavaList = ref(false)
const javaSelectorRef = ref<HTMLElement | null>(null)
const detectingJava = ref(false)

function handleDocumentClick(e: MouseEvent) {
  if (javaSelectorRef.value && !javaSelectorRef.value.contains(e.target as Node)) {
    showJavaList.value = false
  }
}

watch(showJavaList, (open) => {
  if (open) {
    setTimeout(() => document.addEventListener('click', handleDocumentClick), 0)
  } else {
    document.removeEventListener('click', handleDocumentClick)
  }
})

async function handleAutoDetectJava() {
  if (detectingJava.value) return
  detectingJava.value = true
  try {
    showInfo('正在尝试搜索系统中存在的 Java...')
    await javaStore.refreshJava()
    if (javaStore.javaList.length > 0) {
      showSuccess(`已找到 ${javaStore.javaList.length} 个可用 Java，请自行展开下拉框选择`)
    } else {
      showInfo('未检测到已安装的 Java')
    }
  } finally {
    detectingJava.value = false
  }
}

async function handleManualImportJava() {
  try {
    const selected = await tauri.selectFile('选择 javaw.exe', [
      { name: 'Java 可执行文件 (javaw.exe)', extensions: ['exe'] },
    ])
    if (selected) {
      // 验证必须是 javaw.exe
      const fileName = selected.split('\\').pop()?.split('/').pop()?.toLowerCase()
      if (fileName !== 'javaw.exe') {
        showError('提示', '请选择 javaw.exe，而不是 java.exe')
        return
      }
      javaStore.setJavaPath(selected)
    }
  } catch (e) {
    console.error('Failed to select Java:', e)
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
        <button
          class="px-2 py-1 text-xs rounded transition-colors flex items-center"
          :class="detectingJava
            ? 'text-gray-400 bg-gray-50 cursor-not-allowed'
            : 'text-gray-600 bg-gray-100 hover:bg-gray-200'"
          :disabled="detectingJava"
          @click="handleAutoDetectJava"
        >
          <ArrowPathIcon class="w-3.5 h-3.5 mr-1" :class="{ 'animate-spin': detectingJava }" />
          {{ detectingJava ? '检测中...' : '自动检测' }}
        </button>
        <button
          class="px-2 py-1 text-xs text-gray-600 bg-gray-100 hover:bg-gray-200 rounded transition-colors flex items-center"
          @click="handleManualImportJava"
        >
          <DocumentPlusIcon class="w-3.5 h-3.5 mr-1" />
          手动导入
        </button>
      </div>
    </div>
    <!-- 选择器整体容器 -->
    <div ref="javaSelectorRef" class="relative">
      <!-- 输入框 -->
      <div
        class="flex items-center justify-between px-3 py-2 bg-white border rounded-lg cursor-pointer transition-colors"
        :class="showJavaList ? 'border-primary-500 ring-2 ring-primary-100' : 'border-gray-300 hover:border-gray-400'"
        @click="showJavaList = !showJavaList"
      >
        <div class="flex items-center min-w-0 mr-2">
          <span class="text-xs px-1.5 py-0.5 rounded bg-primary-100 text-primary-700 mr-2 shrink-0">
            {{ javaStore.javaPath ? 'Java ' + (javaStore.javaList.find(j => j.executable === javaStore.javaPath)?.major_version || '?') : '自动' }}
          </span>
          <span class="text-sm text-gray-900 truncate">
            {{ javaStore.javaPath || '启动时自动查找最佳 Java' }}
          </span>
        </div>
        <svg class="w-4 h-4 text-gray-400 shrink-0 transition-transform" :class="{ 'rotate-180': showJavaList }" viewBox="0 0 20 20" fill="currentColor">
          <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
        </svg>
      </div>
      <!-- 下拉列表 -->
      <transition
        enter-active-class="transition ease-out duration-150"
        enter-from-class="opacity-0 scale-y-95"
        enter-to-class="opacity-100 scale-y-100"
        leave-active-class="transition ease-in duration-100"
        leave-from-class="opacity-100 scale-y-100"
        leave-to-class="opacity-0 scale-y-95"
      >
        <div
          v-if="showJavaList"
          class="absolute z-10 w-full mt-1 bg-white border border-gray-300 rounded-lg shadow-lg overflow-hidden origin-top"
        >
          <!-- 自动检测 -->
          <div
            class="flex items-center justify-between px-3 py-2.5 hover:bg-primary-50 cursor-pointer transition-colors"
            :class="{ 'bg-primary-50': !javaStore.javaPath }"
            @click="javaStore.setJavaPath(''); showJavaList = false"
          >
            <div class="flex items-center">
              <span class="text-xs px-1.5 py-0.5 rounded bg-gray-100 text-gray-600 mr-2">自动</span>
              <span class="text-sm text-gray-700">启动时自动查找最佳 Java</span>
            </div>
            <span v-if="!javaStore.javaPath" class="text-primary-600 text-xs font-medium">当前</span>
          </div>
          <!-- 已安装列表 -->
          <template v-if="javaStore.javaList.length > 0">
            <div class="border-t border-gray-200 mx-3"></div>
            <div
              v-for="java in javaStore.javaList"
              :key="java.executable"
              class="flex items-center justify-between px-3 py-2.5 hover:bg-primary-50 cursor-pointer transition-colors"
              :class="{ 'bg-primary-50': javaStore.javaPath === java.executable }"
              @click="javaStore.setJavaPath(java.executable); showJavaList = false"
            >
              <div class="flex items-center min-w-0">
                <span class="text-xs px-1.5 py-0.5 rounded bg-blue-100 text-blue-700 mr-2 shrink-0">
                  {{ java.major_version }}
                </span>
                <div class="min-w-0">
                  <div class="text-sm text-gray-900 truncate">{{ java.executable }}</div>
                  <div class="text-xs text-gray-500">{{ java.version }} · {{ java.is_64bit ? '64位' : '32位' }} · {{ java.is_jre ? 'JRE' : 'JDK' }}</div>
                </div>
              </div>
              <span v-if="javaStore.javaPath === java.executable" class="text-primary-600 text-xs font-medium ml-2 shrink-0">当前</span>
            </div>
          </template>
          <div v-else class="px-3 py-2.5 border-t border-gray-100 text-xs text-gray-400">
            未检测到已安装的 Java，请点击「手动导入」
          </div>
        </div>
      </transition>
    </div>
  </div>
</template>
