<script setup lang="ts">
/**
 * 资源包可视化编辑器 - M1 查看器闭环
 *
 * 打开资源包（resourcepacks 目录列表 / 本地 ZIP / 文件夹）→ 包信息栏 +
 * 左文件树右内容分发（mcmeta 表单 / 纹理 2D / 语言表格 / 声音试听 / JSON 文本）。
 */
import { computed, defineAsyncComponent, onMounted, ref } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const RpFileTreeNode = defineAsyncComponent(() => import('./RpFileTreeNode.vue'))
const RpMcmetaForm = defineAsyncComponent(() => import('./RpMcmetaForm.vue'))
const RpTexturePreview = defineAsyncComponent(() => import('./RpTexturePreview.vue'))
const RpLangTable = defineAsyncComponent(() => import('./RpLangTable.vue'))
const RpSoundPreview = defineAsyncComponent(() => import('./RpSoundPreview.vue'))
import { toastError } from '@/utils/toast'
import { pickFile, pickDirectory } from '@/utils/fileDialog'
import { formatBytes } from '@/utils/format'
import { resourcepackList, rpOpen, rpRead } from '@/utils/api/tools'
import type { RpOpenResult, RpReadResult, RpTreeNode, ResourcePackItem } from '@/utils/api/tools'
import {
  ChevronDownIcon,
  CubeIcon,
  FolderOpenIcon,
} from '@heroicons/vue/24/outline'

const packs = ref<ResourcePackItem[]>([])
const current = ref<RpOpenResult | null>(null)
const opening = ref(false)
const selectedNode = ref<RpTreeNode | null>(null)
const fileContent = ref<RpReadResult | null>(null)
const reading = ref(false)
const expandedSet = ref<Set<string>>(new Set())
const listOpen = ref(true)

const fileCount = computed(() => countFiles(current.value?.tree))
const canReadText = computed(() =>
  ['json', 'model', 'text'].includes(selectedNode.value?.file_type ?? ''),
)
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

onMounted(loadPacks)

async function loadPacks() {
  try {
    const res = await resourcepackList()
    packs.value = res.items ?? []
  } catch (e) {
    toastError(`加载资源包列表失败: ${e instanceof Error ? e.message : String(e)}`)
  }
}

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
  if (node.file_type === 'mcmeta') return
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
      <div v-show="listOpen" class="mt-2 grid grid-cols-2 gap-2 md:grid-cols-3">
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
      </div>

      <div class="grid grid-cols-1 border-t border-gray-200 md:grid-cols-[280px_1fr]">
        <!-- 文件树 -->
        <div class="max-h-[560px] overflow-y-auto p-2 md:border-r md:border-gray-200">
          <RpFileTreeNode
            :node="current.tree"
            :selected-path="selectedNode?.rel_path ?? ''"
            :expanded-set="expandedSet"
            @select="selectNode"
            @toggle="toggleNode"
          />
        </div>

        <!-- 内容分发 -->
        <div class="max-h-[560px] overflow-y-auto p-4">
          <RpMcmetaForm
            v-if="selectedNode?.file_type === 'mcmeta'"
            :pack-format="current.pack_format"
            :mc-version="current.mc_version"
            :description="current.description"
          />
          <RpTexturePreview
            v-else-if="selectedNode?.file_type === 'png'"
            :src="mediaContent"
            :animated="selectedNode.animated"
            :name="selectedNode.name"
          />
          <RpLangTable
            v-else-if="selectedNode?.file_type === 'lang'"
            :content="textContent"
          />
          <RpSoundPreview
            v-else-if="selectedNode?.file_type === 'ogg'"
            :src="mediaContent"
          />
          <div v-else-if="selectedNode && canReadText" class="space-y-2">
            <div class="flex items-center gap-2">
              <h4 class="text-sm font-medium text-gray-700">{{ selectedNode.name }}</h4>
              <span class="text-xs text-gray-400">{{ selectedNode.rel_path }}</span>
            </div>
            <pre
              v-if="textContent"
              class="max-h-[500px] overflow-auto rounded border border-gray-200 bg-gray-50 p-3 font-mono text-xs text-gray-700"
            >{{ textContent }}</pre>
            <p v-else-if="reading" class="text-sm text-gray-400">读取中…</p>
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
