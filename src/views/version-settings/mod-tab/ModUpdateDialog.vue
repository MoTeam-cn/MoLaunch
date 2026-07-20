<script setup lang="ts">
/**
 * Mod 版本更新/更改对话框
 *
 * 功能：
 * - 查询当前 mod 的平台工程版本列表（CurseForge / Modrinth）
 * - 按游戏版本和加载器过滤
 * - 显示版本号、发布日期、下载量、发布类型
 * - 选择版本后下载安装到 mods 目录
 * - 支持卸载旧版本（删除当前文件）或保留旧版本
 *
 * 采用 teleport + transition 自承载弹窗（与 ResourceDetail 一致），
 * 不使用 singleton Modal（Modal 仅适合简单确认/提示，不支持自定义宽度和表格内容）。
 */
import { ref, computed, watch } from 'vue'
import Tooltip from '@/components/common/Tooltip.vue'
import Button from '@/components/common/Button.vue'
import { getProjectVersions, downloadResourceToPath } from '@/utils/api/community'
import { getVersionModsDir, deleteMod, type ModInfo } from '@/utils/api/personalization'
import { formatBytes, formatDate, formatDownloads } from '@/utils/format'
import { versionChangeType, type VersionChangeType } from '@/utils/version'
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import type { ResourceVersion, Platform } from '@/types/community'
import {
  XMarkIcon,
  ArrowPathIcon,
  CheckCircleIcon,
  ArrowUpIcon,
  ArrowDownIcon,
} from '@heroicons/vue/24/outline'
// Mod 默认 logo（无平台工程 logo 时使用）
import defaultModLogo from '@/assets/Mods/default-min.png'

interface Props {
  visible: boolean
  /** 要更新/更改的 mod */
  mod: ModInfo | null
  /** 当前版本的游戏版本号（如 "1.20.1"） */
  mcVersion: string
  /** 当前版本的 ID */
  versionId: string
}

const props = defineProps<Props>()
const emit = defineEmits<{
  'update:visible': [val: boolean]
  /** 安装完成后触发，父组件刷新列表 */
  installed: []
}>()

// 版本列表状态
const loading = ref(false)
const versions = ref<ResourceVersion[]>([])
const error = ref('')
const installing = ref(false)

// 过滤器状态（用户不可切换，由当前整合包的 MC 版本和加载器自动确定）
// 更新/更换 mod 时游戏版本和加载器是固定的，不可能切换
const selectedGameVersion = ref<string>('')
const selectedLoader = ref<string>('')

// 过滤后的版本列表（按当前整合包的 MC 版本 + 加载器自动筛选）
const filteredVersions = computed(() => {
  let result = versions.value
  // 按游戏版本过滤
  if (selectedGameVersion.value) {
    result = result.filter(v => v.game_versions.includes(selectedGameVersion.value))
  }
  // 按加载器过滤
  if (selectedLoader.value) {
    const loaderNum = loaderToFlag(selectedLoader.value)
    if (loaderNum > 0) {
      result = result.filter(v => (v.mod_loaders & loaderNum) !== 0)
    }
  }
  return result
})

// 选中的版本对象
const selectedVersion = computed(() =>
  filteredVersions.value.find(v => v.id === selectedVersionId.value) || null,
)

// 选中的版本
const selectedVersionId = ref<string | null>(null)

// 平台（优先 CurseForge，回退 Modrinth）
const platform = computed<Platform | null>(() => {
  if (!props.mod?.project) return null
  return props.mod.project.platform
})

// 当前 mod 的加载器类型
const modLoaderType = computed(() => props.mod?.loader_type || 'unknown')

/**
 * 选中版本相对于当前 mod 版本的变化类型
 *
 * 使用语义化版本比较（而非字符串相等），正确识别升级/降级/同版本。
 * 当 mod.version 或 selectedVersion 为空时返回 'unknown'。
 */
const versionChange = computed<VersionChangeType>(() => {
  if (!props.mod?.version || !selectedVersion.value?.version) return 'unknown'
  return versionChangeType(props.mod.version, selectedVersion.value.version)
})

// 加载器名称转 flag
function loaderToFlag(loader: string): number {
  const flags: Record<string, number> = {
    forge: 1,
    liteloader: 2,
    fabric: 4,
    quilt: 8,
    neoforge: 16,
  }
  return flags[loader] || 0
}

// 查询版本列表
async function loadVersions() {
  if (!props.mod?.project || !platform.value) {
    error.value = '此 Mod 没有关联的平台工程信息，无法查询版本'
    return
  }

  loading.value = true
  error.value = ''
  versions.value = []

  try {
    const result = await getProjectVersions(platform.value, props.mod.project.id)
    versions.value = result

    // 自动用当前整合包的 MC 版本和加载器过滤（用户不可切换）
    if (props.mcVersion) {
      selectedGameVersion.value = props.mcVersion
    }
    if (modLoaderType.value !== 'unknown') {
      selectedLoader.value = modLoaderType.value
    }

    // 自动选中第一个（最新版本）
    if (filteredVersions.value.length > 0) {
      selectedVersionId.value = filteredVersions.value[0].id
    }
  } catch (e: any) {
    error.value = typeof e === 'string' ? e : (e?.message || String(e))
  } finally {
    loading.value = false
  }
}

