<script setup lang="ts">
/**
 * 通用导航侧边栏组件
 *
 * 功能：
 * - 渲染分类菜单（图标 + 标签），选中态高亮
 * - 支持二级子菜单（children 字段，可选）：父项点击展开/收起，子项点击切换
 * - 子菜单展开/收起带动画（grid-template-rows 0fr→1fr + 图标旋转 + 透明度）
 * - 切换菜单时更新 URL query 的 `tab` 参数（router.replace，不产生历史记录）
 * - 初始化时从 URL query `tab` 恢复选中项（刷新页面保留打开的分类）
 *
 * 用法：
 *   <NavSidebar v-model="activeCategory" :categories="categories" />
 *
 * 路由同步说明：
 * - 切换菜单 → router.replace({ query: { ...route.query, tab: id } })
 * - 页面加载 → 读取 route.query.tab，若有效则 emit('update:modelValue', tab)
 * - 保留其他 query 参数（如 VersionSettings 的 id），不冲突
 *
 * 向后兼容：categories 项的 children 字段可选，未传时行为与原版一致（无子菜单）
 */
import { ref, watch, computed, type Component } from 'vue'
import { LockClosedIcon, ChevronDownIcon } from '@heroicons/vue/24/outline'
import { useTabPersistence } from '@/composables/useTabPersistence'
import { useCollapseAnimation } from '@/composables/useCollapseAnimation'

interface NavCategory {
  id: string
  label: string
  icon: Component
  /** 可选图标图片地址（传入时优先渲染 <img>，否则渲染 icon 组件） */
  image?: string
  desc?: string
  /** 可选子菜单项（有 children 时父项点击展开/收起，不切换选中态） */
  children?: NavCategory[]
  /** 可选禁用态：灰色不可点击，用于「房间详情」等需要前置条件的菜单项 */
  disabled?: boolean
  /** 可选封禁态：灰色置灰但点击仍 emit，由父组件拦截弹窗提示原因（区别于 disabled：disabled 完全不响应点击） */
  sealed?: boolean
  /** 可选分组标题（如「社区资源」）：与上一项 group 不同时渲染分组标题 + 分隔线 */
  group?: string
}

const props = defineProps<{
  modelValue: string
  categories: NavCategory[]
}>()

const emit = defineEmits<{
  (e: 'update:modelValue', id: string): void
}>()

/** 展开状态：按父项 id 记录 */
const expandedMap = ref<Record<string, boolean>>({})

/** 所有子项的 id → 父项 id 映射（用于根据 modelValue 自动展开父项） */
const childToParent = computed<Record<string, string>>(() => {
  const map: Record<string, string> = {}
  for (const cat of props.categories) {
    if (cat.children) {
      for (const child of cat.children) {
        map[child.id] = cat.id
      }
    }
  }
  return map
})

/** 判断分类是否展开 */
function isExpanded(id: string): boolean {
  return !!expandedMap.value[id]
}

/** 切换展开状态 */
function toggleExpand(id: string) {
  expandedMap.value = { ...expandedMap.value, [id]: !expandedMap.value[id] }
}

// 折叠动画 class（v-for 外部状态场景，用纯函数；200ms 过渡 + 内容区 opacity 渐隐）
const { contentClassOf, iconClassOf } = useCollapseAnimation({
  contentTransition: 'transition-all duration-200 ease-out',
  iconTransition: 'transition-transform duration-200',
  expandedExtra: 'opacity-100',
  collapsedExtra: 'opacity-0',
})

/** 父项点击：disabled 不响应；sealed 封禁交给父组件拦截提示（不展开子菜单）；有 children 则 toggle 展开，无 children 则 emit id */
function handleClick(cat: NavCategory) {
  if (cat.disabled) return
  if (cat.sealed) {
    emit('update:modelValue', cat.id)
    return
  }
  if (cat.children && cat.children.length > 0) {
    toggleExpand(cat.id)
  } else {
    emit('update:modelValue', cat.id)
  }
}

/** 判断父项是否高亮（自身选中或子项选中） */
function isParentActive(cat: NavCategory): boolean {
  if (props.modelValue === cat.id) return true
  if (cat.children) {
    return cat.children.some(c => c.id === props.modelValue)
  }
  return false
}

