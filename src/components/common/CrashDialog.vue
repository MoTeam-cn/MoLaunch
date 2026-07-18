<template>
  <Transition name="modal">
    <div v-if="visible" class="fixed inset-0 z-50 flex items-center justify-center p-4">
      <!-- 遮罩 -->
      <div class="absolute inset-0 bg-black/50" @click="handleClose" />

      <!-- 弹窗主体 -->
      <div class="relative bg-white rounded-xl shadow-2xl w-full max-w-2xl max-h-[85vh] flex flex-col overflow-hidden">
        <!-- 标题栏 -->
        <div class="flex items-center gap-3 px-6 py-4 border-b border-gray-200">
          <div class="flex h-10 w-10 items-center justify-center rounded-full bg-red-100 flex-none">
            <svg class="h-6 w-6 text-red-600" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-3L13.732 4c-.77-1.333-2.694-1.333-3.464 0L3.34 16c-.77 1.333.192 3 1.732 3z" />
            </svg>
          </div>
          <div class="min-w-0 flex-1">
            <h2 class="text-lg font-semibold text-gray-900">Minecraft 出现错误</h2>
            <p class="text-xs text-gray-500 mt-0.5">游戏已异常退出，以下是崩溃分析结果</p>
          </div>
          <button
            class="text-gray-400 hover:text-gray-600 transition-colors flex-none"
            @click="handleClose"
          >
            <svg class="h-5 w-5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M6 18L18 6M6 6l12 12" />
            </svg>
          </button>
        </div>

        <!-- 内容区域（可滚动）-->
        <div class="flex-1 overflow-y-auto px-6 py-4 space-y-4">
          <!-- 崩溃原因 -->
          <div>
            <div class="flex items-center gap-2 mb-2">
              <span class="text-xs font-semibold uppercase tracking-wide text-gray-500">崩溃原因</span>
              <span
                class="inline-flex items-center px-2 py-0.5 rounded text-xs font-medium"
                :class="categoryColor"
              >
                {{ categoryLabel }}
              </span>
            </div>
            <p class="text-sm text-gray-900 font-medium">{{ crashInfo.reason }}</p>
            <p v-if="crashInfo.problematic_mod" class="text-xs text-gray-500 mt-1">
              相关 Mod：{{ crashInfo.problematic_mod }}
            </p>
          </div>

          <!-- 建议 -->
          <div class="bg-blue-50 border border-blue-200 rounded-lg p-4">
            <div class="flex items-start gap-2">
              <svg class="h-5 w-5 text-blue-500 flex-none mt-0.5" fill="none" viewBox="0 0 24 24" stroke="currentColor">
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z" />
              </svg>
              <div class="min-w-0 flex-1">
                <p class="text-xs font-semibold text-blue-900 mb-1">建议</p>
                <p class="text-sm text-blue-800 whitespace-pre-line">{{ crashInfo.suggestion }}</p>
              </div>
            </div>
          </div>

          <!-- 崩溃报告路径 -->
          <div v-if="crashInfo.crash_report_path" class="bg-gray-50 rounded-lg p-3">
            <p class="text-xs text-gray-500 mb-1">崩溃报告文件</p>
            <div class="flex items-center gap-2">
              <code class="flex-1 text-xs text-gray-700 break-all">{{ crashInfo.crash_report_path }}</code>
              <button
                class="flex-none px-2 py-1 text-xs text-primary-600 hover:bg-primary-50 rounded transition-colors"
                @click="openCrashReport"
              >
                打开
              </button>
            </div>
          </div>

          <!-- 日志详情（可折叠）-->
          <div v-if="crashInfo.log_lines.length > 0 || crashInfo.log_tail.length > 0">
            <button
              class="flex items-center gap-1 text-xs font-semibold text-gray-500 hover:text-gray-700 transition-colors"
              @click="showDetails = !showDetails"
            >
              <svg
                class="h-4 w-4 transition-transform"
                :class="{ 'rotate-90': showDetails }"
                fill="none" viewBox="0 0 24 24" stroke="currentColor"
              >
                <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
              </svg>
              日志详情
            </button>

            <div v-if="showDetails" class="mt-2 space-y-2">
              <!-- 错误日志行 -->
              <div v-if="crashInfo.log_lines.length > 0">
                <p class="text-xs text-gray-500 mb-1">错误日志（{{ crashInfo.log_lines.length }} 行）</p>
                <div class="bg-gray-900 rounded-lg p-3 max-h-48 overflow-y-auto">
                  <pre class="text-xs text-red-300 whitespace-pre-wrap break-all font-mono">{{ crashInfo.log_lines.join('\n') }}</pre>
                </div>
              </div>

              <!-- 游戏日志尾部 -->
              <div v-if="crashInfo.log_tail.length > 0">
                <p class="text-xs text-gray-500 mb-1">游戏日志尾部（{{ crashInfo.log_tail.length }} 行）</p>
                <div class="bg-gray-900 rounded-lg p-3 max-h-48 overflow-y-auto">
                  <pre class="text-xs text-gray-300 whitespace-pre-wrap break-all font-mono">{{ crashInfo.log_tail.join('\n') }}</pre>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 按钮栏 -->
        <div class="flex items-center justify-end gap-2 px-6 py-4 border-t border-gray-200 bg-gray-50">
          <button
            v-if="crashInfo.crash_report_path"
            class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
            @click="openCrashReport"
          >
            查看输出
          </button>
          <button
            class="px-4 py-2 text-sm font-medium text-gray-700 bg-white border border-gray-300 rounded-lg hover:bg-gray-50 transition-colors"
            @click="exportReport"
          >
            导出错误报告
          </button>
          <button
            class="px-4 py-2 text-sm font-medium text-white bg-primary-500 rounded-lg hover:bg-primary-600 transition-colors"
            @click="handleClose"
          >
            确定
          </button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import { open } from '@tauri-apps/plugin-shell'
