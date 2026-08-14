<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted, defineAsyncComponent } from 'vue'
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const AiEndpointSettings = defineAsyncComponent(() => import('./AiEndpointSettings.vue'))
const AiContextSettings = defineAsyncComponent(() => import('./AiContextSettings.vue'))
const AiModelSettings = defineAsyncComponent(() => import('./AiModelSettings.vue'))
import {
  aiLoadConfig,
  aiSaveConfig,
  aiCheckStatus,
  aiListModels,
  type AiConfig,
  type AiProbeParams,
} from '@/utils/api/ai'
import { safeCall } from '@/utils/async'
import { toastSuccess, toastError, toastInfo } from '@/utils/toast'
import { setIconColorMode } from '@/utils/model-icon-mode'

const baseUrl = ref('')
const apiKey = ref('')
const timeoutSecs = ref(60)
const maxInputTokens = ref(184000)
const maxOutputTokens = ref(16000)
const enabledModels = ref<string[]>([])
const defaultModel = ref('')
const iconColorMode = ref<'color' | 'mono'>('color')
const remoteModels = ref<string[]>([])
const loading = ref(false)
const loadingModels = ref(false)
const checking = ref(false)
const available = ref(false)
const checked = ref(false)
let loaded = false

const defaultOptions = computed(() => enabledModels.value.map((model) => ({ label: model, value: model })))

function probeParams(): AiProbeParams {
  return { baseUrl: baseUrl.value.trim(), apiKey: apiKey.value, timeoutSecs: timeoutSecs.value }
}

function toggleModel(model: string): void {
  const index = enabledModels.value.indexOf(model)
  if (index >= 0) {
    enabledModels.value = enabledModels.value.filter((item) => item !== model)
    if (defaultModel.value === model) defaultModel.value = enabledModels.value[0] ?? ''
  } else {
    enabledModels.value = [...enabledModels.value, model]
    if (!defaultModel.value) defaultModel.value = model
  }
}

async function save(): Promise<void> {
  if (!defaultModel.value && enabledModels.value.length > 0) defaultModel.value = enabledModels.value[0]
  const config: AiConfig = {
    baseUrl: baseUrl.value.trim(), apiKey: apiKey.value, timeoutSecs: timeoutSecs.value,
    maxInputTokens: maxInputTokens.value, maxOutputTokens: maxOutputTokens.value,
    models: enabledModels.value, defaultModel: defaultModel.value, iconColorMode: iconColorMode.value,
  }
  const ok = await safeCall(() => aiSaveConfig(config), 'save ai config', () => toastError('保存 AI 配置失败'))
  if (ok !== undefined) {
    setIconColorMode(iconColorMode.value)
    toastSuccess('AI 配置已保存')
  }
}

async function handleLoadModels(): Promise<void> {
  if (!baseUrl.value.trim()) { toastInfo('请先填写服务地址'); return }
  loadingModels.value = true
  const models = await safeCall(() => aiListModels(probeParams()), 'list ai models', () => toastError('获取模型列表失败，请确认服务已启动且地址正确'))
  if (models) { remoteModels.value = models; toastInfo(`获取到 ${models.length} 个模型，勾选需要启用的模型`) }
  loadingModels.value = false
}

async function handleCheck(): Promise<void> {
  checking.value = true
  checked.value = false
  const status = await safeCall(() => aiCheckStatus(probeParams()), 'check ai status', () => toastError('检测 AI 服务失败'))
  if (status) {
    available.value = status.available
    checked.value = true
    if (status.available) toastSuccess(`AI 服务可用（${defaultModel.value || '未选择默认模型'}）`)
    else toastInfo('AI 服务不可用，请确认本地服务已启动且地址正确')
  }
  checking.value = false
}

onMounted(async () => {
  loading.value = true
  const config = await safeCall(() => aiLoadConfig(), 'load ai config', () => toastError('加载 AI 配置失败'))
  if (config) {
    baseUrl.value = config.baseUrl; apiKey.value = config.apiKey; timeoutSecs.value = config.timeoutSecs
    maxInputTokens.value = config.maxInputTokens ?? 184000; maxOutputTokens.value = config.maxOutputTokens ?? 16000
    enabledModels.value = config.models ?? []; defaultModel.value = config.defaultModel
    iconColorMode.value = config.iconColorMode === 'mono' ? 'mono' : 'color'; loaded = true
  }
  loading.value = false
  if (baseUrl.value.trim()) await handleLoadModels()
})

onUnmounted(async () => {
  if (loaded && (baseUrl.value.trim() || enabledModels.value.length > 0)) await save()
})
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">本地 AI 服务</h3>

    <div v-if="loading" class="px-5 pb-5 space-y-3">
      <div class="h-10 bg-gray-100 rounded animate-pulse" />
      <div class="h-10 bg-gray-100 rounded animate-pulse" />
    </div>

    <template v-else>
      <!-- 提示框 -->
      <div class="mx-5 mb-4">
        <Alert
          type="info"
          :truncate="false"
          message="AI 分析使用本地 OpenAI 兼容服务（如 Ollama / LM Studio），日志不会上传到云端。配置地址与 API Key 后，加载并启用模型即可在崩溃分析中使用。"
        />
      </div>

      <div class="divide-y divide-gray-200">
        <AiEndpointSettings
          v-model:base-url="baseUrl"
          v-model:api-key="apiKey"
          v-model:timeout-secs="timeoutSecs"
          v-model:icon-color-mode="iconColorMode"
        />
        <AiContextSettings
          v-model:max-input-tokens="maxInputTokens"
          v-model:max-output-tokens="maxOutputTokens"
        />
        <AiModelSettings
          :remote-models="remoteModels"
          :enabled-models="enabledModels"
          :default-model="defaultModel"
          :loading-models="loadingModels"
          :default-options="defaultOptions"
          @load="handleLoadModels"
          @toggle="toggleModel"
          @update:default-model="defaultModel = $event"
        />

        <div v-if="checked" class="px-5 py-3">
          <p class="text-sm" :class="available ? 'text-green-600' : 'text-amber-600'">
            {{ available ? '服务可用' : '服务不可用：请确认本地服务已启动且地址正确' }}
          </p>
        </div>
      </div>

      <div class="flex items-center gap-3 px-5 py-4 border-t border-gray-200">
        <Button type="primary" size="small" :loading="checking" @click="handleCheck">
          检测连接
        </Button>
        <Button type="outline" size="small" @click="save">
          保存配置
        </Button>
      </div>
    </template>
  </div>
</template>
