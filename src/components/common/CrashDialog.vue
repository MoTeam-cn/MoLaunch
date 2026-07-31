<template>
  <Transition name="crash-modal">
    <div v-if="visible" class="fixed inset-0 z-[9999] flex items-center justify-center p-6">
      <!-- 遮罩（崩溃弹窗是普通蓝色主题不是警告） -->
      <div class="absolute inset-0 bg-black/35" @click="handleClose" />

      <!-- 弹窗主体 -->
      <div class="crash-dialog relative bg-dialog-bg rounded-lg shadow-[0_4px_20px_rgba(52,61,74,0.5)] w-full max-w-2xl max-h-[85vh] flex flex-col overflow-hidden">
        <!-- 标题区 + 分割线 -->
        <div class="px-7 pt-6 pb-3">
          <h2 class="text-[23px] font-normal text-brand-2 leading-tight">Minecraft 出现错误</h2>
          <!-- 分割线 -->
          <div class="mt-3 h-0.5 bg-brand-2 rounded-full"></div>
        </div>

        <!-- 内容区 -->
        <div class="flex-1 overflow-y-auto px-7 py-4">
          <!-- 崩溃原因段落 -->
          <p class="text-[15px] leading-[18px] text-dialog-caption mb-4">
            游戏已异常退出，以下是崩溃分析结果：
          </p>

          <!-- 原因详情（加粗，深色文字） -->
          <p class="text-[15px] leading-[18px] text-brand-1 font-medium mb-2">
            {{ crashInfo?.reason || '未知原因' }}
          </p>

          <!-- 相关 Mod（如果有） -->
          <p v-if="crashInfo?.problematic_mod" class="text-[15px] leading-[18px] text-dialog-caption mb-4">
            相关 Mod：{{ crashInfo.problematic_mod }}
          </p>

          <!-- 建议（纯文本段落） -->
          <p class="text-[15px] leading-[18px] text-dialog-caption whitespace-pre-line mb-4">
            {{ crashInfo?.suggestion || '' }}
          </p>

          <!-- 崩溃报告文件路径（如果有，可点击打开） -->
          <div v-if="crashInfo?.crash_report_path" class="mb-4">
            <p class="text-[15px] leading-[18px] text-dialog-caption mb-1">崩溃报告文件：</p>
            <!-- 保留原生 button：文件路径文本链接（break-all 换行 + brand 色系），
                 Button.vue 的 scoped size 类与样式不适合文本链接场景 -->
            <button
              class="text-[15px] text-brand-2 hover:text-brand-3 hover:underline text-left break-all transition-colors"
              @click="openCrashReport"
            >
              {{ crashInfo.crash_report_path }}
            </button>
          </div>

          <!-- 日志详情（可折叠） -->
          <div v-if="hasLogDetails" class="border-t border-gray-200 pt-3 mt-4">
            <Button
              type="text"
              size="small"
              @click="showDetails = !showDetails"
            >
              <template #icon>
                <svg
                  class="h-4 w-4 transition-transform"
                  :class="{ 'rotate-90': showDetails }"
                  fill="none" viewBox="0 0 24 24" stroke="currentColor"
                >
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 5l7 7-7 7" />
                </svg>
              </template>
              查看日志详情
            </Button>

            <div v-if="showDetails" class="mt-3 space-y-3">
              <!-- 错误日志 -->
              <div v-if="errorLines.length > 0">
                <p class="text-[13px] text-dialog-caption mb-1">错误日志（{{ errorLines.length }} 行）</p>
                <div class="bg-gray-900 rounded-md p-3 max-h-48 overflow-y-auto">
                  <pre class="text-xs text-red-300 whitespace-pre-wrap break-all font-mono">{{ errorLines.join('\n') }}</pre>
                </div>
              </div>

              <!-- 游戏日志尾部 -->
              <div v-if="logTail.length > 0">
                <p class="text-[13px] text-dialog-caption mb-1">游戏日志尾部（{{ logTail.length }} 行）</p>
                <div class="bg-gray-900 rounded-md p-3 max-h-48 overflow-y-auto">
                  <pre class="text-xs text-gray-300 whitespace-pre-wrap break-all font-mono">{{ logTail.join('\n') }}</pre>
                </div>
              </div>
            </div>
          </div>
        </div>

        <!-- 按钮栏（右对齐，3 个按钮） -->
        <!-- 按钮配色：Highlight=#0b5bcb, Normal=#343d4a, hover=#1370f3+bg#e0eafd -->
        <div class="flex items-center justify-end gap-3 px-7 py-4 border-t border-gray-200 bg-gray-50">
          <!-- 查看输出按钮（Normal 态：深灰蓝边框，hover 亮蓝） -->
          <Button
            v-if="crashInfo?.crash_report_path"
            type="outline"
            size="small"
            @click="openCrashReport"
          >
            查看输出
          </Button>
          <!-- 导出错误报告按钮（Normal 态） -->
          <Button
            type="outline"
            size="small"
            @click="exportReport"
          >
            导出错误报告
          </Button>
          <!-- 确定按钮（Highlight 态：主蓝边框 #0b5bcb，hover 亮蓝） -->
          <Button
            type="primary"
            size="small"
            @click="handleClose"
          >
            确定
          </Button>
        </div>
      </div>
    </div>
  </Transition>
