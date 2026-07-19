<script setup lang="ts">
/**
 * 版本设置 - 设置子页
 * 参考 PCL2 PageInstanceSetup：启动选项、内存分配、服务器、高级选项
 * 版本独立设置存 setup.ini（通过 updateVersionPersonalization）
 *
 * Java 选择（4 模式）拆分到 JavaModeSelector 子组件
 * 高级选项字段拆分到 AdvanceFieldsPanel 子组件
 * 进阶开关复用 ToggleRow 公共组件
 */
import { ref, onMounted } from 'vue'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import ToggleRow from '@/components/settings/ToggleRow.vue'
import AdvanceFieldsPanel from '@/components/version-settings/AdvanceFieldsPanel.vue'
import { useVersionSettings } from '@/composables/useVersionSettings'
import MemorySection from './MemorySection.vue'
import JavaModeSelector from './setup-tab/JavaModeSelector.vue'

const { selectedId, personalization, loadPersonalization } = useVersionSettings()

// 版本独立设置（从 setup.ini 读取）
const windowTitle = ref('')
const customInfo = ref('')
const serverEnter = ref('')

// 高级选项开关（参考 PCL2 PageInstanceSetup 高级选项）
const advanceDisableModUpdate = ref(false)
const advanceIgnoreJavaWarning = ref(false)
const advanceDisableAssetsVerify = ref(false)
const advanceDisableJlw = ref(false)
const advanceDisableLua = ref(false)

// personalization 字段名与 camelCase 一致（后端已加 #[serde(rename_all = "camelCase")]），
// 无需 snakeMap 转换，直接用字段名同步共享状态即可

async function loadSetup() {
  try {
    if (!personalization.value && selectedId.value) await loadPersonalization()
    const p = personalization.value
    if (p) {
      windowTitle.value = p.windowTitle
      customInfo.value = p.customInfo
      serverEnter.value = p.serverEnter
      advanceDisableModUpdate.value = p.advanceDisableModUpdate
      advanceIgnoreJavaWarning.value = p.advanceIgnoreJavaWarning
      advanceDisableAssetsVerify.value = p.advanceDisableAssetsVerify
      advanceDisableJlw.value = p.advanceDisableJlw
      advanceDisableLua.value = p.advanceDisableLua
    }
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
      (personalization.value as any)[field] = value
    }
    showSuccess(`${name}已保存`)
  } catch (e) { showError('保存失败：' + String(e)) }
}

async function handleSaveIndie(val: number) {
  if (!selectedId.value) return
  try {
    await tauri.updateVersionPersonalization(selectedId.value, { indieType: val })
    if (personalization.value) personalization.value.indieType = val
    showSuccess(val === 0 ? '已跟随全局设置' : val === 1 ? '已开启版本隔离' : '已关闭版本隔离')
  } catch (e) { showError('保存失败：' + String(e)) }
}

/** 保存高级选项开关 */
async function saveAdvanceSwitch(field: string, value: boolean, name: string) {
  if (!selectedId.value) return
  try {
    const update = { [field]: value } as tauri.PersonalizationUpdate
    await tauri.updateVersionPersonalization(selectedId.value, update)
    if (personalization.value) {
      (personalization.value as any)[field] = value
    }
    showSuccess(`${name}已保存`)
  } catch (e) { showError('保存失败：' + String(e)) }
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
            :model-value="String(personalization?.indieType ?? 0)"
            :options="[
              { value: '0', label: '跟随全局' },
              { value: '1', label: '开启' },
              { value: '2', label: '关闭' },
            ]"
            @update:model-value="(v: string) => handleSaveIndie(Number(v))"
          />
          <Tooltip
            v-if="personalization?.indieType === 1"
            text="与其他版本的存档、Mod 等文件相互独立，互不干涉。
这会使你无法跨版本共享存档，但可以规避 Mod 冲突问题。"
            position="top"
          >
            <span class="cursor-help text-xs text-gray-400">仅对此版本生效</span>
          </Tooltip>
          <Tooltip
            v-else-if="personalization?.indieType === 2"
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
          <JavaModeSelector />
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

    <!-- 高级选项（子组件：3 个文本字段） -->
    <AdvanceFieldsPanel />

    <!-- 进阶开关（参考 PCL2 PageInstanceSetup 高级选项，复用 ToggleRow 公共组件）-->
    <div class="bg-white rounded-xl border border-gray-200 shadow-sm overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">进阶开关</h3>
      <div class="divide-y divide-gray-100">
        <ToggleRow
          v-model="advanceDisableModUpdate"
          label="禁止更新 Mod"
          description="禁止为此版本更新 Mod，防止整合包玩家误操作"
          tooltip-text="禁止为此版本更新 Mod，以防止整合包玩家误操作。"
          @update:model-value="(v) => saveAdvanceSwitch('advanceDisableModUpdate', v, '禁止更新 Mod')"
        />
        <ToggleRow
          v-model="advanceIgnoreJavaWarning"
          label="忽略 Java 兼容性警告"
          description="跳过兼容性警告弹窗，强制使用手动选择的 Java"
          tooltip-text="如果手动选择了与当前版本不兼容的 Java，则自动跳过兼容性警告弹窗，强制使用手动选择的 Java。"
          @update:model-value="(v) => saveAdvanceSwitch('advanceIgnoreJavaWarning', v, '忽略 Java 警告')"
        />
        <ToggleRow
          v-model="advanceDisableAssetsVerify"
          label="关闭文件校验"
          description="不校验 libraries、assets、主 jar 文件是否被修改"
          tooltip-text="完全不更改 assets；不校验 libraries、第三方登录库与版本主 jar 文件是否被修改。&#10;如果你没有修改相关文件，请勿勾选此项。"
          @update:model-value="(v) => saveAdvanceSwitch('advanceDisableAssetsVerify', v, '文件校验')"
        />
        <ToggleRow
          v-model="advanceDisableJlw"
          label="禁用 Java Launch Wrapper"
          description="JLW 修复 Java 18- 中文路径启动问题，异常时可关闭"
          tooltip-text="是否使用 Java Launch Wrapper 修复 Java 18- 在中文路径下可能无法正常启动的问题。&#10;详见：https://github.com/00ll00/java_launch_wrapper"
          @update:model-value="(v) => saveAdvanceSwitch('advanceDisableJlw', v, 'JLW')"
        />
        <ToggleRow
          v-model="advanceDisableLua"
          label="禁用 LWJGL Unsafe Agent"
          description="LUA 修复 LWJGL 3.4.1 性能问题，卡顿时可关闭"
          tooltip-text="是否使用 LWJGL Unsafe Agent 修复 LWJGL 3.4.1 的一个性能问题。&#10;详见：https://github.com/HMCL-dev/lwjgl-unsafe-agent"
          @update:model-value="(v) => saveAdvanceSwitch('advanceDisableLua', v, 'LUA')"
        />
      </div>
    </div>
  </div>
</template>
