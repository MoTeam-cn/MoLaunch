<script setup lang="ts">
/**
 * 社区资源配置卡片
 *
 * 提供 4 项配置：
 * - 来源策略：尽量镜像 / 缓慢时换镜像 / 尽量官方
 * - 文件名格式：5 种译名与原名的组合方式
 * - Mod 管理样式：标题/详情显示译名或文件名
 * - 忽略 Quilt：在显示 Mod 加载器时是否过滤 Quilt
 */
import { ref, watch } from 'vue'
import { useConfigPage } from '@/composables/useConfigPage'
import Select from '@/components/common/Select.vue'
import Button from '@/components/common/Button.vue'

const source = ref<number>(2)
const filenameFormat = ref<number>(1)
const modLocalNameStyle = ref<number>(0)
const ignoreQuilt = ref<boolean>(true)

const { loaded, markDirty } = useConfigPage({
  delay: 800,
  errorLabel: 'save community config',
  onLoad: (cfg) => {
    source.value = cfg.communitySource
    filenameFormat.value = cfg.communityFilenameFormat
    modLocalNameStyle.value = cfg.communityModLocalNameStyle
    ignoreQuilt.value = cfg.communityIgnoreQuilt
  },
})

watch(source, (v) => markDirty('communitySource', v))
watch(filenameFormat, (v) => markDirty('communityFilenameFormat', v))
watch(modLocalNameStyle, (v) => markDirty('communityModLocalNameStyle', v))
watch(ignoreQuilt, (v) => markDirty('communityIgnoreQuilt', v))

const sourceOptions = [
  { value: 0, label: '尽量镜像', desc: '使用镜像源，可能缺少刚刚更新的版本' },
  { value: 1, label: '缓慢时换镜像', desc: '优先官方源，加载缓慢或失败时改用镜像' },
  { value: 2, label: '尽量官方', desc: '使用官方源，速度可能较慢但数据最新' },
]

const filenameOptions = [
  { value: 0, label: '【机械动力】create-1.21.1' },
  { value: 1, label: '[机械动力] create-1.21.1' },
  { value: 2, label: '机械动力-create-1.21.1' },
  { value: 3, label: 'create-1.21.1-机械动力' },
  { value: 4, label: 'create-1.21.1' },
]
</script>

<template>
  <div class="space-y-6">
    <!-- 加载占位（避免初始值与实际值不一致导致的闪烁） -->
    <div v-if="!loaded" class="bg-white rounded-lg border border-gray-300 overflow-hidden">
      <div class="px-5 py-5">
        <div class="h-4 w-24 bg-gray-200 rounded animate-pulse mb-4" />
        <div class="h-10 bg-gray-100 rounded animate-pulse" />
      </div>
    </div>

    <template v-else>
      <!-- 社区资源 -->
      <div class="bg-white rounded-lg border border-gray-300 overflow-hidden">
        <h3 class="text-sm font-semibold text-gray-900 px-5 pt-5 pb-3">社区资源</h3>

        <div class="divide-y divide-gray-200">
          <!-- 来源 -->
          <div class="px-5 py-4">
            <div class="flex items-center justify-between mb-2">
              <div>
                <p class="text-sm font-medium text-gray-900">来源</p>
                <p class="text-xs text-gray-500 mt-0.5">CurseForge 与 Modrinth 资源请求的源策略</p>
              </div>
              <div class="w-72">
                <Select v-model="source" :options="sourceOptions" custom-option>
                  <template #option="{ option }">
                    <div class="flex flex-col min-w-0 flex-1">
                      <span class="text-sm font-medium text-gray-900">{{ option.label }}</span>
                      <span class="text-xs text-gray-400">{{ option.desc }}</span>
                    </div>
                    <svg
                      v-if="option.value === source"
                      class="w-3 h-3 text-blue-600 shrink-0 mt-0.5"
                      viewBox="0 0 1024 1024"
                      fill="currentColor"
                    >
                      <path d="M912 192c-12.8 0-25.6 4.266667-34.133333 12.8L384 699.2 234.666667 548.266667c-17.066667-17.066667-46.933333-17.066667-64 0-17.066667 17.066667-17.066667 46.933333 0 64l179.2 179.2c8.533333 8.533333 21.333333 12.8 34.133333 12.8s25.6-4.266667 34.133333-12.8l520.533334-520.533334c17.066667-17.066667 17.066667-46.933333 0-64-8.533333-8.533333-21.333333-12.8-34.133334-12.8z" />
                    </svg>
                  </template>
                </Select>
              </div>
            </div>
          </div>

          <!-- 文件名格式 -->
          <div class="px-5 py-4">
            <div class="flex items-center justify-between mb-2">
              <div>
                <p class="text-sm font-medium text-gray-900">文件名格式</p>
                <p class="text-xs text-gray-500 mt-0.5">下载社区资源时，默认文件名的格式</p>
              </div>
              <div class="w-72">
                <Select v-model="filenameFormat" :options="filenameOptions" />
              </div>
            </div>
          </div>

          <!-- Mod 管理样式 -->
          <div class="px-5 py-4">
            <div class="flex items-center justify-between mb-2">
              <div>
                <p class="text-sm font-medium text-gray-900">Mod 管理样式</p>
                <p class="text-xs text-gray-500 mt-0.5">在 Mod 管理页面中，Mod 项的显示方式</p>
              </div>
            </div>
            <div class="flex gap-2">
              <Button
                :type="modLocalNameStyle === 0 ? 'primary' : 'outline'"
                size="small"
                class="flex-1"
                @click="modLocalNameStyle = 0"
              >
                标题显示译名，详情显示文件名
              </Button>
              <Button
                :type="modLocalNameStyle === 1 ? 'primary' : 'outline'"
                size="small"
                class="flex-1"
                @click="modLocalNameStyle = 1"
              >
                标题显示文件名，详情显示译名
              </Button>
            </div>
          </div>

          <!-- 忽略 Quilt -->
          <div class="px-5 py-4">
            <div class="flex items-center justify-between mb-2">
              <div>
                <p class="text-sm font-medium text-gray-900">忽略 Quilt 加载器</p>
                <p class="text-xs text-gray-500 mt-0.5">在显示 Mod 加载器时忽略 Quilt</p>
              </div>
            </div>
            <div class="flex gap-2">
              <Button
                :type="ignoreQuilt ? 'primary' : 'outline'"
                size="small"
                class="flex-1"
                @click="ignoreQuilt = true"
              >
                已启用
              </Button>
              <Button
                :type="!ignoreQuilt ? 'primary' : 'outline'"
                size="small"
                class="flex-1"
                @click="ignoreQuilt = false"
              >
                未启用
              </Button>
            </div>
            <p class="text-xs text-gray-400 mt-2">
              <template v-if="ignoreQuilt">已启用：搜索和显示 Mod 加载器时将过滤 Quilt</template>
              <template v-else>未启用：Quilt 加载器将与 Forge/Fabric 一起显示</template>
            </p>
          </div>
        </div>
      </div>
    </template>
  </div>
</template>