import { showSuccess, showError } from '@/utils/toast'
import { saveFile } from '@/utils/api/system'
import type { CrashInfo } from '@/utils/crashDialog'

const visible = ref(false)
const showDetails = ref(false)
const crashInfo = ref<CrashInfo | null>(null)

const categoryColor = computed(() => {
  const map: Record<string, string> = {
    Java: 'bg-orange-100 text-orange-700',
    Memory: 'bg-yellow-100 text-yellow-700',
    Graphics: 'bg-purple-100 text-purple-700',
    Mod: 'bg-blue-100 text-blue-700',
    Forge: 'bg-red-100 text-red-700',
    Fabric: 'bg-green-100 text-green-700',
    OptiFine: 'bg-indigo-100 text-indigo-700',
    Unknown: 'bg-gray-100 text-gray-700',
  }
  return map[crashInfo.value?.category ?? 'Unknown'] ?? 'bg-gray-100 text-gray-700'
})

const categoryLabel = computed(() => {
  const map: Record<string, string> = {
    Java: 'Java',
    Memory: '内存',
    Graphics: '显卡',
    Mod: 'Mod',
    Forge: 'Forge',
    Fabric: 'Fabric',
    OptiFine: 'OptiFine',
    Unknown: '未知',
  }
  return map[crashInfo.value?.category ?? 'Unknown'] ?? '未知'
})

function show(info: CrashInfo) {
  crashInfo.value = info
  showDetails.value = false
  visible.value = true
}

function handleClose() {
  visible.value = false
}

async function openCrashReport() {
  if (!crashInfo.value?.crash_report_path) return
  try {
    await open(crashInfo.value.crash_report_path)
  } catch (e) {
    showError('打开文件失败', String(e))
  }
}

async function exportReport() {
  if (!crashInfo.value) return
  try {
    // 将崩溃信息写入临时文件并打开保存对话框
    const lines: string[] = [
      '===== MoLaunch 崩溃报告 =====',
      `时间: ${new Date().toLocaleString()}`,
      `崩溃原因: ${crashInfo.value.reason}`,
      `类别: ${crashInfo.value.category}`,
      '',
      '--- 建议 ---',
      crashInfo.value.suggestion,
      '',
    ]
    if (crashInfo.value.problematic_mod) {
      lines.push(`--- 相关 Mod ---`, crashInfo.value.problematic_mod, '')
    }
    if (crashInfo.value.log_lines.length > 0) {
      lines.push('--- 错误日志 ---', ...crashInfo.value.log_lines, '')
    }
    if (crashInfo.value.log_tail.length > 0) {
      lines.push('--- 游戏日志尾部 ---', ...crashInfo.value.log_tail, '')
    }

    // 使用 Tauri 的保存文件对话框
    const filePath = await saveFile(
      '选择保存位置',
      `crash-report-${Date.now()}.txt`,
      [{ name: '文本文件', extensions: ['txt'] }],
    )
    if (!filePath) return

    await invoke('plugin:fs|write_text_file', {
      path: filePath,
      contents: lines.join('\n'),
    })
    showSuccess('导出成功', `错误报告已保存到：${filePath}`)
  } catch (e) {
    showError('导出失败', String(e))
  }
}

defineExpose({ show })
</script>
