<script setup lang="ts">
/**
 * 版本设置 - Mod 管理子页（顶层容器）
 *
 * 子组件位于 mod-tab/（ModToolbar / ModListItem / ModEmptyState / ModUpdateDialog），
 * 通用多选操作栏使用 `@/components/common/MultiSelectBar.vue`，
 * 通用多选状态管理使用 `@/composables/useMultiSelect`（在 useModOperations 内部初始化），
 * 业务逻辑（列表加载、过滤、启用/禁用、删除、安装、打开目录、详情查询、预加载监听、
 * 批量操作、版本更新）抽取到 `@/composables/useModOperations`。
 *
 * 多选交互：
 * - 点击列表项切换选中状态（非长按）
 * - Shift+点击 范围选择
 * - 有选中项时顶部显示 MultiSelectBar，所有项显示复选框列
 * - ESC 清空选中
 */
import { ref, computed, onMounted, onUnmounted, defineAsyncComponent } from 'vue'
import { useRouter } from 'vue-router'
import {
  PlayIcon,
  PauseIcon,
  ArrowPathIcon,
  TrashIcon,
} from '@heroicons/vue/24/outline'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { useModOperations } from '@/composables/useModOperations'
import MultiSelectBar, { type MultiSelectAction } from '@/components/common/MultiSelectBar.vue'
const ResourceDetail = defineAsyncComponent(() => import('@/components/community/ResourceDetail.vue'))
const ModListItem = defineAsyncComponent(() => import('./mod-tab/ModListItem.vue'))
const ModToolbar = defineAsyncComponent(() => import('./mod-tab/ModToolbar.vue'))
const ModEmptyState = defineAsyncComponent(() => import('./mod-tab/ModEmptyState.vue'))
const ModUpdateDialog = defineAsyncComponent(() => import('./mod-tab/ModUpdateDialog.vue'))

const router = useRouter()
const { selectedId, isModable } = useVersionSettings()
const modLocalNameStyle = ref(0)

const {
  mods, modsLoading, modFilter, modSearch,
  isModableVersion, checkingModable,
  versionGameVersion, versionModsDir, disableModUpdate,
  filteredMods, filterOptions,
  detailVisible, detailProject, detailLoadingFor,
  // 多选状态（来自 useMultiSelect）
  batchProcessing, selectedCount,
  // 按钮可用性判断
  hasEnabledSelected, hasDisabledSelected, hasUpdatableSelected,
  // Mod 更新对话框
  updateDialogVisible, updateTargetMod,
  // 生命周期
  stopPreloadListener, loadMods,
  // 基础 handler
  handleToggleMod, handleDeleteMod, handleInstallMod,
  handleOpenModsDir, handleOpenFile, onShowInfo, handleOpenWiki,
  // 多选操作
  toggleSelect, selectAll, invertSelection, clearSelection,
  checkSelected, handleMultiSelectKeydown,
  // 批量业务 handler
  batchToggle, batchDelete, batchUpdate,
  openUpdateDialog, onModUpdated,
  init: initModOperations,
} = useModOperations({ selectedId, isModable, modLocalNameStyle })

/**
 * 批量操作按钮配置（响应式，根据选中状态智能禁用）
 *
 * - "禁用"按钮：选中中有已启用的才可用（HasEnabled）
 * - "启用"按钮：选中中有已禁用的才可用（HasDisabled）
 * - "更新"按钮：选中中有可更新的才可用（HasUpdate）
 * - "删除"按钮：始终可用（只要有选中项）
 */
const batchActions = computed<MultiSelectAction[]>(() => [
  { key: 'enable', label: '启用', icon: PlayIcon, variant: 'enable', disabled: !hasDisabledSelected.value },
  { key: 'disable', label: '禁用', icon: PauseIcon, variant: 'disable', disabled: !hasEnabledSelected.value },
  { key: 'update', label: '更新', icon: ArrowPathIcon, variant: 'update', disabled: !hasUpdatableSelected.value },
  { key: 'delete', label: '删除', icon: TrashIcon, variant: 'delete' },
])

/** MultiSelectBar action 事件分发 */
function handleBatchAction(key: string) {
  switch (key) {
    case 'enable': batchToggle(true); break
    case 'disable': batchToggle(false); break
    case 'update': batchUpdate(); break
    case 'delete': batchDelete(); break
  }
}

onMounted(() => {
  initModOperations()
  window.addEventListener('keydown', handleMultiSelectKeydown)
})
onUnmounted(() => {
  stopPreloadListener()
  window.removeEventListener('keydown', handleMultiSelectKeydown)
})
</script>

<template>
  <div class="flex flex-1 flex-col overflow-hidden">
    <ModEmptyState
      v-if="!isModableVersion && !checkingModable"
      variant="not-modable"
      :mods-count="0"
      @go-download="router.push('/apps/downloads')"
      @go-select="router.push('/apps/versions/select')"
    />

    <div v-else class="flex h-full flex-col">
      <ModToolbar
        v-model:mod-filter="modFilter"
        v-model:mod-search="modSearch"
        :mods-loading="modsLoading"
        :filter-options="filterOptions"
        @install="handleInstallMod"
        @open-dir="handleOpenModsDir"
        @refresh="loadMods"
      />

      <!-- 多选操作栏：浮动在视口底部中央，由组件自身 teleport + fixed 定位，不占列表空间 -->
      <MultiSelectBar
        :selected-count="selectedCount"
        :total-count="filteredMods.length"
        :actions="batchActions"
        :batch-processing="batchProcessing"
        @action="handleBatchAction"
        @select-all="selectAll"
        @invert-selection="invertSelection"
        @exit="clearSelection"
      />

      <!-- 列表滚动区 -->
      <div class="flex-1 overflow-y-auto p-6">
        <ModEmptyState v-if="modsLoading" variant="loading" :mods-count="0" />
        <ModEmptyState
          v-else-if="filteredMods.length === 0"
          :variant="mods.length === 0 ? 'empty' : 'no-match'"
          :mods-count="mods.length"
          @install="handleInstallMod"
        />
        <div v-else class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
          <ul class="divide-y divide-gray-100">
            <ModListItem
              v-for="mod in filteredMods"
              :key="mod.file_name"
              :mod="mod"
              :detail-loading-for="detailLoadingFor"
              :mod-local-name-style="modLocalNameStyle"
              :selected="checkSelected(mod.file_name)"
              @toggle="handleToggleMod"
              @delete="handleDeleteMod"
              @show-info="onShowInfo"
              @open-wiki="handleOpenWiki"
              @open-file="handleOpenFile"
              @update="openUpdateDialog"
              @select="toggleSelect"
            />
          </ul>
        </div>
      </div>
    </div>

    <!-- Mod 详情弹窗（复用社区资源详情组件） -->
    <ResourceDetail
      :visible="detailVisible"
      :project="detailProject"
      :version-id="selectedId || undefined"
      :game-version="versionGameVersion || undefined"
      :mods-dir="versionModsDir || undefined"
      :disable-mod-update="disableModUpdate"
      @close="detailVisible = false"
    />

    <!-- Mod 版本更新/更改对话框 -->
    <ModUpdateDialog
      :visible="updateDialogVisible"
      :mod="updateTargetMod"
      :mc-version="versionGameVersion || ''"
      :version-id="selectedId || ''"
      @update:visible="updateDialogVisible = $event"
      @installed="onModUpdated"
    />
  </div>
</template>
