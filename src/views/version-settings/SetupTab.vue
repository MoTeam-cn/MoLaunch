<script setup lang="ts">
/**
 * 版本设置 - 设置子页
 * 参考 PCL2 PageInstanceSetup：启动选项、内存分配、服务器、高级选项
 * 版本独立设置存 setup.ini（通过 updateVersionPersonalization）
 *
 * Java 选择提供 4 种模式（参考 PCL2）：
 *   - auto        自动选择（按 MC 版本兼容性规则）
 *   - auto_version 自动选择指定版本范围的 Java
 *   - folder      使用版本文件夹中的 Java（{version_dir}/runtime/、jre/、java/）
 *   - custom      使用指定的 Java（手动选择 javaw.exe）
 */
import { ref, reactive, computed, onMounted } from 'vue'
import { useJavaStore } from '@/stores/java'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import { useVersionSettings } from '@/composables/useVersionSettings'
import MemorySection from './MemorySection.vue'
import type { JavaRequirements } from '@/types/java'

const javaStore = useJavaStore()
const { selectedId, personalization, loadPersonalization } = useVersionSettings()

// 版本独立设置（从 setup.ini 读取）
const windowTitle = ref('')
const customInfo = ref('')
const serverEnter = ref('')
const javaReqs = ref<JavaRequirements | null>(null)

/** Java 选择模式：auto/auto_version/folder/custom */
const javaMode = ref('auto')
/** 自动选择指定版本范围（仅 auto_version 模式生效，0=不限） */
const javaVersionMin = ref(0)
const javaVersionMax = ref(0)
/** custom 模式下手动选择的 Java 路径 */
const customJavaPath = ref('')
/** Java 列表刷新中状态 */
const refreshingJava = ref(false)

const advanceFields = reactive([
  { label: 'Java 虚拟机参数', field: 'advanceJvmArgs', name: 'JVM 参数', value: '', area: true,
    tip: '启动 Minecraft 时使用的额外 JVM 参数，在没有确定把握的情况下请不要尝试修改。\n若留空，则跟随全局设置的值。' },
  { label: '游戏参数', field: 'advanceGameArgs', name: '游戏参数', value: '', area: false,
    tip: '文本框中的内容将会被直接拼合在启动参数的末尾。\n例如，输入 --demo 则会以试玩模式启动游戏。\n若留空，则跟随全局设置的值。' },
  { label: '启动前执行命令', field: 'advanceRunCmd', name: '启动前命令', value: '', area: false,
    tip: '在 MC 启动前执行特定命令或程序，语法与 Windows 的命令提示符一致。\n涉及路径的操作最好都打上双引号，以避免路径中的空格导致运行失败。\n\n该项不会覆盖全局设置：启动时会先执行全局设置的命令，再执行版本设置的命令。' },
])

// personalization 字段名映射：camelCase → snake_case（用于同步共享状态）
const snakeMap: Record<string, string> = {
  windowTitle: 'window_title', customInfo: 'custom_info', serverEnter: 'server_enter',
  advanceJvmArgs: 'advance_jvm_args', advanceGameArgs: 'advance_game_args',
  advanceRunCmd: 'advance_run_cmd', javaPath: 'java_path',
}

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

/** 判断某个 Java 是否兼容当前版本 */
function isJavaCompatible(majorVersion: number): boolean {
  if (!javaReqs.value) return true
  const { min_java_version: min, max_java_version: max } = javaReqs.value
  if (min && majorVersion < min) return false
  if (max && majorVersion > max) return false
  return true
}

/** 系统中是否存在兼容的 Java */
const hasCompatibleJava = computed(() => {
  if (!javaReqs.value || javaStore.javaList.length === 0) return true
  return javaStore.javaList.some(j => isJavaCompatible(j.major_version))
})

