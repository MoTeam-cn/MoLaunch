<script setup lang="ts">
/**
 * 已安装版本列表组件
 * 参考 Minecraft 原版启动器列表风格
 */

import { computed } from 'vue'
import { CubeIcon, PlayIcon, TrashIcon, StopIcon, ArrowPathIcon } from '@heroicons/vue/24/outline'
import { useVersionStore } from '@/stores/version'
import { useAuthStore } from '@/stores/auth'
import { showWarning, showError } from '@/utils/modal'
import { showSuccess } from '@/utils/toast'
import { inferVersionType, getVersionTypeLabel, getVersionTypeBadgeClass } from '@/composables/useVersionMeta'
import Tooltip from '@/components/common/Tooltip.vue'

const versionStore = useVersionStore()
const authStore = useAuthStore()

interface Props {
  versions: string[]
  versionTypes?: Record<string, string> // 版本ID -> 版本类型
  getVersionIcon: (id: string, type: string) => string
}

const props = withDefaults(defineProps<Props>(), {
  versionTypes: () => ({})
})
const emit = defineEmits<{
  uninstall: [versionId: string]
  refresh: []
}>()

/** 推断版本类型：传入后端类型作为 backendType（old_beta/old_alpha 归一化为 old） */
function inferType(id: string): string {
  return inferVersionType(id, undefined, props.versionTypes[id])
}

async function handleLaunch(versionId: string) {
  // 防呆检查 - 使用toast提示
  if (!authStore.isLoggedIn) {
    showWarning('提示', '请先登录后再启动游戏')
    return
  }

  // 检查是否是当前版本正在启动
  if (versionStore.launchingVersionId === versionId) {
    showWarning('提示', '该版本正在启动中')
    return
  }

  // 检查是否有其他版本正在启动
  if (versionStore.launching) {
    showWarning('提示', '有其他版本正在启动中')
    return
  }

  // 检查当前版本是否正在运行
  if (versionStore.runningVersionId === versionId) {
    showWarning('提示', '该版本已在运行中')
    return
  }

  // 检查是否有其他版本正在运行
  if (versionStore.runningPid) {
    showWarning('提示', '有其他版本正在运行中')
    return
  }

  try {
    const pid = await versionStore.launchGame({
      versionId,
      username: authStore.currentUser?.name || 'Player',
      uuid: authStore.currentUser?.uuid || '',
    })
    showSuccess(`游戏已启动 (PID: ${pid})`)
  } catch (e) {
    showError('启动失败', String(e))
  }
}

async function handleStop(versionId: string) {
  // 只停止当前版本
  if (versionStore.runningVersionId !== versionId) {
    return
  }
  try {
    await versionStore.stopGame()
    showSuccess('已停止', '游戏进程已终止')
  } catch (e) {
    showError('停止失败', String(e))
  }
}

function handleUninstall(versionId: string) {
  // 直接emit，由父组件处理确认逻辑
  emit('uninstall', versionId)
}

// 排序后的版本列表
const sortedVersions = computed(() => {
  return [...props.versions].sort((a, b) => b.localeCompare(a))
})

// 判断是否正在运行
function isRunning(versionId: string) {
  return versionStore.runningVersionId === versionId
}

// 判断是否正在启动
function isLaunching(versionId: string) {
  return versionStore.launchingVersionId === versionId
}
</script>

