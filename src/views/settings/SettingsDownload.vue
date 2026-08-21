<script setup lang="ts">
import { ref, watch, defineAsyncComponent } from 'vue'
import { useConfigPage } from '@/composables/useConfigPage'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Slider = defineAsyncComponent(() => import('@/components/common/Slider.vue'))

const maxThreads = ref(8)
const chunkCount = ref(4)
const mirrorMeta = ref<'official' | 'bmclapi' | 'smart'>('smart')
const mirrorDownload = ref<'official' | 'bmclapi' | 'smart'>('smart')
const maxDownloadSpeed = ref(0)
const speedSlider = ref(0)

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

const { loaded, markDirty } = useConfigPage({
  delay: 1500,
  errorLabel: 'save download settings',
  onLoad: (cfg) => {
    mirrorMeta.value = fromSource(cfg.metaSource)
    mirrorDownload.value = fromSource(cfg.downloadSource)
    maxDownloadSpeed.value = cfg.maxDownloadSpeed
    if (maxDownloadSpeed.value === 0) {
      speedSlider.value = 21
    } else {
      speedSlider.value = Math.round(maxDownloadSpeed.value / 1024 / 1024)
    }
    maxThreads.value = cfg.maxDownloadThreads
    chunkCount.value = cfg.chunkCount
  },
})

watch(mirrorMeta, (v) => markDirty('metaSource', toSource(v)))
watch(mirrorDownload, (v) => markDirty('downloadSource', toSource(v)))
watch(speedSlider, (v) => {
  const speed = v >= 21 ? 0 : (v === 0 ? 1 : v) * 1024 * 1024
  maxDownloadSpeed.value = speed
  markDirty('maxDownloadSpeed', speed)
})
watch(maxThreads, (v) => markDirty('maxDownloadThreads', v))
watch(chunkCount, (v) => markDirty('chunkCount', v))
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
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">版本列表源</p>
              <p class="text-xs text-gray-500 mt-0.5">版本清单、Forge/Fabric 等加载器列表的获取源</p>
            </div>
            <div class="flex-none w-40">
              <Select
                v-model="mirrorMeta"
                :options="[
                  { label: '官方源', value: 'official' },
                  { label: 'BMCLAPI', value: 'bmclapi' },
                  { label: '优先官方', value: 'smart' },
                ]"
              />
            </div>
          </div>
          <p class="text-xs text-gray-400 mt-2">
            <template v-if="mirrorMeta === 'official'">Mojang 官方源，海外快国内可能较慢</template>
            <template v-else-if="mirrorMeta === 'bmclapi'">BMCLAPI 国内镜像，速度快</template>
            <template v-else>优先从官方源下载，速度太慢或不稳定时自动切换到镜像源</template>
          </p>
        </div>
        <!-- 文件下载源 -->
        <div class="px-5 py-4">
          <div class="flex items-center justify-between gap-4">
            <div class="min-w-0">
              <p class="text-sm font-medium text-gray-900">文件下载源</p>
              <p class="text-xs text-gray-500 mt-0.5">客户端 JAR、库文件、资源文件、加载器安装包</p>
            </div>
            <div class="flex-none w-40">
              <Select
                v-model="mirrorDownload"
                :options="[
                  { label: '官方源', value: 'official' },
                  { label: 'BMCLAPI', value: 'bmclapi' },
                  { label: '优先官方', value: 'smart' },
                ]"
              />
            </div>
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
            <Slider v-model="maxThreads" :min="1" :max="16" :step="1" class="w-32" />
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
            <Slider v-model="chunkCount" :min="1" :max="8" :step="1" class="w-32" />
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
            <Slider v-model="speedSlider" :min="0" :max="21" :step="1" class="flex-1" />
            <span class="text-xs text-gray-400">不限</span>
          </div>
        </div>
      </div>
    </div>
    </template>
  </div>
</template>
