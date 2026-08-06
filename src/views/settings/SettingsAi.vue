<script setup lang="ts">
import { ref, computed, onMounted, onUnmounted } from 'vue'
import Alert from '@/components/common/Alert.vue'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import Select from '@/components/common/Select.vue'
import Tag from '@/components/common/Tag.vue'
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

onMounted(async () => {
  loading.value = true
  const cfg = await safeCall(() => aiLoadConfig(), 'load ai config', () => toastError('加载 AI 配置失败'))
  if (cfg) {
    baseUrl.value = cfg.baseUrl
    apiKey.value = cfg.apiKey
    timeoutSecs.value = cfg.timeoutSecs
    maxInputTokens.value = cfg.maxInputTokens ?? 184000
    maxOutputTokens.value = cfg.maxOutputTokens ?? 16000
    enabledModels.value = cfg.models ?? []
    defaultModel.value = cfg.defaultModel
    iconColorMode.value = cfg.iconColorMode === 'mono' ? 'mono' : 'color'
    loaded = true
  }
  loading.value = false
  // 进入页面自动从服务端拉取模型列表（无需手动点击「加载模型」）
  if (baseUrl.value.trim()) {
    await handleLoadModels()
  }
})

onUnmounted(async () => {
  if (loaded && (baseUrl.value.trim() || enabledModels.value.length > 0)) {
    await save()
  }
})

const defaultOptions = computed(() =>
  enabledModels.value.map((m) => ({ label: m, value: m })),
)

/** 模型图标样式选项 */
const iconModeOptions = [
  { label: '彩色', value: 'color' },
  { label: '黑白', value: 'mono' },
]

function isEnabled(model: string): boolean {
  return enabledModels.value.includes(model)
}

function toggleModel(model: string): void {
  const idx = enabledModels.value.indexOf(model)
  if (idx >= 0) {
    enabledModels.value = enabledModels.value.filter((m) => m !== model)
    if (defaultModel.value === model) {
      defaultModel.value = enabledModels.value[0] ?? ''
    }
  } else {
    enabledModels.value = [...enabledModels.value, model]
    if (!defaultModel.value) {
      defaultModel.value = model
    }
  }
}

function probeParams(): AiProbeParams {
  return {
    baseUrl: baseUrl.value.trim(),
    apiKey: apiKey.value,
    timeoutSecs: timeoutSecs.value,
  }
}

async function save(): Promise<void> {
  if (!defaultModel.value && enabledModels.value.length > 0) {
    defaultModel.value = enabledModels.value[0]
  }
  const cfg: AiConfig = {
    baseUrl: baseUrl.value.trim(),
    apiKey: apiKey.value,
    timeoutSecs: timeoutSecs.value,
    maxInputTokens: maxInputTokens.value,
    maxOutputTokens: maxOutputTokens.value,
    models: enabledModels.value,
    defaultModel: defaultModel.value,
    iconColorMode: iconColorMode.value,
  }
  // aiSaveConfig 返回 Promise<void>，成功时 safeCall 结果为 undefined，须用 !== undefined 判断
  const ok = await safeCall(() => aiSaveConfig(cfg), 'save ai config', () => toastError('保存 AI 配置失败'))
  if (ok !== undefined) {
    // 保存后立即同步全局图标显示模式（聊天页/头部无需重新加载即可生效）
    setIconColorMode(iconColorMode.value)
    toastSuccess('AI 配置已保存')
  }
}

async function handleSave(): Promise<void> {
  await save()
}

async function handleLoadModels(): Promise<void> {
  if (!baseUrl.value.trim()) {
    toastInfo('请先填写服务地址')
    return
  }
  loadingModels.value = true
  const models = await safeCall(
    () => aiListModels(probeParams()),
    'list ai models',
    () => toastError('获取模型列表失败，请确认服务已启动且地址正确'),
  )
  if (models) {
    remoteModels.value = models
    toastInfo(`获取到 ${models.length} 个模型，勾选需要启用的模型`)
  }
  loadingModels.value = false
}

