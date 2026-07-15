<script setup lang="ts">
/**
 * 版本设置 - 概览子页
 * 版本展示、个性化、快捷方式、高级管理
 */
import { ref, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useAuthStore } from '@/stores/auth'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError, showWarning, showInfo } from '@/utils/toast'
import { showConfirm, showPrompt } from '@/utils/modal'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { useVersionSettings } from '@/composables/useVersionSettings'

const router = useRouter()
const authStore = useAuthStore()
const javaStore = useJavaStore()
const {
  selectedId,
  personalization,
  versionFolder,
  savesFolder,
  modsFolder,
  resourcepacksFolder,
  shaderpacksFolder,
  currentLogoIcon,
  currentLogo,
  currentMeta,
  iconOptions,
  displayTypeOptions,
  isModable,
  loadPersonalization,
  refreshEffectiveDir,
} = useVersionSettings()

const fixing = ref(false)

async function openFolder(path: string) {
  try {
    await tauri.openPath(path)
  } catch (e) {
    showError('打开失败：' + String(e))
  }
}

function handleEditDesc() {
  if (!selectedId.value) return
  const oldDesc = personalization.value?.custom_info ?? ''
  showPrompt(
    '修改版本描述',
    '修改版本的描述文本，留空则使用默认描述。',
    async (newDesc: string) => {
      if (!selectedId.value) return
      try {
        await tauri.updateVersionPersonalization(selectedId.value, { customInfo: newDesc })
        if (personalization.value) personalization.value.custom_info = newDesc
        showSuccess('描述已更新')
      } catch (e) {
        showError('更新失败：' + String(e))
      }
    },
    { defaultValue: oldDesc, placeholder: '请输入版本描述' },
  )
}

function handleRename() {
  if (!selectedId.value) return
  showPrompt(
    '重命名版本',
    '修改版本文件夹名称（不影响游戏内版本号）',
    async (newName: string) => {
      if (!selectedId.value || !newName.trim()) return
      if (newName === selectedId.value) return
      try {
        const oldName = selectedId.value
        await tauri.renameVersion(oldName, newName.trim())
        // 等待 selectedId computed 更新
        await nextTick()
        await loadPersonalization()
        await refreshEffectiveDir()
        showSuccess('重命名成功')
      } catch (e) {
        showError('重命名失败：' + String(e))
      }
    },
    { defaultValue: selectedId.value, placeholder: '请输入新版本名' },
  )
}

async function handleToggleStar() {
  if (!selectedId.value || !personalization.value) return
  const newVal = !personalization.value.is_star
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { isStar: newVal })
    personalization.value.is_star = newVal
    showSuccess(newVal ? '已加入收藏' : '已取消收藏')
  } catch (e) {
    showError('操作失败：' + String(e))
  }
}

async function handleChangeDisplayType(newType: number) {
  if (!selectedId.value || !personalization.value) return
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { displayType: newType })
    personalization.value.display_type = newType
    showSuccess('分类已更新')
  } catch (e) { showError('更新失败：' + String(e)) }
}

async function handleChangeLogo(newLogo: string) {
  if (!selectedId.value || !personalization.value) return
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { logo: newLogo })
    // 替换整个 personalization 对象，确保所有依赖该 ref 的组件（如首页 VersionSelector）都能响应式更新
    personalization.value = { ...personalization.value, logo: newLogo }
    showSuccess('图标已更新')
  } catch (e) { showError('更新失败：' + String(e)) }
}

async function handleExportScript() {
  if (!selectedId.value) return
  if (!authStore.isLoggedIn) return showWarning('请先登录账号')
  const user = authStore.currentUser!
  try {
    const savePath = await tauri.saveFile('选择脚本保存位置', `Run_${selectedId.value}.bat`, [{ name: '批处理文件', extensions: ['bat'] }])
    if (!savePath) return
    await tauri.exportLaunchScript(selectedId.value, user.name, user.uuid, user.access_token, user.login_type, javaStore.javaPath || null, savePath)
    showSuccess('启动脚本已导出')
    // 导出后自动打开所在文件夹并选中导出的文件
    await tauri.revealInExplorer(savePath)
  } catch (e) { showError('导出失败：' + String(e)) }
}

async function handleFixFiles() {
  if (!selectedId.value || fixing.value) return
  showConfirm('补全文件', `将检查并下载版本"${selectedId.value}"缺失的 libraries 和 assets 文件，可能耗时较长。`, async () => {
    fixing.value = true
    showInfo('开始补全文件...')
    try {
      await tauri.fixVersionFiles(selectedId.value!)
      showSuccess('文件补全完成')
    } catch (e) { showError('补全失败：' + String(e)) }
    finally { fixing.value = false }
  })
}

