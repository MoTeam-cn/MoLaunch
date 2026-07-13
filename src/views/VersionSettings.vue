<script setup lang="ts">
/**
 * 版本设置页（参考 PCL2 PageInstanceLeft + PageInstanceOverall）
 *
 * 左侧导航：概览 / 设置 / Mod 管理 / 导出
 * 概览：版本展示、文件夹快捷方式、高级管理（删除版本）
 * 其他子页：占位（后端尚未支持版本独立设置）
 */

import { ref, computed, onMounted, nextTick } from 'vue'
import { useRouter } from 'vue-router'
import { useVersionStore } from '@/stores/version'
import { useAuthStore } from '@/stores/auth'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError, showWarning, showInfo } from '@/utils/toast'
import { showConfirm, showPrompt } from '@/utils/modal'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import {
  Squares2X2Icon,
  Cog6ToothIcon,
  PuzzlePieceIcon,
  ArrowUpTrayIcon,
} from '@heroicons/vue/24/outline'
import grassIcon from '@/assets/blocks/Grass.png'
import cobblestoneIcon from '@/assets/blocks/CobbleStone.png'
import commandBlockIcon from '@/assets/blocks/CommandBlock.png'
import goldBlockIcon from '@/assets/blocks/GoldBlock.png'
import anvilIcon from '@/assets/blocks/Anvil.png'
import fabricIcon from '@/assets/blocks/Fabric.png'
import neoforgeIcon from '@/assets/blocks/NeoForge.png'
import optifineIcon from '@/assets/blocks/RedstoneLampOn.png'
import liteloaderIcon from '@/assets/blocks/Egg.png'

const router = useRouter()
const versionStore = useVersionStore()
const authStore = useAuthStore()
const javaStore = useJavaStore()

const activeCategory = ref('overview')
const gameDir = ref('')
const effectiveDir = ref('')
const personalization = ref<tauri.VersionPersonalization | null>(null)
const fixing = ref(false)

/** 图标选项（参考 PCL2 ComboDisplayLogo，自动判断 + 各方块图标 + 自定义） */
const iconOptions = [
  { value: '', label: '自动判断', icon: '' },
  { value: 'Grass', label: '草方块', icon: grassIcon },
  { value: 'CobbleStone', label: '圆石', icon: cobblestoneIcon },
  { value: 'CommandBlock', label: '命令方块', icon: commandBlockIcon },
  { value: 'GoldBlock', label: '金块', icon: goldBlockIcon },
  { value: 'Anvil', label: '铁砧', icon: anvilIcon },
  { value: 'Fabric', label: 'Fabric', icon: fabricIcon },
  { value: 'NeoForge', label: 'NeoForge', icon: neoforgeIcon },
  { value: 'RedstoneLampOn', label: '红石灯', icon: optifineIcon },
  { value: 'Egg', label: '蛋', icon: liteloaderIcon },
]

/** 分类选项（参考 PCL2 McInstanceCardType 枚举） */
const displayTypeOptions = [
  { value: 0, label: '自动判断' },
  { value: 2, label: '可安装 Mod' },
  { value: 3, label: '原版类似' },
  { value: 5, label: '愚人节' },
  { value: 1, label: '隐藏' },
]

/** 当前选中的图标值（logo 字段） */
const currentLogo = computed(() => personalization.value?.logo ?? '')

/** 当前选中的图标资源 */
const currentLogoIcon = computed(() => {
  const opt = iconOptions.find(o => o.value === currentLogo.value)
  return opt?.icon || currentMeta.value.icon
})

const categories = [
  { id: 'overview', label: '概览', icon: Squares2X2Icon, desc: '版本信息、文件夹快捷方式、高级管理' },
  { id: 'setup', label: '设置', icon: Cog6ToothIcon, desc: '版本独立的 Java、内存、窗口等启动参数' },
  { id: 'mod', label: 'Mod 管理', icon: PuzzlePieceIcon, desc: '管理当前版本的 Mod' },
  { id: 'export', label: '导出', icon: ArrowUpTrayIcon, desc: '导出整合包或版本' },
]

const selectedId = computed(() => versionStore.selectedVersion)

/** 推断版本类型（仅根据 ID 字符串匹配） */
function inferVersionType(id: string): string {
  if (!id) return 'release'
  const lower = id.toLowerCase()
  if (lower.includes('neoforge')) return 'neoforge'
  if (lower.includes('forge')) return 'forge'
  if (lower.includes('fabric')) return 'fabric'
  if (lower.includes('optifine')) return 'optifine'
  if (lower.includes('liteloader')) return 'liteloader'
  if (/^\d{2}w\d{2}[a-z]/.test(id)) return 'snapshot'
  return 'release'
}