async function handleCheck(): Promise<void> {
  checking.value = true
  checked.value = false
  const status = await safeCall(
    () => aiCheckStatus(probeParams()),
    'check ai status',
    () => toastError('检测 AI 服务失败'),
  )
  if (status) {
    available.value = status.available
    checked.value = true
    if (status.available) {
      toastSuccess(`AI 服务可用（${defaultModel.value || '未选择默认模型'}）`)
    } else {
      toastInfo('AI 服务不可用，请确认本地服务已启动且地址正确')
    }
  }
  checking.value = false
}
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
        <!-- 服务地址 -->
        <div class="px-5 py-4">
          <p class="text-sm font-medium text-gray-900 mb-2">服务地址</p>
          <Input
            v-model="baseUrl"
            placeholder="http://127.0.0.1:11434/v1"
            hint="OpenAI 兼容 API 地址，例如 Ollama 默认 http://127.0.0.1:11434/v1"
          />
        </div>

        <!-- API Key -->
        <div class="px-5 py-4">
          <p class="text-sm font-medium text-gray-900 mb-2">API Key</p>
          <Input
            v-model="apiKey"
            type="password"
            placeholder="留空表示无需认证"
            hint="写入时经 SDK 加密存储（config.ini），本地 Ollama 通常无需填写"
          />
        </div>

        <!-- 请求超时（滑块） -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <p class="text-sm font-medium text-gray-900">请求超时</p>
            <span class="text-sm font-medium text-primary-600">{{ timeoutSecs }} 秒</span>
          </div>
          <div class="flex items-center gap-3">
            <span class="text-xs text-gray-400">10</span>
            <input
              v-model.number="timeoutSecs"
              type="range"
              class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              min="10"
              max="300"
              step="10"
            />
            <span class="text-xs text-gray-400">300</span>
          </div>
          <p class="text-xs text-gray-500 mt-1.5">模型分析耗时可较长，默认 60 秒</p>
        </div>

        <!-- Token 限制 -->
        <div class="px-5 py-4">
          <p class="text-sm font-medium text-gray-900 mb-3">上下文窗口（Token）</p>
          <div class="grid grid-cols-2 gap-4">
            <div>
              <label class="block text-xs text-gray-500 mb-1.5">输入上限（窗口）</label>
              <Input
                v-model.number="maxInputTokens"
                type="number"
                min="2000"
                max="1000000"
                placeholder="184000"
                hint="接近此上限时自动压缩历史上下文"
              />
            </div>
            <div>
              <label class="block text-xs text-gray-500 mb-1.5">单次回复上限（输出）</label>
              <Input
                v-model.number="maxOutputTokens"
                type="number"
                min="256"
                max="128000"
                placeholder="16000"
                hint="请求时作为 max_tokens 下发"
              />
            </div>
          </div>
        </div>

        <!-- 模型图标样式 -->
        <div class="px-5 py-4">
          <p class="text-sm font-medium text-gray-900 mb-2">模型图标</p>
          <Select v-model="iconColorMode" :options="iconModeOptions" />
          <p class="text-xs text-gray-500 mt-1.5">
            彩色为品牌官方配色；黑白为单色图标。未识别的模型统一使用 HuggingFace 图标。
          </p>
        </div>

        <!-- 模型管理 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <p class="text-sm font-medium text-gray-900">模型管理</p>
            <Button type="outline" size="mini" :loading="loadingModels" @click="handleLoadModels">
              加载模型
            </Button>
          </div>
          <p class="text-xs text-gray-500 mb-3">
            从服务端加载模型列表后，勾选需要启用的模型；未勾选的模型不会被使用。
          </p>

          <!-- 模型勾选列表 -->
          <div v-if="remoteModels.length > 0" class="border border-gray-200 rounded-md max-h-44 overflow-y-auto p-1.5 space-y-0.5 mb-3">
            <label
              v-for="m in remoteModels"
              :key="m"
              class="flex items-center justify-between px-2 py-1 rounded hover:bg-gray-50 cursor-pointer"
            >
              <Checkbox :checked="isEnabled(m)" @change="() => toggleModel(m)">
                {{ m }}
              </Checkbox>
              <Tag v-if="defaultModel === m" color="primary" size="small">默认</Tag>
            </label>
          </div>
          <p v-else class="text-xs text-gray-400 py-2 mb-3">
            点击「加载模型」从当前服务地址拉取可用模型
          </p>

          <!-- 默认模型选择 -->
          <template v-if="enabledModels.length > 0">
            <p class="text-sm font-medium text-gray-900 mb-2">默认模型</p>
            <Select
              v-model="defaultModel"
              :options="defaultOptions"
              placeholder="请选择默认模型"
            />
            <p class="text-xs text-gray-500 mt-1.5">未指定模型时，崩溃分析默认使用该模型</p>
          </template>
        </div>

        <!-- 状态提示 -->
        <div v-if="checked" class="px-5 py-3">
          <p class="text-sm" :class="available ? 'text-green-600' : 'text-amber-600'">
            {{ available ? '服务可用' : '服务不可用：请确认本地服务已启动且地址正确' }}
          </p>
        </div>
      </div>

      <!-- 按钮栏 -->
      <div class="flex items-center gap-3 px-5 py-4 border-t border-gray-200">
        <Button type="primary" size="small" :loading="checking" @click="handleCheck">
          检测连接
        </Button>
        <Button type="outline" size="small" @click="handleSave">
          保存配置
        </Button>
      </div>
    </template>
  </div>
</template>
