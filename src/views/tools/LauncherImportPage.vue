<script setup lang="ts">
/**
 * 启动器数据导入：探测外部启动器实例并迁移到 MoLaunch
 *
 * 支持来源：PCL2 / PCL2CE / HMCL / MultiMC / Prism Launcher / CurseForge / 手动选择文件夹。
 * 导入模式：复制（与原实例独立）或符号链接（共享数据，节省空间）。
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'
import { InboxIcon } from '@heroicons/vue/24/outline'
import {
  listLauncherSources,
  runLauncherImport,
  scanGenericPath,
  type ImportResultItem,
  type LauncherSource,
} from '@/utils/api/tools/launcher-import'
import { pickDirectory } from '@/utils/fileDialog'
import { toastError, toastInfo, toastSuccess } from '@/utils/toast'

const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))

const sources = ref<LauncherSource[]>([])
const loading = ref(false)
const scanning = ref(false)
const importing = ref(false)
const symlink = ref(false)
const selected = ref<Set<string>>(new Set())
const results = ref<Record<string, ImportResultItem>>({})

const selectedCount = computed(() => selected.value.size)

function instanceKey(source: LauncherSource, path: string): string {
  return `${source.kind}|${path}`
}

async function loadSources() {
  loading.value = true
  try {
    sources.value = await listLauncherSources()
    if (sources.value.length === 0) {
      toastInfo('未检测到其他启动器的实例，可点击「选择文件夹」手动导入')
    }
  } catch (e) {
    toastError(`扫描失败：${e}`)
  } finally {
    loading.value = false
  }
}

async function chooseFolder() {
  const dir = await pickDirectory()
  if (!dir) return
  scanning.value = true
  try {
    const source = await scanGenericPath(dir)
    const exists = sources.value.some((s) => s.base_path === source.base_path)
    if (!exists) {
      sources.value.push(source)
    }
    toastSuccess(`发现 ${source.instances.length} 个可导入实例`)
  } catch (e) {
    toastError(`扫描失败：${e}`)
  } finally {
    scanning.value = false
  }
}

function toggleSelect(source: LauncherSource, path: string) {
  const key = instanceKey(source, path)
  const next = new Set(selected.value)
  if (next.has(key)) {
    next.delete(key)
  } else {
    next.add(key)
  }
  selected.value = next
}

async function startImport() {
  if (selected.value.size === 0 || importing.value) return
  importing.value = true
  const targets = [...selected.value]
  let ok = 0
  for (const key of targets) {
    const [kind, path] = key.split('|')
    try {
      const result = await runLauncherImport({
        kind: kind as LauncherSource['kind'],
        source_path: path,
        symlink: symlink.value,
      })
      results.value = { ...results.value, [key]: result }
      if (result.success) ok++
    } catch (e) {
      results.value = {
        ...results.value,
        [key]: {
          name: path.split(/[\\/]/).pop() ?? path,
          success: false,
          message: String(e),
          mc_version: null,
          loader: null,
        },
      }
    }
  }
  importing.value = false
  if (ok > 0) {
    toastSuccess(`导入完成：成功 ${ok} 个，失败 ${targets.length - ok} 个，可在「版本管理」中查看`)
  } else {
    toastError('导入失败，请查看实例状态')
  }
}

function loaderBadge(loader: string | null): { text: string; cls: string } {
  switch (loader) {
    case 'forge':
      return { text: 'Forge', cls: 'bg-orange-100 text-orange-600' }
    case 'neoforge':
      return { text: 'NeoForge', cls: 'bg-red-100 text-red-600' }
    case 'fabric':
      return { text: 'Fabric', cls: 'bg-green-100 text-green-600' }
    case 'quilt':
      return { text: 'Quilt', cls: 'bg-purple-100 text-purple-600' }
    case 'optifine':
      return { text: 'OptiFine', cls: 'bg-blue-100 text-blue-600' }
    case 'liteloader':
      return { text: 'LiteLoader', cls: 'bg-gray-100 text-gray-600' }
    default:
      return { text: '原版', cls: 'bg-gray-100 text-gray-500' }
  }
}

onMounted(loadSources)
</script>

<template>
  <div class="flex h-full flex-col">
    <!-- 顶部操作区 -->
    <div class="flex items-center justify-between px-6 pb-4 pt-6">
      <div>
        <h3 class="text-lg font-semibold text-gray-800">启动器数据导入</h3>
        <p class="mt-1 text-sm text-gray-400">
          从 PCL2 / HMCL / MultiMC / CurseForge 等启动器导入实例，支持复制或符号链接（共享数据）
        </p>
      </div>
      <div class="flex gap-2">
        <Button type="outline" :loading="scanning" @click="chooseFolder">选择文件夹</Button>
        <Button type="secondary" :loading="loading" @click="loadSources">重新扫描</Button>
      </div>
    </div>

    <!-- 来源列表 -->
    <div class="flex-1 space-y-4 overflow-y-auto px-6 pb-4">
      <div
        v-for="source in sources"
        :key="source.kind + source.base_path"
        class="rounded-xl border border-gray-100 bg-white p-4 shadow-sm"
      >
        <div class="mb-3 flex items-center justify-between">
          <div class="flex items-center gap-2">
            <span class="text-sm font-semibold text-gray-700">{{ source.label }}</span>
            <span class="rounded bg-gray-100 px-1.5 py-0.5 text-xs text-gray-500">
              {{ source.instances.length }} 个实例
            </span>
          </div>
          <span class="truncate text-xs text-gray-400">{{ source.base_path }}</span>
        </div>
        <div class="space-y-1">
          <div
            v-for="inst in source.instances"
            :key="inst.path"
            class="flex cursor-pointer items-center gap-3 rounded-lg px-2 py-2 hover:bg-gray-50"
            @click="toggleSelect(source, inst.path)"
          >
            <Checkbox
              class="flex-none"
              :checked="selected.has(instanceKey(source, inst.path))"
              @click.stop
              @change="toggleSelect(source, inst.path)"
            />
            <div class="min-w-0 flex-1">
              <div class="flex items-center gap-2">
                <span class="truncate text-sm text-gray-700">{{ inst.name }}</span>
                <span v-if="inst.mc_version" class="rounded bg-blue-50 px-1.5 py-0.5 text-xs text-blue-600">
                  {{ inst.mc_version }}
                </span>
                <span :class="['rounded px-1.5 py-0.5 text-xs', loaderBadge(inst.loader).cls]">
                  {{ loaderBadge(inst.loader).text }}
                </span>
              </div>
              <div class="mt-0.5 truncate text-xs text-gray-400">{{ inst.path }}</div>
            </div>
            <span
              v-if="results[instanceKey(source, inst.path)]"
              :class="results[instanceKey(source, inst.path)].success ? 'text-xs text-green-500' : 'text-xs text-red-500'"
            >
              {{ results[instanceKey(source, inst.path)].message }}
            </span>
          </div>
        </div>
      </div>

      <!-- 空状态 -->
      <div
        v-if="!loading && sources.length === 0"
        class="flex h-64 flex-col items-center justify-center gap-3 text-gray-300"
      >
        <InboxIcon class="h-12 w-12" />
        <p class="text-sm">未检测到其他启动器的实例</p>
        <p class="text-xs">可点击右上角「选择文件夹」手动选择实例目录</p>
      </div>
    </div>

    <!-- 底部操作条 -->
    <div class="flex items-center justify-between border-t border-gray-100 bg-white px-6 py-3">
      <div class="flex items-center gap-4 text-sm text-gray-600">
        <label class="flex cursor-pointer items-center gap-1.5">
          <input v-model="symlink" type="radio" :value="false" class="accent-primary-500" />
          复制（与原实例独立）
        </label>
        <label class="flex cursor-pointer items-center gap-1.5">
          <input v-model="symlink" type="radio" :value="true" class="accent-primary-500" />
          符号链接（共享数据，节省空间）
        </label>
      </div>
      <div class="flex items-center gap-3">
        <span class="text-sm text-gray-500">已选 {{ selectedCount }} 个实例</span>
        <Button type="primary" :loading="importing" :disabled="selectedCount === 0" @click="startImport">
          开始导入
        </Button>
      </div>
    </div>
  </div>
</template>