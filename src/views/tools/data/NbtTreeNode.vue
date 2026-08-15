<script setup lang="ts">
/**
 * NBT 树节点（递归组件，支持编辑）
 *
 * 叶子节点点击值可编辑（输入框），容器节点（compound/list）支持新增子节点，
 * 非根节点可删除。值/增删直接修改响应式 node 对象，删除通过 emit 交由父级定位。
 */
import { computed, ref, defineAsyncComponent } from 'vue'
import {
  ChevronRightIcon,
  ChevronDownIcon,
  PlusIcon,
  TrashIcon,
  CheckIcon,
  XMarkIcon,
} from '@heroicons/vue/24/outline'
const Tag = defineAsyncComponent(() => import('@/components/common/Tag.vue'))
const Input = defineAsyncComponent(() => import('@/components/common/Input.vue'))
import type { NbtNode } from '@/utils/api/tools'

const props = withDefaults(
  defineProps<{
    node: NbtNode
    path: string
    expandedSet: Set<string>
    editable?: boolean
    isRoot?: boolean
  }>(),
  { editable: false, isRoot: false },
)

const emit = defineEmits<{
  toggle: [key: string]
  remove: [node: NbtNode, key: string]
}>()

/** 本地引用：编辑操作修改响应式对象本身（树形编辑的常用做法） */
const node = props.node

const NBT_TYPES = [
  'byte', 'short', 'int', 'long', 'float', 'double',
  'string', 'compound', 'list', 'byte_array', 'int_array', 'long_array',
]

const key = computed(() => props.path + '/' + node.name + ':' + node.tag_type)
const isExpanded = computed(() => props.expandedSet.has(key.value))
const isContainer = computed(() => node.tag_type === 'compound' || node.tag_type === 'list')
const isNumberType = computed(() => ['byte', 'short', 'int', 'long', 'float', 'double'].includes(node.tag_type))
const isArrayType = computed(() => ['byte_array', 'int_array', 'long_array'].includes(node.tag_type))

// 值编辑状态
const editing = ref(false)
const editText = ref('')
// 数组文本编辑
const arrayEditing = ref(false)
const arrayText = ref('')
// 新增子节点（compound 专用）
const addMode = ref(false)
const addName = ref('')
const addType = ref('string')

/** 按内容估算文本像素宽度（中文 14px、其他 7.5px），输入框宽度随内容自适应 */
function textWidth(s: string): number {
  let w = 0
  for (const ch of s) w += ch.charCodeAt(0) > 255 ? 14 : 7.5
  return w
}
const editInputWidth = computed(() => `${Math.min(Math.max(textWidth(editText.value) + 32, 80), 240)}px`)
const arrayInputWidth = computed(() => `${Math.min(Math.max(textWidth(arrayText.value) + 32, 140), 320)}px`)

function toggle() {
  if (isContainer.value) emit('toggle', key.value)
}

function tagColor(tagType: string): string {
  switch (tagType) {
    case 'compound': return 'blue'
    case 'list': return 'purple'
    case 'string': return 'green'
    case 'int':
    case 'short':
    case 'long':
    case 'byte': return 'orange'
    case 'float':
    case 'double': return 'cyan'
    default: return 'gray'
  }
}

function formatValue(node: NbtNode): string {
  if (node.value === null || node.value === undefined) return ''
  if (typeof node.value === 'string') return '"' + node.value + '"'
  if (Array.isArray(node.value)) {
    if (node.value.length <= 8) return '[' + node.value.join(', ') + ']'
    return '[' + node.value.slice(0, 8).join(', ') + ', ... ] (' + node.value.length + ' items)'
  }
  return String(node.value)
}

function defaultValue(type: string): unknown {
  switch (type) {
    case 'byte':
    case 'short':
    case 'int':
    case 'long':
    case 'float':
    case 'double': return 0
    case 'byte_array':
    case 'int_array':
    case 'long_array': return []
    case 'string': return ''
    default: return null
  }
}

function makeNode(name: string, type: string): NbtNode {
  return { name, tag_type: type, value: defaultValue(type), children: [] }
}

// ===== 值编辑 =====
function startEdit() {
  if (!props.editable || isContainer.value || isArrayType.value) return
  editing.value = true
  editText.value = node.value === null || node.value === undefined ? '' : String(node.value)
}

function commitEdit() {
  const text = editText.value
  if (node.tag_type === 'string') {
    node.value = text
  } else if (isNumberType.value) {
    const n = Number(text)
    if (!Number.isFinite(n)) return
    node.value = n
  }
  editing.value = false
}

function cancelEdit() {
  editing.value = false
}

// ===== 数组编辑（逗号分隔） =====
function startArrayEdit() {
  if (!props.editable || !isArrayType.value) return
  arrayEditing.value = true
  arrayText.value = Array.isArray(node.value) ? node.value.join(', ') : ''
}

function commitArrayEdit() {
  const parts = arrayText.value.split(',').map((s) => s.trim()).filter(Boolean)
  node.value = parts.map((p) => Number(p)).filter((n) => Number.isFinite(n))
  arrayEditing.value = false
}

