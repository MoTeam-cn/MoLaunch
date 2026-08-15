<script setup lang="ts">
/** 创建房间表单：MC 版本 + 房间设置 + 整合包关联（Scaffolding 收敛版，一站式创建） */
import { ref, computed, watch, defineAsyncComponent } from 'vue'
import { PlusIcon, ArrowPathIcon, CheckCircleIcon } from '@heroicons/vue/24/outline'
import { useCreateRoomForm } from '@/composables/useCreateRoomForm'
const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))
const Card = defineAsyncComponent(() => import('@/components/common/Card.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const Drawer = defineAsyncComponent(() => import('@/components/common/Drawer.vue'))
const AlertV2 = defineAsyncComponent(() => import('@/components/common/AlertV2.vue'))
const ModpackSelector = defineAsyncComponent(() => import('./ModpackSelector.vue'))
const ModpackRequirementCard = defineAsyncComponent(() => import('./ModpackRequirementCard.vue'))

const {
  store,
  createForm,
  creating,
  createSteps,
  createStep,
  modpackMeta,
  onModpackEnabledChange,
  publicRoomHint,
  versionOptions,
  versionsLoading,
  onVersionSelect,
  handleCreateRoom,
} = useCreateRoomForm()

/** 创建进度抽屉开关：提交创建时自动弹出，完成/失败自动收起 */
const createProgressOpen = ref(false)
watch(
  () => createStep.value,
  (step) => {
    createProgressOpen.value = step !== 'idle'
  },
)
const currentStepIndex = computed(() => createSteps.findIndex((s) => s.key === createStep.value))
</script>

<template>
  <div class="space-y-4">
    <AlertV2 type="info" message="联机基于 easytier 虚拟局域网（Scaffolding）：房主需先在游戏中开启「对局域网开放」，再填写下方信息创建房间" />
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
          <Input v-model.number="createForm.mcPort" type="number" placeholder="25565" />
        </div>
        <!-- 房间备注 -->
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">房间备注</label>
          <Input v-model="createForm.remark" placeholder="例如：开黑速通，随到随玩（选填）" maxlength="40" />
        </div>
        <!-- 房间类型：公开 / 私密 -->
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">房间类型</label>
          <div class="flex-1 space-y-1">
            <div class="flex gap-2">
              <Button
                :type="createForm.isPublic ? 'primary' : 'outline'"
                long
                size="small"
                @click="createForm.isPublic = true"
              >
                公开
              </Button>
              <Button
                :type="!createForm.isPublic ? 'primary' : 'outline'"
                long
                size="small"
                @click="createForm.isPublic = false"
              >
                私密
              </Button>
            </div>
            <p class="text-xs text-gray-500">{{ publicRoomHint }}</p>
          </div>
        </div>
        <!-- 房间密码 -->
        <div class="flex items-center gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0">房间密码</label>
          <Input v-model="createForm.password" placeholder="留空表示无密码" />
        </div>
        <!-- 整合包关联 -->
        <div class="flex items-start gap-3">
          <label class="w-24 text-xs text-gray-600 shrink-0 pt-2">整合包</label>
          <div class="flex-1 space-y-2">
            <ModpackSelector
              v-model="modpackMeta"
              :version-id="createForm.selectedVersionId"
              @enabled-change="onModpackEnabledChange"
            />
            <ModpackRequirementCard v-if="modpackMeta" :modpack="modpackMeta" />
          </div>
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
