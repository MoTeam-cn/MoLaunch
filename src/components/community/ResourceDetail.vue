<script setup lang="ts">
/**
 * 资源详情弹窗
 *
 * - 顶部资源预览 + 操作按钮（ResourceDetailHeader 子组件）
 * - 版本筛选 RadioButton（HorizontalFilter）
 * - 版本按游戏版本分组卡片（VersionGroupCard 子组件，可折叠/展开带动画）
 * - 加载进度条
 * - 下载进度浮层（DownloadProgressOverlay 子组件）
 */

import { ref, watch, onUnmounted, nextTick } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import type { ResourceProject, ResourceVersion } from '@/types/community'
import { getProjectVersions, downloadResourceToPath, formatDownloadFilename, installModpack } from '@/utils/api/community'
import { installMerged } from '@/utils/api/loader'
import { useVersionStore } from '@/stores/version'
import { saveFile } from '@/utils/api/system'
import { showSuccess, showError } from '@/utils/toast'
import { showPrompt } from '@/utils/modal'
import { useVersionGroups, getFilterVersionName } from '@/composables/useVersionGroups'
import { useSearchProgress } from '@/composables/useSearchProgress'
import { useCommunityDownload } from '@/composables/useCommunityDownload'
import HorizontalFilter from '@/components/common/HorizontalFilter.vue'
import ResourceDetailHeader from './resource-detail/ResourceDetailHeader.vue'
import VersionGroupCard from './resource-detail/VersionGroupCard.vue'
import DownloadProgressOverlay from './resource-detail/DownloadProgressOverlay.vue'

const versionStore = useVersionStore()

interface Props {
  visible: boolean
  project: ResourceProject | null
  versionId?: string
  /** 整合包对应的 MC 版本号，设置后自动选中顶部筛选 tag */
  gameVersion?: string
  /** 整合包的 mods 目录路径，设置后下载 Mod 默认保存到该目录 */
  modsDir?: string
  /** 是否禁止更新 Mod（版本独立设置 advance_disable_mod_update），开启后下载已存在文件时拦截 */
  disableModUpdate?: boolean
}

const props = defineProps<Props>()
const emit = defineEmits<{ close: [] }>()

const versions = ref<ResourceVersion[]>([])
const loading = ref(false)
const downloading = ref<string | null>(null)

const { groups, filterOptions, versionFilter, toggleGroup, setFilter, expandedOf, mountedOf } = useVersionGroups(() => versions.value)
const { percent, slowMerging, stageText, start, finish, fail } = useSearchProgress()
const { downloading: communityDownloading, progress: downloadProgress, startDownload, startListener, stopListener } = useCommunityDownload()

startListener()

watch(
  [() => props.visible, () => props.project],
  async ([v, p], [oldV, oldP]) => {
    // 仅在 visible 变为 true 或 project 变化时触发（避免 visible/gameVersion 单独变化重复加载）
    if (!v || !p) return
    if (v === oldV && p === oldP) return
    loading.value = true
    versions.value = []
    setFilter('')
    start(p.platform === 'CurseForge' ? 1 : p.platform === 'Modrinth' ? 2 : 0)
    try {
      versions.value = await getProjectVersions(p.platform, p.id)
      finish()
      // 整合包来自 ModTab 时自动选中对应版本筛选
      if (props.gameVersion) {
        const target = getFilterVersionName(props.gameVersion)
        if (target && filterOptions.value.includes(target)) {
          nextTick(() => setFilter(target))
        }
      }
    } catch (e: any) {
      showError('加载版本列表失败: ' + (e?.message || String(e)))
      fail()
    } finally {
      loading.value = false
    }
  },
)

async function handleDownload(v: ResourceVersion) {
  if (!props.project) return
  const finalFileName = await formatDownloadFilename(v.file_name, props.project.translated_name)

  // 禁止更新 Mod 拦截：如果目标文件在 mods 目录已存在（即"更新"场景），阻止下载
  if (props.disableModUpdate && props.modsDir) {
    const targetPath = `${props.modsDir}/${finalFileName}`.replace(/\\/g, '/')
    try {
      const exists = await invoke<boolean>('plugin:fs|exists', { path: targetPath })
      if (exists) {
        showError(`此版本已禁止更新 Mod：${finalFileName} 已存在。\n如需更新，请前往 版本设置 → 高级选项 关闭「禁止更新 Mod」`)
        return
      }
    } catch (e) {
      console.debug('[ResourceDetail] 检查文件存在性失败:', e)
    }
  }

  const savePath = await saveFile('选择保存位置', finalFileName, [{ name: '所有文件', extensions: ['*'] }], props.modsDir)
  if (!savePath) return

  downloading.value = v.id
  startDownload()
  try {
    await downloadResourceToPath(v.download_url, finalFileName, savePath)
    showSuccess(`已下载: ${finalFileName}`)
  } catch (e: any) {
    showError('下载失败: ' + (e?.message || String(e)))
  } finally {
    downloading.value = null
  }
}