/** custom 模式下的 Java 下拉选项（末尾追加"导入 Java"特殊项） */
const javaOptionsForCustom = computed(() => {
  const opts = javaStore.javaList.map(j => {
    const compat = isJavaCompatible(j.major_version)
    return {
      value: j.executable,
      label: `Java ${j.version}（${j.major_version}${compat ? ' ✓' : ' ✗'}）`,
    }
  })
  // 末尾追加"导入 Java"项，选中后触发文件选择器
  opts.push({ value: '__import__', label: '导入 Java' })
  return opts
})

/** 导入 Java 特殊值 */
const IMPORT_JAVA_VALUE = '__import__'

/**
 * 自动选择适配的 Java 路径（用于切换到 custom 模式时初始化）
 * 优先级：已存在的 customJavaPath > 第一个兼容的 Java > 列表第一项 > 空
 */
function pickDefaultJavaPath(): string {
  if (customJavaPath.value && javaStore.javaList.some(j => j.executable === customJavaPath.value)) {
    return customJavaPath.value
  }
  if (javaStore.javaList.length === 0) return ''
  const compatible = javaStore.javaList.find(j => isJavaCompatible(j.major_version))
  return (compatible ?? javaStore.javaList[0]).executable
}

/** custom 模式下选中的 Java 是否兼容 */
const customJavaWarning = computed(() => {
  if (javaMode.value !== 'custom' || !customJavaPath.value || !javaReqs.value) return ''
  const sel = javaStore.javaList.find(j => j.executable === customJavaPath.value)
  if (!sel) return ''
  const { min_java_version: min, max_java_version: max } = javaReqs.value
  if (min && sel.major_version < min) {
    return `当前版本至少需要 Java ${min}，你选择的 Java ${sel.major_version} 不兼容，可能导致游戏崩溃`
  }
  if (max && sel.major_version > max) {
    return `当前版本最高兼容到 Java ${max}，你选择的 Java ${sel.major_version} 不兼容，可能导致游戏崩溃`
  }
  return ''
})

async function loadSetup() {
  try {
    if (!personalization.value && selectedId.value) await loadPersonalization()
    const p = personalization.value
    if (p) {
      windowTitle.value = p.window_title
      customInfo.value = p.custom_info
      serverEnter.value = p.server_enter
      advanceFields[0].value = p.advance_jvm_args
      advanceFields[1].value = p.advance_game_args
      advanceFields[2].value = p.advance_run_cmd
      // Java 模式相关字段
      const mode = (p as any).java_mode || ''
      javaMode.value = ['auto', 'auto_version', 'folder', 'custom'].includes(mode) ? mode : 'auto'
      javaVersionMin.value = (p as any).java_version_min || 0
      javaVersionMax.value = (p as any).java_version_max || 0
      customJavaPath.value = p.java_path || ''
      // 加载 Java 需求（用 original_version 和 version_type 判断）
      const loader = ['forge', 'neoforge', 'fabric', 'quilt', 'optifine', 'liteloader'].includes(p.version_type) ? p.version_type : null
      javaReqs.value = await tauri.getJavaRequirements(p.original_version || p.version_type || '', loader)
    }
    if (!javaStore.javaLoaded) await javaStore.detectJava()
  } catch (e) {
    console.error('Failed to load setup:', e)
  }
}

/** 保存版本独立字段到 setup.ini */
async function savePersonalField(field: string, value: string, name: string) {
  if (!selectedId.value) return
  try {
    const update = { [field]: value } as tauri.PersonalizationUpdate
    await tauri.updateVersionPersonalization(selectedId.value, update)
    if (personalization.value) {
      const sk = snakeMap[field]
      if (sk) (personalization.value as any)[sk] = value
    }
    showSuccess(`${name}已保存`)
  } catch (e) { showError('保存失败：' + String(e)) }
}

async function handleSaveIndie(val: number) {
  if (!selectedId.value) return
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { indieType: val })
    if (personalization.value) personalization.value.indie_type = val
    showSuccess(val === 0 ? '已跟随全局设置' : val === 1 ? '已开启版本隔离' : '已关闭版本隔离')
  } catch (e) { showError('保存失败：' + String(e)) }
}

