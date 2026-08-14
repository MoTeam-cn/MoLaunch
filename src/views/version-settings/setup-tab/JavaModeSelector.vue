<script setup lang="ts">
/**
 * Java 选择模式组件（版本独立设置）
 *
 * 4 种模式：
 *   - auto         自动选择（按 MC 版本兼容性规则）
 *   - auto_version 自动选择指定版本范围的 Java
 *   - folder       使用版本文件夹中的 Java（{version_dir}/runtime/、jre/、java/）
 *   - custom       使用指定的 Java（手动选择 javaw.exe）
 *
 * 子组件（setup-tab/）：
 *   - JavaCustomMode  custom 模式 UI（Java 列表选择/导入/刷新/警告）
 *
 * 通过 useVersionSettings 共享状态（模块级单例），无需 props。
 */
import { ref, computed, watch, defineAsyncComponent } from 'vue'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import { isJavaCompatible } from '@/utils/api/java'
import { toastSuccess, toastError } from '@/utils/toast'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
import { useVersionSettings } from '@/composables/useVersionSettings'
import type { JavaRequirements } from '@/types/java'
const JavaCustomMode = defineAsyncComponent(() => import('./JavaCustomMode.vue'))
import { safeCall } from '@/utils/async'

const javaStore = useJavaStore()
const { selectedId, personalization } = useVersionSettings()

/** Java 选择模式：auto/auto_version/folder/custom */
const javaMode = ref('auto')
/** 自动选择指定版本范围（仅 auto_version 模式生效，0=不限） */
const javaVersionMin = ref(0)
const javaVersionMax = ref(0)
/** custom 模式下手动选择的 Java 路径 */
const customJavaPath = ref('')
/** Java 列表刷新中状态 */
const refreshingJava = ref(false)
/** Java 需求（从后端加载，用于兼容性检查） */
const javaReqs = ref<JavaRequirements | null>(null)

/** Java 下拉框 4 个固定选项 */
const javaModeOptions = [
  { value: 'auto', label: '自动选择（推荐）' },
  { value: 'auto_version', label: '自动选择指定版本的 Java' },
  { value: 'folder', label: '使用版本文件夹中的 Java' },
  { value: 'custom', label: '使用指定的 Java' },
]

/** Java 需求描述（用于提示文案） */
const javaReqDesc = computed(() => {
  if (!javaReqs.value) return ''
  const { min_java_version: min, max_java_version: max } = javaReqs.value
  if (min && max) return `当前版本需要 Java ${min}~${max}`
  if (min) return `当前版本需要 Java ${min}+`
  if (max) return `当前版本最高兼容 Java ${max}`
  return ''
})

/** auto_version 模式下输入的版本范围是否合法 */
const javaVersionRangeTip = computed(() => {
  if (javaMode.value !== 'auto_version') return ''
  const min = javaVersionMin.value
  const max = javaVersionMax.value
  if (min > 0 && max > 0 && min > max) return '最小版本不能大于最大版本'
  return ''
})

/** 系统中是否存在兼容的 Java */
const hasCompatibleJava = computed(() => {
  if (!javaReqs.value || javaStore.javaList.length === 0) return true
  return javaStore.javaList.some(j => isJavaCompatible(j.major_version, javaReqs.value))
})

/** 切换到 custom 模式时自动选中适配的 Java */
function pickDefaultJavaPath(): string {
  if (customJavaPath.value && javaStore.javaList.some(j => j.executable === customJavaPath.value)) {
    return customJavaPath.value
  }
  if (javaStore.javaList.length === 0) return ''
  const compatible = javaStore.javaList.find(j => isJavaCompatible(j.major_version, javaReqs.value))
  return (compatible ?? javaStore.javaList[0]).executable
}

/** 切换 Java 选择模式 */
async function handleSaveJavaMode(mode: string) {
  if (!selectedId.value) return
  try {
    const update: tauri.PersonalizationUpdate = { javaMode: mode }
    if (mode === 'custom') {
      const picked = pickDefaultJavaPath()
      if (picked) {
        customJavaPath.value = picked
        update.javaPath = picked
      } else {
        update.javaPath = ''
      }
    } else {
      update.javaPath = ''
      customJavaPath.value = ''
    }
    // 切换到 auto_version 时若 min/max 都为 0，初始化为当前版本需求的 min/max
    if (mode === 'auto_version' && javaVersionMin.value === 0 && javaVersionMax.value === 0 && javaReqs.value) {
      javaVersionMin.value = javaReqs.value.min_java_version || 0
      javaVersionMax.value = javaReqs.value.max_java_version || 0
      update.javaVersionMin = javaVersionMin.value
      update.javaVersionMax = javaVersionMax.value
    }
    await tauri.updateVersionPersonalization(selectedId.value, update)
    if (personalization.value) {
      personalization.value.javaMode = mode
      personalization.value.javaPath = update.javaPath ?? ''
      personalization.value.javaVersionMin = javaVersionMin.value
      personalization.value.javaVersionMax = javaVersionMax.value
    }
    javaMode.value = mode
    const labelMap: Record<string, string> = {
      auto: '已设置为自动选择',
      auto_version: '已设置为按版本范围自动选择',
      folder: '已设置为使用版本文件夹中的 Java',
      custom: '已切换为指定 Java',
    }
    toastSuccess(labelMap[mode] || 'Java 模式已保存')
  } catch (e) { toastError('保存失败：' + String(e)) }
}