function handleDelete() {
  if (!selectedId.value) return
  showConfirm('删除版本', `确定要删除版本"${selectedId.value}"吗？此操作不可恢复。`, async () => {
    try {
      await tauri.uninstallVersion(selectedId.value!)
      showSuccess('版本已删除')
      router.push('/apps')
    } catch (e) { showError(String(e)) }
  })
}
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
            <span class="inline-block rounded-full bg-primary-50 px-2.5 py-0.5 text-xs font-medium text-primary-600">
              {{ currentMeta.label }}
            </span>
            <span v-if="personalization?.original_version" class="text-xs text-gray-400">
              原版 {{ personalization.original_version }}
            </span>
          </div>
          <p v-if="personalization?.custom_info" class="mt-1.5 text-xs text-gray-500">
            {{ personalization.custom_info }}
          </p>
        </div>
        <button
          class="flex flex-none items-center gap-1.5 rounded-lg border px-3 py-1.5 text-xs transition-colors"
          :class="personalization?.is_star
            ? 'border-yellow-400 bg-yellow-50 text-yellow-600'
            : 'border-gray-300 bg-white text-gray-500 hover:border-yellow-400 hover:text-yellow-600'"
          @click="handleToggleStar"
        >
          <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" :fill="personalization?.is_star ? 'currentColor' : 'none'" stroke="currentColor" stroke-width="1.5">
            <path d="M9.049 2.927c.3-.921 1.603-.921 1.902 0l1.07 3.292a1 1 0 00.95.69h3.462c.969 0 1.371 1.24.588 1.81l-2.8 2.034a1 1 0 00-.364 1.118l1.07 3.292c.3.921-.755 1.688-1.54 1.118l-2.8-2.034a1 1 0 00-1.175 0l-2.8 2.034c-.784.57-1.838-.197-1.539-1.118l1.07-3.292a1 1 0 00-.364-1.118L2.98 8.72c-.783-.57-.38-1.81.588-1.81h3.461a1 1 0 00.951-.69l1.07-3.292z" />
          </svg>
          {{ personalization?.is_star ? '已收藏' : '收藏' }}
        </button>
      </div>
    </section>

    <!-- 个性化 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-3 text-sm font-semibold text-gray-700">个性化</h3>
      <div class="space-y-3">
        <div class="flex items-center gap-3">
          <span class="w-20 flex-none text-xs text-gray-500">版本名</span>
          <span class="flex-1 truncate text-sm text-gray-800">{{ selectedId }}</span>
          <button class="flex flex-none items-center gap-1 rounded-md border border-gray-300 bg-white px-2.5 py-1 text-xs text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50" @click="handleRename">
            <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.379-8.379-2.828-2.828z" /></svg>
            重命名
          </button>
        </div>
        <div class="flex items-center gap-3">
          <span class="w-20 flex-none text-xs text-gray-500">描述</span>
          <span class="flex-1 truncate text-sm" :class="personalization?.custom_info ? 'text-gray-800' : 'text-gray-400'">
            {{ personalization?.custom_info || '默认描述' }}
          </span>
          <button class="flex flex-none items-center gap-1 rounded-md border border-gray-300 bg-white px-2.5 py-1 text-xs text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50" @click="handleEditDesc">
            <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor"><path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.379-8.379-2.828-2.828z" /></svg>
            修改
          </button>
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
              <button class="flex w-full items-center justify-between rounded-md border border-gray-300 bg-white px-2.5 py-1.5 text-sm text-gray-700 transition-colors hover:border-primary-500" @click="toggle">
                <span class="flex items-center gap-2">
                  <img v-if="currentLogoIcon" :src="currentLogoIcon" class="h-4 w-4 rounded-sm" alt="">
                  <span>{{ label }}</span>
                </span>
                <svg class="h-3.5 w-3.5 text-gray-400 transition-transform" :class="{ 'rotate-180': open }" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" /></svg>
              </button>
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
            :model-value="personalization?.display_type ?? 0"
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
        <button
          v-for="f in [
            { label: '版本文件夹', path: versionFolder, show: true },
            { label: '存档文件夹', path: savesFolder, show: true },
            { label: 'Mod 文件夹', path: modsFolder, show: isModable },
            { label: '材质包文件夹', path: resourcepacksFolder, show: true },
            { label: '光影文件夹', path: shaderpacksFolder, show: isModable },
          ]"
          v-show="f.show"
          :key="f.label"
          class="flex items-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
          @click="openFolder(f.path)"
        >
          <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" /></svg>
          {{ f.label }}
        </button>
      </div>
    </section>

    <!-- 高级管理 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-3 text-sm font-semibold text-gray-700">高级管理</h3>
      <div class="flex flex-wrap gap-3">
        <button class="flex items-center gap-2 rounded-lg border border-blue-300 bg-white px-4 py-2 text-sm text-blue-600 transition-colors hover:bg-blue-50 hover:border-blue-500" :disabled="fixing" :class="{ 'opacity-50 cursor-not-allowed': fixing }" @click="handleExportScript">
          <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" /></svg>
          导出启动脚本
        </button>
        <Tooltip text="校验并下载该版本缺失的文件（库文件、资源文件等）。当游戏无法启动或缺少文件时使用。" position="top">
          <button class="flex items-center gap-2 rounded-lg border border-green-300 bg-white px-4 py-2 text-sm text-green-600 transition-colors hover:bg-green-50 hover:border-green-500" :disabled="fixing" :class="{ 'opacity-50 cursor-not-allowed': fixing }" @click="handleFixFiles">
            <svg v-if="fixing" class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none"><circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" /><path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" /></svg>
            <svg v-else class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" /></svg>
            {{ fixing ? '补全中...' : '补全文件' }}
          </button>
        </Tooltip>
        <button class="flex items-center gap-2 rounded-lg border border-red-300 bg-white px-4 py-2 text-sm text-red-600 transition-colors hover:bg-red-50 hover:border-red-500" @click="handleDelete">
          <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor"><path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" /></svg>
          删除版本
        </button>
      </div>
    </section>
  </div>
</template>