<template>
  <div class="h-full flex flex-col bg-[#f5f5f5]">
    <!-- 空状态 -->
    <div v-if="versions.length === 0" class="flex-1 flex items-center justify-center">
      <div class="text-center py-12">
        <div class="w-20 h-20 mx-auto mb-4 rounded-full bg-gray-100 flex items-center justify-center">
          <CubeIcon class="w-10 h-10 text-gray-400" />
        </div>
        <h3 class="text-base font-medium text-gray-900 mb-1">暂无已安装版本</h3>
        <p class="text-sm text-gray-500">前往版本页面下载 Minecraft</p>
      </div>
    </div>

    <!-- 版本列表 -->
    <div v-else class="flex-1 overflow-y-auto px-4 py-3 space-y-2">
      <div
        v-for="versionId in sortedVersions"
        :key="versionId"
        class="version-card group relative bg-white rounded-lg border border-gray-200 hover:border-gray-300 hover:shadow-sm transition-all duration-150"
      >
        <div class="flex items-center p-3 gap-3">
          <!-- 版本图标 -->
          <div class="relative flex-shrink-0">
            <div class="w-12 h-12 rounded-lg overflow-hidden bg-gradient-to-br from-green-400 to-emerald-600 flex items-center justify-center">
              <img
                v-if="getVersionIcon(versionId, inferType(versionId))"
                :src="getVersionIcon(versionId, inferType(versionId))"
                :alt="versionId"
                class="w-full h-full object-cover"
              />
              <CubeIcon v-else class="w-7 h-7 text-white" />
            </div>
            <!-- 类型标记 -->
            <div
              class="absolute -bottom-1 -right-1 w-5 h-5 rounded-full flex items-center justify-center"
              :class="getVersionTypeBadgeClass(inferType(versionId))"
            >
              <span class="text-[8px] font-bold text-white">
                {{ getVersionTypeLabel(inferType(versionId)).charAt(0) }}
              </span>
            </div>
          </div>

          <!-- 版本信息 -->
          <div class="flex-1 min-w-0">
            <h3 class="text-sm font-semibold text-gray-900 truncate">{{ versionId }}</h3>
            <p class="text-xs text-gray-500 mt-0.5">
              {{ getVersionTypeLabel(inferType(versionId)) }}
            </p>
          </div>

          <!-- 操作按钮 -->
          <div class="flex items-center gap-2 flex-shrink-0">
            <!-- 运行中 - 显示停止按钮 -->
            <template v-if="isRunning(versionId)">
              <button
                class="play-btn stop-btn"
                @click.stop="handleStop(versionId)"
              >
                <StopIcon class="w-4 h-4" />
                <span>Stop</span>
              </button>
            </template>

            <!-- 启动中 - 显示加载按钮 -->
            <template v-else-if="isLaunching(versionId)">
              <button class="play-btn launching-btn" disabled>
                <ArrowPathIcon class="w-4 h-4 animate-spin" />
                <span>Launching...</span>
              </button>
            </template>

            <!-- 默认 - 显示启动按钮 (原版蓝色) -->
            <template v-else>
              <button
                class="play-btn launch-btn"
                @click.stop="handleLaunch(versionId)"
              >
                <PlayIcon class="w-4 h-4" />
                <span>Play</span>
              </button>
            </template>

            <!-- 卸载按钮 -->
            <Tooltip text="卸载" position="top">
              <button
                class="delete-btn"
                @click.stop="handleUninstall(versionId)"
              >
                <TrashIcon class="w-4 h-4" />
              </button>
            </Tooltip>
          </div>
        </div>
      </div>
    </div>

    <!-- 底部状态栏 -->
    <div v-if="versions.length > 0" class="px-4 py-2 bg-white border-t border-gray-200">
      <div class="flex items-center justify-between text-xs text-gray-500">
        <span>{{ versions.length }} 个版本已安装</span>
        <div v-if="versionStore.launchProgress" class="flex items-center gap-2">
          <span class="text-blue-600 font-medium">{{ versionStore.launchStageName }}</span>
          <div class="w-24 bg-gray-200 rounded-full h-1.5">
            <div
              class="bg-blue-500 h-1.5 rounded-full transition-all duration-300"
              :style="{ width: `${versionStore.launchProgress.overall_progress * 100}%` }"
            ></div>
          </div>
        </div>
        <span v-else-if="versionStore.runningPid" class="text-green-600 font-medium">
          游戏运行中 (PID: {{ versionStore.runningPid }})
        </span>
      </div>
    </div>
  </div>
</template>

<style scoped>
.version-card {
  @apply cursor-default;
}

.play-btn {
  @apply inline-flex items-center gap-1.5 px-4 py-2 rounded text-xs font-medium transition-all duration-150 focus:outline-none;
}

/* 原版蓝色启动按钮 */
.launch-btn {
  background: #3c8527;
  color: white;
}

.launch-btn:hover {
  background: #337322;
}

.launch-btn:active {
  background: #2a611c;
}

/* 停止按钮 */
.stop-btn {
  background: #c75050;
  color: white;
}

.stop-btn:hover {
  background: #a74040;
}

/* 启动中按钮 */
.launching-btn {
  background: #e5e7eb;
  color: #9ca3af;
  cursor: not-allowed;
}

.delete-btn {
  @apply p-2 rounded text-gray-400 hover:text-red-500 hover:bg-red-50 transition-colors opacity-0 group-hover:opacity-100 focus:opacity-100 focus:outline-none;
}
</style>
