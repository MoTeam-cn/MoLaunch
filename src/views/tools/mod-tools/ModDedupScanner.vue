<script setup lang="ts">
/**
 * Mod 文件去重
 *
 * 选择已安装版本 → 调用 modDedupScan → 展示重复 mod 列表。
 * 重复指同一 mod_id（slug）在 mods 目录下存在多个版本文件，
 * 可据此手动清理冗余旧版本。
 */
import { ref, onMounted } from 'vue'
import {
  Squares2X2Icon,
  MagnifyingGlassIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { toastError } from '@/utils/toast'
import { modDedupScan } from '@/utils/api/tools'
import type { ModDedupResult } from '@/utils/api/tools'
import { listInstalledVersionsWithType } from '@/utils/api/version'
import type { InstalledVersionInfo } from '@/utils/api/version'
import { formatBytes } from '@/utils/format'

const versions = ref<InstalledVersionInfo[]>([])
const selectedVersion = ref<string>('')
const loading = ref(false)
const result = ref<ModDedupResult | null>(null)

const versionOptions = ref<{ label: string; value: string }[]>([])

onMounted(async () => {
  try {
    versions.value = await listInstalledVersionsWithType()
    versionOptions.value = versions.value.map((v) => ({ label: v.id, value: v.id }))
  } catch (e) {
    toastError(`加载版本列表失败: ${e instanceof Error ? e.message : String(e)}`)
  }
})

async function runScan() {
  if (!selectedVersion.value) {
    toastError('请先选择游戏版本')
    return
  }
  loading.value = true
  result.value = null
  try {
    result.value = await modDedupScan(selectedVersion.value)
  } catch (e) {
    toastError(`去重扫描失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <Squares2X2Icon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">Mod 文件去重</h3>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        扫描所选版本 mods 目录，按 mod_id 分组找出存在多个版本的 mod，便于手动清理冗余旧版本。
      </p>

      <!-- 版本选择 + 扫描按钮 -->
      <div class="flex items-end gap-3">
        <div class="flex-1">
          <label class="mb-1 block text-xs font-medium text-gray-700">选择游戏版本</label>
          <Select
            v-model="selectedVersion"
            :options="versionOptions"
            placeholder="请选择版本"
          />
        </div>
        <Button type="primary" :loading="loading" :disabled="!selectedVersion" @click="runScan">
          <template #icon><MagnifyingGlassIcon class="h-4 w-4" /></template>
          {{ loading ? '扫描中...' : '开始扫描' }}
        </Button>
      </div>

      <!-- 结果区 -->
      <div v-if="result">
        <div v-if="result.duplicates.length === 0" class="flex flex-col items-center justify-center py-8 text-gray-400">
          <CheckCircleIcon class="h-8 w-8 mb-2 text-green-400" />
          <span class="text-xs">未发现重复 mod，mods 目录已很整洁</span>
        </div>

        <div v-else class="space-y-3">
          <div class="flex items-center gap-2 text-sm text-amber-600">
            <ExclamationCircleIcon class="h-4 w-4" />
            发现 {{ result.duplicates.length }} 个重复 mod
          </div>

          <div
            v-for="dup in result.duplicates"
            :key="dup.mod_id"
            class="rounded-lg border border-gray-200 overflow-hidden"
          >
            <div class="flex items-center gap-2 bg-gray-50 px-3 py-2">
              <Squares2X2Icon class="h-4 w-4 flex-none text-amber-500" />
              <code class="text-sm font-medium text-gray-800">{{ dup.mod_id }}</code>
              <span class="ml-auto flex-none text-xs text-gray-400">{{ dup.versions.length }} 个版本</span>
            </div>
            <div class="divide-y divide-gray-100">
              <div
                v-for="v in dup.versions"
                :key="v.file_name"
                class="flex items-center gap-3 px-3 py-2"
              >
                <span class="flex-none rounded bg-blue-100 px-1.5 py-0.5 text-xs font-medium text-blue-700">
                  {{ v.version }}
                </span>
                <Tooltip :text="v.file_name" position="top" :delay="200" block>
                  <span class="flex-1 min-w-0 truncate text-sm text-gray-700">{{ v.file_name }}</span>
                </Tooltip>
                <span class="flex-none text-xs text-gray-400">{{ formatBytes(v.file_size) }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
