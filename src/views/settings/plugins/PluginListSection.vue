<script setup lang="ts">
/**
 * 已安装插件列表（含权限 tag 展示、启用/禁用切换、卸载、刷新）
 */
import { computed, defineAsyncComponent } from 'vue'
import { usePluginStore } from '@/stores/plugins'
import {
  PERMISSION_REGISTRY,
  getPermissionMeta,
  type PermissionMeta,
} from '@/plugins/permissions'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import { toastSuccess, toastError } from '@/utils/toast'
import { showConfirmAsync } from '@/utils/modal'
import {
  PuzzlePieceIcon,
  ComputerDesktopIcon,
  CloudArrowDownIcon,
  TrashIcon,
  ArrowPathIcon,
  InformationCircleIcon,
  ShieldCheckIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'

const pluginStore = usePluginStore()

/** 插件列表（按 ID 排序，内置在前） */
const sortedPlugins = computed(() =>
  [...pluginStore.manifests].sort((a, b) => {
    if (a.builtin !== b.builtin) return a.builtin ? -1 : 1
    return a.id.localeCompare(b.id)
  }),
)

/** 已启用数量 */
const enabledCount = computed(
  () => pluginStore.manifests.filter((m) => pluginStore.runtimeStates[m.id]?.enabled).length,
)

/** 内置插件数量 */
const builtinCount = computed(() => pluginStore.manifests.filter((m) => m.builtin).length)

/** 外部插件数量 */
const externalCount = computed(() => pluginStore.manifests.filter((m) => !m.builtin).length)

/** 获取插件能力描述 */
function getCapabilityText(manifestId: string): string {
  const manifest = pluginStore.manifests.find((m) => m.id === manifestId)
  if (!manifest) return ''
  const caps = manifest.capabilities?.()
  const parts: string[] = []
  if (caps?.homePanel) parts.push('主页内容区')
  if (caps?.settingsPanel) parts.push('插件设置')
  return parts.length > 0 ? parts.join(' / ') : '无'
}

/**
 * 获取插件已声明的权限列表
 *
 * - 内置插件：返回 null（不受沙箱限制）
 * - 外部插件：返回 manifest.permissions 数组
 */
function getDeclaredPermissions(manifestId: string): string[] | null {
  const manifest = pluginStore.manifests.find((m) => m.id === manifestId)
  if (!manifest) return null
  if (manifest.builtin) return null
  return manifest.permissions ?? []
}

/** 获取权限的元信息（带 fallback） */
function getPermMeta(name: string): PermissionMeta {
  return (
    getPermissionMeta(name) ?? {
      name,
      description: '未知权限',
      useCase: '未在权限注册表中找到说明',
      risk: 'medium' as const,
    }
  )
}

/** 切换插件启用状态 */
async function onTogglePlugin(pluginId: string, value: string | number) {
  try {
    await pluginStore.setPluginEnabled(pluginId, value === 'true')
  } catch (e) {
    toastError('切换插件状态失败：' + e)
  }
}

/** 卸载 */
async function onUninstallPlugin(pluginId: string, pluginName: string) {
  const confirmed = await showConfirmAsync(
    '确认卸载',
    `确定卸载插件「${pluginName}」吗？此操作将删除插件目录，不可恢复。`,
  )
  if (!confirmed) return
  try {
    await pluginStore.uninstallExternal(pluginId)
    toastSuccess(`插件已卸载：${pluginName}`)
  } catch (e) {
    toastError('卸载插件失败：' + e)
  }
}

/** 刷新 */
async function onRefresh() {
  try {
    await pluginStore.loadExternalPlugins()
    toastSuccess('已刷新外部插件列表')
  } catch (e) {
    toastError('刷新失败：' + e)
  }
}
</script>

<template>
  <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
    <div class="flex items-center justify-between px-5 pt-5 pb-3">
      <h3 class="text-sm font-semibold text-gray-900">已安装插件</h3>
      <div class="flex items-center gap-3">
        <span class="text-xs text-gray-500">
          共 {{ sortedPlugins.length }} 个（内置 {{ builtinCount }} · 外部 {{ externalCount }}）· 已启用 {{ enabledCount }} 个
        </span>
        <Button
          type="ghost"
          size="mini"
          @click="onRefresh"
        >
          <template #icon><ArrowPathIcon class="h-3.5 w-3.5" /></template>
          刷新
        </Button>
      </div>
    </div>

    <!-- 空状态 -->
    <div
      v-if="sortedPlugins.length === 0"
      class="flex h-full flex-col items-center justify-center px-5 py-12"
    >
      <PuzzlePieceIcon class="mb-3 h-10 w-10 text-gray-300" />
      <p class="text-sm text-gray-500">暂无已安装的插件</p>
      <p class="mt-1 text-xs text-gray-400">可从下方「外部插件」区域从文件夹或 ZIP 安装</p>
    </div>

    <!-- 列表 -->
    <div v-else class="divide-y divide-gray-200">
      <div v-for="manifest in sortedPlugins" :key="manifest.id" class="px-5 py-4">
        <div class="flex items-start justify-between gap-4">
          <!-- 左侧：插件信息 -->
          <div class="min-w-0 flex-1">
            <div class="flex flex-wrap items-center gap-2">
              <p class="text-sm font-medium text-gray-900">{{ manifest.name }}</p>
              <span
                class="inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 text-[10px] font-medium"
                :class="manifest.builtin
                  ? 'bg-gray-100 text-gray-600'
                  : 'bg-blue-50 text-blue-700'"
              >
                <ComputerDesktopIcon v-if="manifest.builtin" class="h-3 w-3" />
                <CloudArrowDownIcon v-else class="h-3 w-3" />
                {{ manifest.builtin ? '内置' : '外部' }}
              </span>
              <Tooltip :text="`能力：${getCapabilityText(manifest.id)}`">
                <span class="inline-flex items-center gap-0.5 rounded bg-primary-50 px-1.5 py-0.5 text-[10px] font-medium text-primary-700">
                  <InformationCircleIcon class="h-3 w-3" />
                  {{ getCapabilityText(manifest.id) }}
                </span>
              </Tooltip>
            </div>
            <p class="mt-1 text-xs text-gray-500">{{ manifest.description }}</p>
            <p class="mt-1 text-[11px] text-gray-400">
              v{{ manifest.version }} · {{ manifest.author }} · ID: {{ manifest.id }}
            </p>

            <!-- 已声明权限（tag 列表，带 Tooltip 说明） -->
            <div class="mt-2">
              <p class="mb-1 text-[11px] text-gray-500">已声明权限：</p>
              <div class="flex flex-wrap items-center gap-1">
                <!-- 内置插件：全部 -->
                <span
                  v-if="getDeclaredPermissions(manifest.id) === null"
                  class="inline-flex items-center rounded bg-green-50 px-1.5 py-0.5 text-[10px] font-medium text-green-700"
                >
                  <ShieldCheckIcon class="mr-0.5 h-2.5 w-2.5" />
                  全部（无沙箱限制）
                </span>
                <!-- 外部插件：声明的权限 tag（带 Tooltip） -->
                <template v-else>
                  <Tooltip
                    v-for="perm in getDeclaredPermissions(manifest.id)"
                    :key="perm"
                    :text="`${getPermMeta(perm).description} — ${getPermMeta(perm).useCase}`"
                  >
                    <span
                      class="inline-flex items-center gap-0.5 rounded px-1.5 py-0.5 text-[10px] font-medium"
                      :class="getPermMeta(perm).risk === 'high'
                        ? 'bg-red-50 text-red-700'
                        : 'bg-blue-50 text-blue-700'"
                    >
                      <ExclamationTriangleIcon v-if="getPermMeta(perm).risk === 'high'" class="h-2.5 w-2.5" />
                      {{ perm }}
                    </span>
                  </Tooltip>
                  <!-- 始终允许的权限（灰色） -->
                  <span
                    v-for="perm in PERMISSION_REGISTRY.filter((p) => p.alwaysAllowed)"
                    :key="perm.name"
                    class="inline-flex items-center rounded bg-gray-100 px-1.5 py-0.5 text-[10px] font-medium text-gray-500"
                  >
                    {{ perm.name }} *
                  </span>
                  <span
                    v-if="getDeclaredPermissions(manifest.id)?.length === 0"
                    class="text-[10px] text-gray-400"
                  >
                    （未声明任何 SDK 权限）
                  </span>
                </template>
              </div>
            </div>

            <!-- 错误信息 -->
            <p
              v-if="pluginStore.runtimeStates[manifest.id]?.lastError"
              class="mt-1 text-[11px] text-red-500"
            >
              最近错误：{{ pluginStore.runtimeStates[manifest.id]?.lastError }}
            </p>
          </div>

          <!-- 右侧：操作区 -->
          <div class="flex flex-none flex-col items-end gap-2">
            <div class="w-32">
              <Select
                :model-value="pluginStore.runtimeStates[manifest.id]?.enabled ? 'true' : 'false'"
                :options="[
                  { label: '已启用', value: 'true' },
                  { label: '已禁用', value: 'false' },
                ]"
                @update:model-value="onTogglePlugin(manifest.id, $event)"
              />
            </div>
            <!-- 保留原生 button：卸载按钮（icon+text，自定义 px-2 py-1 text-[11px] text-red-500），
                 Button.vue 的 scoped size 类固定 padding 会破坏紧凑尺寸 -->
            <button
              v-if="!manifest.builtin"
              class="inline-flex items-center gap-1 rounded px-2 py-1 text-[11px] text-red-500 hover:bg-red-50"
              @click="onUninstallPlugin(manifest.id, manifest.name)"
            >
              <TrashIcon class="h-3 w-3" />
              卸载
            </button>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
