<script setup lang="ts">
/**
 * 启动设置页
 * - Java 路径选择（JavaPathSelector 子组件）
 * - 内存分配（MemoryAllocation 子组件，内部复用 useMemoryVisualizer + useConfigPage）
 * - 版本隔离（Select 下拉）
 * - 高级选项（ToggleRow 公共组件 × 3）
 * - 游戏目录（只读展示）
 */
import { ref, watch, defineAsyncComponent } from 'vue'
import { useConfigPage } from '@/composables/useConfigPage'
const Select = defineAsyncComponent(() => import('@/components/common/Select.vue'))
const ToggleRow = defineAsyncComponent(() => import('@/components/settings/ToggleRow.vue'))
const JavaPathSelector = defineAsyncComponent(() => import('./settings-launch/JavaPathSelector.vue'))
const MemoryAllocation = defineAsyncComponent(() => import('./settings-launch/MemoryAllocation.vue'))

const gameDir = ref('')
const isolationMode = ref(4)
// 启动高级选项
const launchDisableJlw = ref(false)
const launchDisableLua = ref(false)
const launchUseDedicatedGpu = ref(false)

const isolationOptions = [
  { label: '关闭', value: 0 },
  { label: '隔离 Mod 版本', value: 1 },
  { label: '隔离非正式版', value: 2 },
  { label: '隔离非正式版 + Mod 版本', value: 3 },
  { label: '隔离所有版本（推荐）', value: 4 },
]

const { loaded, markDirty } = useConfigPage({
  delay: 500,
  errorLabel: 'save launch settings',
  onLoad: async (cfg) => {
    gameDir.value = cfg.gameDir || '.minecraft'
    isolationMode.value = cfg.isolationMode
    launchDisableJlw.value = cfg.launchDisableJlw
    launchDisableLua.value = cfg.launchDisableLua
    launchUseDedicatedGpu.value = cfg.launchUseDedicatedGpu
  },
})

// 版本隔离模式保存
watch(isolationMode, (mode) => {
  if (!loaded.value) return
  markDirty('isolationMode', mode)
})

// 启动高级选项保存（ToggleRow 的 @update:model-value 仅在用户点击时触发，
// loaded 守卫防止初始加载期间误保存）
function saveLaunchSwitch(key: 'launchDisableJlw' | 'launchDisableLua' | 'launchUseDedicatedGpu', v: boolean) {
  if (!loaded.value) return
  markDirty(key, v)
}
</script>

<template>
  <div class="space-y-6">
    <!-- 启动参数 -->
    <div class="bg-white rounded-lg border border-gray-300">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">启动参数</h3>
      <div class="divide-y divide-gray-200">
        <!-- Java 路径（子组件） -->
        <JavaPathSelector />
      </div>
    </div>

    <!-- 内存分配（子组件，内部复用 useMemoryVisualizer + useConfigPage） -->
    <MemoryAllocation />

    <!-- 版本隔离 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">版本隔离</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">隔离模式</p>
            <p class="text-xs text-gray-500 mt-0.5">控制不同版本是否共享存档、Mod、资源包等</p>
          </div>
          <Select
            :model-value="isolationMode"
            :options="isolationOptions"
            class="w-56"
            @update:model-value="isolationMode = $event as number"
          />
        </div>
      </div>
    </div>

    <!-- 高级选项（复用 ToggleRow 公共组件）-->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">高级选项</h3>
      <div class="divide-y divide-gray-200">
        <ToggleRow
          v-model="launchDisableJlw"
          label="禁用 Java Launch Wrapper"
          description="JLW 用于修复 Java 18- 在中文路径下可能无法正常启动的问题，若启动异常可尝试关闭"
          :hover="false"
          @update:model-value="(v) => saveLaunchSwitch('launchDisableJlw', v)"
        />
        <ToggleRow
          v-model="launchDisableLua"
          label="禁用 LWJGL Unsafe Agent"
          description="LUA 用于修复 LWJGL 3.4.1 的性能问题，若游戏卡顿可尝试关闭"
          :hover="false"
          @update:model-value="(v) => saveLaunchSwitch('launchDisableLua', v)"
        />
        <ToggleRow
          v-model="launchUseDedicatedGpu"
          label="使用高性能显卡"
          description="自动在 Windows 设置中将 Java 改为使用独立显卡，提升游戏帧率"
          :hover="false"
          @update:model-value="(v) => saveLaunchSwitch('launchUseDedicatedGpu', v)"
        />
      </div>
    </div>

    <!-- 游戏目录 -->
    <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">游戏目录</h3>
      <div class="divide-y divide-gray-200">
        <div class="px-5 py-4 flex items-center justify-between">
          <div>
            <p class="text-sm font-medium text-gray-900">存储路径</p>
            <p class="text-xs text-gray-500 mt-0.5">Minecraft 游戏数据存放位置（固定）</p>
          </div>
          <div class="flex items-center gap-2">
            <span class="text-sm text-gray-600 max-w-xs truncate">{{ gameDir }}</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
