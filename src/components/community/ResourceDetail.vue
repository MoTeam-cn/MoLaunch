<script setup lang="ts">
/**
 * 资源详情弹窗（参考 PCL2 PageDownloadCompDetail）
 * - 顶部资源预览 + 操作按钮
 * - 版本筛选 RadioButton（参考 PCL2 CardFilter）
 * - 版本按游戏版本分组卡片，可折叠/展开（带动画 + icon）
 * - 加载进度条
 */

import { ref, watch, onUnmounted, nextTick } from 'vue'
import type { ResourceProject, ResourceVersion } from '@/types/community'
import { ModLoaderFlags } from '@/types/community'
import { getProjectVersions, downloadResourceToPath, formatDownloadFilename, getMcmodUrl, installModpack } from '@/utils/api/community'
import { installMerged } from '@/utils/api/loader'
import { useVersionStore } from '@/stores/version'
import { saveFile } from '@/utils/api/system'
import { showSuccess, showError } from '@/utils/toast'
import { formatBytes } from '@/utils/format'
import { useVersionGroups, getFilterVersionName } from '@/composables/useVersionGroups'
import { useSearchProgress } from '@/composables/useSearchProgress'
import { useCommunityDownload } from '@/composables/useCommunityDownload'
import { open as openUrl } from '@tauri-apps/plugin-shell'
import HorizontalFilter from '@/components/common/HorizontalFilter.vue'
import {
  XMarkIcon,
  ArrowDownTrayIcon,
  ArrowTopRightOnSquareIcon,
  ChevronDownIcon,
  CubeIcon,
  ClipboardDocumentIcon,
  BookOpenIcon,
  RocketLaunchIcon,
} from '@heroicons/vue/24/outline'

const versionStore = useVersionStore()

interface Props {
  visible: boolean
  project: ResourceProject | null
  versionId?: string
  /**
   * 整合包对应的 MC 版本号（如 "1.20.1"）。
   *
   * 设置后，弹窗加载版本列表完成时会自动选中顶部筛选 tag
   * （把 "1.20.1" 截断成 "1.20" 后匹配 filterOptions）。
   * 用于从 ModTab 打开资源详情时自动定位到整合包对应版本。
   */
  gameVersion?: string
  /**
   * 整合包的 mods 目录绝对路径。
   *
   * 设置后，资源类型为 Mod 时，下载按钮默认保存到该目录
   * （saveFile 对话框的默认路径会拼接 modsDir + 文件名）。
   * 用于从 ModTab 打开资源详情时，下载 mod 直接放到整合包的 mods 文件夹。
   */
  modsDir?: string
}

const props = defineProps<Props>()
const emit = defineEmits<{ close: [] }>()

const versions = ref<ResourceVersion[]>([])
const loading = ref(false)
const downloading = ref<string | null>(null)

const { groups, filterOptions, versionFilter, toggleGroup, setFilter, expandedOf, mountedOf } = useVersionGroups(() => versions.value)
const { percent, slowMerging, stageText, start, finish, fail } = useSearchProgress()
const { downloading: communityDownloading, progress: downloadProgress, startDownload, startListener, stopListener } = useCommunityDownload()

// 组件挂载时启动事件监听
startListener()