// 安装选中的版本（使用 showConfirm 回调模式）
function installSelected() {
  if (!selectedVersion.value || !props.mod) return

  const version = selectedVersion.value
  const oldFileName = props.mod.file_name

  showConfirm(
    '确认安装',
    `将下载 ${version.version} 并替换当前文件 ${oldFileName}。\n\n新文件名：${version.file_name}\n大小：${formatBytes(version.size)}`,
    async () => {
      installing.value = true
      try {
        // 获取 mods 目录
        const modsDir = await getVersionModsDir(props.versionId)

        // 下载新版本到 mods 目录
        await downloadResourceToPath(version.download_url, version.file_name, modsDir)

        // 删除旧版本文件（如果文件名不同）
        if (version.file_name !== oldFileName && version.file_name !== props.mod.enabled_name) {
          try {
            await deleteMod(props.versionId, oldFileName)
          } catch {
            // 删除旧文件失败不阻断流程
          }
        }

        toastSuccess(`已安装 ${version.version}`)
        emit('installed')
        emit('update:visible', false)
      } catch (e: any) {
        const msg = typeof e === 'string' ? e : (e?.message || String(e))
        toastError(`安装失败：${msg}`)
      } finally {
        installing.value = false
      }
    },
  )
}

// 监听 visible 变化，打开时加载版本
watch(() => props.visible, async (val) => {
  if (val && props.mod) {
    await loadVersions()
  } else {
    // 关闭时重置状态
    versions.value = []
    error.value = ''
    selectedVersionId.value = null
    selectedGameVersion.value = ''
    selectedLoader.value = ''
  }
})

// 发布类型样式
function releaseTypeClass(type: string): string {
  switch (type) {
    case 'Release': return 'bg-green-100 text-green-700'
    case 'Beta': return 'bg-blue-100 text-blue-700'
    case 'Alpha': return 'bg-yellow-100 text-yellow-700'
    default: return 'bg-gray-100 text-gray-600'
  }
}
</script>

