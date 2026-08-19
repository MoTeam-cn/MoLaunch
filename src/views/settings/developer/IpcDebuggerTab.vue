<script setup lang="ts">
/**
 * 开发者 - IPC 命令调试器子页签
 *
 * 输入 Tauri 命令名 + JSON 参数直接 invoke，查看返回结果或错误。
 * 命令名提供 datalist 自动补全（后端 generate_handler 注册的命令）。
 */
import { ref, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
import { invoke } from '@tauri-apps/api/core'
import { toastError } from '@/utils/toast'
import { BeakerIcon, PlayIcon, ClipboardDocumentIcon } from '@heroicons/vue/24/outline'

/** 后端 generate_handler 注册的全部命令（lib.rs invoke_handler 列表） */
const KNOWN_COMMANDS = [
  'sdk_manager',
  'meta_manager',
  'skin_manager',
  'image_cache_manager',
  'version_list_manager',
  'version_install_manager',
  'version_mods_manager',
  'version_packs_manager',
  'version_progress_manager',
  'version_launch_manager',
  'version_export_manager',
  'java_manager',
  'system_manager',
  'config_manager',
  'community_manager',
  'plugins_manager',
  'relaunch_snapshot',
  'tools_manager',
  'experimental_manager',
  'online_manager',
  'redstone_manager',
  'frp_manager',
  'request_exit',
  'frontend_ready',
]

const command = ref('')
const argsText = ref('{}')
const loading = ref(false)
const result = ref<string | null>(null)
const error = ref<string | null>(null)
const elapsed = ref<number | null>(null)

async function run() {
  const name = command.value.trim()
  if (!name) {
    toastError('请输入命令名')
    return
  }
  let args: unknown
  try {
    args = argsText.value.trim() ? JSON.parse(argsText.value) : undefined
  } catch {
    toastError('参数不是合法的 JSON')
    return
  }
  if (args !== undefined && (typeof args !== 'object' || args === null || Array.isArray(args))) {
    toastError('参数必须是 JSON 对象（如 {} 或 {"req": {...}}）')
    return
  }
  loading.value = true
  result.value = null
  error.value = null
  elapsed.value = null
  const start = performance.now()
  try {
    const r = await invoke(name, args as Record<string, unknown>)
    result.value = JSON.stringify(r, null, 2)
  } catch (e) {
    error.value = e instanceof Error ? e.message : String(e)
  } finally {
    elapsed.value = Math.round(performance.now() - start)
    loading.value = false
  }
}

async function copyResult() {
  const text = result.value ?? error.value
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
  } catch {
    toastError('复制失败')
  }
}
</script>

<template>
  <div class="space-y-6">
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3 flex items-center gap-2">
        <BeakerIcon class="w-4 h-4 text-gray-500" />
        IPC 命令调试器
      </h3>

      <div class="mx-5 mb-4">
        <Alert
          type="info"
          :truncate="false"
          message="输入后端注册的 Tauri 命令名与 JSON 参数直接 invoke，用于排查 IPC 调用问题。参数留空或 {} 表示无参数；manager 类命令需传 { req: { action, params } } 结构。"
        />
      </div>

      <div class="px-5 pb-5 space-y-4">
        <div>
          <p class="text-sm font-medium text-gray-900 mb-1.5">命令名</p>
          <Input
            v-model="command"
            placeholder="如 version_launch_manager"
            list="ipc-command-list"
            clearable
          />
          <datalist id="ipc-command-list">
            <option v-for="c in KNOWN_COMMANDS" :key="c" :value="c" />
          </datalist>
        </div>

        <div>
          <p class="text-sm font-medium text-gray-900 mb-1.5">参数（JSON）</p>
          <Input
            v-model="argsText"
            textarea
            :rows="4"
            resize="vertical"
            placeholder='{"req": {"action": "get_launch_history", "params": null}}'
          />
        </div>

        <div class="flex items-center gap-2">
          <Button type="primary" :loading="loading" @click="run">
            <template #icon><PlayIcon class="w-4 h-4" /></template>
            执行
          </Button>
          <Button v-if="result || error" type="secondary" @click="copyResult">
            <template #icon><ClipboardDocumentIcon class="w-4 h-4" /></template>
            复制结果
          </Button>
          <span v-if="elapsed !== null" class="text-xs text-gray-400">
            耗时 {{ elapsed }} ms
          </span>
        </div>

        <!-- 结果区：固定标题 + 内容滚动 -->
        <div v-if="result || error" class="rounded-md border border-gray-200 overflow-hidden">
          <div class="flex items-center justify-between px-3 py-2 bg-gray-50 border-b border-gray-200">
            <span class="text-xs font-medium" :class="error ? 'text-red-600' : 'text-green-600'">
              {{ error ? '调用失败' : '调用成功' }}
            </span>
          </div>
          <pre
            data-inner-scroll
            class="max-h-80 overflow-y-auto px-3 py-2 text-xs leading-relaxed font-mono whitespace-pre-wrap break-all"
            :class="error ? 'text-red-600' : 'text-gray-800'"
          >{{ error ?? result }}</pre>
        </div>
      </div>
    </div>
  </div>
</template>