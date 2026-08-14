<script setup lang="ts">
/** 创建房间表单：MC 版本 Select 下拉 + 高级设置（白名单/整合包关联，置于抽屉内） */
import { ref, watch, defineAsyncComponent } from 'vue'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const WhitelistEditor = defineAsyncComponent(() => import('./WhitelistEditor.vue'))
const ModpackSelector = defineAsyncComponent(() => import('./ModpackSelector.vue'))
import { Cog8ToothIcon, PlusIcon, ArrowPathIcon, CheckCircleIcon } from '@heroicons/vue/24/outline'
import { useCreateRoomForm } from '@/composables/useCreateRoomForm'

const {
  store,
  createForm,
  creating,
  modpackMeta,
  onModpackEnabledChange,
  publicRoomHint,
  versionOptions,
  versionsLoading,
  onVersionSelect,
  maxPlayersHint,
  maxPlayersHintType,
  whitelistForm,
  advancedBadge,
  advancedBadgeActive,
  createSteps,
  currentStepIndex,
  handleCreateRoom,
} = useCreateRoomForm()

/** 高级设置抽屉开关（详情页仅保留入口按钮） */
const advancedDrawerOpen = ref(false)

/** 创建进度抽屉开关：提交创建时自动弹出，完成/失败自动收起 */
const createProgressOpen = ref(false)
watch(
  () => store.roomCreateStep,
  (step) => {
    createProgressOpen.value = step !== null
  },
)
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="P2P联机对房主的网络质量要求较高，如遇连接不上可尝试更换房主" />
    <AlertV2 type="info" message="如遇到违法违规房间，请及时向我们举报" />
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
            allow-clear
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
        <!-- 房间类型：私密 / 公开（联机大厅阶段 2） -->
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">房间类型</label>
          <div class="flex-1 space-y-1">
            <div class="flex gap-2">
              <Button
                :type="createForm.roomType === 'private' ? 'primary' : 'outline'"
                long
                size="small"
                @click="createForm.roomType = 'private'"
              >
                私密
              </Button>
              <Button
                :type="createForm.roomType === 'lobby' ? 'primary' : 'outline'"
                long
                size="small"
                @click="createForm.roomType = 'lobby'"
              >
                公开
              </Button>
            </div>
            <p class="text-xs text-gray-500">{{ publicRoomHint }}</p>
          </div>
        </div>
        <!-- 高级设置入口（内容置于抽屉内，详情页仅保留按钮；未启用时不显示状态徽章） -->
        <div class="pt-2">
          <Button type="outline" long @click="advancedDrawerOpen = true">
            <template #icon><Cog8ToothIcon class="w-4 h-4" /></template>
            <span class="flex items-center justify-center gap-1">
              高级设置
              <Tag v-if="advancedBadgeActive" size="small" color="arcoblue">
                {{ advancedBadge }}
              </Tag>
            </span>
          </Button>
        </div>
        <!-- 创建按钮 -->
        <div class="pt-1">
          <Button type="primary" long :loading="store.roomLoading || creating" :disabled="creating" @click="handleCreateRoom">
            <template #icon><PlusIcon class="w-4 h-4" /></template>
            创建房间
          </Button>
        </div>
      </div>
    </Card>

    <!-- 高级设置抽屉：整合包关联 + 白名单管理 -->
    <Drawer
      v-model:visible="advancedDrawerOpen"
      title="高级设置"
      placement="right"
      :width="420"
      render-in-place
      popup-container="#app-content"
    >
      <div class="space-y-3">
        <!-- 整合包关联（联机大厅阶段 3） -->
        <ModpackSelector
          v-model="modpackMeta"
          :version-id="createForm.selectedVersionId"
          @enabled-change="onModpackEnabledChange"
        />
        <div class="border-t border-gray-100 pt-3">
          <div class="text-xs text-gray-500 mb-2">白名单管理：启用后仅白名单内设备可加入房间</div>
          <WhitelistEditor v-model="whitelistForm" mode="create" />
        </div>
      </div>
    </Drawer>

    <!-- 创建进度抽屉：提交创建时自动弹出，完成/失败自动收起 -->
    <Drawer
      v-model:visible="createProgressOpen"
      title="正在创建房间"
      placement="right"
      :width="320"
      :mask="false"
      :closable="false"
      render-in-place
      popup-container="#app-content"
    >
      <div class="space-y-1.5 py-2">
        <div
          v-for="(step, idx) in createSteps"
          :key="step.key"
          class="flex items-center gap-2 text-xs transition-colors"
          :class="idx === currentStepIndex ? 'text-primary-700 font-medium' : idx < currentStepIndex ? 'text-green-600' : 'text-gray-400'"
        >
          <ArrowPathIcon v-if="idx === currentStepIndex" class="w-3.5 h-3.5 animate-spin" />
          <CheckCircleIcon v-else-if="idx < currentStepIndex" class="w-3.5 h-3.5" />
          <span v-else class="w-3.5 h-3.5 rounded-full border border-current opacity-40" />
          <span>{{ step.label }}</span>
        </div>
      </div>
    </Drawer>
  </div>
</template>