// ===== 新增 / 删除 =====
function startAdd() {
  if (!props.editable || node.tag_type !== 'compound') return
  addMode.value = true
  addName.value = ''
  addType.value = 'string'
}

function confirmAdd() {
  const name = addName.value.trim()
  if (!name) return
  node.children.push(makeNode(name, addType.value))
  addMode.value = false
  addName.value = ''
}

function addListItem() {
  if (!props.editable || node.tag_type !== 'list') return
  const first = node.children[0]
  node.children.push(makeNode('', first ? first.tag_type : 'string'))
}

function removeNode() {
  if (props.isRoot) return
  emit('remove', node, key.value)
}
</script>

<template>
  <div>
    <!-- 当前节点行 -->
    <div class="group flex items-center gap-1.5 py-1 rounded px-1 hover:bg-gray-50">
      <div class="flex flex-1 min-w-0 items-center gap-1.5 cursor-pointer" @click="toggle">
        <ChevronDownIcon v-if="isContainer && isExpanded" class="h-3.5 w-3.5 flex-none text-gray-400" />
        <ChevronRightIcon v-else-if="isContainer" class="h-3.5 w-3.5 flex-none text-gray-400" />
        <span v-else class="inline-block w-3.5 flex-none" />
        <Tag size="small" class="flex-none" :color="tagColor(node.tag_type)">{{ node.tag_type }}</Tag>
        <span v-if="node.name" class="text-sm text-gray-800 font-medium truncate">{{ node.name }}</span>
        <span v-else class="text-sm text-gray-400 italic">(unnamed)</span>

        <!-- 叶子值：编辑态 / 展示态 -->
        <template v-if="!isContainer && !isArrayType && node.value !== null && node.value !== undefined">
          <Input
            v-if="editing"
            v-model="editText"
            size="small"
            class="flex-none"
            :width="editInputWidth"
            @keydown.enter="commitEdit"
            @keydown.esc="cancelEdit"
            @blur="commitEdit"
          />
          <span v-else-if="editable" class="text-sm text-gray-500 truncate hover:text-primary-600" @click.stop="startEdit">
            {{ formatValue(node) }}
          </span>
          <span v-else class="text-sm text-gray-500 truncate">{{ formatValue(node) }}</span>
        </template>

        <!-- 数组值 -->
        <template v-else-if="!isContainer && isArrayType">
          <Input
            v-if="arrayEditing"
            v-model="arrayText"
            size="small"
            class="flex-none"
            :width="arrayInputWidth"
            placeholder="逗号分隔，如 1, 2, 3"
            @keydown.enter="commitArrayEdit"
            @keydown.esc="arrayEditing = false"
            @blur="commitArrayEdit"
          />
          <span v-else-if="editable" class="text-sm text-gray-500 truncate hover:text-primary-600" @click.stop="startArrayEdit">
            {{ formatValue(node) }}
          </span>
          <span v-else class="text-sm text-gray-500 truncate">{{ formatValue(node) }}</span>
        </template>

        <span v-if="isContainer && node.children.length > 0" class="text-xs text-gray-400">({{ node.children.length }})</span>
      </div>

      <!-- 操作按钮（编辑模式 + hover 显示） -->
      <div v-if="editable && !isRoot" class="hidden group-hover:flex flex-none items-center gap-1">
        <button
          v-if="isContainer"
          class="rounded p-0.5 text-gray-400 hover:text-primary-600 transition-colors"
          title="新增"
          @click.stop="node.tag_type === 'compound' ? startAdd() : addListItem()"
        >
          <PlusIcon class="h-3.5 w-3.5" />
        </button>
        <button
          class="rounded p-0.5 text-gray-400 hover:text-red-500 transition-colors"
          title="删除"
          @click.stop="removeNode"
        >
          <TrashIcon class="h-3.5 w-3.5" />
        </button>
      </div>
    </div>

    <!-- 新增子节点表单（compound） -->
    <div v-if="addMode" class="ml-6 flex items-center gap-1.5 py-1">
      <Input v-model="addName" size="small" class="w-32" placeholder="字段名" @keydown.enter="confirmAdd" />
      <select
        v-model="addType"
        class="h-7 rounded-md border border-gray-200 bg-white px-1.5 text-xs text-gray-700 outline-none focus:border-primary-400"
      >
        <option v-for="t in NBT_TYPES" :key="t" :value="t">{{ t }}</option>
      </select>
      <button class="rounded p-1 text-gray-500 hover:text-primary-600" title="确认" @click="confirmAdd">
        <CheckIcon class="h-3.5 w-3.5" />
      </button>
      <button class="rounded p-1 text-gray-500 hover:text-gray-700" title="取消" @click="addMode = false">
        <XMarkIcon class="h-3.5 w-3.5" />
      </button>
    </div>

    <!-- 子节点（展开时） -->
    <div v-if="isContainer && isExpanded" class="ml-4 border-l border-gray-100 pl-2">
      <NbtTreeNode
        v-for="(child, idx) in node.children"
        :key="idx"
        :node="child"
        :path="key"
        :expanded-set="expandedSet"
        :editable="editable"
        @toggle="emit('toggle', $event)"
        @remove="(n, k) => emit('remove', n, k)"
      />
    </div>
  </div>
</template>
