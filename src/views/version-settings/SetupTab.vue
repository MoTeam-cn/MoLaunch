<script setup lang="ts">
/**
 * 版本设置 - 设置子页
 * 参考 PCL2 PageInstanceSetup：启动选项、内存分配、服务器、高级选项
 * 版本独立设置存 setup.ini（通过 updateVersionPersonalization）
 *
 * Java 选择（4 模式）拆分到 JavaModeSelector 子组件
 */
import { ref, reactive, onMounted } from 'vue'
import * as tauri from '@/utils/tauri'
import { showSuccess, showError } from '@/utils/toast'
import Select from '@/components/common/Select.vue'
import Tooltip from '@/components/common/Tooltip.vue'
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

const advanceFields = reactive([
  { label: 'Java 虚拟机参数', field: 'advanceJvmArgs', name: 'JVM 参数', value: '', area: true,
    tip: '启动 Minecraft 时使用的额外 JVM 参数，在没有确定把握的情况下请不要尝试修改。\n若留空，则跟随全局设置的值。' },
  { label: '游戏参数', field: 'advanceGameArgs', name: '游戏参数', value: '', area: false,
    tip: '文本框中的内容将会被直接拼合在启动参数的末尾。\n例如，输入 --demo 则会以试玩模式启动游戏。\n若留空，则跟随全局设置的值。' },
  { label: '启动前执行命令', field: 'advanceRunCmd', name: '启动前命令', value: '', area: false,
    tip: '在 MC 启动前执行特定命令或程序，语法与 Windows 的命令提示符一致。\n涉及路径的操作最好都打上双引号，以避免路径中的空格导致运行失败。\n\n⚠️ 安全警告：此命令将通过系统 shell 执行，请勿输入来源不明的命令。共享整合包时请检查此字段。\n\n该项不会覆盖全局设置：启动时会先执行全局设置的命令，再执行版本设置的命令。' },
])

// personalization 字段名映射：camelCase → snake_case（用于同步共享状态）
const snakeMap: Record<string, string> = {
  windowTitle: 'window_title', customInfo: 'custom_info', serverEnter: 'server_enter',
  advanceJvmArgs: 'advance_jvm_args', advanceGameArgs: 'advance_game_args',
  advanceRunCmd: 'advance_run_cmd', javaPath: 'java_path',
  advanceDisableModUpdate: 'advance_disable_mod_update',
  advanceIgnoreJavaWarning: 'advance_ignore_java_warning',
  advanceDisableAssetsVerify: 'advance_disable_assets_verify',
  advanceDisableJlw: 'advance_disable_jlw',
  advanceDisableLua: 'advance_disable_lua',
}

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
      advanceDisableModUpdate.value = p.advance_disable_mod_update
      advanceIgnoreJavaWarning.value = p.advance_ignore_java_warning
      advanceDisableAssetsVerify.value = p.advance_disable_assets_verify
      advanceDisableJlw.value = p.advance_disable_jlw
      advanceDisableLua.value = p.advance_disable_lua
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