/**
 * 安装整合包：下载原始包 + 解析 + 下载依赖 mods + 复制 overrides → 安装游戏本体 + 加载器
 * 进度走 download_state，复用 DownloadPanel
 *
 * 点击下载后先弹窗询问安装名称，用户可自定义；取消则中止安装。
 */
async function handleInstallModpack(v: ResourceVersion) {
  if (!props.project) return
  const { platform, resource_type, translated_name, raw_name } = props.project
  if (resource_type !== 'ModPack') return

  const defaultName = translated_name || raw_name || v.file_name.replace(/\.(zip|mrpack)$/i, '')

  // 弹窗询问安装名称，允许用户自定义（取消则中止）
  const instanceName = await promptForInstanceName(defaultName)
  if (!instanceName) return

  downloading.value = v.id
  versionStore.startDownload(instanceName)

  try {
    const result = await installModpack({ platform, downloadUrl: v.download_url, fileName: v.file_name, instanceName })
    const loader = result.loader
    const lv = result.loaderVersion
    await installMerged(
      result.gameVersion,
      loader === 'forge' ? lv : undefined,
      loader === 'neoforge' ? lv : undefined,
      loader === 'fabric' || loader === 'quilt' ? lv : undefined,
      undefined, undefined, instanceName,
    )
    showSuccess(`整合包 ${instanceName} 安装完成`)
  } catch (e: any) {
    showError('整合包安装失败: ' + (e?.message || String(e)))
    versionStore.finishDownload()
  } finally {
    downloading.value = null
  }
}

/**
 * 弹窗询问整合包安装名称
 *
 * 将 callback 风格的 showPrompt 包装为 Promise，便于在 async 流程中 await。
 * 用户确认返回 trim 后的名称（空则回退默认名）；取消返回 null。
 */
function promptForInstanceName(defaultName: string): Promise<string | null> {
  return new Promise((resolve) => {
    showPrompt(
      '安装整合包',
      '请输入整合包的安装名称：',
      (value: string) => {
        const trimmed = value.trim()
        resolve(trimmed || defaultName)
      },
      {
        defaultValue: defaultName,
        placeholder: '请输入安装名称',
        onCancel: () => resolve(null),
      },
    )
  })
}

onUnmounted(() => stopListener())
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
        v-if="visible && project"
        class="fixed inset-0 z-[9999] flex items-center justify-center p-4"
        @click.self="emit('close')"
      >
        <div class="absolute inset-0 bg-black/40" />
        <div class="relative w-full max-w-2xl bg-white rounded-lg shadow-xl flex flex-col max-h-[85vh]">
          <!-- 头部 + 操作按钮 -->
          <ResourceDetailHeader :project="project" @close="emit('close')" />

          <!-- 版本筛选 -->
          <div v-if="filterOptions.length > 1" class="px-4 py-2 border-b border-gray-100 bg-gray-50/50">
            <HorizontalFilter
              :model-value="versionFilter"
              :options="filterOptions.map(o => ({ label: o, value: o }))"
              @update:model-value="setFilter"
            />
          </div>

          <!-- 版本列表区 -->
          <div class="flex-1 overflow-y-auto p-2">
            <!-- 加载中：进度条 -->
            <div v-if="loading" class="py-12 px-4">
              <div class="flex flex-col items-center">
                <svg class="mb-4 h-8 w-8 animate-spin text-primary-500" viewBox="0 0 24 24" fill="none">
                  <circle cx="12" cy="12" r="10" stroke="currentColor" stroke-width="3" class="opacity-25" />
                  <path d="M12 2a10 10 0 0 1 10 10" stroke="currentColor" stroke-width="3" stroke-linecap="round" />
                </svg>
                <div class="w-full max-w-sm">
                  <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
                    <div
                      class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
                      :style="{ width: Math.min(100, percent) + '%' }"
                    />
                  </div>
                  <div class="mt-2 flex items-center justify-between text-xs">
                    <span class="text-gray-500">{{ stageText }}</span>
                    <span class="font-medium text-primary-600">{{ percent.toFixed(1) }}%</span>
                  </div>
                  <p v-if="slowMerging" class="mt-2 text-center text-[11px] text-gray-400">
                    资源有点多，稍安勿躁，静候处理
                  </p>
                </div>
              </div>
            </div>

            <!-- 版本分组卡片 -->
            <div v-else-if="groups.length > 0" class="space-y-1.5">
              <VersionGroupCard
                v-for="g in groups"
                :key="g.title"
                :title="g.title"
                :versions="g.versions"
                :expanded="expandedOf(g.title)"
                :mounted="mountedOf(g.title)"
                :downloading="downloading"
                :is-modpack="project.resource_type === 'ModPack'"
                @toggle="toggleGroup(g.title)"
                @download="handleDownload"
                @install="handleInstallModpack"
              />
            </div>

            <!-- 空状态 -->
            <div v-else class="py-12 text-center text-sm text-gray-400">
              暂无版本数据
            </div>

            <!-- 下载进度浮层 -->
            <DownloadProgressOverlay
              v-if="communityDownloading && downloadProgress"
              :progress="downloadProgress"
            />
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>