/** 切换 Java 选择模式 */
async function handleSaveJavaMode(mode: string) {
  if (!selectedId.value) return
  try {
    const update: tauri.PersonalizationUpdate = { javaMode: mode }
    // 切换到 custom 模式：自动选中适配的 Java（无适配则选第一项）
    if (mode === 'custom') {
      const picked = pickDefaultJavaPath()
      if (picked) {
        customJavaPath.value = picked
        update.javaPath = picked
      } else {
        update.javaPath = ''
      }
    } else {
      // 非 custom 模式清空 javaPath（后端会忽略）
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
      (personalization.value as any).java_mode = mode
      personalization.value.java_path = update.javaPath ?? ''
      ;(personalization.value as any).java_version_min = javaVersionMin.value
      ;(personalization.value as any).java_version_max = javaVersionMax.value
    }
    javaMode.value = mode
    const labelMap: Record<string, string> = {
      auto: '已设置为自动选择',
      auto_version: '已设置为按版本范围自动选择',
      folder: '已设置为使用版本文件夹中的 Java',
      custom: '已切换为指定 Java',
    }
    showSuccess(labelMap[mode] || 'Java 模式已保存')
  } catch (e) { showError('保存失败：' + String(e)) }
}

/** 保存 auto_version 模式的版本范围 */
async function handleSaveJavaVersionRange() {
  if (!selectedId.value) return
  if (javaVersionRangeTip.value) {
    showError(javaVersionRangeTip.value)
    return
  }
  try {
    const update: tauri.PersonalizationUpdate = {
      javaVersionMin: javaVersionMin.value,
      javaVersionMax: javaVersionMax.value,
    }
    await tauri.updateVersionPersonalization(selectedId.value, update)
    if (personalization.value) {
      (personalization.value as any).java_version_min = javaVersionMin.value
      ;(personalization.value as any).java_version_max = javaVersionMax.value
    }
    showSuccess('Java 版本范围已保存')
  } catch (e) { showError('保存失败：' + String(e)) }
}

/** custom 模式：从已找到的 Java 列表中选择，或选择"导入 Java"项触发文件选择器 */
async function handleSelectJavaFromList(value: string) {
  if (!selectedId.value) return
  // 选中"导入 Java"特殊项 → 弹出文件选择器
  if (value === IMPORT_JAVA_VALUE) {
    await handleImportJava()
    return
  }
  // 选中正常 Java 项
  customJavaPath.value = value
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { javaPath: value })
    if (personalization.value) personalization.value.java_path = value
    showSuccess('Java 路径已保存')
  } catch (e) { showError('保存失败：' + String(e)) }
}

/** 手动导入 Java（选择 javaw.exe），导入后自动选中并保存 */
async function handleImportJava() {
  const filePath = await tauri.selectFile('选择 Java 可执行文件', [
    { name: 'Java 可执行文件', extensions: ['exe'] },
  ])
  if (!filePath) return
  // 刷新 Java 列表（后端会自动检测新导入的 Java）
  await javaStore.refreshJava()
  const found = javaStore.javaList.find(j => j.executable === filePath)
  if (!found) {
    showError('所选文件不是有效的 Java 可执行文件')
    return
  }
  customJavaPath.value = filePath
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { javaPath: filePath })
    if (personalization.value) personalization.value.java_path = filePath
    showSuccess('Java 路径已保存')
  } catch (e) { showError('保存失败：' + String(e)) }
}

/** 刷新 Java 列表 */
async function handleRefreshJavaList() {
  refreshingJava.value = true
  try {
    await javaStore.refreshJava()
    showSuccess(`已刷新 Java 列表，共找到 ${javaStore.javaList.length} 个 Java`)
  } catch (e) {
    showError('刷新 Java 列表失败：' + String(e))
  } finally {
    refreshingJava.value = false
  }
}

onMounted(loadSetup)
</script>

