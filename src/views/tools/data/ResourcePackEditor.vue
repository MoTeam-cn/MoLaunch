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
const RpPackList = defineAsyncComponent(() => import('./RpPackList.vue'))
const RpPackInfo = defineAsyncComponent(() => import('./RpPackInfo.vue'))
const RpFileTreePanel = defineAsyncComponent(() => import('./RpFileTreePanel.vue'))
const RpContentPanel = defineAsyncComponent(() => import('./RpContentPanel.vue'))
import { toastError, toastSuccess } from '@/utils/toast'
import { pickFile, pickDirectory, pickSavePath } from '@/utils/fileDialog'
import { showConfirmAsync } from '@/utils/modal'
import { resourcepackList, rpOpen, rpRead, rpExport } from '@/utils/api/tools'
import type { RpOpenResult, RpReadResult, RpTreeNode, ResourcePackItem } from '@/utils/api/tools'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import { CubeIcon } from '@heroicons/vue/24/outline'

const packs = ref<ResourcePackItem[]>([])
const current = ref<RpOpenResult | null>(null)
const opening = ref(false)
const selectedNode = ref<RpTreeNode | null>(null)
const fileContent = ref<RpReadResult | null>(null)
const reading = ref(false)
const listOpen = ref(true)
const exporting = ref(false)
/** 版本隔离：'' = 全局（不隔离），其他 = 具体版本 ID（参考资源包转换器） */
const selectedVersionId = ref('')
const installedVersions = ref<InstalledVersionInfo[]>([])
const versionOptions = computed(() => [
  { label: '全局（不隔离）', value: '' },
  ...installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
])

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

    <RpPackList
      :packs="packs"
      :list-open="listOpen"
      :selected-version="selectedVersionId"
      :version-options="versionOptions"
      :opening="opening"
      @open="openPath"
      @update:list-open="listOpen = $event"
      @update:version="selectedVersionId = $event"
    />

    <!-- 已打开包：包信息 + 左树右内容 -->
    <div v-if="current" class="border-t border-gray-200">
      <RpPackInfo
        :current="current"
        :exporting="exporting"
        @save-zip="saveZip"
        @save-as-zip="saveAsZip"
      />

      <div class="grid grid-cols-1 border-t border-gray-200 md:grid-cols-[280px_1fr]">
        <RpFileTreePanel
          :key="current.work_dir"
          :tree="current.tree"
          :selected-path="selectedNode?.rel_path ?? ''"
          @select="selectNode"
        />
        <RpContentPanel
          :selected-node="selectedNode"
          :file-content="fileContent"
          :reading="reading"
          :work-dir="current.work_dir"
          :mc-version="current.mc_version"
          @mcmeta-saved="onMcmetaSaved"
        />
      </div>
    </div>

    <!-- 未打开：空状态 -->
    <div v-else class="flex flex-col items-center justify-center gap-2 px-5 py-16 text-gray-400">
      <CubeIcon class="h-10 w-10 text-gray-300" />
      <p class="text-sm">打开一个资源包（ZIP / 文件夹）开始浏览</p>
      <p class="text-xs">可从上方列表选择，或点击「打开 ZIP / 打开文件夹」载入</p>
    </div>
  </section>
</template>