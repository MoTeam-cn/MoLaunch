<script setup lang="ts">
import { defineAsyncComponent } from 'vue'
/**
 * 版本设置 - 概览子页：版本展示、个性化、快捷方式、高级管理
 * 业务逻辑已抽取到 `@/composables/useVersionOverviewActions`，本文件仅负责模板组装。
 */
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useJavaStore } from '@/stores/java'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
import { useVersionSettings } from '@/composables/useVersionSettings'
import { useVersionOverviewActions } from '@/composables/useVersionOverviewActions'
const RepairLoaderDrawer = defineAsyncComponent(() => import('./repair-loader/RepairLoaderDrawer.vue'))

const router = useRouter()
const authStore = useAuthStore()
const javaStore = useJavaStore()
const {
  selectedId, personalization,
  versionFolder, savesFolder, modsFolder, resourcepacksFolder, shaderpacksFolder,
  currentLogoIcon, currentLogo, currentMeta,
  iconOptions, displayTypeOptions,
  isModable, shaderAvailable, loadPersonalization, refreshEffectiveDir,
} = useVersionSettings()

const {
  fixing, repairDrawerOpen, openFolder,
  handleEditDesc, handleRename, handleToggleStar,
  handleChangeDisplayType, handleChangeLogo,
  handleExportScript, handleFixFiles, handleRepairLoader, handleDelete,
} = useVersionOverviewActions({
  selectedId, personalization, loadPersonalization, refreshEffectiveDir,
  router, authStore, javaStore,
})
</script>