interface TypeMeta {
  icon: string
  label: string
}
const typeMetaMap: Record<string, TypeMeta> = {
  release:    { icon: grassIcon,        label: '正式版' },
  snapshot:   { icon: commandBlockIcon, label: '快照' },
  forge:      { icon: anvilIcon,        label: 'Forge' },
  neoforge:   { icon: neoforgeIcon,     label: 'NeoForge' },
  fabric:     { icon: fabricIcon,       label: 'Fabric' },
  optifine:   { icon: optifineIcon,     label: 'OptiFine' },
  liteloader: { icon: liteloaderIcon,   label: 'LiteLoader' },
  old:        { icon: cobblestoneIcon,  label: '旧版' },
  fool:       { icon: goldBlockIcon,    label: '愚人节版' },
}
const defaultMeta: TypeMeta = { icon: grassIcon, label: '其他' }

const currentMeta = computed<TypeMeta>(() => {
  if (!selectedId.value) return defaultMeta
  return typeMetaMap[inferVersionType(selectedId.value)] ?? defaultMeta
})

/** 路径分隔符（根据游戏目录格式推断） */
const sep = computed(() => gameDir.value.includes('\\') ? '\\' : '/')

/** 拼接路径 */
function joinPath(base: string, ...parts: string[]): string {
  return [base, ...parts].join(sep.value)
}

/** 版本文件夹路径（始终是 gameDir/versions/versionId） */
const versionFolder = computed(() => {
  if (!selectedId.value || !gameDir.value) return ''
  return joinPath(gameDir.value, 'versions', selectedId.value)
})

/** 存档文件夹路径（基于版本隔离后的有效游戏目录） */
const savesFolder = computed(() => {
  if (!effectiveDir.value) return ''
  return joinPath(effectiveDir.value, 'saves')
})

/** Mod 文件夹路径（基于版本隔离后的有效游戏目录） */
const modsFolder = computed(() => {
  if (!effectiveDir.value) return ''
  return joinPath(effectiveDir.value, 'mods')
})

/** 材质包文件夹路径（基于版本隔离后的有效游戏目录） */
const resourcepacksFolder = computed(() => {
  if (!effectiveDir.value) return ''
  return joinPath(effectiveDir.value, 'resourcepacks')
})

/** 光影文件夹路径（基于版本隔离后的有效游戏目录） */
const shaderpacksFolder = computed(() => {
  if (!effectiveDir.value) return ''
  return joinPath(effectiveDir.value, 'shaderpacks')
})

/** 当前版本是否支持 Mod（仅 modloader 版本） */
const isModable = computed(() => {
  if (!selectedId.value) return false
  const type = inferVersionType(selectedId.value)
  return ['forge', 'neoforge', 'fabric', 'optifine', 'liteloader'].includes(type)
})

/** 打开文件夹（后端会在文件夹不存在时自动创建） */
async function openFolder(path: string) {
  try {
    await tauri.openPath(path)
  } catch (e) {
    showError('打开失败：' + String(e))
  }
}

/** 修改版本描述 */
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

/** 重命名版本 */
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
        versionStore.selectedVersion = newName.trim()
        // 等待 selectedId computed 更新
        await nextTick()
        await loadPersonalization()
        // 重新加载有效目录
        effectiveDir.value = await tauri.getVersionEffectiveDir(newName.trim())
        showSuccess('重命名成功')
      } catch (e) {
        showError('重命名失败：' + String(e))
      }
    },
    { defaultValue: selectedId.value, placeholder: '请输入新版本名' },
  )
}

/** 切换收藏 */
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

/** 更改版本分类 */
async function handleChangeDisplayType(newType: number) {
  if (!selectedId.value || !personalization.value) return
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { displayType: newType })
    personalization.value.display_type = newType
    showSuccess('分类已更新')
  } catch (e) {
    showError('更新失败：' + String(e))
  }
}

/** 更改版本图标 */
async function handleChangeLogo(newLogo: string) {
  if (!selectedId.value || !personalization.value) return
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { logo: newLogo })
    personalization.value.logo = newLogo
    showSuccess('图标已更新')
  } catch (e) {
    showError('更新失败：' + String(e))
  }
}

