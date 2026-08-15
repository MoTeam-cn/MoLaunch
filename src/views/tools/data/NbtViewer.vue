<script setup lang="ts">
/**
 * NBT 数据编辑器（level.dat / playerdata / region .mca 等）
 *
 * 支持直接选择文件或通过「从存档选择」抽屉选取版本存档内的 NBT 文件；
 * .mca 按 Anvil 容器解析为区块列表，可切换区块编辑；树节点支持改值/新增/删除，
 * 保存时后端序列化回写（普通 NBT 保持 gzip，mca 整体重打包）。
 */
import { ref, computed, watch, defineAsyncComponent } from 'vue'
import {
  CubeIcon,
  BoltIcon,
  DocumentIcon,
  FolderOpenIcon,
  CheckIcon,
  ExclamationTriangleIcon,
} from '@heroicons/vue/24/outline'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const NbtTreeNode = defineAsyncComponent(() => import('@/views/tools/data/NbtTreeNode.vue'))
const NbtSaveDrawer = defineAsyncComponent(() => import('@/views/tools/data/NbtSaveDrawer.vue'))
import { toastError, toastSuccess, toastWarning } from '@/utils/toast'
import { nbtParse, nbtSave } from '@/utils/api/tools'
import type { NbtNode, NbtChunkInfo } from '@/utils/api/tools'
import { pickFile } from '@/utils/fileDialog'

const filePath = ref('')
const parsing = ref(false)
const saving = ref(false)
const root = ref<NbtNode | null>(null)
const fileType = ref<'nbt' | 'mca'>('nbt')
const chunks = ref<NbtChunkInfo[]>([])
const chunkSelect = ref('')
const expandedSet = ref<Set<string>>(new Set())
const saveDrawerVisible = ref(false)

const currentChunk = computed(() => chunks.value[Number(chunkSelect.value)])

/** 当前展示的树：普通文件取根，mca 取选中区块 */
const displayRoot = computed<NbtNode | null>(() =>
  fileType.value === 'mca' ? currentChunk.value?.root ?? null : root.value,
)

const chunkOptions = computed(() =>
  chunks.value.map((c, i) => ({
    label: `区块 (${c.x}, ${c.z}) · 索引 ${c.index}`,
    value: String(i),
  })),
)

/** 递归收集所有容器节点 key（默认全部展开） */
function collectKeys(node: NbtNode): Set<string> {
  const keys = new Set<string>()
  function walk(n: NbtNode, path: string) {
    const key = path + '/' + n.name + ':' + n.tag_type
    if (n.tag_type === 'compound' || n.tag_type === 'list') {
      keys.add(key)
      for (const c of n.children) walk(c, key)
    }
  }
  walk(node, '')
  return keys
}