<template>
  <div class="mx-auto max-w-3xl space-y-5">
    <!-- 启动选项 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-4 text-sm font-semibold text-gray-700">启动选项</h3>
      <div class="space-y-4">
        <div class="flex items-center gap-3">
          <label class="w-28 flex-none text-xs text-gray-500">版本隔离</label>
          <Select
            :model-value="String(personalization?.indie_type ?? 0)"
            :options="[
              { value: '0', label: '跟随全局' },
              { value: '1', label: '开启' },
              { value: '2', label: '关闭' },
            ]"
            @update:model-value="(v: string) => handleSaveIndie(Number(v))"
          />
          <Tooltip
            v-if="personalization?.indie_type === 1"
            text="与其他版本的存档、Mod 等文件相互独立，互不干涉。
这会使你无法跨版本共享存档，但可以规避 Mod 冲突问题。"
            position="top"
          >
            <span class="cursor-help text-xs text-gray-400">仅对此版本生效</span>
          </Tooltip>
          <Tooltip
            v-else-if="personalization?.indie_type === 2"
            text="与其余关闭隔离的版本共享存档、Mod 等文件。
若存在多个安装了 Mod 的版本，可能会由于 Mod 冲突而导致崩溃。"
            position="top"
          >
            <span class="cursor-help text-xs text-gray-400">仅对此版本生效</span>
          </Tooltip>
          <span v-else class="text-xs text-gray-400">仅对此版本生效</span>
        </div>

        <div class="flex items-center gap-3">
          <label class="w-28 flex-none text-xs text-gray-500">游戏窗口标题</label>
          <Tooltip text="自定义游戏窗口的标题，若留空则跟随全局设置的值。" position="top" :delay="0" class="flex-1">
            <input v-model="windowTitle" type="text" placeholder="跟随全局设置" class="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500" @blur="savePersonalField('windowTitle', windowTitle, '窗口标题')">
          </Tooltip>
        </div>

        <div class="flex items-center gap-3">
          <label class="w-28 flex-none text-xs text-gray-500">自定义信息</label>
          <Tooltip
            text="注意：Mojang 于 Minecraft 26.1 移除了该设置，因此该设置在新版本中无效。

