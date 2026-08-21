<script setup lang="ts">
/**
 * 版本选择页左侧文件夹列表
 *
 * 管理 Minecraft 文件夹的列表展示、切换、添加、移除。
 * 父组件在文件夹切换后需要重新加载版本列表（通过 @switched 事件）。
 */
import { ref, onMounted, onUnmounted, defineAsyncComponent } from 'vue'
import * as tauri from '@/utils/tauri'
import { pickDirectory, pickFile } from '@/utils/fileDialog'
import { toastSuccess, toastWarning, toastError, toastInfo } from '@/utils/toast'
import { showConfirm, showPrompt } from '@/utils/modal'
import { handleModpackDrop } from '@/composables/useDragDrop/handlers'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
import {
  FolderIcon,
  PlusIcon,
  XMarkIcon,
  ArrowDownTrayIcon,
} from '@heroicons/vue/24/outline'
import { safeCall } from '@/utils/async'

interface McFolder {
  name: string
  path: string
}

const emit = defineEmits<{
  switched: [path: string]
}>()

const folders = ref<McFolder[]>([])
const currentPath = ref<string>('')
const switchingFolder = ref(false)

/** 加载文件夹列表 */
async function loadFolders() {
  await safeCall(async () => {
    folders.value = await tauri.listMcFolders()
    currentPath.value = await tauri.getGameDir()
  }, 'load folders', () => toastError('加载文件夹列表失败'))
}

/** 切换文件夹 */
async function switchFolder(folder: McFolder) {
  if (switchingFolder.value) return
  if (folder.path === currentPath.value) return
  switchingFolder.value = true
  try {
    await tauri.switchMcFolder(folder.path)
    currentPath.value = folder.path
    emit('switched', folder.path)
    toastSuccess(`已切换到：${folder.name}`)
  } catch (e) {
    toastError(String(e))
  } finally {
    switchingFolder.value = false
  }
}

/** 添加文件夹 */
async function addFolder() {
  try {
    const selected = await pickDirectory({ title: '选择 .minecraft 文件夹' })
    if (!selected) { toastInfo('已取消选择'); return }

    const normalized = selected.replace(/[\\/]+$/, '')
    const parts = normalized.split(/[\\/]/)
    let defaultName = parts[parts.length - 1] || '文件夹'
    if (defaultName.toLowerCase() === '.minecraft' && parts.length >= 2) {
      defaultName = parts[parts.length - 2]
    }

    showPrompt(
      '添加文件夹',
      '请输入文件夹显示名称：',
      async (name) => {
        if (!name.trim()) return
        try {
          folders.value = await tauri.addMcFolder(name.trim(), selected)
          toastSuccess('文件夹已添加')
        } catch (e) {
          toastError(String(e))
        }
      },
      { defaultValue: defaultName, placeholder: '文件夹名称' },
    )
  } catch (e) {
    toastError(String(e))
  }
}

/** 导入本地整合包（复用全局拖拽的安装流程：预览 → 实例名 → 可选 Mod → 安装） */
async function importModpack() {
  try {
    // 文件选择器仅开放后端支持导入的整合包格式
    const selected = await pickFile({
      title: '选择整合包',
      filters: [{ name: '整合包', extensions: ['zip', 'mrpack'] }],
    })
    if (!selected) { toastInfo('已取消选择'); return }
    await handleModpackDrop(selected)
  } catch (e) {
    toastError(String(e))
  }
}

/** 移除文件夹 */
async function removeFolder(folder: McFolder, event: Event) {
  event.stopPropagation()
  if (folders.value.length <= 1) {
    toastWarning('至少需要保留一个文件夹')
    return
  }
  showConfirm(
    '移除文件夹',
    `确定要移除文件夹"${folder.name}"吗？（不会删除实际文件）`,
    async () => {
      try {
        folders.value = await tauri.removeMcFolder(folder.path)
        currentPath.value = await tauri.getGameDir()
        emit('switched', currentPath.value)
        toastSuccess('文件夹已移除')
      } catch (e) {
        toastError(String(e))
      }
    },
  )
}

/** dev-api 测试：注入模拟文件夹（查看列表铺满/截断样式） */
function onMockFolders(e: Event) {
  const detail = (e as CustomEvent<McFolder[]>).detail
  if (Array.isArray(detail)) folders.value = detail
}

onMounted(() => {
  loadFolders()
  window.addEventListener('molaunch:mock-folders', onMockFolders)
})
onUnmounted(() => window.removeEventListener('molaunch:mock-folders', onMockFolders))

defineExpose({ loadFolders })
</script>

<template>
  <aside class="flex w-[23%] flex-none flex-col border-r border-gray-200 bg-white">
    <!-- 滚动区（对齐 Settings 侧边栏：py-4，按钮自带 px-4） -->
    <div data-inner-scroll class="flex-1 overflow-y-auto py-4">
      <!-- 文件夹项（对齐 Settings 选中态：右侧 border 高亮 + bg-primary-50 满色 + Heroicons 图标 w-5 h-5 mr-3） -->
      <!-- 保留原生 button：文件夹列表项（w-full + active 状态 + 图标），
           Button.vue 的 scoped size 类无法承载列表项布局 -->
      <button
        v-for="folder in folders"
        :key="folder.path"
        class="group relative flex w-full items-center px-4 py-2.5 text-left text-sm font-medium transition-colors"
        :class="folder.path === currentPath
          ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500'
          : 'text-gray-700 hover:bg-gray-50'"
        :disabled="switchingFolder"
        @click="switchFolder(folder)"
      >
        <FolderIcon
          class="w-5 h-5 mr-3 flex-none"
          :class="folder.path === currentPath ? 'text-primary-500' : 'text-gray-400'"
        />
        <!-- 名称 + 路径 -->
        <div class="min-w-0 flex-1">
          <div class="truncate">{{ folder.name }}</div>
          <div class="truncate text-xs font-normal text-gray-400">{{ folder.path }}</div>
        </div>
        <!-- hover 时显示的移除按钮 -->
        <XMarkIcon
          v-if="folders.length > 1"
          class="ml-1 h-4 w-4 flex-none text-gray-400 opacity-0 transition-opacity hover:text-gray-600 group-hover:opacity-100"
          @click="removeFolder(folder, $event)"
        />
      </button>

      <!-- 分隔线 + 添加按钮 -->
      <div class="mx-4 my-3 border-t border-gray-100" />
      <Button
        type="ghost"
        long
        class="!justify-start !px-4"
        @click="addFolder"
      >
        <template #icon>
          <PlusIcon class="h-4 w-4 flex-none text-gray-400" />
        </template>
        添加已有文件夹
      </Button>
      <Button
        type="ghost"
        long
        class="!justify-start !px-4"
        @click="importModpack"
      >
        <template #icon>
          <ArrowDownTrayIcon class="h-4 w-4 flex-none text-gray-400" />
        </template>
        导入整合包
      </Button>
    </div>
  </aside>
</template>