/** 保存高级选项开关 */
async function saveAdvanceSwitch(field: string, value: boolean, name: string) {
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

    <!-- 高级选项 -->
    <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
      <h3 class="mb-4 text-sm font-semibold text-gray-700">高级选项</h3>
      <div class="space-y-4">
        <div v-for="f in advanceFields" :key="f.field">
          <label class="block mb-1.5 text-xs text-gray-500">{{ f.label }}</label>
          <Tooltip :text="f.tip" position="top" :delay="0" block>
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
          </Tooltip>
        </div>
        <!-- 高级选项开关（参考 PCL2 PageInstanceSetup 高级选项）-->
        <div class="space-y-2 pt-2">
          <label class="block text-xs font-medium text-gray-500 mb-1">进阶开关</label>
          <!-- 禁止更新 Mod -->
          <Tooltip
            text="禁止为此版本更新 Mod，以防止整合包玩家误操作。"
            position="top" :delay="0" block
          >
            <div class="flex items-center justify-between py-1.5">
              <span class="text-sm text-gray-700">禁止更新 Mod</span>
              <button
                class="relative inline-flex h-5 w-9 flex-none items-center rounded-full transition-colors"
                :class="advanceDisableModUpdate ? 'bg-primary-500' : 'bg-gray-300'"
                @click="advanceDisableModUpdate = !advanceDisableModUpdate; saveAdvanceSwitch('advanceDisableModUpdate', advanceDisableModUpdate, '禁止更新 Mod')"
              >
                <span
                  class="inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform"
                  :class="advanceDisableModUpdate ? 'translate-x-5' : 'translate-x-1'"
                />
              </button>
            </div>
          </Tooltip>
          <!-- 忽略 Java 兼容性警告 -->
          <Tooltip
            text="如果手动选择了与当前版本不兼容的 Java，则自动跳过兼容性警告弹窗，强制使用手动选择的 Java。"
            position="top" :delay="0" block
          >
            <div class="flex items-center justify-between py-1.5">
              <span class="text-sm text-gray-700">忽略 Java 兼容性警告</span>
              <button
                class="relative inline-flex h-5 w-9 flex-none items-center rounded-full transition-colors"
                :class="advanceIgnoreJavaWarning ? 'bg-primary-500' : 'bg-gray-300'"
                @click="advanceIgnoreJavaWarning = !advanceIgnoreJavaWarning; saveAdvanceSwitch('advanceIgnoreJavaWarning', advanceIgnoreJavaWarning, '忽略 Java 警告')"
              >
                <span
                  class="inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform"
                  :class="advanceIgnoreJavaWarning ? 'translate-x-5' : 'translate-x-1'"
                />
              </button>
            </div>
          </Tooltip>
          <!-- 关闭文件校验 -->
          <Tooltip
            text="完全不更改 assets；不校验 libraries、第三方登录库与版本主 jar 文件是否被修改。&#10;如果你没有修改相关文件，请勿勾选此项。"
            position="top" :delay="0" block
          >
            <div class="flex items-center justify-between py-1.5">
              <span class="text-sm text-gray-700">关闭文件校验</span>
              <button
                class="relative inline-flex h-5 w-9 flex-none items-center rounded-full transition-colors"
                :class="advanceDisableAssetsVerify ? 'bg-primary-500' : 'bg-gray-300'"
                @click="advanceDisableAssetsVerify = !advanceDisableAssetsVerify; saveAdvanceSwitch('advanceDisableAssetsVerify', advanceDisableAssetsVerify, '文件校验')"
              >
                <span
                  class="inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform"
                  :class="advanceDisableAssetsVerify ? 'translate-x-5' : 'translate-x-1'"
                />
              </button>
            </div>
          </Tooltip>
          <!-- 禁用 Java Launch Wrapper -->
          <Tooltip
            text="是否使用 Java Launch Wrapper 修复 Java 18- 在中文路径下可能无法正常启动的问题。&#10;详见：https://github.com/00ll00/java_launch_wrapper"
            position="top" :delay="0" block
          >
            <div class="flex items-center justify-between py-1.5">
              <span class="text-sm text-gray-700">禁用 Java Launch Wrapper</span>
              <button
                class="relative inline-flex h-5 w-9 flex-none items-center rounded-full transition-colors"
                :class="advanceDisableJlw ? 'bg-primary-500' : 'bg-gray-300'"
                @click="advanceDisableJlw = !advanceDisableJlw; saveAdvanceSwitch('advanceDisableJlw', advanceDisableJlw, 'JLW')"
              >
                <span
                  class="inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform"
                  :class="advanceDisableJlw ? 'translate-x-5' : 'translate-x-1'"
                />
              </button>
            </div>
          </Tooltip>
          <!-- 禁用 LWJGL Unsafe Agent -->
          <Tooltip
            text="是否使用 LWJGL Unsafe Agent 修复 LWJGL 3.4.1 的一个性能问题。&#10;详见：https://github.com/HMCL-dev/lwjgl-unsafe-agent"
            position="top" :delay="0" block
          >
            <div class="flex items-center justify-between py-1.5">
              <span class="text-sm text-gray-700">禁用 LWJGL Unsafe Agent</span>
              <button
                class="relative inline-flex h-5 w-9 flex-none items-center rounded-full transition-colors"
                :class="advanceDisableLua ? 'bg-primary-500' : 'bg-gray-300'"
                @click="advanceDisableLua = !advanceDisableLua; saveAdvanceSwitch('advanceDisableLua', advanceDisableLua, 'LUA')"
              >
                <span
                  class="inline-block h-3.5 w-3.5 transform rounded-full bg-white transition-transform"
                  :class="advanceDisableLua ? 'translate-x-5' : 'translate-x-1'"
                />
              </button>
            </div>
          </Tooltip>
        </div>
      </div>
    </section>
  </div>
</template>
