<script setup lang="ts">
/**
 * 高级选项字段面板（SetupTab 子组件）
 *
 * 从 SetupTab.vue 抽取的「高级选项」区块，包含 3 个文本字段：
 * - Java 虚拟机参数（textarea）
 * - 游戏参数（input）
 * - 启动前执行命令（input）
 *
 * 字段值版本独立，通过 updateVersionPersonalization 保存到 setup.ini。
 * 复用 useVersionSettings 共享状态，自行加载 + 失焦保存。
 */
import { reactive, onMounted } from 'vue'
import * as tauri from '@/utils/tauri'
import { toastSuccess, toastError } from '@/utils/toast'
import Tooltip from '@/components/common/Tooltip.vue'
import Input from '@/components/common/Input.vue'
import { useVersionSettings } from '@/composables/useVersionSettings'

const { selectedId, personalization, loadPersonalization } = useVersionSettings()

interface AdvanceField {
  label: string
  field: 'advanceJvmArgs' | 'advanceGameArgs' | 'advanceRunCmd'
  name: string
  value: string
  area: boolean
  tip: string
}

const advanceFields = reactive<AdvanceField[]>([
  { label: 'Java 虚拟机参数', field: 'advanceJvmArgs', name: 'JVM 参数', value: '', area: true,
    tip: '启动 Minecraft 时使用的额外 JVM 参数，在没有确定把握的情况下请不要尝试修改。\n若留空，则跟随全局设置的值。' },
  { label: '游戏参数', field: 'advanceGameArgs', name: '游戏参数', value: '', area: false,
    tip: '文本框中的内容将会被直接拼合在启动参数的末尾。\n例如，输入 --demo 则会以试玩模式启动游戏。\n若留空，则跟随全局设置的值。' },
  { label: '启动前执行命令', field: 'advanceRunCmd', name: '启动前命令', value: '', area: false,
    tip: '在 MC 启动前执行特定命令或程序，语法与 Windows 的命令提示符一致。\n涉及路径的操作最好都打上双引号，以避免路径中的空格导致运行失败。\n\n【安全警告】此命令将通过系统 shell 执行，请勿输入来源不明的命令。共享整合包时请检查此字段。\n\n该项不会覆盖全局设置：启动时会先执行全局设置的命令，再执行版本设置的命令。' },
])

async function loadSetup() {
  try {
    if (!personalization.value && selectedId.value) await loadPersonalization()
    const p = personalization.value
    if (p) {
      advanceFields[0].value = p.advanceJvmArgs
      advanceFields[1].value = p.advanceGameArgs
      advanceFields[2].value = p.advanceRunCmd
    }
  } catch (e) {
    console.error('Failed to load advance fields:', e)
  }
}

/** 保存版本独立字段到 setup.ini */
async function savePersonalField(field: AdvanceField['field'], value: string, name: string) {
  if (!selectedId.value) return
  try {
    const update = { [field]: value } as tauri.PersonalizationUpdate
    await tauri.updateVersionPersonalization(selectedId.value, update)
    if (personalization.value) {
      (personalization.value as any)[field] = value
    }
    toastSuccess(`${name}已保存`)
  } catch (e) { toastError('保存失败：' + String(e)) }
}

onMounted(loadSetup)
</script>

<template>
  <section class="rounded-xl border border-gray-200 bg-white p-5 shadow-sm">
    <h3 class="mb-4 text-sm font-semibold text-gray-700">高级选项</h3>
    <div class="space-y-4">
      <div v-for="f in advanceFields" :key="f.field">
        <label class="block mb-1.5 text-xs text-gray-500">{{ f.label }}</label>
        <Tooltip :text="f.tip" position="top" :delay="0" block>
          <Input
            v-model="f.value"
            :textarea="f.area"
            :rows="3"
            placeholder="跟随全局设置"
            @blur="savePersonalField(f.field, f.value, f.name)"
          />
        </Tooltip>
      </div>
    </div>
  </section>
</template>
