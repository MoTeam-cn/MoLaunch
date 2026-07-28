<script setup lang="ts">
/**
 * 创建房间表单（从 RoomManager.vue 拆分）
 *
 * 改造要点：
 * - MC 版本从手打 Input 改为 Select 下拉，数据源 `listInstalledVersionsWithType`
 *   选择后用 version_id 作为 mcVersion 上报（含 loader 信息，比单纯版本号更有意义）
 * - 白名单从平铺改为 CollapsibleCard「高级设置」折叠，默认收起
 * - 布局美化：字段标签加宽 w-24、间距 space-y-4、白名单独立成卡
 */
import { ref, computed, onMounted } from 'vue'
import { useOnlineStore } from '@/stores/online'
import Button from '@/components/common/Button.vue'
import Card from '@/components/common/Card.vue'
import Input from '@/components/common/Input.vue'
import Select from '@/components/common/Select.vue'
import CollapsibleCard from '@/components/common/CollapsibleCard.vue'
import WhitelistEditor from './WhitelistEditor.vue'
import { listInstalledVersionsWithType, type InstalledVersionInfo } from '@/utils/api/version'
import {
  PlusIcon,
  ArrowPathIcon,
  CheckCircleIcon,
} from '@heroicons/vue/24/outline'
import { toastError } from '@/utils/toast'

const store = useOnlineStore()

/** 创建房间表单 */
const createForm = ref({
  maxPlayers: 4,
  password: '',
  mcVersion: '',
  mcPort: 25565,
  selectedVersionId: '',
})

/** 已安装版本列表（用于 MC 版本下拉选择） */
const installedVersions = ref<InstalledVersionInfo[]>([])
const versionOptions = computed(() =>
  installedVersions.value.map((v) => ({ label: v.id, value: v.id })),
)
const versionsLoading = ref(false)

onMounted(async () => {
  versionsLoading.value = true
  try {
    installedVersions.value = await listInstalledVersionsWithType()
  } catch (e) {
    console.error('Failed to load installed versions:', e)
  } finally {
    versionsLoading.value = false
  }
})

/** 选择已安装版本后自动填充 mcVersion（用 version_id 上报，含 loader 信息） */
function onVersionSelect(value: string | number) {
  createForm.value.selectedVersionId = String(value)
  createForm.value.mcVersion = String(value)
}

/** 最大人数提示：mesh 拓扑 5+ 人带宽压力陡增，超限显示 error */
const maxPlayersHint = computed(() => {
  const v = createForm.value.maxPlayers
  if (v < 2) return '至少需要 2 人（房主 + 1 参与者）'
  if (v > 5) return 'mesh 模式不建议超过 5 人，请使用专业服务器'
  return 'mesh 模式建议 2-5 人，超过请使用专业服务器'
})
const maxPlayersHintType = computed<'default' | 'error' | 'success'>(() => {
  const v = createForm.value.maxPlayers
  if (v < 2 || v > 5) return 'error'
  return 'default'
})

/** 白名单表单状态 */
const whitelistForm = ref({ enabled: false, deviceIds: [] as string[] })

/** 创建房间步骤指示器（mesh 拓扑两步：stun → create） */
const createSteps = [
  { key: 'stun' as const, label: '获取 STUN 服务器' },
  { key: 'create' as const, label: '创建房间' },
]
const stepOrder = ['stun', 'create'] as const
const currentStepIndex = computed(() => {
  if (!store.roomCreateStep) return -1
  return stepOrder.indexOf(store.roomCreateStep)
})

