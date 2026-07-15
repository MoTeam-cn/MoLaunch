<script setup lang="ts">
import { ref, watch, onMounted, nextTick } from 'vue'
import * as tauri from '@/utils/tauri'
import { useDebouncedSave } from '@/composables/useDebouncedSave'

const maxThreads = ref(8)
const chunkCount = ref(4)
const mirrorMeta = ref<'official' | 'bmclapi' | 'smart'>('smart')
const mirrorDownload = ref<'official' | 'bmclapi' | 'smart'>('smart')
const maxDownloadSpeed = ref(0)
const speedSlider = ref(0)
const loaded = ref(false)

// 待保存的设置队列
const pendingChanges = new Set<string>()

/** 前端选项 → 后端 source 值 */
function toSource(v: 'official' | 'bmclapi' | 'smart'): string {
  return v === 'bmclapi' ? 'mirror' : v
}

/** 后端 source 值 → 前端选项 */
function fromSource(s: string): 'official' | 'bmclapi' | 'smart' {
  if (s === 'mirror') return 'bmclapi'
  if (s === 'official') return 'official'
  return 'smart'
}

async function flushSave() {
  const changes = [...pendingChanges]
  pendingChanges.clear()

  try {
    const tasks: Promise<void>[] = []

    if (changes.includes('meta')) {
      tasks.push(tauri.setMetaSource(toSource(mirrorMeta.value), true))
    }
    if (changes.includes('download')) {
      tasks.push(tauri.setDownloadSource(toSource(mirrorDownload.value), true))
    }
    if (changes.includes('speed')) {
      const speed = speedSlider.value >= 21 ? 0 : (speedSlider.value === 0 ? 1 : speedSlider.value) * 1024 * 1024
      maxDownloadSpeed.value = speed
      tasks.push(tauri.setMaxDownloadSpeed(speed, true))
    }
    if (changes.includes('threads')) {
      tasks.push(tauri.setMaxDownloadThreads(maxThreads.value))
    }
    if (changes.includes('chunks')) {
      tasks.push(tauri.setChunkCount(chunkCount.value))
    }

    await Promise.all(tasks)
  } catch (e) {
    console.error('Failed to save download settings:', e)
  }
}

const { scheduleSave: scheduleDebouncedSave } = useDebouncedSave(flushSave, 1500)

function scheduleSave(changeType: string) {
  if (!loaded.value) return
  pendingChanges.add(changeType)
  scheduleDebouncedSave()
}

watch(mirrorMeta, () => scheduleSave('meta'))
watch(mirrorDownload, () => scheduleSave('download'))
watch(speedSlider, () => scheduleSave('speed'))
watch(maxThreads, () => scheduleSave('threads'))
watch(chunkCount, () => scheduleSave('chunks'))

onMounted(async () => {
  try {
    mirrorMeta.value = fromSource(await tauri.getMetaSource())
  } catch {
    mirrorMeta.value = 'smart'
  }
  try {
    mirrorDownload.value = fromSource(await tauri.getDownloadSource())
  } catch {
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
  try {
    maxThreads.value = await tauri.getMaxDownloadThreads()
  } catch {
    // ignore
  }
  try {
    chunkCount.value = await tauri.getChunkCount()
  } catch {
    // ignore
  }
  // 等待 watch 回调执行完毕（避免加载值被误判为用户改动触发保存）
  await nextTick()
  loaded.value = true
})
</script>

<template>
  <div class="space-y-6">
    <!-- 加载占位（避免初始值与实际值不一致导致的闪烁） -->
    <div v-if="!loaded" class="space-y-6">
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 py-5">
          <div class="h-4 w-24 bg-gray-200 rounded animate-pulse mb-4" />
          <div class="h-10 bg-gray-100 rounded animate-pulse" />
        </div>
      </div>
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <div class="px-5 py-5">
          <div class="h-4 w-24 bg-gray-200 rounded animate-pulse mb-4" />
          <div class="h-10 bg-gray-100 rounded animate-pulse" />
        </div>
      </div>
    </div>

    <template v-else>
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
        <!-- 并行下载数 -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">并行下载数</p>
            <p class="text-xs text-gray-500 mt-0.5">控制同时下载文件的数量，一般情况下推荐设置为 8</p>
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
        <!-- 分片数量 -->
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">下载线程数</p>
            <p class="text-xs text-gray-500 mt-0.5">跟 IDM等多线程下载器一样，将文件分片下载，提升单文件下载速度，推荐设置为 4</p>
          </div>
          <div class="flex items-center gap-3">
            <input
              v-model.number="chunkCount"
              type="range"
              class="w-32 h-2 bg-gray-200 rounded-lg appearance-none cursor-pointer"
              min="1"
              max="8"
              step="1"
            />
            <span class="text-sm font-medium text-primary-600 w-6 text-right">{{ chunkCount }}</span>
          </div>
        </div>
        <!-- 下载限速 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between mb-2">
            <div>
              <p class="text-sm font-medium text-gray-900">下载限速</p>
              <p class="text-xs text-gray-500 mt-0.5">现在下载资源时候的速度，对于电脑性能不好的，可以防止其他程序突然卡死</p>
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
    </template>
  </div>
</template>
