<script setup lang="ts">
/**
 * 已安装游戏版本 Java 环境检测
 *
 * 对每个已安装的游戏版本，解析其 MC 版本与加载器 → 查询 Java 需求（min/max/recommended）
 * → 判断当前系统已安装的 Java 是否满足。不满足时提供一键预下载对应 Java 到 runtime 目录。
 *
 * - 兼容判断复用 `isJavaCompatible`（纯函数），需求描述复用 `describeJavaRequirement`
 * - 下载复用 `JavaDownloadBar`（按 targetMajor 驱动），完成后刷新 store 列表
 * - 单个版本探测失败不影响其他版本
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
import {
  ShieldCheckIcon,
  ArrowPathIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const JavaDownloadBar = defineAsyncComponent(() => import('@/views/version-settings/JavaDownloadBar.vue'))
import { useJavaStore } from '@/stores/java'
import { toastSuccess, toastError } from '@/utils/toast'
import {
  listInstalledVersionsWithType,
  getVersionLoaderInfo,
} from '@/utils/api/version'
import { getVersionGameVersion } from '@/utils/api/personalization/version'
import {
  getJavaRequirements,
  describeJavaRequirement,
  isJavaCompatible,
} from '@/utils/api/java'
import type { JavaRequirements } from '@/types/java'

interface EnvRow {
  id: string
  mcVersion: string
  loader: string
  reqs: JavaRequirements | null
  /** 已安装的 Java 中是否存在满足需求的版本 */
  compatible: boolean
}

/** 与后端 java_selector 规则一致的加载器类型集合（其余类型按原版处理） */
const LOADER_TYPES = ['forge', 'neoforge', 'fabric', 'quilt', 'optifine', 'liteloader']

const javaStore = useJavaStore()
const rows = ref<EnvRow[]>([])
const loading = ref(false)
const loaded = ref(false)

function normalizeLoader(type: string): string | null {
  return LOADER_TYPES.includes(type) ? type : null
}

/** 行内推荐的下载 Java 大版本号（无需求/无需下载时为 0） */
function downloadMajorFor(row: EnvRow): number {
  if (row.compatible || !row.reqs) return 0
  return row.reqs.recommended_java_version || row.reqs.min_java_version || 0
}

async function check() {
  loading.value = true
  try {
    if (!javaStore.javaLoaded) await javaStore.detectJava()
    const versions = await listInstalledVersionsWithType()
    const list: EnvRow[] = []
    for (const v of versions) {
      let reqs: JavaRequirements | null = null
      let mcVersion = ''
      let loader = ''
      try {
        const [mc, loaderInfo] = await Promise.all([
          getVersionGameVersion(v.id),
          getVersionLoaderInfo(v.id),
        ])
        mcVersion = mc ?? ''
        loader = loaderInfo.loaderType
        const loaderForReq = normalizeLoader(loader)
        if (mcVersion) {
          reqs = await getJavaRequirements(mcVersion, loaderForReq)
        }
      } catch {
        // 单个版本探测失败（如 JSON 缺失）不阻断其他版本
      }
      const compatible = reqs
        ? javaStore.javaList.some((j) => isJavaCompatible(j.major_version, reqs))
        : true
      list.push({ id: v.id, mcVersion, loader, reqs, compatible })
    }
    rows.value = list
    loaded.value = true
  } catch (e) {
    toastError(`环境检测失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
}

async function refresh() {
  await javaStore.refreshJava()
  await check()
  toastSuccess('已重新检测')
}

async function onDownloaded() {
  await javaStore.listJava()
  toastSuccess('Java 已下载，正在重新校验兼容状态')
  await check()
}

onMounted(check)
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <ShieldCheckIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">已安装版本 Java 环境检测</h3>
      <span class="ml-auto text-xs text-gray-400">{{ rows.length }} 个版本</span>
      <Button type="outline" size="small" :loading="loading" @click="refresh">
        <template #icon><ArrowPathIcon class="h-4 w-4" /></template>
        重新检测
      </Button>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <AlertV2
        type="info"
        message="逐个检查已安装的游戏版本是否需要尚未安装的 Java。缺少时可直接一键预下载到固定 runtime 目录，之后启动该版本时会自动命中。"
      />

      <!-- 加载中 -->
      <div
        v-if="loading && rows.length === 0"
        class="flex flex-col items-center justify-center py-8 text-gray-400"
      >
        <ArrowPathIcon class="h-8 w-8 mb-2 animate-spin" />
        <span class="text-xs">正在检测各版本的 Java 需求...</span>
      </div>

      <!-- 版本行列表 -->
      <div v-else-if="rows.length > 0" class="space-y-2">
        <div
          v-for="row in rows"
          :key="row.id"
          class="rounded-lg border border-gray-200 bg-white px-3 py-2.5"
        >
          <div class="flex items-center gap-3">
            <div class="flex-1 min-w-0">
              <div class="truncate text-sm font-medium text-gray-900">{{ row.id }}</div>
              <div class="mt-0.5 text-xs text-gray-400">
                <template v-if="row.mcVersion">
                  MC {{ row.mcVersion }}
                  <span v-if="row.loader && row.loader !== 'release'"> · {{ row.loader }}</span>
                </template>
                <template v-else>无法识别游戏版本号</template>
              </div>
            </div>

            <div class="text-xs text-gray-500">
              {{ row.reqs ? describeJavaRequirement(row.reqs) : '—' }}
            </div>

            <Tag
              size="small"
              :color="row.compatible ? 'green' : row.reqs ? 'red' : 'gray'"
              class="flex-none"
            >
              {{ row.compatible ? '已满足' : row.reqs ? '缺 Java' : '未知需求' }}
            </Tag>
          </div>

          <!-- 缺 Java 时的一键下载（目标版本为 recommended||min） -->
          <div v-if="downloadMajorFor(row) > 0" class="mt-2 flex items-center gap-2">
            <ExclamationTriangleIcon class="h-4 w-4 flex-none text-amber-500" />
            <span class="text-xs text-amber-600">当前没有可用的 Java 满足该版本需求</span>
            <div class="ml-auto">
              <JavaDownloadBar :target-major="downloadMajorFor(row)" @downloaded="onDownloaded" />
            </div>
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-else-if="loaded"
        class="flex flex-col items-center justify-center py-8 text-gray-400"
      >
        <ShieldCheckIcon class="h-8 w-8 mb-2" />
        <span class="text-xs">未检测到已安装的游戏版本</span>
      </div>
    </div>
  </section>
</template>