/** 房主创建房间（mesh 拓扑：不生成本地 Offer，参与者加入后再 per-participant 生成） */
async function handleCreateRoom() {
  if (!createForm.value.mcVersion) {
    toastError('请选择 MC 版本：创建房间前需指明房主的 Minecraft 版本')
    return
  }
  if (createForm.value.mcPort <= 0 || createForm.value.mcPort > 65535) {
    toastError('MC 端口无效：端口范围 1-65535')
    return
  }
  if (createForm.value.maxPlayers < 2 || createForm.value.maxPlayers > 5) {
    toastError('人数无效：mesh 模式最大人数范围为 2-5')
    return
  }

  try {
    store.roomCreateStep = 'stun'
    const stun = await store.fetchStunServers()

    store.roomCreateStep = 'create'
    await store.hostCreateRoom(
      '',
      [],
      createForm.value.maxPlayers,
      createForm.value.password,
      createForm.value.mcVersion,
      createForm.value.mcPort,
      stun,
      whitelistForm.value.enabled,
      whitelistForm.value.deviceIds,
    )
  } catch (e) {
    toastError(`创建房间失败：${e instanceof Error ? e.message : String(e)}`)
  } finally {
    store.roomCreateStep = null
  }
}
</script>

<template>
  <div class="space-y-4">
    <!-- 基础信息卡片 -->
    <Card title="创建房间">
      <div class="space-y-4 py-1">
        <!-- MC 版本：下拉选择已安装版本 -->
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">MC 版本</label>
          <Select
            :model-value="createForm.selectedVersionId"
            :options="versionOptions"
            placeholder="选择已安装的版本"
            class="flex-1"
            :disabled="versionsLoading"
            @update:model-value="onVersionSelect"
          />
        </div>
        <!-- MC 端口 -->
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">MC 端口</label>
          <Input v-model="createForm.mcPort" type="number" placeholder="25565" />
        </div>
        <!-- 最大人数 -->
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">最大人数</label>
          <Input
            v-model="createForm.maxPlayers"
            type="number"
            placeholder="4"
            :hint="maxPlayersHint"
            :hint-type="maxPlayersHintType"
          />
        </div>
        <!-- 房间密码 -->
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">房间密码</label>
          <Input v-model="createForm.password" placeholder="留空表示无密码" />
        </div>
        <!-- 创建按钮 -->
        <div class="pt-2">
          <Button type="primary" long :loading="store.roomLoading" @click="handleCreateRoom">
            <template #icon><PlusIcon class="w-4 h-4" /></template>
            创建房间
          </Button>
          <!-- 创建进度反馈：两步指示器 -->
          <div
            v-if="store.roomCreateStep"
            class="mt-2 space-y-1.5 px-3 py-2.5 bg-primary-50/50 rounded-lg border border-primary-100"
          >
            <div
              v-for="(step, idx) in createSteps"
              :key="step.key"
              class="flex items-center gap-2 text-xs transition-colors"
              :class="[
                idx === currentStepIndex
                  ? 'text-primary-700 font-medium'
                  : idx < currentStepIndex
                    ? 'text-green-600'
                    : 'text-gray-400',
              ]"
            >
              <ArrowPathIcon
                v-if="idx === currentStepIndex"
                class="w-3.5 h-3.5 animate-spin"
              />
              <CheckCircleIcon
                v-else-if="idx < currentStepIndex"
                class="w-3.5 h-3.5"
              />
              <span
                v-else
                class="w-3.5 h-3.5 rounded-full border border-current opacity-40"
              />
              <span>{{ step.label }}</span>
            </div>
          </div>
        </div>
      </div>
    </Card>

    <!-- 高级设置：白名单（默认收起，点击展开） -->
    <CollapsibleCard :default-open="false">
      <template #title>
        <div class="flex items-center gap-2">
          <span>高级设置</span>
          <span
            v-if="whitelistForm.enabled"
            class="px-1.5 py-0.5 text-xs rounded bg-primary-100 text-primary-700"
          >
            白名单已启用
          </span>
          <span
            v-else
            class="px-1.5 py-0.5 text-xs rounded bg-gray-100 text-gray-500"
          >
            未启用
          </span>
        </div>
      </template>
      <div class="space-y-2">
        <div class="text-xs text-gray-500">白名单管理：启用后仅白名单内设备可加入房间</div>
        <WhitelistEditor v-model="whitelistForm" mode="create" />
      </div>
    </CollapsibleCard>
  </div>
</template>
