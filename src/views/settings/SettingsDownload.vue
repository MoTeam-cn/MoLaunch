<script setup lang="ts">
import { ref, watch, onMounted } from 'vue'
import * as tauri from '@/utils/tauri'

const maxThreads = ref(8)
const mirrorMeta = ref<'official' | 'bmclapi' | 'smart'>('bmclapi')
const mirrorDownload = ref<'official' | 'bmclapi' | 'smart'>('bmclapi')
const maxDownloadSpeed = ref(0)
const speedSlider = ref(0)
const loaded = ref(false)

watch([mirrorMeta, mirrorDownload], async ([meta, dl]) => {
  if (!loaded.value) return
  let source = 'official'
  if (meta === 'bmclapi' && dl === 'bmclapi') source = 'mirror'
  else if (meta === 'official' && dl === 'bmclapi') source = 'smart'
  else if (meta === 'smart' || dl === 'smart') source = 'smart'
  try {
    await tauri.setDownloadSource(source)
  } catch (e) {
    console.error('Failed to set download source:', e)
  }
})

watch(speedSlider, async (val) => {
  if (!loaded.value) return
  const speed = val >= 21 ? 0 : (val === 0 ? 1 : val) * 1024 * 1024
  maxDownloadSpeed.value = speed
  try {
    await tauri.setMaxDownloadSpeed(speed)
  } catch (e) {
    console.error('Failed to set max download speed:', e)
  }
})

let saveTimer: ReturnType<typeof setTimeout> | null = null
function autoSave() {
  if (!loaded.value) return
  if (saveTimer) clearTimeout(saveTimer)
  saveTimer = setTimeout(() => {
    // TODO: 持久化下载设置到后端
  }, 500)
}

watch(maxThreads, autoSave)

onMounted(async () => {
  try {
    const source = await tauri.getDownloadSource()
    if (source === 'mirror') {
      mirrorMeta.value = 'bmclapi'
      mirrorDownload.value = 'bmclapi'
    } else if (source === 'official') {
      mirrorMeta.value = 'official'
      mirrorDownload.value = 'official'
    } else {
      mirrorMeta.value = 'smart'
      mirrorDownload.value = 'smart'
    }
  } catch {
    mirrorMeta.value = 'smart'
    mirrorDownload.value = 'smart'
  }
  try {
    maxDownloadSpeed.value = await tauri.getMaxDownloadSpeed()
    if (maxDownloadSpeed.value === 0) {
      speedSlider.value = 21
    } else {
      speedSlider.value = Math.round(maxDownloadSpeed.value / 1024 / 1024)
    }
  } catch {
    // ignore
  }
  loaded.value = true
})
</script>

<template>
  <div class="space-y-6">
    <!-- 下载源 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">下载源</h3>
      <div class="divide-y divide-gray-200">
        <!-- 版本列表源 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">版本列表源</p>
              <p class="text-xs text-gray-500 mt-0.5">版本清单、Forge/Fabric 等加载器列表的获取源</p>
            </div>
          </div>
          <div class="flex gap-2">
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="mirrorMeta === 'official'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="mirrorMeta = 'official'"
            >
              官方源
            </button>
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="mirrorMeta === 'bmclapi'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="mirrorMeta = 'bmclapi'"
            >
              BMCLAPI
            </button>
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="mirrorMeta === 'smart'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="mirrorMeta = 'smart'"
            >
              优先官方
            </button>
          </div>
          <p class="text-xs text-gray-400 mt-2">
            <template v-if="mirrorMeta === 'official'">Mojang 官方源，海外快国内可能较慢</template>
            <template v-else-if="mirrorMeta === 'bmclapi'">BMCLAPI 国内镜像，速度快</template>
            <template v-else>优先从官方源下载，速度太慢或不稳定时自动切换到镜像源</template>
          </p>
        </div>
        <!-- 文件下载源 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">文件下载源</p>
              <p class="text-xs text-gray-500 mt-0.5">客户端 JAR、库文件、资源文件、加载器安装包</p>
            </div>
          </div>
          <div class="flex gap-2">
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="mirrorDownload === 'official'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="mirrorDownload = 'official'"
            >
              官方源
            </button>
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="mirrorDownload === 'bmclapi'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="mirrorDownload = 'bmclapi'"
            >
              BMCLAPI
            </button>
            <button
              class="flex-1 px-3 py-2 text-xs font-medium rounded-lg border-2 transition-colors"
              :class="mirrorDownload === 'smart'
                ? 'border-primary-500 bg-primary-50 text-primary-700'
                : 'border-gray-200 text-gray-600 hover:border-gray-300'"
              @click="mirrorDownload = 'smart'"
            >
              优先官方
            </button>
          </div>
          <p class="text-xs text-gray-400 mt-2">
            <template v-if="mirrorDownload === 'official'">Mojang 官方源，海外快国内可能较慢</template>
            <template v-else-if="mirrorDownload === 'bmclapi'">BMCLAPI 国内镜像，速度快</template>
            <template v-else>优先从官方源下载，速度太慢或不稳定时自动切换到镜像源</template>
          </p>
        </div>
      </div>
    </div>

    <!-- 下载控制 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">下载控制</h3>
      <div class="divide-y divide-gray-200">
        <!-- 下载线程数 -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">下载线程数</p>
            <p class="text-xs text-gray-500 mt-0.5">并发下载线程，数值越大下载越快</p>
          </div>
          <div class="flex items-center gap-3">
            <input
              v-model.number="maxThreads"
              type="range"
              class="w-32 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              min="1"
              max="16"
              step="1"
            />
            <span class="text-sm font-medium text-primary-600 w-6 text-right">{{ maxThreads }}</span>
          </div>
        </div>
        <!-- 下载限速 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">下载限速</p>
              <p class="text-xs text-gray-500 mt-0.5">拖到最右边为不限制</p>
            </div>
            <span class="text-sm font-medium text-primary-600">
              {{ speedSlider >= 21 ? '不限制' : speedSlider + ' MB/s' }}
            </span>
          </div>
          <div class="flex items-center gap-3">
            <span class="text-xs text-gray-400">1</span>
            <input
              v-model.number="speedSlider"
              type="range"
              class="flex-1 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              min="0"
              max="21"
              step="1"
            />
            <span class="text-xs text-gray-400">不限</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