// modelValue 变化时：如果是某父项的子项，自动展开该父项
watch(() => props.modelValue, (val) => {
  const parentId = childToParent.value[val]
  if (parentId && !isExpanded(parentId)) {
    expandedMap.value = { ...expandedMap.value, [parentId]: true }
  }
}, { immediate: true })

// tab 选中态 URL 持久化（onMounted 恢复 + watch 写入，逻辑抽取到 useTabPersistence）
useTabPersistence(
  () => props.modelValue,
  (tab) => {
    // 校验 tab 是否在 categories 中（含 children）且未禁用/未封禁
    for (const cat of props.categories) {
      if (cat.id === tab && !cat.disabled && !cat.sealed) return true
      if (cat.children) {
        for (const child of cat.children) {
          if (child.id === tab && !child.disabled && !child.sealed) return true
        }
      }
    }
    return false
  },
  (tab) => emit('update:modelValue', tab),
)
</script>

<template>
  <aside class="w-48 bg-white border-r border-gray-200 flex flex-col shrink-0">
    <div data-inner-scroll class="flex-1 overflow-y-auto py-4">
      <template v-for="(cat, idx) in categories" :key="cat.id">
        <!-- 分组标题：group 非空且与上一项不同时渲染（分隔线 + 标题） -->
        <div v-if="cat.group && cat.group !== categories[idx - 1]?.group">
          <div v-if="idx > 0" class="my-2 mx-4 border-t border-gray-200"></div>
          <div class="px-4 py-1 text-[11px] font-semibold text-gray-400 uppercase tracking-wide">{{ cat.group }}</div>
        </div>
        <!-- 保留原生 button：导航菜单项（w-full 布局 + active 状态 + 图标 + 展开箭头），
             Button.vue 的 scoped size 类固定 height/padding 无法承载列表项布局 -->
        <!-- 父项（无 children 时为普通项，有 children 时为可展开项） -->
        <button
          type="button"
          class="w-full flex items-center py-2.5 text-sm font-medium transition-colors"
          :class="[
            cat.group ? 'pl-8 pr-4' : 'px-4',
            cat.disabled || cat.sealed
              ? 'text-gray-300 cursor-not-allowed'
              : isParentActive(cat)
                ? 'bg-primary-50 text-primary-700 border-r-2 border-primary-500 cursor-pointer'
                : 'text-gray-700 hover:bg-gray-50 cursor-pointer',
          ]"
          @click="handleClick(cat)"
        >
          <img v-if="cat.image" :src="cat.image" class="w-5 h-5 mr-3 shrink-0 object-contain" alt="" />
          <component :is="cat.icon" v-else class="w-5 h-5 mr-3 shrink-0" />
          <span class="flex-1 text-left">{{ cat.label }}</span>
          <!-- 展开图标（仅有 children 且非封禁时显示，带旋转动画）；封禁项显示锁图标 -->
          <LockClosedIcon v-if="cat.sealed" class="w-4 h-4 text-gray-300 shrink-0" />
          <ChevronDownIcon
            v-else-if="cat.children && cat.children.length > 0"
            class="w-4 h-4 text-gray-400"
            :class="iconClassOf(isExpanded(cat.id))"
          />
        </button>

        <!-- 子菜单（grid-template-rows 动画：0fr → 1fr） -->
        <div
          v-if="cat.children && cat.children.length > 0"
          :class="contentClassOf(isExpanded(cat.id))"
        >
          <div class="overflow-hidden">
            <button
              v-for="child in cat.children"
              :key="child.id"
              type="button"
              class="w-full flex items-center pl-11 pr-4 py-2 text-sm transition-colors"
              :class="[
                child.disabled || child.sealed
                  ? 'text-gray-300 cursor-not-allowed'
                  : modelValue === child.id
                    ? 'text-primary-700 bg-primary-50/50 font-medium cursor-pointer'
                    : 'text-gray-600 hover:bg-gray-50 cursor-pointer',
              ]"
              @click="child.disabled ? null : emit('update:modelValue', child.id)"
            >
              <component :is="child.icon" class="w-4 h-4 mr-2.5 shrink-0" />
              {{ child.label }}
              <LockClosedIcon v-if="child.sealed" class="w-3.5 h-3.5 ml-auto text-gray-300 shrink-0" />
            </button>
          </div>
        </div>
      </template>
    </div>
    <!-- 底部插槽（如「打开游戏目录」按钮） -->
    <slot />
  </aside>
</template>