该信息会显示在游戏主界面的左下角，与 F3 调试页面的左上角。
若留空，则跟随全局设置的值。"
            position="top"
            :delay="0"
            class="flex-1"
          >
            <input v-model="customInfo" type="text" placeholder="跟随全局设置" class="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500" @blur="savePersonalField('customInfo', customInfo, '自定义信息')">
          </Tooltip>
        </div>

        <div class="flex items-start gap-3">
          <label class="w-28 flex-none pt-1.5 text-xs text-gray-500">游戏 Java</label>
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
                @update:model-value="(v: string) => handleSaveJavaMode(v)"
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

            <!-- custom 模式：从列表选择（末尾含"导入 Java"项） -->
            <div v-else-if="javaMode === 'custom'" class="space-y-2">
              <div class="flex items-center gap-2">
                <!-- Java 列表下拉框（末尾项为"导入 Java"） -->
                <Tooltip
                  :text="customJavaPath || '从已找到的 Java 中选择，或选择导入项'"
                  position="top"
                  :delay="0"
                  class="min-w-0 flex-1"
                >
                  <Select
                    :model-value="customJavaPath"
                    :options="javaOptionsForCustom"
                    placeholder="从已找到的 Java 中选择"
                    @update:model-value="(v: string) => handleSelectJavaFromList(v)"
                  />
                </Tooltip>
                <!-- 刷新列表按钮 -->
                <Tooltip text="刷新 Java 列表" position="top" :delay="0">
                  <button
                    type="button"
                    class="flex h-[35px] w-[35px] flex-none items-center justify-center rounded-md border border-gray-300 bg-gray-50 text-gray-600 transition-colors hover:border-primary-500 hover:text-primary-600 disabled:opacity-50"
                    :disabled="refreshingJava"
                    @click="handleRefreshJavaList"
                  >
                    <svg class="h-4 w-4" :class="{ 'animate-spin': refreshingJava }" viewBox="0 0 20 20" fill="currentColor">
                      <path fill-rule="evenodd" d="M4 2a1 1 0 011 1v2.101a7.002 7.002 0 0111.601 2.566 1 1 0 11-1.885.666A5.002 5.002 0 005.999 7H9a1 1 0 010 2H4a1 1 0 01-1-1V3a1 1 0 011-1zm.008 9.057a1 1 0 011.276.61A5.002 5.002 0 0014.001 13H11a1 1 0 110-2h5a1 1 0 011 1v5a1 1 0 11-2 0v-2.101a7.002 7.002 0 01-11.601-2.566 1 1 0 01.61-1.276z" clip-rule="evenodd" />
                    </svg>
                  </button>
                </Tooltip>
              </div>
              <!-- 列表为空提示 -->
              <div v-if="javaStore.javaList.length === 0" class="rounded-md bg-amber-50 px-3 py-2 text-xs text-amber-600">
                未找到已安装的 Java，请选择列表中的"导入 Java"项手动选择。
              </div>
              <!-- 兼容性警告 -->
              <div v-if="customJavaWarning" class="flex items-start gap-2 rounded-md bg-red-50 px-3 py-2 text-xs text-red-600">
                <svg class="mt-0.5 h-4 w-4 flex-none" viewBox="0 0 20 20" fill="currentColor">
                  <path fill-rule="evenodd" d="M8.257 3.099c.765-1.36 2.722-1.36 3.486 0l5.58 9.92c.75 1.334-.213 2.98-1.742 2.98H4.42c-1.53 0-2.493-1.646-1.743-2.98l5.58-9.92zM11 13a1 1 0 11-2 0 1 1 0 012 0zm-1-8a1 1 0 00-1 1v3a1 1 0 002 0V6a1 1 0 00-1-1z" clip-rule="evenodd" />
                </svg>
                <span>{{ customJavaWarning }}</span>
              </div>
            </div>
          </div>
        </div>
      </div>
    </section>

    <!-- 内存分配（版本独立，子组件） -->
    <MemorySection />

    <!-- 服务器 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-4 text-sm font-semibold text-gray-700">服务器</h3>
      <div class="flex items-center gap-3">
        <label class="w-28 flex-none text-xs text-gray-500">自动进入服务器</label>
        <Tooltip
          text="在打开 Minecraft 后自动进入某服务器。
用英文冒号间隔 IP 与端口，例如 233.233.233.233:12345。"
          position="top"
          :delay="0"
          class="flex-1"
        >
          <input
            v-model="serverEnter"
            type="text"
            placeholder="例如：233.233.233.233:12345"
            class="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            @blur="savePersonalField('serverEnter', serverEnter, '服务器')"
          >
        </Tooltip>
      </div>
    </section>

    <!-- 高级选项（label 在上方，输入框全宽） -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-4 text-sm font-semibold text-gray-700">高级选项</h3>
      <div class="space-y-4">
        <div v-for="f in advanceFields" :key="f.field">
          <Tooltip :text="f.tip" position="top" :delay="0" class="mb-1.5 inline-flex">
            <label class="cursor-help text-xs text-gray-500">{{ f.label }}</label>
          </Tooltip>
          <textarea
            v-if="f.area"
            v-model="f.value"
            rows="3"
            placeholder="跟随全局设置"
            class="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            @blur="savePersonalField(f.field, f.value, f.name)"
          />
          <input
            v-else
            v-model="f.value"
            type="text"
            placeholder="跟随全局设置"
            class="w-full rounded-md border border-gray-300 px-3 py-1.5 text-sm focus:border-primary-500 focus:outline-none focus:ring-1 focus:ring-primary-500"
            @blur="savePersonalField(f.field, f.value, f.name)"
          >
        </div>
      </div>
    </section>
  </div>
</template>