/** 保存 auto_version 模式的版本范围 */
async function handleSaveJavaVersionRange() {
  if (!selectedId.value) return
  if (javaVersionRangeTip.value) {
    toastError(javaVersionRangeTip.value)
    return
  }
  try {
    const update: tauri.PersonalizationUpdate = {
      javaVersionMin: javaVersionMin.value,
      javaVersionMax: javaVersionMax.value,
    }
    await tauri.updateVersionPersonalization(selectedId.value, update)
    if (personalization.value) {
      personalization.value.javaVersionMin = javaVersionMin.value
      personalization.value.javaVersionMax = javaVersionMax.value
    }
    toastSuccess('Java 版本范围已保存')
  } catch (e) { toastError('保存失败：' + String(e)) }
}

/** 监听 personalization 加载完成，初始化 Java 相关字段 + 加载 Java 需求 */
watch(personalization, async (p) => {
  if (!p) return
  const mode = p.javaMode || ''
  javaMode.value = ['auto', 'auto_version', 'folder', 'custom'].includes(mode) ? mode : 'auto'
  javaVersionMin.value = p.javaVersionMin || 0
  javaVersionMax.value = p.javaVersionMax || 0
  customJavaPath.value = p.javaPath || ''

  // 加载 Java 需求（用 originalVersion 和 versionType 判断）
  const loader = ['forge', 'neoforge', 'fabric', 'quilt', 'optifine', 'liteloader'].includes(p.versionType) ? p.versionType : null
  const reqs = await safeCall(() => tauri.getJavaRequirements(p.originalVersion || p.versionType || '', loader), 'load Java requirements', () => toastError('加载 Java 需求失败'))
  if (reqs) javaReqs.value = reqs

  if (!javaStore.javaLoaded) await javaStore.detectJava()
}, { immediate: true })
</script>

<template>
  <div class="min-w-0 flex-1 space-y-2">
    <!-- 4 模式下拉框 -->
    <Tooltip
      :text="javaReqDesc ? javaReqDesc : '若将 Java 放在版本文件夹，在选择“使用版本文件夹中的 Java”时会优先使用它'"
      position="top"
      class="block"
    >
      <Select
        :model-value="javaMode"
        :options="javaModeOptions"
        @update:model-value="(v: string | number) => handleSaveJavaMode(String(v))"
      />
    </Tooltip>

    <!-- auto 模式：提示 -->
    <div v-if="javaMode === 'auto'" class="rounded-md bg-blue-50 px-3 py-2 text-xs text-blue-600">
      <span v-if="javaReqDesc">{{ javaReqDesc }}，启动时将自动选择合适的 Java。</span>
      <span v-else>启动时将根据版本要求自动选择合适的 Java。</span>
      <span v-if="!hasCompatibleJava" class="mt-1 block text-amber-600">你的电脑上没有可供该版本使用的 Java，启动游戏时将自动下载。</span>
    </div>

    <!-- auto_version 模式：版本范围输入 -->
    <div v-else-if="javaMode === 'auto_version'" class="space-y-2">
      <div class="flex items-center gap-2">
        <span class="text-xs text-gray-500">Java 主版本范围：</span>
        <input
          v-model.number="javaVersionMin"
          type="number"
          min="0"
          placeholder="最低"
          class="w-20 rounded-md border border-gray-300 px-2 py-1 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          @blur="handleSaveJavaVersionRange"
        >
        <span class="text-xs text-gray-400">~</span>
        <input
          v-model.number="javaVersionMax"
          type="number"
          min="0"
          placeholder="最高"
          class="w-20 rounded-md border border-gray-300 px-2 py-1 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
          @blur="handleSaveJavaVersionRange"
        >
        <span class="text-xs text-gray-400">（0 = 不限）</span>
      </div>
      <div v-if="javaVersionRangeTip" class="rounded-md bg-red-50 px-3 py-1.5 text-xs text-red-600">
        {{ javaVersionRangeTip }}
      </div>
      <div v-else class="rounded-md bg-blue-50 px-3 py-2 text-xs text-blue-600">
        将在指定版本范围内自动选择 Java。{{ javaReqDesc ? `参考：${javaReqDesc}。` : '' }}
        <span v-if="!hasCompatibleJava" class="mt-1 block text-amber-600">你的电脑上没有可供该版本使用的 Java，启动游戏时将自动下载。</span>
      </div>
    </div>

    <!-- folder 模式：提示 -->
    <div v-else-if="javaMode === 'folder'" class="rounded-md bg-blue-50 px-3 py-2 text-xs text-blue-600">
      将在版本文件夹下的 <code class="rounded bg-blue-100 px-1">runtime/</code>、<code class="rounded bg-blue-100 px-1">jre/</code>、<code class="rounded bg-blue-100 px-1">java/</code> 子目录中查找 Java。若未找到则回退到自动选择。
    </div>

    <!-- custom 模式：子组件接管（列表选择 + 导入 + 刷新 + 警告） -->
    <JavaCustomMode
      v-else-if="javaMode === 'custom'"
      v-model:custom-java-path="customJavaPath"
      v-model:refreshing-java="refreshingJava"
      :java-reqs="javaReqs"
    />
  </div>
</template>