<template>
  <teleport to="body">
    <transition
      enter-active-class="transition ease-out duration-150"
      enter-from-class="opacity-0"
      enter-to-class="opacity-100"
      leave-active-class="transition ease-in duration-100"
      leave-from-class="opacity-100"
      leave-to-class="opacity-0"
    >
      <div
        v-if="visible && mod"
        class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
        @click.self="$emit('update:visible', false)"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="relative w-full max-w-2xl bg-white rounded-lg shadow-xl flex flex-col max-h-[85vh]">
          <!-- 标题栏 -->
          <div class="flex items-center justify-between px-5 py-3 border-b border-gray-200">
            <h3 class="text-sm font-semibold text-gray-900 flex items-center gap-2">
              <ArrowPathIcon class="w-4 h-4 text-blue-500" />
              更新 / 更改 Mod 版本
            </h3>
            <Button
              type="ghost"
              size="small"
              @click="$emit('update:visible', false)"
            >
              <template #icon><XMarkIcon class="w-5 h-5" /></template>
            </Button>
          </div>

          <!-- 内容区 -->
          <div class="flex-1 overflow-y-auto p-5">
            <div class="flex flex-col gap-3">
              <!-- 当前 mod 信息 -->
              <div class="flex items-center gap-3 p-3 bg-gray-50 rounded-lg">
                <img
                  :src="mod.cached_logo_url || defaultModLogo"
                  class="w-10 h-10 rounded-lg object-cover"
                  alt=""
                  @error="(e) => { (e.target as HTMLImageElement).src = defaultModLogo }"
                >
                <div class="flex-1 min-w-0">
                  <div class="text-sm font-medium text-gray-800 truncate">{{ mod.project?.raw_name || mod.file_name }}</div>
                  <div class="text-xs text-gray-500">
                    当前版本：{{ mod.version || '未知' }}
                    <span v-if="mod.project" class="ml-2 text-gray-400">·</span>
                    <span v-if="mod.project" class="ml-2">{{ mod.project.platform }}</span>
                  </div>
                </div>
              </div>

              <!-- 加载中 -->
              <div v-if="loading" class="flex items-center justify-center py-8">
                <svg class="animate-spin w-6 h-6 text-blue-500" viewBox="0 0 24 24" fill="none">
                  <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4" />
                  <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4z" />
                </svg>
                <span class="ml-2 text-sm text-gray-500">正在查询版本列表...</span>
              </div>

              <!-- 错误 -->
              <div v-else-if="error" class="p-4 bg-red-50 rounded-lg">
                <p class="text-sm text-red-600">{{ error }}</p>
              </div>

              <!-- 无版本数据 -->
              <div v-else-if="versions.length === 0" class="py-8 text-center">
                <p class="text-sm text-gray-500">未找到任何版本信息</p>
              </div>

              <!-- 版本列表 -->
              <div v-else class="border border-gray-200 rounded-lg overflow-hidden">
                <div class="max-h-80 overflow-y-auto">
                  <table class="w-full text-sm">
                    <thead class="sticky top-0 bg-gray-50 text-xs text-gray-500">
                      <tr>
                        <th class="w-8 px-2 py-2"></th>
                        <th class="px-2 py-2 text-left">文件名</th>
                        <th class="px-2 py-2 text-left">发布日期</th>
                        <th class="px-2 py-2 text-left">类型</th>
                        <th class="px-2 py-2 text-right">大小</th>
                      </tr>
                    </thead>
                    <tbody class="divide-y divide-gray-100">
                      <tr
                        v-for="ver in filteredVersions"
                        :key="ver.id"
                        class="cursor-pointer transition-colors"
                        :class="selectedVersionId === ver.id
                          ? 'bg-blue-50'
                          : 'hover:bg-gray-50'"
                        @click="selectedVersionId = ver.id"
                      >
                        <td class="px-2 py-2 text-center">
                          <CheckCircleIcon
                            v-if="selectedVersionId === ver.id"
                            class="w-4 h-4 text-blue-500 inline-block"
                          />
                        </td>
                        <td class="px-2 py-2">
                          <Tooltip
                            v-if="ver.file_name.length > 28"
                            :text="ver.file_name"
                            position="top"
                            :delay="200"
                          >
                            <div class="text-gray-800 truncate max-w-[260px] cursor-help">{{ ver.file_name }}</div>
                          </Tooltip>
                          <div v-else class="text-gray-800 truncate max-w-[260px]">{{ ver.file_name }}</div>
                        </td>
                        <td class="px-2 py-2 text-xs text-gray-500">{{ formatDate(ver.release_date) }}</td>
                        <td class="px-2 py-2">
                          <span class="text-[10px] px-1.5 py-0.5 rounded font-medium" :class="releaseTypeClass(ver.release_type)">
                            {{ ver.release_type }}
                          </span>
                        </td>
                        <td class="px-2 py-2 text-xs text-gray-500 text-right">{{ formatBytes(ver.size, 1) }}</td>
                      </tr>
                    </tbody>
                  </table>
                </div>
              </div>
            </div>
          </div>

          <!-- 底部操作栏 -->
          <div class="flex items-center justify-between gap-3 px-5 py-3 border-t border-gray-200 bg-gray-50 rounded-b-lg">
            <!-- 左侧：版本变化徽章 + 下载量 -->
            <div v-if="selectedVersion" class="flex items-center gap-2 min-w-0">
              <!-- 胶囊式版本变化徽章：图标 + 旧版本(删除线) + 箭头 + 新版本(高亮) -->
              <div
                class="flex items-center gap-1 pl-2 pr-2.5 py-1 rounded-full border transition-colors"
                :class="{
                  'bg-green-50 border-green-200': versionChange === 'upgrade',
                  'bg-amber-50 border-amber-200': versionChange === 'downgrade',
                  'bg-gray-100 border-gray-200': versionChange === 'same',
                  'bg-blue-50 border-blue-200': versionChange === 'unknown',
                }"
              >
                <!-- 状态图标 -->
                <ArrowUpIcon v-if="versionChange === 'upgrade'" class="w-3.5 h-3.5 text-green-600 shrink-0" />
                <ArrowDownIcon v-else-if="versionChange === 'downgrade'" class="w-3.5 h-3.5 text-amber-600 shrink-0" />
                <CheckCircleIcon v-else-if="versionChange === 'same'" class="w-3.5 h-3.5 text-gray-500 shrink-0" />
                <span v-else class="w-1.5 h-1.5 rounded-full bg-blue-400 shrink-0"></span>

                <!-- 旧版本（删除线，表示将被替换） -->
                <span
                  v-if="mod.version && versionChange !== 'same'"
                  class="text-xs font-mono text-gray-400 line-through decoration-gray-300"
                >{{ mod.version }}</span>

                <!-- 箭头 -->
                <span v-if="versionChange !== 'same'" class="text-xs text-gray-400">→</span>

                <!-- 新版本（彩色高亮） -->
                <span
                  class="text-xs font-mono font-semibold"
                  :class="{
                    'text-green-700': versionChange === 'upgrade',
                    'text-amber-700': versionChange === 'downgrade',
                    'text-gray-700': versionChange === 'same',
                    'text-blue-700': versionChange === 'unknown',
                  }"
                >{{ selectedVersion.version || '?' }}</span>
              </div>

              <!-- 下载量 -->
              <span
                v-if="selectedVersion.download_count > 0"
                class="text-xs text-gray-400 whitespace-nowrap"
              >
                · {{ formatDownloads(selectedVersion.download_count) }} 次下载
              </span>
            </div>
            <div v-else class="flex-1"></div>
            <!-- 右侧：操作按钮 -->
            <div class="flex gap-2 shrink-0">
              <Button
                type="ghost"
                @click="$emit('update:visible', false)"
              >
                取消
              </Button>
              <Button
                type="primary"
                :loading="installing"
                :disabled="!selectedVersion || installing"
                @click="installSelected"
              >
                {{ installing ? '安装中...' : '安装' }}
              </Button>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