<template>
  <div class="mx-auto max-w-2xl space-y-5">
    <!-- 版本展示卡片 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <div class="flex items-center gap-4">
        <img :src="currentLogoIcon" class="h-16 w-16 flex-none rounded-lg shadow-sm" alt="">
        <div class="min-w-0 flex-1">
          <div class="truncate text-xl font-semibold text-gray-900">{{ selectedId }}</div>
          <div class="mt-1 flex flex-wrap items-center gap-2">
            <Tag size="small" color="arcoblue">{{ currentMeta.label }}</Tag>
            <span v-if="personalization?.originalVersion" class="text-xs text-gray-400">
              原版 {{ personalization.originalVersion }}
            </span>
          </div>
          <p v-if="personalization?.customInfo" class="mt-1.5 text-xs text-gray-500">
            {{ personalization.customInfo }}
          </p>
        </div>
        <Button
          type="text"
          size="small"
          class="!rounded-lg !border !px-3 !py-1.5 !text-xs !gap-1.5"
          :class="personalization?.isStar
            ? '!border-yellow-400 !bg-yellow-50 !text-yellow-600'
            : '!border-gray-300 !bg-white !text-gray-500 hover:!border-yellow-400 hover:!text-yellow-600'"
          @click="handleToggleStar"
        >
          <svg class="h-3.5 w-3.5 !mr-0" viewBox="0 0 20 20" :fill="personalization?.isStar ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="1.5">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
          {{ personalization?.isStar ? '已收藏' : '收藏' }}
        </Button>
      </div>
    </section>

    <!-- 个性化 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-3 text-sm font-semibold text-gray-700">个性化</h3>
      <div class="space-y-3">
        <div class="flex items-center gap-3">
          <span class="w-20 flex-none text-xs text-gray-500">版本名</span>
          <span class="flex-1 truncate text-sm text-gray-800">{{ selectedId }}</span>
          <Button type="outline" size="small" @click="handleRename">
            <template #icon><svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.379-8.379-2.828-2.828z" /></svg></template>
            重命名
          </Button>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-20 flex-none text-xs text-gray-500">描述</span>
          <span class="flex-1 truncate text-sm" :class="personalization?.customInfo ? 'text-gray-800' : 'text-gray-400'">
            {{ personalization?.customInfo || '默认描述' }}
          </span>
          <Button type="outline" size="small" @click="handleEditDesc">
            <template #icon><svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.379-8.379-2.828-2.828z" /></svg></template>
            修改
          </Button>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-20 flex-none text-xs text-gray-500">图标</span>
          <Select
            :model-value="currentLogo"
            :options="iconOptions"
            class="flex-1"
            @update:model-value="handleChangeLogo($event as string)"
          >
            <template #trigger="{ label, open, toggle }">
              <!-- Select 组件的自定义触发器（结构性元素，非视觉按钮） -->
              <div role="button" tabindex="0" class="flex w-full items-center justify-between rounded-md border border-gray-300 bg-white px-2.5 py-1.5 text-sm text-gray-700 transition-colors hover:border-primary-500 cursor-pointer" @click="toggle" @keydown.enter="toggle">
                <span class="flex items-center gap-2">
                  <img v-if="currentLogoIcon" :src="currentLogoIcon" class="h-4 w-4 rounded-sm" alt="">
                  <span>{{ label }}</span>
                </span>
                <svg class="h-3.5 w-3.5 text-gray-400 transition-transform" :class="{ 'rotate-180': open }" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" /></svg>
              </div>
            </template>
            <template #option="{ option, selected }">
              <span class="flex items-center gap-2">
                <img v-if="option.icon" :src="option.icon" class="h-4 w-4 rounded-sm" alt="">
                <span>{{ option.label }}</span>
              </span>
              <svg v-if="selected" class="h-4 w-4 text-primary-500" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z" clip-rule="evenodd" /></svg>
            </template>
          </Select>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-20 flex-none text-xs text-gray-500">分类</span>
          <Select
            :model-value="personalization?.displayType ?? 0"
            :options="displayTypeOptions"
            class="flex-1"
            @update:model-value="handleChangeDisplayType($event as number)"
          />
        </div>
      </div>
    </section>

    <!-- 快捷方式 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-3 text-sm font-semibold text-gray-700">快捷方式</h3>
      <div class="flex flex-wrap gap-3">
        <Button
          v-for="f in [
            { label: '版本文件夹', path: versionFolder, show: true },
            { label: '存档文件夹', path: savesFolder, show: true },
            { label: 'Mod 文件夹', path: modsFolder, show: isModable },
            { label: '材质包文件夹', path: resourcepacksFolder, show: true },
            { label: '光影文件夹', path: shaderpacksFolder, show: shaderAvailable },
          ]"
          v-show="f.show"
          :key="f.label"
          type="outline"
          @click="openFolder(f.path)"
        >
          <template #icon><svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" /></svg></template>
          {{ f.label }}
        </Button>
      </div>
    </section>

    <!-- 高级管理 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-3 text-sm font-semibold text-gray-700">高级管理</h3>
      <div class="flex flex-wrap gap-3">
        <Button type="outline" :disabled="fixing" @click="handleExportScript">
          <template #icon><svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" /></svg></template>
          导出启动脚本
        </Button>
        <Tooltip text="校验并下载该版本缺失的文件（库文件、资源文件等）。当游戏无法启动或缺少文件时使用。" position="top">
          <Button type="outline" :loading="fixing" :disabled="fixing" @click="handleFixFiles">
            {{ fixing ? '补全中...' : '补全文件' }}
          </Button>
        </Tooltip>
        <Tooltip text="检测 Forge/Fabric/LiteLoader 是否损坏，若损坏将询问是否重新安装。" position="top">
          <Button type="outline" @click="handleRepairLoader">
            <template #icon><svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M18 10a1 1 0 011 1v2a1 1 0 01-.832.988l-4.52.648a1 1 0 01-.746-.207l-1.319-1.1a5 5 0 00-5.166 0l-1.319 1.1a1 1 0 01-.746.207l-4.52-.648A1 1 0 010 13v-2a1 1 0 011-1h.388a3 3 0 001.032-5.8l.62-.62A2 2 0 014.243 3h2.514a2 2 0 011.202.42l.62.62a3 3 0 001.032 5.8H10v.25A4.75 4.75 0 005.25 15H4a1 1 0 100 2h1.25A6.75 6.75 0 0012 10.25V10h.38a3 3 0 001.032-5.8l.62-.62A2 2 0 0115.243 3h2.514a2 2 0 011.202.42l.62.62A3 3 0 0019.388 10H18z" clip-rule="evenodd" /></svg></template>
            检测并重装加载器
          </Button>
        </Tooltip>
        <Button type="outline" @click="handleDelete">
          <template #icon><svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" /></svg></template>
          删除版本
        </Button>
      </div>
    </section>
  </div>

  <RepairLoaderDrawer v-if="selectedId" v-model:visible="repairDrawerOpen" :version-id="selectedId" />
</template>
