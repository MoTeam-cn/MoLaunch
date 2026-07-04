<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import * as tauri from '@/utils/tauri'
import Alert from '@/components/common/Alert.vue'

const proxyMode = ref<'none' | 'system' | 'custom'>('none')
const proxyType = ref<'http' | 'https' | 'socks5'>('http')
const proxyUrl = ref('')
const loaded = ref(false)

let saveTimer: ReturnType<typeof setTimeout> | null = null
const pendingChanges = new Set<string>()

function scheduleSave(changeType: string) {
  if (!loaded.value) return
  pendingChanges.add(changeType)
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(flushSave, 1500)
}

async function flushSave() {
  const changes = [...pendingChanges]
  pendingChanges.clear()
  saveTimer = null

  try {
    const tasks: Promise<void>[] = []

    if (changes.includes('mode')) {
      tasks.push(tauri.setProxyMode(proxyMode.value))
    }
    if (changes.includes('type')) {
      tasks.push(tauri.setProxyType(proxyType.value))
    }
    if (changes.includes('url')) {
      tasks.push(tauri.setProxyUrl(proxyUrl.value))
    }

    await Promise.all(tasks)
  } catch (e) {
    console.error('Failed to save proxy settings:', e)
  }
}

watch(proxyMode, () => scheduleSave('mode'))
watch(proxyType, () => scheduleSave('type'))
watch(proxyUrl, () => scheduleSave('url'))

onMounted(async () => {
  try {
    proxyMode.value = (await tauri.getProxyMode()) as typeof proxyMode.value
  } catch { /* ignore */ }
  try {
    proxyType.value = (await tauri.getProxyType()) as typeof proxyType.value
  } catch { /* ignore */ }
  try {
    proxyUrl.value = await tauri.getProxyUrl()
  } catch { /* ignore */ }
  loaded.value = true
})
</script>

<template>
  <div class="space-y-6">
    <!-- 代理配置 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">代理配置</h3>

      <!-- 提示框 -->
      <div class="mx-5 mb-4">
        <Alert type="warning" message="默认不走系统代理。即使开启了系统代理设置，本启动器也不会自动使用，除非您在此处手动配置。" />
      </div>

      <div class="divide-y divide-gray-200">
        <!-- 代理模式 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">代理模式</p>
              <p class="text-xs text-gray-500 mt-0.5">选择启动器的网络代理方式</p>
            </div>
          </div>
          <div class="flex gap-2">
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="proxyMode === 'none'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="proxyMode = 'none'"
            >
              不使用代理
            </button>
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="proxyMode === 'system'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="proxyMode = 'system'"
            >
              系统代理
            </button>
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="proxyMode === 'custom'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="proxyMode = 'custom'"
            >
              自定义代理
            </button>
          </div>
          <p class="text-xs text-gray-400 mt-2">
            <template v-if="proxyMode === 'none'">不使用任何代理，直接连接</template>
            <template v-else-if="proxyMode === 'system'">使用操作系统中配置的代理设置</template>
            <template v-else>手动配置代理服务器地址和端口</template>
          </p>
        </div>

        <!-- 自定义代理配置 -->
        <div v-if="proxyMode === 'custom'" class="px-5 py-4 space-y-4">
          <!-- 代理类型 -->
          <div>
            <p class="text-sm font-medium text-gray-900 mb-2">代理类型</p>
            <div class="flex gap-2">
              <button
                class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                :class="proxyType === 'http'
                  ? 'border-primary-500 bg-primary-50 text-primary-700'
                  : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                @click="proxyType = 'http'"
              >
                HTTP
              </button>
              <button
                class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                :class="proxyType === 'https'
                  ? 'border-primary-500 bg-primary-50 text-primary-700'
                  : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                @click="proxyType = 'https'"
              >
                HTTPS
              </button>
              <button
                class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
                :class="proxyType === 'socks5'
                  ? 'border-primary-500 bg-primary-50 text-primary-700'
                  : 'border-gray-200 text-gray-600 hover:border-gray-300'"
                @click="proxyType = 'socks5'"
              >
                SOCKS5
              </button>
            </div>
          </div>

          <!-- 代理地址 -->
          <div>
            <p class="text-sm font-medium text-gray-900 mb-2">代理地址</p>
            <input
              v-model="proxyUrl"
              type="text"
              placeholder="127.0.0.1:7890"
              class="w-full px-3 py-2 text-sm border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-primary-500 focus:border-transparent"
            />
            <p class="text-xs text-gray-400 mt-1">格式：IP地址:端口号，例如 127.0.0.1:7890</p>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
