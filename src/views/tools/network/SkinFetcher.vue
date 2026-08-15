<script setup lang="ts">
/**
 * 正版玩家皮肤下载
 *
 * 输入正版玩家名，通过 Mojang 官方 API 获取该玩家的 UUID / 皮肤模型 / 皮肤与披风
 * （流程参照 PCL2：api.mojang.com 取 UUID → sessionserver.mojang.com 取 textures →
 * 下载 textures.minecraft.net 的皮肤/披风 PNG），支持预览与保存到本地。
 */
import { ref, computed, defineAsyncComponent } from 'vue'
import {
  UserIcon,
  BoltIcon,
  ArrowDownTrayIcon,
  ClipboardDocumentIcon,
  CheckIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Alert = defineAsyncComponent(() => import('@/components/common/Alert.vue'))
import { toastSuccess, toastError } from '@/utils/toast'
import { copyToClipboard } from '@/utils/clipboard'
import { pickSavePath } from '@/utils/fileDialog'
import { skinFetch, skinSaveImage } from '@/utils/api/tools'
import type { SkinFetchResult } from '@/utils/api/tools'

const name = ref('')
const loading = ref(false)
const saving = ref<'skin' | 'cape' | null>(null)
const result = ref<SkinFetchResult | null>(null)

async function doFetch() {
  const playerName = name.value.trim()
  if (!playerName) return
  loading.value = true
  result.value = null
  try {
    const res = await skinFetch(playerName)
    result.value = res
    if (!res.error) toastSuccess(`已获取 ${res.name} 的皮肤`)
  } catch (e) {
    toastError('获取失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    loading.value = false
  }
}

/** 模型标签：slim=Alex（细手臂），classic=Steve（粗手臂） */
const modelLabel = computed(() =>
  result.value?.skin_model === 'slim' ? 'Alex 模型（细手臂）' : 'Steve 模型（粗手臂）',
)

/** 32 位 UUID 格式化为 8-4-4-4-12 标准形式 */
function formatUuid(uuid: string): string {
  if (uuid.length !== 32) return uuid
  return `${uuid.slice(0, 8)}-${uuid.slice(8, 12)}-${uuid.slice(12, 16)}-${uuid.slice(16, 20)}-${uuid.slice(20)}`
}

async function copyUuid() {
  if (!result.value?.uuid) return
  await copyToClipboard(formatUuid(result.value.uuid), { toast: true })
}

/** 去掉 data URI 前缀，取纯 base64 */
function stripDataUri(dataUri: string): string {
  const idx = dataUri.indexOf(',')
  return idx >= 0 ? dataUri.slice(idx + 1) : dataUri
}

async function saveImage(kind: 'skin' | 'cape') {
  const res = result.value
  const image = kind === 'skin' ? res?.skin_image : res?.cape_image
  if (!image) return
  const label = kind === 'skin' ? '皮肤' : '披风'
  const path = await pickSavePath({
    title: `保存${label}`,
    defaultPath: `${res?.name ?? 'player'}_${kind}.png`,
    filters: [{ name: 'PNG 图片', extensions: ['png'] }],
  })
  if (!path) return
  saving.value = kind
  try {
    await skinSaveImage(path, stripDataUri(image))
    toastSuccess(`已保存到 ${path}`)
  } catch (e) {
    toastError('保存失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    saving.value = null
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <UserIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">正版玩家皮肤下载</h3>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <Alert
        type="info"
        message="输入正版玩家名，获取该玩家在 Mojang 官方服务器上的皮肤与披风。国内网络访问 Mojang API 可能不稳定，若获取失败请稍后重试或检查网络。"
        :truncate="false"
      />

      <!-- 输入区 -->
      <div class="flex items-end gap-3">
        <div class="flex-1">
          <label class="mb-1 block text-xs font-medium text-gray-700">玩家名</label>
          <Input
            v-model="name"
            placeholder="如 Notch"
            clearable
            @keydown.enter="doFetch"
          />
        </div>
        <Button type="primary" :loading="loading" :disabled="!name.trim()" @click="doFetch">
          <template #icon><BoltIcon class="h-4 w-4" /></template>
          {{ loading ? '获取中...' : '获取皮肤' }}
        </Button>
      </div>

      <!-- 结果区 -->
      <div v-if="result" class="rounded-lg border border-gray-200 p-4 space-y-3">
        <template v-if="result.error">
          <div class="flex items-center gap-2">
            <XMarkIcon class="h-5 w-5 text-red-500" />
            <span class="text-sm font-medium text-red-600">获取失败</span>
          </div>
          <div class="rounded-lg bg-red-50 px-3 py-2 text-xs text-red-600">
            {{ result.error }}
          </div>
        </template>

        <template v-else>
          <!-- 玩家信息 -->
          <div class="flex flex-wrap items-center gap-x-4 gap-y-2 rounded-lg bg-gray-50 px-3 py-2.5">
            <div class="flex items-center gap-2">
              <UserIcon class="h-4 w-4 text-gray-500" />
              <span class="text-sm font-semibold text-gray-900">{{ result.name }}</span>
              <span
                class="rounded bg-blue-100 px-1.5 py-0.5 text-xs font-medium text-blue-700"
              >{{ modelLabel }}</span>
            </div>
            <div class="flex items-center gap-1 text-xs text-gray-500">
              <code class="select-all">{{ formatUuid(result.uuid) }}</code>
              <Button type="text" size="small" @click="copyUuid">
                <template #icon><ClipboardDocumentIcon class="h-3.5 w-3.5" /></template>
                复制
              </Button>
            </div>
          </div>

          <!-- 皮肤 / 披风 -->
          <div class="flex flex-wrap gap-4">
            <div class="rounded-lg border border-gray-200 p-3">
              <div class="mb-2 flex items-center justify-between">
                <span class="text-xs font-medium text-gray-700">皮肤</span>
                <Button
                  type="secondary"
                  size="small"
                  :loading="saving === 'skin'"
                  @click="saveImage('skin')"
                >
                  <template #icon><ArrowDownTrayIcon class="h-3.5 w-3.5" /></template>
                  保存
                </Button>
              </div>
              <img
                :src="result.skin_image"
                :alt="result.name + ' 的皮肤'"
                class="h-44 w-44 object-contain rounded bg-gray-100"
              />
            </div>
            <div v-if="result.cape_image" class="rounded-lg border border-gray-200 p-3">
              <div class="mb-2 flex items-center justify-between">
                <span class="text-xs font-medium text-gray-700">披风</span>
                <Button
                  type="secondary"
                  size="small"
                  :loading="saving === 'cape'"
                  @click="saveImage('cape')"
                >
                  <template #icon><ArrowDownTrayIcon class="h-3.5 w-3.5" /></template>
                  保存
                </Button>
              </div>
              <img
                :src="result.cape_image"
                :alt="result.name + ' 的披风'"
                class="h-44 w-44 object-contain rounded bg-gray-100"
              />
            </div>
            <div
              v-else
              class="flex items-center justify-center gap-2 rounded-lg border border-dashed border-gray-300 p-3 text-xs text-gray-400"
            >
              <CheckIcon class="h-4 w-4" />
              该玩家未设置披风
            </div>
          </div>
        </template>
      </div>
    </div>
  </section>
</template>