watch(
  () => props.visible,
  async (v) => {
    if (!v || !props.project) return
    loading.value = true
    versions.value = []
    setFilter('')
    start(props.project.platform === 'CurseForge' ? 1 : props.project.platform === 'Modrinth' ? 2 : 0)
    try {
      versions.value = await getProjectVersions(props.project.platform, props.project.id)
      finish()
      // 版本列表加载完成后，若传入了 gameVersion（来自 ModTab 的整合包版本），
      // 自动选中顶部筛选 tag（截断到二级：1.20.1 → "1.20"）
      if (props.gameVersion) {
        const target = getFilterVersionName(props.gameVersion)
        if (target && filterOptions.value.includes(target)) {
          // 下一 tick 再设置，确保 useVersionGroups 的 watch（flush:sync）已重建 groups
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

function loaderNames(flags: number): string[] {
  const list: string[] = []
  if (flags & ModLoaderFlags.Forge) list.push('Forge')
  if (flags & ModLoaderFlags.NeoForge) list.push('NeoForge')
  if (flags & ModLoaderFlags.Fabric) list.push('Fabric')
  if (flags & ModLoaderFlags.Quilt) list.push('Quilt')
  return list
}

function releaseColor(rt: string): string {
  if (rt === 'Release') return 'bg-green-100 text-green-700'
  if (rt === 'Beta') return 'bg-yellow-100 text-yellow-700'
  return 'bg-gray-100 text-gray-600'
}

function formatDownloads(n: number): string {
  if (n >= 100_000_000) return (n / 100_000_000).toFixed(2) + ' 亿'
  if (n >= 10_000) return (n / 10_000).toFixed(1).replace(/\.0$/, '') + ' 万'
  return String(n)
}

async function handleDownload(v: ResourceVersion) {
  if (!props.project) return
  // 1. 根据用户设置的下载文件名格式生成默认文件名（译名+原名拼接）
  const finalFileName = await formatDownloadFilename(v.file_name, props.project.translated_name)
  // 2. 弹出系统文件管理器让用户选择保存位置
  //    若传入了 modsDir（从 ModTab 打开），默认定位到整合包的 mods 文件夹
  const savePath = await saveFile(
    '选择保存位置',
    finalFileName,
    [{ name: '所有文件', extensions: ['*'] }],
    props.modsDir,
  )
  if (!savePath) return // 用户取消

  // 3. 启动下载（流式 + 进度推送）
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
 * 安装整合包（参考 PCL2 PageDownloadCompDetail.Install_Click）
 *
 * 流程：
 * 1. versionStore.startDownload 触发 DownloadPanel 显示
 * 2. installModpack：下载原始包 + 解析 + 下载依赖 mods + 复制 overrides（进度走 download_state）
 * 3. installMerged：安装游戏本体 + 加载器（进度走 download_state，复用同一 DownloadPanel）
 */
async function handleInstallModpack(v: ResourceVersion) {
  if (!props.project) return
  const { platform, resource_type, translated_name, raw_name } = props.project
  if (resource_type !== 'ModPack') return

  const instanceName = translated_name || raw_name || v.file_name.replace(/\.(zip|mrpack)$/i, '')
  downloading.value = v.id

  // 触发 DownloadPanel（右下角进度环 + 下载管理页面）
  versionStore.startDownload(instanceName)

  try {
    // 1. 安装整合包专属部分（下载原始包 + 依赖 mods + overrides）
    const result = await installModpack({
      platform,
      downloadUrl: v.download_url,
      fileName: v.file_name,
      instanceName,
    })

    // 2. 安装游戏本体 + 加载器（使用 manifest 中的 game_version + loader_version）
    const loader = result.loader
    const lv = result.loaderVersion
    await installMerged(
      result.gameVersion,
      loader === 'forge' ? lv : undefined,
      loader === 'neoforge' ? lv : undefined,
      loader === 'fabric' || loader === 'quilt' ? lv : undefined, // quilt 暂复用 fabric 通道
      undefined, // optifine
      undefined, // liteloader
      instanceName,
    )

    showSuccess(`整合包 ${instanceName} 安装完成`)
  } catch (e: any) {
    showError('整合包安装失败: ' + (e?.message || String(e)))
    versionStore.finishDownload()
  } finally {
    downloading.value = null
  }
}

/** 格式化下载速度 */
function formatSpeed(bytesPerSec: number): string {
  if (bytesPerSec >= 1_048_576) return (bytesPerSec / 1_048_576).toFixed(1) + ' MB/s'
  if (bytesPerSec >= 1024) return (bytesPerSec / 1024).toFixed(0) + ' KB/s'
  return bytesPerSec + ' B/s'
}

/** 点击"转到 MC百科"：先查 class id 直链，查不到回退搜索 URL，toast 提示后跳转 */
async function openMcmod() {
  if (!props.project) return
  const { platform, slug, translated_name, raw_name } = props.project
  // 优先用 slug 查 moddata.txt 的 class id 直链
  let url = await getMcmodUrl(platform, slug)
  if (!url) {
    // 回退到搜索 URL
    const name = translated_name || raw_name
    url = `https://search.mcmod.cn/s?key=${encodeURIComponent(name)}`
    showSuccess('未找到 MC 百科直链，已跳转到搜索页')
  } else {
    showSuccess('正在打开 MC 百科详情页')
  }
  // 用 Tauri shell opener 打开系统默认浏览器（webview 内 window.open 会被拦截）
  await openUrl(url)
}

/** 复制资源名称到剪贴板 */
async function copyName() {
  const name = props.project?.translated_name || props.project?.raw_name || ''
  if (!name) return
  try {
    await navigator.clipboard.writeText(name)
    showSuccess('已复制: ' + name)
  } catch {
    showError('复制失败')
  }
}

/** 下载进度百分比 */
function downloadPercent(): number {
  if (!downloadProgress.value || downloadProgress.value.total === 0) return 0
  return Math.min(100, (downloadProgress.value.downloaded / downloadProgress.value.total) * 100)
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
          <!-- 头部：Logo + 标题 + 操作按钮 -->
          <div class="flex items-start gap-3 p-4 border-b border-gray-200">
            <div class="shrink-0 w-12 h-12 rounded-md overflow-hidden bg-gray-100 flex items-center justify-center">
              <img v-if="project.logo_url" :src="project.logo_url" :alt="project.raw_name" class="w-full h-full object-cover">
              <CubeIcon v-else class="w-6 h-6 text-gray-400" />
            </div>
            <div class="flex-1 min-w-0">
              <h2 class="text-base font-semibold text-gray-900 truncate">
                {{ project.translated_name || project.raw_name }}
              </h2>
              <p v-if="project.translated_name" class="text-xs text-gray-400 mt-0.5 truncate">
                {{ project.raw_name }}
              </p>
              <p class="text-xs text-gray-500 mt-0.5 line-clamp-2">{{ project.description }}</p>
              <div class="flex items-center gap-2 mt-1.5">
                <span class="px-1.5 py-0.5 rounded text-[10px] font-medium"
                  :class="project.platform === 'CurseForge' ? 'bg-orange-100 text-orange-700' : 'bg-green-100 text-green-700'">
                  {{ project.platform }}
                </span>
                <span class="text-[11px] text-gray-400">{{ formatDownloads(project.download_count) }} 下载</span>
              </div>
            </div>
            <button class="p-1 rounded text-gray-400 hover:bg-gray-100" @click="emit('close')">
              <XMarkIcon class="w-5 h-5" />
            </button>
          </div>

          <!-- 操作按钮行 -->
          <div class="flex items-center gap-2 px-4 py-2 border-b border-gray-100">
            <button
              v-if="project.website"
              class="px-3 py-1.5 rounded-md text-xs font-medium text-white bg-primary-600 hover:bg-primary-700 transition-colors flex items-center gap-1"
              @click="openUrl(project.website)"
            >
              <ArrowTopRightOnSquareIcon class="w-3.5 h-3.5" />
              转到 {{ project.platform }}
            </button>
            <button
              v-if="project.resource_type === 'Mod'"
              class="px-3 py-1.5 rounded-md text-xs font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 transition-colors flex items-center gap-1"
              @click="openMcmod"
            >
              <BookOpenIcon class="w-3.5 h-3.5" />
              转到 MC百科
            </button>
            <button
              class="px-3 py-1.5 rounded-md text-xs font-medium text-gray-700 bg-gray-100 hover:bg-gray-200 transition-colors flex items-center gap-1"
              @click="copyName"
            >
              <ClipboardDocumentIcon class="w-3.5 h-3.5" />
              复制名称
            </button>
          </div>

          <!-- 版本筛选（横向滚动，参考 PCL2 CardFilter） -->
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
              <div
                v-for="g in groups"
                :key="g.title"
                class="border border-gray-200 rounded-md overflow-hidden transition-colors duration-200"
                :class="expandedOf(g.title) ? 'border-primary-200 bg-primary-50/30' : 'bg-white hover:border-gray-300'"
              >
                <!-- 卡片标题栏（点击折叠/展开） -->
                <button
                  class="w-full flex items-center justify-between px-3 py-2.5 transition-colors duration-200"
                  :class="expandedOf(g.title) ? 'bg-primary-50/50 hover:bg-primary-100/50' : 'bg-gray-50 hover:bg-gray-100'"
                  @click="toggleGroup(g.title)"
                >
                  <div class="flex items-center gap-1.5">
                    <CubeIcon
                      class="w-3.5 h-3.5 transition-colors duration-300"
                      :class="expandedOf(g.title) ? 'text-primary-500' : 'text-gray-400'"
                    />
                    <span
                      class="text-sm font-medium transition-colors duration-200"
                      :class="expandedOf(g.title) ? 'text-primary-700' : 'text-gray-700'"
                    >{{ g.title }}</span>
                  </div>
                  <div class="flex items-center gap-2">
                    <span
                      class="text-xs transition-colors duration-200"
                      :class="expandedOf(g.title) ? 'text-primary-400' : 'text-gray-400'"
                    >{{ g.versions.length }} 个版本</span>
                    <span
                      class="inline-flex items-center justify-center w-5 h-5 rounded-full transition-all duration-300 ease-[cubic-bezier(0.4,0,0.2,1)]"
                      :class="expandedOf(g.title)
                        ? 'bg-primary-100 text-primary-600 rotate-180'
                        : 'bg-gray-100 text-gray-500 rotate-0'"
                    >
                      <ChevronDownIcon class="w-3.5 h-3.5" />
                    </span>
                  </div>
                </button>

                <!-- 卡片内容（懒挂载：首次展开才渲染版本条目，折叠卡片无内容 DOM；grid-template-rows 0fr→1fr 过渡） -->
                <div
                  v-if="mountedOf(g.title)"
                  class="grid transition-[grid-template-rows] duration-[400ms] ease-[cubic-bezier(0.4,0,0.2,1)]"
                  :class="expandedOf(g.title) ? 'grid-rows-[1fr]' : 'grid-rows-[0fr]'"
                >
                  <div class="overflow-hidden">
                    <div
                      class="p-1.5 space-y-0.5 transition-all duration-[350ms] ease-[cubic-bezier(0.4,0,0.2,1)]"
                      :class="expandedOf(g.title)
                        ? 'opacity-100 translate-y-0 transition-delay-[50ms]'
                        : 'opacity-0 -translate-y-2 transition-delay-0'"
                    >
                      <div
                        v-for="v in g.versions"
                        :key="v.id"
                        class="flex items-center gap-2 px-2 py-2 rounded-md hover:bg-gray-50 transition-colors"
                      >
                        <div class="flex-1 min-w-0">
                          <div class="flex items-center gap-1.5">
                            <span class="px-1 py-0.5 rounded text-[9px] font-medium" :class="releaseColor(v.release_type)">{{ v.release_type }}</span>
                            <span class="text-sm text-gray-900 truncate">{{ v.display || v.file_name }}</span>
                          </div>
                          <div class="flex items-center gap-2 mt-0.5 text-[11px] text-gray-400">
                            <span>{{ v.game_versions.slice(0, 3).join(', ') }}</span>
                            <span v-for="l in loaderNames(v.mod_loaders)" :key="l" class="text-blue-500">{{ l }}</span>
                            <span>{{ formatBytes(v.size) }}</span>
                            <span>{{ formatDownloads(v.download_count) }} 下载</span>
                          </div>
                        </div>
                        <button
                          class="shrink-0 px-2.5 py-1.5 rounded-md text-xs font-medium text-white bg-primary-600 hover:bg-primary-700 disabled:opacity-50 transition-colors flex items-center gap-1"
                          :disabled="downloading === v.id"
                          @click="project.resource_type === 'ModPack' ? handleInstallModpack(v) : handleDownload(v)"
                        >
                          <RocketLaunchIcon v-if="project.resource_type === 'ModPack'" class="w-3.5 h-3.5" />
                          <ArrowDownTrayIcon v-else class="w-3.5 h-3.5" />
                          <template v-if="downloading === v.id">
                            {{ project.resource_type === 'ModPack' ? '安装中...' : '下载中...' }}
                          </template>
                          <template v-else>
                            {{ project.resource_type === 'ModPack' ? '安装' : '下载' }}
                          </template>
                        </button>
                      </div>
                    </div>
                  </div>
                </div>
              </div>
            </div>

            <!-- 空状态 -->
            <div v-else class="py-12 text-center text-sm text-gray-400">
              暂无版本数据
            </div>

            <!-- 下载进度浮层 -->
            <div
              v-if="communityDownloading && downloadProgress"
              class="sticky bottom-0 left-0 right-0 bg-white border-t border-gray-200 px-4 py-2 shadow-lg"
            >
              <div class="flex items-center justify-between mb-1">
                <span class="text-xs text-gray-600 truncate flex-1">
                  {{ downloadProgress.fileName }}
                </span>
                <span class="text-xs text-gray-500 ml-2">
                  {{ formatSpeed(downloadProgress.speed) }}
                </span>
              </div>
              <div class="h-1.5 overflow-hidden rounded-full bg-gray-100">
                <div
                  class="h-full rounded-full bg-gradient-to-r from-primary-400 to-primary-600 transition-all duration-300 ease-out"
                  :style="{ width: downloadPercent() + '%' }"
                />
              </div>
              <div class="flex items-center justify-between mt-1 text-[11px] text-gray-400">
                <span>{{ formatBytes(downloadProgress.downloaded) }} / {{ downloadProgress.total ? formatBytes(downloadProgress.total) : '未知' }}</span>
                <span>{{ downloadPercent().toFixed(1) }}%</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </transition>
  </teleport>
</template>

<style scoped>
.line-clamp-2 {
  display: -webkit-box;
  -webkit-line-clamp: 2;
  -webkit-box-orient: vertical;
  overflow: hidden;
}
</style>
