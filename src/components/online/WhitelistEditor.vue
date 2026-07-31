<script setup lang="ts">
/**
 * 房主白名单编辑器（阶段三子任务 8 安全加强）
 *
 * 两种使用场景：
 * - **创建房间表单**（`mode='create'`）：纯本地编辑，v-model 双向绑定
 *   `{ enabled: boolean, deviceIds: string[] }`，不调用后端；房主创建房间时
 *   一并提交到 `createRoom`。
 * - **运行期管理**（`mode='runtime'`）：调用后端 API 实时增删，通过
 *   `roomCode` 触发 `room_list_whitelist` / `room_add_whitelist` /
 *   `room_remove_whitelist` / `room_set_whitelist_enabled`。
 *
 * # 复用约定
 *
 * - 使用 [Input.vue](src/components/common/Input.vue) 而非原生 `<input>`
 * - 使用 [Button.vue](src/components/common/Button.vue) 而非原生 `<button>`
 * - 使用 [Tooltip.vue](src/components/common/Tooltip.vue) 而非原生 `title`
 * - checkbox 沿用项目惯例（`accent-primary-500`），与 ExportTab / ArchiveManager 一致
 */
import { ref, computed, watch, onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Checkbox from '@/components/common/Checkbox.vue'
import Tooltip from '@/components/common/Tooltip.vue'
import {
  PlusIcon,
  TrashIcon,
  ShieldCheckIcon,
  ShieldExclamationIcon,
  UserPlusIcon,
} from '@heroicons/vue/24/outline'
import { toastError } from '@/utils/toast'
import { stripMcsdkPrefix, ensureMcsdkPrefix } from '@/utils/online/device-id'

const props = withDefaults(
  defineProps<{
    /**
     * 模式：
     * - `create`：本地编辑模式，v-model 绑定 `{ enabled, deviceIds }`
     * - `runtime`：运行期管理模式，通过 `roomCode` 调用后端 API
     */
    mode: 'create' | 'runtime'
    /** 房间码（仅 `runtime` 模式使用） */
    roomCode?: string
    /** create 模式下的 v-model 绑定值 */
    modelValue?: { enabled: boolean; deviceIds: string[] }
  }>(),
  {
    roomCode: '',
    modelValue: () => ({ enabled: false, deviceIds: [] }),
  },
)

const emit = defineEmits<{
  'update:modelValue': [value: { enabled: boolean; deviceIds: string[] }]
}>()

const store = useOnlineStore()

/** create 模式：本地输入框值 */
const createInput = ref('')
/** runtime 模式：本地输入框值 */
const runtimeInput = ref('')

/** create 模式：双向绑定的启用状态 */
const createEnabled = computed({
  get: () => props.modelValue.enabled,
  set: (v: boolean) => {
    emit('update:modelValue', { ...props.modelValue, enabled: v })
  },
})

/** create 模式：双向绑定的设备 ID 数组 */
const createDeviceIds = computed({
  get: () => props.modelValue.deviceIds,
  set: (v: string[]) => {
    emit('update:modelValue', { ...props.modelValue, deviceIds: v })
  },
})

/** runtime 模式：使用 store 中的状态 */
const runtimeEnabled = computed(() => store.roomState.whitelistEnabled)
const runtimeEntries = computed(() => store.whitelistEntries)
const loading = computed(() => store.whitelistLoading)

/** 当前展示的设备 ID 列表（按模式区分） */
const displayDeviceIds = computed(() => {
  if (props.mode === 'create') return createDeviceIds.value
  return runtimeEntries.value.map((e) => e.deviceId)
})

/**
 * 展示用条目列表
 *
 * - `raw`：完整设备 ID（含 `mcsdk-` 前缀），用于内部存储与后端交互
 * - `display`：去前缀的设备 ID，用于 UI 展示
 */
const displayEntries = computed(() =>
  displayDeviceIds.value.map((id) => ({ raw: id, display: stripMcsdkPrefix(id) })),
)

/** 当前是否启用白名单 */
const isEnabled = computed(() => {
  if (props.mode === 'create') return createEnabled.value
  return runtimeEnabled.value
})

/** 是否展示"启用且为空"的警告（拒绝所有人加入） */
const showEmptyWarning = computed(
  () => isEnabled.value && displayEntries.value.length === 0,
)

/** 校验设备 ID 格式（mcsdk-xxxx-xxxx-xxxx-xxxx 或非空字符串） */
function validateDeviceId(id: string): boolean {
  const trimmed = id.trim()
  if (!trimmed) return false
  return true
}

/** create 模式：添加本地设备 ID（自动补全 mcsdk- 前缀） */
function handleCreateAdd() {
  const trimmed = createInput.value.trim()
  if (!validateDeviceId(trimmed)) {
    toastError('设备 ID 不能为空')
    return
  }
  const fullId = ensureMcsdkPrefix(trimmed)
  if (createDeviceIds.value.includes(fullId)) {
    toastError(`设备 ID 已存在：${stripMcsdkPrefix(fullId)}`)
    return
  }
  createDeviceIds.value = [...createDeviceIds.value, fullId]
  createInput.value = ''
}

/** create 模式：移除本地设备 ID */
function handleCreateRemove(deviceId: string) {
  createDeviceIds.value = createDeviceIds.value.filter((id) => id !== deviceId)
}

/** runtime 模式：调用后端添加白名单（自动补全 mcsdk- 前缀） */
async function handleRuntimeAdd() {
  const trimmed = runtimeInput.value.trim()
  if (!validateDeviceId(trimmed)) {
    toastError('设备 ID 不能为空')
    return
  }
  const fullId = ensureMcsdkPrefix(trimmed)
  const ok = await store.addWhitelistEntry(fullId)
  if (ok) runtimeInput.value = ''
}

/** runtime 模式：调用后端移除白名单 */
async function handleRuntimeRemove(deviceId: string) {
  await store.removeWhitelistEntry(deviceId)
}

/** runtime 模式：切换启用状态 */
async function handleRuntimeToggle(enabled: boolean) {
  await store.updateWhitelistEnabled(enabled)
}

/** 切换启用状态（按模式分发） */
function onToggleEnabled(v: boolean) {
  if (props.mode === 'create') {
    createEnabled.value = v
  } else {
    void handleRuntimeToggle(v)
  }
}

/** 添加按钮处理（按模式分发） */
function onAdd() {
  if (props.mode === 'create') handleCreateAdd()
  else void handleRuntimeAdd()
}

/** 移除按钮处理（按模式分发） */
function onRemove(deviceId: string) {
  if (props.mode === 'create') handleCreateRemove(deviceId)
  else void handleRuntimeRemove(deviceId)
}

/** 当前输入框值（按模式区分） */
const currentInput = computed({
  get: () => (props.mode === 'create' ? createInput.value : runtimeInput.value),
  set: (v: string) => {
    if (props.mode === 'create') createInput.value = v
    else runtimeInput.value = v
  },
})

/** runtime 模式：进入房间后自动拉取一次白名单 */
watch(
  () => [props.mode, props.roomCode] as const,
  ([mode, code]) => {
    if (mode === 'runtime' && code) {
      void store.refreshWhitelist()
    }
  },
  { immediate: true },
)

onMounted(() => {
  if (props.mode === 'runtime' && props.roomCode) {
    void store.refreshWhitelist()
  }
})
</script>

<template>
  <div class="space-y-3">
    <!-- 启用开关 -->
    <div class="flex items-center gap-2 cursor-pointer">
      <Checkbox
        :checked="isEnabled"
        :disabled="mode === 'runtime' && loading"
        @change="onToggleEnabled"
      />
      <ShieldCheckIcon
        class="w-4 h-4"
        :class="isEnabled ? 'text-primary-600' : 'text-gray-400'"
      />
      <span class="text-sm text-gray-800">启用白名单</span>
      <span class="text-xs text-gray-400">
        {{ isEnabled ? '仅白名单内设备可加入' : '允许任何已注册设备加入' }}
      </span>
    </div>

    <!-- 启用且为空的警告 -->
    <div
      v-if="showEmptyWarning"
      class="p-2 bg-yellow-50 rounded text-xs text-yellow-700 flex gap-1.5 items-start"
    >
      <ShieldExclamationIcon class="w-3.5 h-3.5 mt-0.5 shrink-0" />
      <span>白名单为空：启用后<b>拒绝所有人加入</b>（仅房主可进入）。请在下方添加设备 ID，或关闭白名单</span>
    </div>

    <!-- 添加输入框 -->
    <div class="flex items-center gap-2">
      <Input
        v-model="currentInput"
        placeholder="设备 ID（如 xxxx-xxxx-xxxx-xxxx）"
        class="font-mono"
        :disabled="mode === 'runtime' && loading"
        @keyup.enter="onAdd"
      />
      <Tooltip text="添加到白名单">
        <Button
          type="primary"
          size="small"
          :disabled="mode === 'runtime' && loading"
          @click="onAdd"
        >
          <template #icon><PlusIcon class="w-3.5 h-3.5" /></template>
          添加
        </Button>
      </Tooltip>
    </div>

    <!-- 白名单列表 -->
    <div v-if="displayEntries.length > 0" class="space-y-1.5">
      <div class="text-xs font-medium text-gray-600 flex items-center gap-1.5">
        <UserPlusIcon class="w-3.5 h-3.5" />
        <span>已添加 {{ displayEntries.length }} 个设备</span>
      </div>
      <div class="space-y-1 max-h-40 overflow-y-auto">
        <div
          v-for="entry in displayEntries"
          :key="entry.raw"
          class="flex items-center justify-between px-3 py-1.5 bg-gray-50 rounded"
        >
          <code class="text-xs text-gray-900 truncate">{{ entry.display }}</code>
          <Tooltip text="移除">
            <Button
              type="ghost"
              size="mini"
              class="!h-6 !w-6 !p-0 text-gray-400 hover:!text-red-500 shrink-0 ml-2"
              :disabled="mode === 'runtime' && loading"
              @click="onRemove(entry.raw)"
            >
              <TrashIcon class="w-3.5 h-3.5" />
            </Button>
          </Tooltip>
        </div>
      </div>
    </div>

    <!-- 空状态提示（icon + text 垂直水平居中） -->
    <div
      v-else
      class="py-6 flex flex-col items-center justify-center gap-2 text-gray-400"
    >
      <UserPlusIcon class="w-6 h-6" />
      <span class="text-xs">尚未添加任何设备 ID</span>
    </div>
  </div>
</template>
