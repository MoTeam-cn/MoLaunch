<script setup lang="ts">
/**
 * 资源包可视化编辑器 - M1 查看器闭环 + M2 编辑闭环
 *
 * 打开资源包（resourcepacks 目录列表 / 本地 ZIP / 文件夹）→ 包信息栏 +
 * 左文件树右内容分发（mcmeta 表单编辑 / 纹理 2D 预览与替换 / 语言表格编辑 /
 * JSON 文本编辑 / 声音试听）→ 保存回原包 / 另存为 ZIP。
 */
import { computed, defineAsyncComponent, onMounted, ref, watch } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const RpFileTreeNode = defineAsyncComponent(() => import('./RpFileTreeNode.vue'))
const RpMcmetaForm = defineAsyncComponent(() => import('./RpMcmetaForm.vue'))
const RpTexturePreview = defineAsyncComponent(() => import('./RpTexturePreview.vue'))
const RpLangTable = defineAsyncComponent(() => import('./RpLangTable.vue'))
const RpSoundPreview = defineAsyncComponent(() => import('./RpSoundPreview.vue'))
const RpTextEditor = defineAsyncComponent(() => import('./RpTextEditor.vue'))
const RpModelPreview = defineAsyncComponent(() => import('./RpModelPreview.vue'))
import { toastError, toastSuccess } from '@/utils/toast'
import { pickFile, pickDirectory, pickSavePath } from '@/utils/fileDialog'
import { formatBytes } from '@/utils/format'
import { showConfirmAsync } from '@/utils/modal'
import { collectExpandPaths, filterTreeNode, normalizeKeyword } from '@/utils/resourcepack/filterTree'
import { resourcepackList, rpOpen, rpRead, rpExport } from '@/utils/api/tools'
import type { RpOpenResult, RpReadResult, RpTreeNode, ResourcePackItem } from '@/utils/api/tools'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import {
  ArrowDownTrayIcon,
  ChevronDownIcon,
  CubeIcon,
  FolderOpenIcon,
  MagnifyingGlassIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'

const packs = ref<ResourcePackItem[]>([])
const current = ref<RpOpenResult | null>(null)
const opening = ref(false)
const selectedNode = ref<RpTreeNode | null>(null)
const fileContent = ref<RpReadResult | null>(null)
const reading = ref(false)
const expandedSet = ref<Set<string>>(new Set())
const listOpen = ref(true)
const exporting = ref(false)
/** 文件树搜索关键字（空 = 不过滤） */
const searchQuery = ref('')
/** 模型文件视图模式：false = 3D 预览，true = JSON 文本编辑 */
const modelEditMode = ref(false)
/** 版本隔离：'' = 全局（不隔离），其他 = 具体版本 ID（参考资源包转换器） */
const selectedVersionId = ref('')
const installedVersions = ref<InstalledVersionInfo[]>([])
const versionOptions = computed(() => [
  { label: '全局（不隔离）', value: '' },
  ...installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
])

/** 过滤后的文件树（无关键字或未打开包时返回原树） */
const filteredTree = computed(() => {
  const tree = current.value?.tree
  if (!tree) return null
  const kw = normalizeKeyword(searchQuery.value)
  return filterTreeNode(tree, kw)
})

watch(searchQuery, (v) => {
  const kw = normalizeKeyword(v)
  if (!current.value || !kw) return
  // 搜索时自动展开所有命中路径的祖先目录
  const paths = collectExpandPaths(current.value.tree, kw)
  const next = new Set(expandedSet.value)
  paths.forEach((p) => next.add(p))
  expandedSet.value = next
})

watch(selectedNode, () => {
  modelEditMode.value = false
})

const fileCount = computed(() => countFiles(current.value?.tree))
const canEditText = computed(() =>
  ['json', 'model', 'text'].includes(selectedNode.value?.file_type ?? ''),
)
/** 模型 / blockstate JSON → 3D 预览 */
const isModelFile = computed(() => {
  const t = selectedNode.value?.file_type
  if (t === 'model') return true
  return t === 'json' && (selectedNode.value?.rel_path ?? '').includes('/blockstates/')
})
const textContent = computed(() =>
  fileContent.value?.kind === 'text' ? fileContent.value.content : '',
)
const mediaContent = computed(() =>
  fileContent.value?.kind === 'data_uri' ? fileContent.value.content : '',
)

function countFiles(node?: RpTreeNode): number {
  if (!node) return 0
  return node.children.reduce(
    (sum, c) => sum + (c.kind === 'file' ? 1 : countFiles(c)),
    0,
  )
}

/** Windows canonicalize 返回的路径带 `\\?\` 前缀，仅展示时去掉 */
function displayPath(p: string): string {
  return p.startsWith('\\\\?\\') ? p.slice(4) : p
}

onMounted(async () => {
  await loadVersions()
  await loadPacks()
})

async function loadPacks() {
  try {
    const res = await resourcepackList(selectedVersionId.value || undefined)
    packs.value = res.items ?? []
  } catch (e) {
    toastError(`加载资源包列表失败: ${e instanceof Error ? e.message : String(e)}`)
  }
}

async function loadVersions() {
  try {
    installedVersions.value = await listInstalledVersionsWithType()
  } catch {
    toastError('加载版本列表失败')
  }
}

// 版本切换时按隔离目录重新加载列表（首次加载由 onMounted 触发，跳过初始回调）
watch(selectedVersionId, (newVal, oldVal) => {
  if (oldVal !== '' || newVal !== '') {
    loadPacks()
  }
})

async function openPath(path: string) {
  if (opening.value) return
  opening.value = true
  try {
    const res = await rpOpen(path, current.value?.work_dir)
    if (res.error) {
      toastError(res.error)
      return
    }
    current.value = res
    selectedNode.value = null
    fileContent.value = null
    expandedSet.value = new Set(
      res.tree.children.filter((c) => c.kind === 'dir').map((c) => c.rel_path),
    )
    listOpen.value = false
  } catch (e) {
    toastError(`打开失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    opening.value = false
  }
}

async function pickZip() {
  const file = await pickFile({ filters: [{ name: 'ZIP', extensions: ['zip'] }], title: '选择资源包 ZIP' })
  if (file) await openPath(file)
}

async function pickFolder() {
  const dir = await pickDirectory({ title: '选择资源包文件夹' })
  if (dir) await openPath(dir)
}

function toggleNode(relPath: string) {
  const next = new Set(expandedSet.value)
  if (next.has(relPath)) next.delete(relPath)
  else next.add(relPath)
  expandedSet.value = next
}

async function selectNode(node: RpTreeNode) {
  if (node.kind !== 'file') return
  selectedNode.value = node
  fileContent.value = null
  if (!current.value) return
  reading.value = true
  try {
    const res = await rpRead(current.value.work_dir, node.rel_path)
    if (res.error) {
      toastError(res.error)
      return
    }
    fileContent.value = res
  } catch (e) {
    toastError(`读取失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    reading.value = false
  }
}

function onMcmetaSaved(meta: { packFormat: number; description: string | null }) {
  if (!current.value) return
  const old = current.value.pack_format
  current.value.pack_format = meta.packFormat
  current.value.description = meta.description
  if (old !== meta.packFormat) current.value.mc_version = null
}

/** zip 会话保存回原 zip（覆盖原包前二次确认） */
async function saveZip() {
  const c = current.value
  if (!c || !c.is_zip || !c.src_path || exporting.value) return
  const ok = await showConfirmAsync(
    '保存 ZIP',
    `将把当前编辑内容打包并覆盖原 ZIP：\n${displayPath(c.src_path)}\n确定保存？`,
  )
  if (!ok) return
  exporting.value = true
  try {
    const res = await rpExport({
      work_dir: c.work_dir,
      path: c.src_path,
      format: 'zip',
      src_path: c.src_path,
    })
    if (!res.success) {
      toastError(res.message)
      return
    }
    toastSuccess(`已保存 ZIP：${res.output_path}`)
  } catch (e) {
    toastError(`导出失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    exporting.value = false
  }
}

/** 另存为 ZIP（pickSavePath 选择目标路径） */
async function saveAsZip() {
  const c = current.value
  if (!c || exporting.value) return
  const path = await pickSavePath({
    title: '导出资源包 ZIP',
    filters: [{ name: 'ZIP', extensions: ['zip'] }],
    defaultPath: `${c.name.replace(/\.zip$/i, '')}.zip`,
  })
  if (!path) return
  exporting.value = true
  try {
    const res = await rpExport({
      work_dir: c.work_dir,
      path,
      format: 'zip',
      src_path: c.src_path,
    })
    if (!res.success) {
      toastError(res.message)
      return
    }
    toastSuccess(`已导出：${res.output_path}`)
  } catch (e) {
    toastError(`导出失败: ${e instanceof Error ? e.message : String(e)}`)
  } finally {
    exporting.value = false
  }
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <!-- 标题 + 操作 -->
    <div class="flex flex-wrap items-center gap-2 border-b border-gray-200 px-5 py-3">
      <CubeIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-base font-semibold text-gray-800">资源包编辑器</h3>
      <span class="text-xs text-gray-400">不进入游戏可视化查看资源包</span>
      <div class="ml-auto flex items-center gap-2">
        <Button size="small" :disabled="opening" @click="pickZip">打开 ZIP</Button>
        <Button size="small" type="outline" :disabled="opening" @click="pickFolder">打开文件夹</Button>
        <Button size="small" type="text" @click="loadPacks">刷新列表</Button>
      </div>
    </div>

    <div class="px-5 py-4">
      <!-- 资源包列表 -->
      <div class="flex flex-wrap items-center gap-2">
        <button
          class="flex items-center gap-1 text-sm text-gray-600 hover:text-gray-800"
          @click="listOpen = !listOpen"
        >
          <ChevronDownIcon
            class="h-4 w-4 transition-transform"
            :class="listOpen ? '' : '-rotate-90'"
          />
          资源包列表
          <span class="text-xs text-gray-400">（{{ packs.length }}）</span>
        </button>
        <div class="ml-auto flex items-center gap-1.5">
          <span class="text-xs text-gray-400">版本</span>
          <Select
            v-model="selectedVersionId"
            :options="versionOptions"
            class="w-40"
            title="选择资源包隔离目录（按 MC 版本）"
          />
        </div>
      </div>
      <div v-show="listOpen" class="mt-2 grid max-h-[132px] grid-cols-2 gap-2 overflow-y-auto md:grid-cols-3">
        <button
          v-for="p in packs"
          :key="p.path"
          class="flex items-center gap-2 rounded border border-gray-200 px-3 py-2 text-left text-sm text-gray-700 hover:border-blue-400 hover:bg-blue-50"
          :disabled="opening"
          @click="openPath(p.path)"
        >
          <FolderOpenIcon class="h-4 w-4 shrink-0 text-gray-400" />
          <span class="truncate">{{ p.name }}</span>
          <span class="ml-auto shrink-0 text-[10px] text-gray-400">{{ formatBytes(p.size) }}</span>
        </button>
        <p v-if="!packs.length" class="col-span-full py-4 text-center text-sm text-gray-400">
          暂无资源包，可点击「打开 ZIP / 打开文件夹」载入
        </p>
      </div>
    </div>

    <!-- 已打开包：包信息 + 左树右内容 -->
    <div v-if="current" class="border-t border-gray-200">
      <div class="flex items-center gap-3 px-5 py-3">
        <img
          v-if="current.icon_data_url"
          :src="current.icon_data_url"
          class="h-11 w-11 shrink-0 rounded border border-gray-300 object-contain"
          alt="包图标"
        />
        <div v-else class="grid h-11 w-11 shrink-0 place-items-center rounded border border-gray-300 bg-gray-50">
          <CubeIcon class="h-6 w-6 text-gray-400" />
        </div>
        <div class="min-w-0 flex-1">
          <div class="flex items-center gap-2">
            <span class="truncate font-medium text-gray-800">{{ current.name }}</span>
            <Tag :color="current.format === 'zip' ? 'blue' : 'green'">
              {{ current.format === 'zip' ? 'ZIP' : '文件夹' }}
            </Tag>
            <Tag v-if="current.pack_format != null">pack_format {{ current.pack_format }}</Tag>
            <Tag v-if="current.mc_version" color="purple">{{ current.mc_version }}</Tag>
          </div>
          <p class="mt-0.5 truncate text-xs text-gray-500">
            {{ formatBytes(current.size) }} · {{ fileCount }} 个文件
            <span v-if="current.description"> · {{ current.description }}</span>
          </p>
        </div>
        <div class="flex shrink-0 items-center gap-2">
          <Button
            v-if="current.is_zip && current.src_path"
            size="small"
            type="outline"
            :loading="exporting"
            :disabled="exporting"
            @click="saveZip"
          >
            <template #icon><ArrowDownTrayIcon class="h-4 w-4" /></template>
            保存 ZIP
          </Button>
          <Button
            size="small"
            :loading="exporting"
            :disabled="exporting"
            @click="saveAsZip"
          >
            <template #icon><ArrowDownTrayIcon class="h-4 w-4" /></template>
            另存为 ZIP
          </Button>
        </div>
      </div>

      <div class="grid grid-cols-1 border-t border-gray-200 md:grid-cols-[280px_1fr]">
        <!-- 文件树 -->
        <div class="max-h-[400px] overflow-y-auto p-2 md:border-r md:border-gray-200">
          <div class="relative mb-2 px-1">
            <MagnifyingGlassIcon class="pointer-events-none absolute left-3 top-1/2 h-4 w-4 -translate-y-1/2 text-gray-400" />
            <input
              v-model="searchQuery"
              type="text"
              class="w-full rounded border border-gray-300 py-1 pl-8 pr-7 text-sm text-gray-700 placeholder:text-gray-400 focus:border-blue-400 focus:outline-none"
              placeholder="搜索文件…"
            />
            <button
              v-if="searchQuery"
              type="button"
              class="absolute right-2 top-1/2 -translate-y-1/2 text-gray-400 hover:text-gray-600"
              @click="searchQuery = ''"
            >
              <XMarkIcon class="h-4 w-4" />
            </button>
          </div>
          <RpFileTreeNode
            v-if="filteredTree"
            :node="filteredTree"
            :selected-path="selectedNode?.rel_path ?? ''"
            :expanded-set="expandedSet"
            @select="selectNode"
            @toggle="toggleNode"
          />
          <p v-else class="px-1 py-4 text-center text-xs text-gray-400">未找到匹配的文件</p>
        </div>

        <!-- 内容分发 -->
        <div class="max-h-[400px] overflow-y-auto p-4">
          <RpMcmetaForm
            v-if="selectedNode?.file_type === 'mcmeta'"
            :work-dir="current.work_dir"
            :rel-path="selectedNode.rel_path"
            :content="textContent"
            :mc-version="current.mc_version"
            @saved="onMcmetaSaved"
          />
          <RpTexturePreview
            v-else-if="selectedNode?.file_type === 'png'"
            :work-dir="current.work_dir"
            :rel-path="selectedNode.rel_path"
            :src="mediaContent"
            :animated="selectedNode.animated"
            :name="selectedNode.name"
          />
          <RpLangTable
            v-else-if="selectedNode?.file_type === 'lang'"
            :work-dir="current.work_dir"
            :rel-path="selectedNode.rel_path"
            :content="textContent"
          />
          <RpSoundPreview
            v-else-if="selectedNode?.file_type === 'ogg'"
            :src="mediaContent"
          />
          <!-- 模型 / blockstate：3D 预览 ⇄ JSON 文本编辑 -->
          <div v-else-if="selectedNode && isModelFile">
            <RpModelPreview
              v-if="!modelEditMode"
              :work-dir="current.work_dir"
              :rel-path="selectedNode.rel_path"
              :name="selectedNode.name"
            />
            <div v-else class="space-y-2">
              <p class="text-xs text-gray-400">JSON 文本编辑（切换后内容即时生效）</p>
              <RpTextEditor
                :work-dir="current.work_dir"
                :rel-path="selectedNode.rel_path"
                :name="selectedNode.name"
                :file-type="selectedNode.file_type"
                :content="textContent"
              />
            </div>
            <button
              class="mt-2 flex items-center gap-1 text-xs text-blue-600 hover:text-blue-700"
              @click="modelEditMode = !modelEditMode"
            >
              {{ modelEditMode ? '返回 3D 预览' : '编辑 JSON' }}
            </button>
          </div>
          <div v-else-if="selectedNode && canEditText">
            <p v-if="reading && !fileContent" class="py-8 text-center text-sm text-gray-400">读取中…</p>
            <RpTextEditor
              v-else
              :work-dir="current.work_dir"
              :rel-path="selectedNode.rel_path"
              :name="selectedNode.name"
              :file-type="selectedNode.file_type"
              :content="textContent"
            />
          </div>
          <div v-else-if="selectedNode" class="flex flex-col items-center justify-center gap-1 py-16 text-gray-400">
            <p class="text-sm">暂不支持预览该类型文件</p>
            <p class="text-xs">{{ selectedNode.file_type }}</p>
          </div>
          <div v-else-if="reading" class="py-16 text-center text-sm text-gray-400">读取中…</div>
          <div v-else class="flex flex-col items-center justify-center gap-2 py-16 text-gray-400">
            <CubeIcon class="h-9 w-9 text-gray-300" />
            <p class="text-sm">在左侧选择文件以预览</p>
          </div>
        </div>
      </div>
    </div>

    <!-- 未打开：空状态 -->
    <div v-else class="flex flex-col items-center justify-center gap-2 px-5 py-16 text-gray-400">
      <CubeIcon class="h-10 w-10 text-gray-300" />
      <p class="text-sm">打开一个资源包（ZIP / 文件夹）开始浏览</p>
      <p class="text-xs">可从上方列表选择，或点击「打开 ZIP / 打开文件夹」</p>
    </div>
  </section>
</template>
