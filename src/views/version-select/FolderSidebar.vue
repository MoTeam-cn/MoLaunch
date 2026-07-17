<script setup lang="ts">
/**
 * 版本选择页左侧文件夹列表（参考 PCL2 PageSelectLeft）
 *
 * 管理 Minecraft 文件夹的列表展示、切换、添加、移除。
 * 父组件在文件夹切换后需要重新加载版本列表（通过 @switched 事件）。
 */
import { ref, onMounted } from 'vue'
import { invoke } from '@tauri-apps/api/core'
import * as tauri from '@/utils/tauri'
import { showSuccess, showWarning, showError } from '@/utils/toast'
import { showConfirm, showPrompt } from '@/utils/modal'

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
  try {
    folders.value = await tauri.listMcFolders()
    currentPath.value = await invoke<string>('get_game_dir')
  } catch (e) {
    console.error('Failed to load folders:', e)
  }
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
    showSuccess(`已切换到：${folder.name}`)
  } catch (e) {
    showError(String(e))
  } finally {
    switchingFolder.value = false
  }
}

/** 添加文件夹 */
async function addFolder() {
  try {
    const selected = await invoke<string | null>('select_folder')
    if (!selected) return

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
          showSuccess('文件夹已添加')
        } catch (e) {
          showError(String(e))
        }
      },
      { defaultValue: defaultName, placeholder: '文件夹名称' },
    )
  } catch (e) {
    showError(String(e))
  }
}

/** 移除文件夹 */
async function removeFolder(folder: McFolder, event: Event) {
  event.stopPropagation()
  if (folders.value.length <= 1) {
    showWarning('至少需要保留一个文件夹')
    return
  }
  showConfirm(
    '移除文件夹',
    `确定要移除文件夹"${folder.name}"吗？（不会删除实际文件）`,
    async () => {
      try {
        folders.value = await tauri.removeMcFolder(folder.path)
        currentPath.value = await invoke<string>('get_game_dir')
        emit('switched', currentPath.value)
        showSuccess('文件夹已移除')
      } catch (e) {
        showError(String(e))
      }
    },
  )
}

onMounted(() => loadFolders())

defineExpose({ loadFolders })
</script>

<template>
  <aside class="flex w-64 flex-none flex-col border-r border-gray-200 bg-white">
    <!-- 滚动区 -->
    <div class="flex-1 overflow-y-auto px-3 pt-5">
      <!-- 分组标题 -->
      <div class="mb-1 px-2 text-xs font-medium text-gray-400">文件夹列表</div>
      <!-- 文件夹项 -->
      <ul class="space-y-0.5">
        <li v-for="folder in folders" :key="folder.path">
          <button
            class="group relative flex w-full items-center pl-3 pr-2 py-2.5 text-left transition-colors"
            :class="folder.path === currentPath
              ? 'bg-primary-50/70 text-primary-700'
              : 'text-gray-700 hover:bg-gray-50'"
            :disabled="switchingFolder"
            @click="switchFolder(folder)"
          >
            <!-- 选中时左侧高亮条（参考 PCL2 MyListItem RadioBox 样式） -->
            <span
              v-if="folder.path === currentPath"
              class="absolute left-0 top-1/2 h-5 w-0.5 -translate-y-1/2 rounded-full bg-primary-500"
            />
            <!-- 文件夹图标 -->
            <svg
              class="mr-2.5 h-4 w-4 flex-none"
              :class="folder.path === currentPath ? 'text-primary-500' : 'text-gray-400'"
              viewBox="0 0 20 20" fill="currentColor"
            >
              <path d="M2 6a2 2 0 012-2h4l2 2h6a2 2 0 012 2v6a2 2 0 01-2 2H4a2 2 0 01-2-2V6z" />
            </svg>
            <!-- 名称 + 路径 -->
            <div class="min-w-0 flex-1">
              <div class="truncate text-sm font-medium">{{ folder.name }}</div>
              <div class="truncate text-xs text-gray-400">{{ folder.path }}</div>
            </div>
            <!-- hover 时显示的移除按钮 -->
            <svg
              v-if="folders.length > 1"
              class="ml-1 h-4 w-4 flex-none text-gray-400 opacity-0 transition-opacity hover:text-gray-600 group-hover:opacity-100"
              viewBox="0 0 20 20" fill="currentColor"
              @click="removeFolder(folder, $event)"
            >
              <path fill-rule="evenodd" d="M4.3 4.3a1 1 0 011.4 0L10 8.6l4.3-4.3a1 1 0 111.4 1.4L11.4 10l4.3 4.3a1 1 0 01-1.4 1.4L10 11.4l-4.3 4.3a1 1 0 01-1.4-1.4L8.6 10 4.3 5.7a1 1 0 010-1.4z" clip-rule="evenodd" />
            </svg>
          </button>
        </li>
      </ul>

      <!-- 分组标题：添加或导入 -->
      <div class="mb-1 mt-5 px-2 text-xs font-medium text-gray-400">添加或导入</div>
      <ul class="space-y-0.5">
        <li>
          <button
            class="flex w-full items-center rounded-md px-3 py-2 text-left text-sm text-gray-600 transition-colors hover:bg-gray-50 hover:text-primary-600"
            @click="addFolder"
          >
            <svg class="mr-2.5 h-4 w-4 flex-none text-gray-400" viewBox="0 0 20 20" fill="currentColor">
              <path d="M10 3a1 1 0 011 1v5h5a1 1 0 110 2h-5v5a1 1 0 11-2 0v-5H4a1 1 0 110-2h5V4a1 1 0 011-1z" />
            </svg>
            添加已有文件夹
          </button>
        </li>
      </ul>
    </div>
  </aside>
</template>
