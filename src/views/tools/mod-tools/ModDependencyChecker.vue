<script setup lang="ts">
/**
 * Mod 依赖检测
 *
 * 选择已安装版本 → 调用 modDependencyCheck → 展示缺失依赖列表。
 * 缺失依赖指某 mod 声明依赖的 mod_id 不在 mods 目录已安装集合中
 * （排除 minecraft / java / fabricloader / fabric-api 等内置依赖）。
 */
import { ref, onMounted, defineAsyncComponent } from 'vue'
import {
  PuzzlePieceIcon,
  MagnifyingGlassIcon,
  CheckCircleIcon,
  ExclamationCircleIcon,
  ArrowRightIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { toastSuccess, toastWarning, toastError } from '@/utils/toast'
import { modDependencyCheck } from '@/utils/api/tools'
import type { ModDependencyResult, MissingDep } from '@/utils/api/tools'
import { listInstalledVersionsWithType } from '@/utils/api/version'
import type { InstalledVersionInfo } from '@/utils/api/version'

const versions = ref<InstalledVersionInfo[]>([])
const selectedVersion = ref<string>('')
const loading = ref(false)
const result = ref<ModDependencyResult | null>(null)

const versionOptions = ref<{ label: string; value: string }[]>([])

onMounted(async () => {
  try {
    versions.value = await listInstalledVersionsWithType()
    versionOptions.value = versions.value.map((v) => ({ label: v.id, value: v.id }))
  } catch (e) {
    toastError(`加载版本列表失败: ${e instanceof Error ? e.message : String(e)}`)
  }
})

async function runCheck() {
  if (!selectedVersion.value) {
    toastError('请先选择游戏版本')
    return
  }
  loading.value = true
  result.value = null
  try {
    result.value = await modDependencyCheck(selectedVersion.value)
    if (result.value.missing.length > 0) {
      toastWarning('检测完成，发现 ' + result.value.missing.length + ' 个缺失依赖')
    } else {
      toastSuccess('检测完成，所有依赖均已满足')
    }
  } catch (e) {
    toastError(`依赖检测失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    loading.value = false
  }
}

function depGroups(missing: MissingDep[]): Map<string, string[]> {
  // 按 required_by 分组：required_by -> [mod_id, ...]
  const map = new Map<string, string[]>()
  for (const dep of missing) {
    if (!map.has(dep.required_by)) map.set(dep.required_by, [])
    map.get(dep.required_by)!.push(dep.mod_id)
  }
  return map
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <PuzzlePieceIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">Mod 依赖检测</h3>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        扫描所选版本 mods 目录下所有 mod 的依赖声明，找出缺失的依赖项（不含 minecraft / java / 加载器等内置依赖）。
      </p>

      <!-- 版本选择 + 检测按钮 -->
      <div class="flex items-end gap-3">
        <div class="flex-1">
          <label class="mb-1 block text-xs font-medium text-gray-700">选择游戏版本</label>
          <Select
            v-model="selectedVersion"
            :options="versionOptions"
            placeholder="请选择版本"
          />
        </div>
        <Button type="primary" :loading="loading" :disabled="!selectedVersion" @click="runCheck">
          <template #icon><MagnifyingGlassIcon class="h-4 w-4" /></template>
          {{ loading ? '检测中...' : '开始检测' }}
        </Button>
      </div>

      <!-- 结果区 -->
      <div v-if="result">
        <div v-if="result.missing.length === 0" class="flex flex-col items-center justify-center py-8 text-gray-400">
          <CheckCircleIcon class="h-8 w-8 mb-2 text-green-400" />
          <span class="text-xs">所有 mod 依赖均已满足，无缺失项</span>
        </div>

        <div v-else class="space-y-3">
          <div class="flex items-center gap-2 text-sm text-red-600">
            <ExclamationCircleIcon class="h-4 w-4" />
            发现 {{ result.missing.length }} 个缺失依赖（来自 {{ depGroups(result.missing).size }} 个 mod）
          </div>

          <div
            v-for="[requiredBy, depIds] in depGroups(result.missing)"
            :key="requiredBy"
            class="rounded-lg border border-gray-200 overflow-hidden"
          >
            <div class="flex items-center gap-2 bg-gray-50 px-3 py-2">
              <PuzzlePieceIcon class="h-4 w-4 flex-none text-primary-500" />
              <Tooltip :text="requiredBy" position="top" :delay="200" block>
                <span class="truncate text-sm font-medium text-gray-800">{{ requiredBy }}</span>
              </Tooltip>
              <span class="ml-auto flex-none text-xs text-gray-400">{{ depIds.length }} 个缺失</span>
            </div>
            <div class="divide-y divide-gray-100">
              <div
                v-for="depId in depIds"
                :key="depId"
                class="flex items-center gap-2 px-3 py-2"
              >
                <ArrowRightIcon class="h-3 w-3 flex-none text-gray-300" />
                <code class="text-sm text-red-600">{{ depId }}</code>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </section>
</template>