</template>

<script setup lang="ts">
import { ref, computed } from 'vue'
import { open } from '@tauri-apps/plugin-shell'
import { toastSuccess, toastError } from '@/utils/toast'
import { pickSavePath } from '@/utils/fileDialog'
import { writeTextFile } from '@/utils/api/system'
import type { CrashInfo } from '@/utils/crashDialog'
import Button from '@/components/common/Button.vue'

const visible = ref(false)
const showDetails = ref(false)
const crashInfo = ref<CrashInfo | null>(null)

// 防御性处理：crashInfo 可能为 null 或字段 undefined
const errorLines = computed<string[]>(() => crashInfo.value?.log_lines ?? [])
const logTail = computed<string[]>(() => crashInfo.value?.log_tail ?? [])
const hasLogDetails = computed(() => errorLines.value.length > 0 || logTail.value.length > 0)

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
    toastError('打开文件失败：' + String(e))
  }
}

async function exportReport() {
  if (!crashInfo.value) return
  try {
    const info = crashInfo.value
    const lines: string[] = [
      '===== MoLaunch 崩溃报告 =====',
      `时间: ${new Date().toLocaleString()}`,
      `崩溃原因: ${info.reason}`,
      `类别: ${info.category}`,
      '',
      '--- 建议 ---',
      info.suggestion,
      '',
    ]
    if (info.problematic_mod) {
      lines.push(`--- 相关 Mod ---`, info.problematic_mod, '')
    }
    if (info.log_lines && info.log_lines.length > 0) {
      lines.push('--- 错误日志 ---', ...info.log_lines, '')
    }
    if (info.log_tail && info.log_tail.length > 0) {
      lines.push('--- 游戏日志尾部 ---', ...info.log_tail, '')
    }

    const filePath = await pickSavePath({
      title: '选择保存位置',
      defaultPath: `crash-report-${Date.now()}.txt`,
      filters: [{ name: '文本文件', extensions: ['txt'] }],
    })
    if (!filePath) return

    await writeTextFile(filePath, lines.join('\n'))
    toastSuccess('导出成功', `错误报告已保存到：${filePath}`)
  } catch (e) {
    toastError('导出失败', String(e))
  }
}

defineExpose({ show })
</script>

<style scoped>
/* 进入动画：
   - 背景遮罩：透明 → rgba(0,0,0,0.353)
   - 弹窗：透明度 0→1（120ms），Y 偏移 40→0（300ms，回弹缓动 OutBack） */
.crash-modal-enter-active {
  transition: opacity 0.2s ease;
}
.crash-modal-enter-active .crash-dialog {
  transition: transform 0.3s cubic-bezier(0.34, 1.56, 0.64, 1), opacity 0.15s ease;
}
.crash-modal-enter-from {
  opacity: 0;
}
.crash-modal-enter-from .crash-dialog {
  transform: translateY(40px);
  opacity: 0;
}
/* 关闭动画（MyMsgText.xaml.vb 第 53-70 行）：
   - 下沉 20px + 旋转 6° + 淡出 */
.crash-modal-leave-active {
  transition: opacity 0.15s ease;
}
.crash-modal-leave-to {
  opacity: 0;
}
.crash-modal-leave-to .crash-dialog {
  transform: translateY(20px);
  opacity: 0;
}
</style>
