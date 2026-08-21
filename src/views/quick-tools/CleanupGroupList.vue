<script setup lang="ts">
/**
 * 清理垃圾 - 分组列表子组件
 *
 * 按分组（全局 / 各版本）以文件树形式展示扫描结果，每组可折叠、可全选。
 * 单项可勾选，路径用 Tooltip 组件展示完整内容。
 *
 * 分组规则：display_name 形如 "游戏日志 - 1.19.2" 归入版本 "1.19.2"，
 * 无 " - " 后缀的归入 "全局" 分组。分组计算内聚于此组件，父级只传原始 items。
 */
import { computed, defineAsyncComponent } from 'vue'
import { ChevronDownIcon } from '@heroicons/vue/24/outline'
const Tooltip = defineAsyncComponent(() => import('@/components/common/Tooltip.vue'))
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Checkbox = defineAsyncComponent(() => import('@/components/common/Checkbox.vue'))
import type { CleanupItem } from '@/utils/api/tools'
import { formatBytes } from '@/utils/format'
import { groupIcon } from '@/utils/cleanup-group'
import { useCollapseAnimation } from '@/composables/useCollapseAnimation'

interface CleanupGroup {
  key: string
  label: string
  items: CleanupItem[]
  groupSize: number
  groupFileCount: number
  selectedCount: number
  selectedSize: number
}

interface Props {
  items: CleanupItem[]
  selectedPaths: Set<string>
  collapsedGroups: Set<string>
}

const props = defineProps<Props>()

const emit = defineEmits<{
  (e: 'toggleSelect', path: string): void
  (e: 'toggleGroup', key: string): void
  (e: 'toggleGroupSelect', groupKey: string): void
}>()

const groups = computed<CleanupGroup[]>(() => {
  const map = new Map<string, CleanupItem[]>()
  for (const item of props.items) {
    const dashIdx = item.display_name.indexOf(' - ')
    const groupKey = dashIdx > 0 ? item.display_name.substring(dashIdx + 3) : '全局'
    if (!map.has(groupKey)) map.set(groupKey, [])
    map.get(groupKey)!.push(item)
  }

  const result: CleanupGroup[] = []
  if (map.has('全局')) {
    result.push(buildGroup('全局', map.get('全局')!))
    map.delete('全局')
  }
  for (const key of Array.from(map.keys()).sort()) {
    result.push(buildGroup(key, map.get(key)!))
  }
  return result
})

function buildGroup(key: string, items: CleanupItem[]): CleanupGroup {
  let groupSize = 0
  let groupFileCount = 0
  let selectedCount = 0
  let selectedSize = 0
  for (const item of items) {
    groupSize += item.size
    groupFileCount += item.file_count
    if (props.selectedPaths.has(item.path)) {
      selectedCount++
      selectedSize += item.size
    }
  }
  return { key, label: key, items, groupSize, groupFileCount, selectedCount, selectedSize }
}

function itemDisplayName(displayName: string): string {
  const idx = displayName.indexOf(' - ')
  return idx > 0 ? displayName.substring(0, idx) : displayName
}

// 折叠动画 class（v-for 外部状态场景，用纯函数；折叠时图标 -rotate-90、展开不旋转）
const { contentClassOf, iconClassOf } = useCollapseAnimation({
  iconTransition: 'transition-transform duration-300',
  rotateClass: '',
  collapsedRotateClass: '-rotate-90',
})
</script>

<template>
  <div class="space-y-2 py-1">
    <div
      v-for="group in groups"
      :key="group.key"
      class="rounded-lg border border-gray-200 overflow-hidden"
    >
      <!-- 分组标题：整行点击展开/折叠，右侧独立全选按钮 -->
      <div
        class="flex items-center gap-2 bg-gray-50 px-3 py-2 cursor-pointer select-none hover:bg-gray-100"
        @click="emit('toggleGroup', group.key)"
      >
        <ChevronDownIcon
          class="h-4 w-4 flex-none text-gray-400"
          :class="iconClassOf(!collapsedGroups.has(group.key))"
        />
        <component :is="groupIcon(group.key)" class="h-4 w-4 flex-none text-primary-500" />
        <div class="flex-1 min-w-0">
          <div class="flex items-center gap-2">
            <span class="text-sm font-medium text-gray-800 truncate">{{ group.label }}</span>
            <span class="text-xs text-gray-400">
              {{ formatBytes(group.groupSize) }} · {{ group.groupFileCount }} 个文件
            </span>
          </div>
        </div>
        <!-- 组内全选切换：Checkbox 半选表示部分选中，右侧徽章显示已选/总数 -->
        <div class="flex flex-none items-center gap-1.5" @click.stop>
          <Checkbox
            :checked="group.items.length > 0 && group.selectedCount === group.items.length"
            :indeterminate="group.selectedCount > 0 && group.selectedCount < group.items.length"
            @change="emit('toggleGroupSelect', group.key)"
          />
          <span class="text-xs text-gray-500">{{ group.selectedCount }}/{{ group.items.length }}</span>
        </div>
      </div>

      <!-- 分组内容（折叠动画） -->
      <div
        :class="contentClassOf(!collapsedGroups.has(group.key))"
      >
        <div class="overflow-hidden">
          <div class="divide-y divide-gray-100">
            <div
              v-for="item in group.items"
              :key="item.path"
              class="flex items-center gap-3 px-3 py-2.5 transition-colors cursor-pointer"
              :class="
                selectedPaths.has(item.path)
                  ? 'bg-primary-50/60'
                  : 'bg-white hover:bg-gray-50'
              "
              @click="emit('toggleSelect', item.path)"
            >
              <!-- 复选框 -->
              <div class="flex-none" @click.stop>
                <Checkbox
                  :checked="selectedPaths.has(item.path)"
                  @change="emit('toggleSelect', item.path)"
                />
              </div>
              <!-- 名称 + 类别 -->
              <div class="min-w-0 flex-1">
                <div class="flex items-center gap-2">
                  <span class="text-sm font-medium text-gray-900">
                    {{ itemDisplayName(item.display_name) }}
                  </span>
                  <Tag
                    size="small"
                    :color="item.category === '可选' ? 'gold' : 'arcoblue'"
                  >
                    {{ item.category }}
                  </Tag>
                </div>
                <!-- 路径：用 Tooltip 组件展示完整路径，避免原生 title -->
                <!-- block prop 让 trigger 撑满父容器宽度，内部 truncate 才能在 flex 布局下生效 -->
                <Tooltip :text="item.path" position="top" :delay="200" block>
                  <div class="mt-0.5 truncate text-xs text-gray-400">{{ item.path }}</div>
                </Tooltip>
              </div>
              <!-- 大小 + 文件数 -->
              <div class="flex-none text-right">
                <div class="text-sm font-medium text-gray-700">{{ formatBytes(item.size) }}</div>
                <div class="text-xs text-gray-400">{{ item.file_count }} 个文件</div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