/** 导出启动脚本 */
async function handleExportScript() {
  if (!selectedId.value) return
  if (!authStore.isLoggedIn) {
    showWarning('请先登录账号')
    return
  }
  const user = authStore.currentUser!
  try {
    const savePath = await tauri.saveFile(
      '选择脚本保存位置',
      `Run_${selectedId.value}.bat`,
      [{ name: '批处理文件', extensions: ['bat'] }],
    )
    if (!savePath) return
    await tauri.exportLaunchScript(
      selectedId.value,
      user.name,
      user.uuid,
      user.access_token,
      user.login_type,
      javaStore.javaPath || null,
      savePath,
    )
    showSuccess('启动脚本已导出')
    // 打开所在文件夹
    const dir = savePath.replace(/[\\/][^\\/]+$/, '')
    await tauri.openPath(dir)
  } catch (e) {
    showError('导出失败：' + String(e))
  }
}

/** 补全文件 */
async function handleFixFiles() {
  if (!selectedId.value || fixing.value) return
  showConfirm(
    '补全文件',
    `将检查并下载版本"${selectedId.value}"缺失的 libraries 和 assets 文件，可能耗时较长。`,
    async () => {
      fixing.value = true
      showInfo('开始补全文件...')
      try {
        await tauri.fixVersionFiles(selectedId.value!)
        showSuccess('文件补全完成')
      } catch (e) {
        showError('补全失败：' + String(e))
      } finally {
        fixing.value = false
      }
    },
  )
}

/** 删除版本 */
function handleDelete() {
  if (!selectedId.value) return
  showConfirm(
    '删除版本',
    `确定要删除版本"${selectedId.value}"吗？此操作不可恢复。`,
    async () => {
      try {
        await tauri.uninstallVersion(selectedId.value!)
        showSuccess('版本已删除')
        versionStore.selectedVersion = null
        router.push('/')
      } catch (e) {
        showError(String(e))
      }
    },
  )
}

/** 加载个性化数据 */
async function loadPersonalization() {
  if (!selectedId.value) {
    personalization.value = null
    return
  }
  try {
    personalization.value = await tauri.getVersionPersonalization(selectedId.value)
  } catch (e) {
    console.error('Failed to load personalization:', e)
  }
}

function goBack() {
  router.push('/')
}

onMounted(async () => {
  try {
    gameDir.value = await tauri.getGameDir()
    if (selectedId.value) {
      effectiveDir.value = await tauri.getVersionEffectiveDir(selectedId.value)
      await loadPersonalization()
    }
  } catch (e) {
    console.error('Failed to load dirs:', e)
  }
})
</script>

