<script setup lang="ts">
/**
 * 内置插件：版本统计
 *
 * 在主页右侧内容区显示已安装版本统计：
 * - 总数 + 按加载器分类的横向条形图（vanilla/forge/fabric/neoforge/...）
 * - 主版本号分布（1.20 / 1.19 / 1.18 ...）
 *
 * 数据来源：pluginSdk.listInstalledVersionsWithType（一次拉取，无轮询）。
 */
import { ref, computed, onMounted } from 'vue'
import { pluginSdk } from '@/plugins/sdk'
import { ArrowPathIcon, ChartBarIcon } from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'

interface InstalledVersion {
  id: string
  version_type: string
  logo: string
}

const versions = ref<InstalledVersion[]>([])
const loading = ref(true)
const error = ref<string | null>(null)

async function loadVersions() {
  try {
    versions.value = await pluginSdk.listInstalledVersionsWithType()
    error.value = null
  } catch (e) {
    error.value = String(e)
    pluginSdk.log('error', `[VersionStats] 加载失败: ${e}`)
  } finally {
    loading.value = false
  }
}

/** 加载器类型显示信息 */
const LOADER_INFO: Record<string, { label: string; color: string }> = {
  vanilla: { label: '原版', color: 'bg-green-500' },
  forge: { label: 'Forge', color: 'bg-blue-500' },
  fabric: { label: 'Fabric', color: 'bg-yellow-500' },
  neoforge: { label: 'NeoForge', color: 'bg-orange-500' },
  optifine: { label: 'OptiFine', color: 'bg-purple-500' },
  liteloader: { label: 'LiteLoader', color: 'bg-pink-500' },
}

/** 按加载器类型分组统计 */
const statsByLoader = computed(() => {
  const counts = new Map<string, number>()
  for (const v of versions.value) {
    const type = v.version_type || 'unknown'
    counts.set(type, (counts.get(type) ?? 0) + 1)
  }
  // 转为数组并按数量倒序
  const arr = Array.from(counts.entries()).map(([type, count]) => ({
    type,
    count,
    info: LOADER_INFO[type] ?? { label: type, color: 'bg-gray-400' },
  }))
  arr.sort((a, b) => b.count - a.count)
  return arr
})

/** 按主版本号分组统计（如 1.20 / 1.19 / 1.18） */
const statsByMajor = computed(() => {
  const counts = new Map<string, number>()
  for (const v of versions.value) {
    // 提取主版本号（前两段数字）
    const match = v.id.match(/^(\d+\.\d+)/)
    const major = match ? match[1] : '其他'
    counts.set(major, (counts.get(major) ?? 0) + 1)
  }
  // 转为数组并按版本号倒序
  const arr = Array.from(counts.entries()).map(([major, count]) => ({ major, count }))
  arr.sort((a, b) => {
    const [a1, a2] = a.major.split('.').map(Number)
    const [b1, b2] = b.major.split('.').map(Number)
    if (a1 !== b1) return b1 - a1
    return b2 - a2
  })
  return arr
})

/** 最大数量（用于条形图宽度计算） */
const maxLoaderCount = computed(() =>
  Math.max(1, ...statsByLoader.value.map((s) => s.count)),
)
const maxMajorCount = computed(() =>
  Math.max(1, ...statsByMajor.value.map((s) => s.count)),
)

onMounted(loadVersions)
</script>

<template>
  <div class="flex h-full flex-col p-6">
    <!-- 标题栏 -->
    <div class="mb-4 flex items-center justify-between">
      <h3 class="text-base font-semibold text-gray-900">版本统计</h3>
      <Button
        type="ghost"
        size="mini"
        :disabled="loading"
        @click="loadVersions"
      >
        <template #icon>
          <ArrowPathIcon class="h-3.5 w-3.5" :class="{ 'animate-spin': loading }" />
        </template>
        刷新
      </Button>
    </div>

    <!-- 加载中 -->
    <div v-if="loading" class="text-sm text-gray-500">加载中...</div>

    <!-- 错误 -->
    <div v-else-if="error" class="text-sm text-red-500">
      加载失败：{{ error }}
    </div>

    <!-- 空状态 -->
    <div
      v-else-if="versions.length === 0"
      class="flex flex-1 flex-col items-center justify-center text-center"
    >
      <ChartBarIcon class="mb-3 h-10 w-10 text-gray-300" />
      <p class="text-sm text-gray-500">暂无已安装版本</p>
      <p class="mt-1 text-xs text-gray-400">下载版本后可在此查看统计</p>
    </div>

    <!-- 统计内容 -->
    <div v-else class="flex-1 space-y-4 overflow-y-auto pr-1">
      <!-- 总数 -->
      <div class="rounded-md border border-gray-200 p-4">
        <div class="flex items-center justify-between">
          <span class="text-sm text-gray-600">已安装版本总数</span>
          <span class="text-2xl font-semibold text-primary-600">{{ versions.length }}</span>
        </div>
      </div>

      <!-- 按加载器分类 -->
      <div class="rounded-md border border-gray-200 p-4">
        <p class="mb-3 text-sm font-medium text-gray-900">按加载器分类</p>
        <div class="space-y-2">
          <div v-for="item in statsByLoader" :key="item.type" class="space-y-1">
            <div class="flex items-center justify-between text-xs">
              <span class="text-gray-700">{{ item.info.label }}</span>
              <span class="text-gray-500">{{ item.count }}</span>
            </div>
            <div class="h-1.5 w-full overflow-hidden rounded-full bg-gray-100">
              <div
                class="h-full transition-all duration-500"
                :class="item.info.color"
                :style="{ width: `${(item.count / maxLoaderCount) * 100}%` }"
              />
            </div>
          </div>
          <p v-if="statsByLoader.length === 0" class="text-xs text-gray-400">无数据</p>
        </div>
      </div>

      <!-- 按主版本号分类 -->
      <div class="rounded-md border border-gray-200 p-4">
        <p class="mb-3 text-sm font-medium text-gray-900">按主版本号分布</p>
        <div class="space-y-2">
          <div v-for="item in statsByMajor" :key="item.major" class="space-y-1">
            <div class="flex items-center justify-between text-xs">
              <span class="text-gray-700">{{ item.major }}</span>
              <span class="text-gray-500">{{ item.count }}</span>
            </div>
            <div class="h-1.5 w-full overflow-hidden rounded-full bg-gray-100">
              <div
                class="h-full bg-primary-400 transition-all duration-500"
                :style="{ width: `${(item.count / maxMajorCount) * 100}%` }"
              />
            </div>
          </div>
          <p v-if="statsByMajor.length === 0" class="text-xs text-gray-400">无数据</p>
        </div>
      </div>
    </div>
  </div>
</template>