function toggleExpand(key: string) {
  const next = new Set(expandedSet.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  expandedSet.value = next
}

function countNodes(node: NbtNode): number {
  let count = 1
  for (const child of node.children) count += countNodes(child)
  return count
}

async function doParse() {
  const path = filePath.value.trim()
  if (!path) return
  parsing.value = true
  try {
    const res = await nbtParse(path)
    fileType.value = res.file_type
    chunks.value = res.chunks
    root.value = null
    if (res.file_type === 'mca') {
      chunkSelect.value = res.chunks.length ? '0' : ''
      expandedSet.value = res.chunks.length ? collectKeys(res.chunks[0].root) : new Set()
    } else {
      root.value = res.root
      expandedSet.value = collectKeys(res.root)
    }
    toastSuccess(`解析成功：${res.file_type === 'mca' ? res.chunks.length + ' 个区块' : countNodes(res.root) + ' 个节点'}已加载`)
  } catch (e) {
    toastError('解析失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    parsing.value = false
  }
}

async function pickNbtFile() {
  const path = await pickFile({
    title: '选择 NBT 文件',
    filters: [
      { name: 'NBT 数据文件', extensions: ['dat', 'nbt', 'mca'] },
      { name: '所有文件', extensions: ['*'] },
    ],
  })
  if (path) {
    filePath.value = path
    doParse()
  }
}

async function handleSave() {
  const path = filePath.value.trim()
  const tree = displayRoot.value
  if (!path || !tree) return
  const chunk = fileType.value === 'mca' ? currentChunk.value : null
  toastWarning('保存将覆盖原文件，建议提前备份')
  const targetDesc = chunk ? `区块 (${chunk.x}, ${chunk.z})` : '文件'
  if (!window.confirm(`确认将修改写回${targetDesc}？\n${path}`)) return
  saving.value = true
  try {
    if (fileType.value === 'mca') {
      await nbtSave(path, tree, chunk!.index)
    } else {
      await nbtSave(path, tree)
    }
    toastSuccess('保存成功')
  } catch (e) {
    toastError('保存失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    saving.value = false
  }
}

/** 递归查找并移除目标节点（含展开状态清理） */
function removeNode(target: NbtNode, key: string) {
  const tree = displayRoot.value
  if (!tree) return
  function walk(node: NbtNode): boolean {
    const idx = node.children.indexOf(target)
    if (idx >= 0) {
      node.children.splice(idx, 1)
      return true
    }
    for (const c of node.children) {
      if (walk(c)) return true
    }
    return false
  }
  if (walk(tree)) expandedSet.value.delete(key)
}

function handleDrawerSelect(payload: { path: string }) {
  filePath.value = payload.path
  doParse()
}

watch(chunkSelect, (idx) => {
  const c = chunks.value[Number(idx)]
  if (c) expandedSet.value = collectKeys(c.root)
})
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <CubeIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">NBT 数据编辑器</h3>
      <span v-if="displayRoot" class="ml-auto text-xs text-gray-400">
        {{ countNodes(displayRoot) }} 个节点
      </span>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        解析并编辑 NBT 文件（level.dat、playerdata/*.dat、region/*.mca 等），以树形结构展示。
        支持 gzip 压缩格式；mca 文件按区块容器解析，可切换区块编辑。
      </p>

      <!-- 路径输入与操作 -->
      <div class="flex items-end gap-3">
        <div class="flex-1">
          <label class="mb-1 block text-xs font-medium text-gray-700">NBT 文件路径</label>
          <Input v-model="filePath" placeholder="如 .minecraft/saves/MyWorld/level.dat" clearable>
            <template #append>
              <FolderOpenIcon
                class="h-4 w-4 cursor-pointer text-gray-500 hover:text-primary-600 transition-colors"
                @click="pickNbtFile"
              />
            </template>
          </Input>
        </div>
        <Button type="secondary" @click="saveDrawerVisible = true">从存档选择</Button>
        <Button type="primary" :loading="parsing" :disabled="!filePath.trim()" @click="doParse">
          <template #icon><BoltIcon class="h-4 w-4" /></template>
          {{ parsing ? '解析中...' : '解析' }}
        </Button>
      </div>

      <!-- mca 区块选择 -->
      <div v-if="fileType === 'mca'" class="flex items-center gap-3">
        <Select v-model="chunkSelect" :options="chunkOptions" class="w-80" />
        <span class="text-xs text-gray-400">共 {{ chunks.length }} 个区块，选择后编辑其 NBT 数据</span>
      </div>

      <!-- 树形编辑 -->
      <div
        v-if="displayRoot"
        data-inner-scroll
        class="max-h-[500px] overflow-y-auto rounded-lg border border-gray-200 p-3"
      >
        <NbtTreeNode
          :node="displayRoot"
          :path="''"
          :expanded-set="expandedSet"
          :editable="true"
          is-root
          @toggle="toggleExpand"
          @remove="removeNode"
        />
      </div>

      <!-- 空状态 -->
      <div v-else class="flex flex-col items-center justify-center py-8 text-gray-400">
        <DocumentIcon class="h-8 w-8 mb-2 text-gray-300" />
        <span class="text-xs">选择 NBT 文件（level.dat / playerdata / region .mca）后点击解析</span>
      </div>

      <!-- 保存栏 -->
      <div
        v-if="displayRoot"
        class="flex items-center justify-between gap-3 rounded-lg border border-amber-200 bg-amber-50 px-4 py-3"
      >
        <div class="flex items-center gap-2 text-xs text-amber-700">
          <ExclamationTriangleIcon class="h-4 w-4 flex-none" />
          <span>点击叶子节点值可直接编辑，悬停节点可新增/删除；保存将覆盖原文件，建议先备份。</span>
        </div>
        <Button type="primary" :loading="saving" class="flex-none" @click="handleSave">
          <template #icon><CheckIcon class="h-4 w-4" /></template>
          保存修改
        </Button>
      </div>
    </div>

    <NbtSaveDrawer v-model="saveDrawerVisible" @select="handleDrawerSelect" />
  </section>
</template>