<template>
  <div class="flex h-full flex-col overflow-hidden">
    <!-- 顶部栏 -->
    <header class="flex flex-none items-center justify-between border-b border-gray-200 bg-white px-4 py-3">
      <div class="flex items-center gap-3">
        <button
          class="flex items-center gap-1.5 rounded-lg px-2.5 py-1.5 text-sm text-gray-500 transition-colors hover:bg-gray-100 hover:text-gray-700"
          @click="goBack"
        >
          <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
            <path fill-rule="evenodd" d="M12.7 4.3a1 1 0 010 1.4L8.4 10l4.3 4.3a1 1 0 01-1.4 1.4l-5-5a1 1 0 010-1.4l5-5a1 1 0 011.4 0z" clip-rule="evenodd" />
          </svg>
          返回
        </button>
        <h1 class="text-base font-semibold text-gray-800">
          版本设置<span v-if="selectedId" class="text-gray-400"> - {{ selectedId }}</span>
        </h1>
      </div>
    </header>

    <!-- 未选择版本 -->
    <div v-if="!selectedId" class="flex flex-1 items-center justify-center">
      <div class="flex flex-col items-center gap-3 text-gray-400">
        <svg class="h-12 w-12" viewBox="0 0 24 24" fill="currentColor">
          <path d="M4 4a2 2 0 012-2h12a2 2 0 012 2v16a2 2 0 01-2 2H6a2 2 0 01-2-2V4zm2 0v4h12V4H6zm0 6v4h12v-4H6zm0 6v4h12v-4H6z" />
        </svg>
        <p class="text-sm">请先在主页选择一个版本</p>
        <button
          class="rounded-lg bg-primary-600 px-4 py-2 text-sm font-medium text-white transition-colors hover:bg-primary-700"
          @click="router.push('/select')"
        >
          去选择版本
        </button>
      </div>
    </div>

    <!-- 主体：左导航 + 右内容 -->
    <div v-else class="flex flex-1 overflow-hidden">
      <aside class="w-48 flex-none border-r border-gray-200 bg-white">
        <div class="flex-1 overflow-y-auto py-4">
          <button
            v-for="cat in categories"
            :key="cat.id"
            class="flex w-full items-center px-4 py-2.5 text-sm font-medium transition-colors"
            :class="[
              activeCategory === cat.id
                ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
                : 'text-gray-700 hover:bg-gray-50'
            ]"
            @click="activeCategory = cat.id"
          >
            <component :is="cat.icon" class="mr-3 h-5 w-5" />
            {{ cat.label }}
          </button>
        </div>
      </aside>

      <!-- 右侧内容区 -->
      <div class="flex flex-1 flex-col overflow-hidden">
        <div class="flex-none border-b border-gray-200 bg-white px-6 py-4">
          <h2 class="text-lg font-semibold text-gray-900">
            {{ categories.find(c => c.id === activeCategory)?.label }}
          </h2>
          <p class="mt-1 text-xs text-gray-500">{{ categories.find(c => c.id === activeCategory)?.desc }}</p>
        </div>

        <div class="flex-1 overflow-y-auto p-6">
          <!-- 概览 -->
          <div v-if="activeCategory === 'overview'" class="mx-auto max-w-2xl space-y-5">
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
                <!-- 版本名 -->
                <div class="flex items-center gap-3">
                  <span class="w-20 flex-none text-xs text-gray-500">版本名</span>
                  <span class="flex-1 truncate text-sm text-gray-800">{{ selectedId }}</span>
                  <button
                    class="flex flex-none items-center gap-1 rounded-md border border-gray-300 bg-white px-2.5 py-1 text-xs text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
                    @click="handleRename"
                  >
                    <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                      <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.379-8.379-2.828-2.828z" />
                    </svg>
                    重命名
                  </button>
                </div>
                <!-- 描述 -->
                <div class="flex items-center gap-3">
                  <span class="w-20 flex-none text-xs text-gray-500">描述</span>
                  <span class="flex-1 truncate text-sm" :class="personalization?.custom_info ? 'text-gray-800' : 'text-gray-400'">
                    {{ personalization?.custom_info || '默认描述' }}
                  </span>
                  <button
                    class="flex flex-none items-center gap-1 rounded-md border border-gray-300 bg-white px-2.5 py-1 text-xs text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
                    @click="handleEditDesc"
                  >
                    <svg class="h-3.5 w-3.5" viewBox="0 0 20 20" fill="currentColor">
                      <path d="M13.586 3.586a2 2 0 112.828 2.828l-.793.793-2.828-2.828.793-.793zM11.379 5.793L3 14.172V17h2.828l8.379-8.379-2.828-2.828z" />
                    </svg>
                    修改
                  </button>
                </div>
                <!-- 图标（使用 Select 组件 + 自定义 option slot） -->
                <div class="flex items-center gap-3">
                  <span class="w-20 flex-none text-xs text-gray-500">图标</span>
                  <Select
                    :model-value="currentLogo"
                    :options="iconOptions"
                    @update:model-value="handleChangeLogo($event as string)"
                  >
                    <template #trigger="{ label, open, toggle }">
                      <button
                        class="flex w-full items-center justify-between rounded-md border border-gray-300 bg-white px-2.5 py-1.5 text-sm text-gray-700 transition-colors hover:border-primary-500"
                        @click="toggle"
                      >
                        <span class="flex items-center gap-2">
                          <img v-if="currentLogoIcon" :src="currentLogoIcon" class="h-4 w-4 rounded-sm" alt="">
                          <span>{{ label }}</span>
                        </span>
                        <svg class="h-3.5 w-3.5 text-gray-400 transition-transform" :class="{ 'rotate-180': open }" viewBox="0 0 20 20" fill="currentColor">
                          <path fill-rule="evenodd" d="M5.23 7.21a.75.75 0 011.06.02L10 11.168l3.71-3.938a.75.75 0 111.08 1.04l-4.25 4.5a.75.75 0 01-1.08 0l-4.25-4.5a.75.75 0 01.02-1.06z" clip-rule="evenodd" />
                        </svg>
                      </button>
                    </template>
                    <template #option="{ option, selected }">
                      <span class="flex items-center gap-2">
                        <img v-if="option.icon" :src="option.icon" class="h-4 w-4 rounded-sm" alt="">
                        <span>{{ option.label }}</span>
                      </span>
                      <svg v-if="selected" class="h-4 w-4 text-primary-500" viewBox="0 0 20 20" fill="currentColor">
                        <path fill-rule="evenodd" d="M16.704 4.153a.75.75 0 01.143 1.052l-8 10.5a.75.75 0 01-1.127.075l-4.5-4.5a.75.75 0 011.06-1.06l3.894 3.893 7.48-9.817a.75.75 0 011.05-.143z" clip-rule="evenodd" />
                      </svg>
                    </template>
                  </Select>
                </div>
                <!-- 分类（使用 Select 组件） -->
                <div class="flex items-center gap-3">
                  <span class="w-20 flex-none text-xs text-gray-500">分类</span>
                  <Select
                    :model-value="personalization?.display_type ?? 0"
                    :options="displayTypeOptions"
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
                  class="flex items-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
                  @click="openFolder(versionFolder)"
                >
                  <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                  </svg>
                  版本文件夹
                </button>
                <button
                  class="flex items-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
                  @click="openFolder(savesFolder)"
                >
                  <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                  </svg>
                  存档文件夹
                </button>
                <button
                  v-if="isModable"
                  class="flex items-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
                  @click="openFolder(modsFolder)"
                >
                  <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                  </svg>
                  Mod 文件夹
                </button>
                <button
                  class="flex items-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
                  @click="openFolder(resourcepacksFolder)"
                >
                  <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                  </svg>
                  材质包文件夹
                </button>
                <button
                  v-if="isModable"
                  class="flex items-center gap-2 rounded-lg border border-gray-300 bg-white px-4 py-2 text-sm text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 hover:bg-primary-50"
                  @click="openFolder(shaderpacksFolder)"
                >
                  <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
                  </svg>
                  光影文件夹
                </button>
              </div>
              <p class="mt-3 break-all text-xs text-gray-400">{{ versionFolder }}</p>
            </section>

            <!-- 高级管理 -->
            <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
              <h3 class="mb-3 text-sm font-semibold text-gray-700">高级管理</h3>
              <div class="flex flex-wrap gap-3">
                <button
                  class="flex items-center gap-2 rounded-lg border border-blue-300 bg-white px-4 py-2 text-sm text-blue-600 transition-colors hover:bg-blue-50 hover:border-blue-500"
                  :disabled="fixing"
                  :class="{ 'opacity-50 cursor-not-allowed': fixing }"
                  @click="handleExportScript"
                >
                  <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M3 17a1 1 0 011-1h12a1 1 0 110 2H4a1 1 0 01-1-1zm3.293-7.707a1 1 0 011.414 0L9 10.586V3a1 1 0 112 0v7.586l1.293-1.293a1 1 0 111.414 1.414l-3 3a1 1 0 01-1.414 0l-3-3a1 1 0 010-1.414z" clip-rule="evenodd" />
                  </svg>
                  导出启动脚本
                </button>
                <Tooltip
                  text="校验并下载该版本缺失的文件（库文件、资源文件等）。当游戏无法启动或缺少文件时使用。"
                  position="top"
                >
                  <button
                    class="flex items-center gap-2 rounded-lg border border-green-300 bg-white px-4 py-2 text-sm text-green-600 transition-colors hover:bg-green-50 hover:border-green-500"
                    :disabled="fixing"
                    :class="{ 'opacity-50 cursor-not-allowed': fixing }"
                    @click="handleFixFiles"
                  >
                    <svg v-if="fixing" class="h-4 w-4 animate-spin" viewBox="0 0 24 24" fill="none">
                      <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                      <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                    </svg>
                    <svg v-else class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
                    </svg>
                    {{ fixing ? '补全中...' : '补全文件' }}
                  </button>
                </Tooltip>
                <button
                  class="flex items-center gap-2 rounded-lg border border-red-300 bg-white px-4 py-2 text-sm text-red-600 transition-colors hover:bg-red-50 hover:border-red-500"
                  @click="handleDelete"
                >
                  <svg class="h-4 w-4" viewBox="0 0 20 20" fill="currentColor">
                    <path fill-rule="evenodd" d="M9 2a1 1 0 00-.894.553L7.382 4H4a1 1 0 000 2v10a2 2 0 002 2h8a2 2 0 002-2V6a1 1 0 100-2h-3.382l-.724-1.447A1 1 0 0011 2H9zM7 8a1 1 0 012 0v6a1 1 0 11-2 0V8zm5-1a1 1 0 00-1 1v6a1 1 0 102 0V8a1 1 0 00-1-1z" clip-rule="evenodd" />
                  </svg>
                  删除版本
                </button>
              </div>
            </section>
          </div>

          <!-- 占位子页 -->
          <div v-else class="flex h-full items-center justify-center">
            <div class="flex flex-col items-center gap-3 text-gray-400">
              <component :is="categories.find(c => c.id === activeCategory)?.icon" class="h-10 w-10" />
              <p class="text-sm">功能开发中</p>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
