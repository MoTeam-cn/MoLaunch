<script setup lang="ts">
/**
 * 版本设置 - Mod 管理子页（顶层容器）
 * 子组件位于 mod-tab/（ModToolbar / ModListItem / ModEmptyState），列表项视觉与按钮设计细节见 ModListItem.vue。
 *
 * 业务逻辑（列表加载、过滤、启用/禁用、删除、安装、打开目录、详情查询、预加载监听）
 * 已抽取到 `@/composables/useModOperations`，本文件仅负责模板组装与生命周期钩子。
 */
import { ref, onMounted, onUnmounted } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionSettings } from '@/composables/useVersionSettings'
import { useModOperations } from '@/composables/useModOperations'
import ResourceDetail from '@/components/community/ResourceDetail.vue'
import ModListItem from './mod-tab/ModListItem.vue'
import ModToolbar from './mod-tab/ModToolbar.vue'
import ModEmptyState from './mod-tab/ModEmptyState.vue'

const router = useRouter()
const { selectedId, isModable } = useVersionSettings()

const modLocalNameStyle = ref(0)
const {
  mods, modsLoading, modFilter, modSearch,
  isModableVersion, checkingModable,
  versionGameVersion, versionModsDir, disableModUpdate,
  filteredMods, filterOptions,
  detailVisible, detailProject, detailLoadingFor,
  stopPreloadListener,
  loadMods,
  handleToggleMod, handleDeleteMod, handleInstallMod,
  handleOpenModsDir, handleOpenFile,
  onShowInfo, handleOpenWiki,
  init: initModOperations,
} = useModOperations({ selectedId, isModable, modLocalNameStyle })

onMounted(initModOperations)
onUnmounted(stopPreloadListener)
</script>

<template>
  <div class="flex flex-1 flex-col overflow-hidden">
    <!-- 不可安装 Mod 的提示 -->
    <ModEmptyState
      v-if="!isModableVersion && !checkingModable"
      variant="not-modable"
      :mods-count="0"
      @go-download="router.push('/apps/downloads')"
      @go-select="router.push('/apps/versions/select')"
    />

    <!-- Mod 管理主体：工具栏固定不滚动，列表区独立滚动 -->
    <div v-else class="flex h-full flex-col">
      <ModToolbar
        v-model:modFilter="modFilter"
        v-model:modSearch="modSearch"
        :mods-loading="modsLoading"
        :filter-options="filterOptions"
        @install="handleInstallMod"
        @open-dir="handleOpenModsDir"
        @refresh="loadMods"
      />

      <!-- 列表滚动区（只有这里滚动，工具栏固定不动） -->
      <div class="flex-1 overflow-y-auto p-6">
        <ModEmptyState v-if="modsLoading" variant="loading" :mods-count="0" />
        <ModEmptyState
          v-else-if="filteredMods.length === 0"
          :variant="mods.length === 0 ? 'empty' : 'no-match'"
          :mods-count="mods.length"
          @install="handleInstallMod"
        />
        <!-- Mod 列表 -->
        <div v-else class="overflow-hidden rounded-xl border border-gray-200 bg-white shadow-sm">
          <ul class="divide-y divide-gray-100">
            <ModListItem
              v-for="mod in filteredMods"
              :key="mod.file_name"
              :mod="mod"
              :detail-loading-for="detailLoadingFor"
              :mod-local-name-style="modLocalNameStyle"
              @toggle="handleToggleMod"
              @delete="handleDeleteMod"
              @show-info="onShowInfo"
              @open-wiki="handleOpenWiki"
              @open-file="handleOpenFile"
            />
          </ul>
        </div>
      </div>
    </div>

    <!-- Mod 详情弹窗（关联到 CF/MR 平台工程时弹出，复用社区资源详情组件） -->
    <ResourceDetail
      :visible="detailVisible"
      :project="detailProject"
      :version-id="selectedId || undefined"
      :game-version="versionGameVersion || undefined"
      :mods-dir="versionModsDir || undefined"
      :disable-mod-update="disableModUpdate"
      @close="detailVisible = false"
    />
  </div>
</template>
