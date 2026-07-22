<script setup lang="ts">
/**
 * NBT 数据查看
 *
 * 输入 NBT 文件路径（如 level.dat / playerdata/{uuid}.dat），后端解析为树形结构。
 * 前端用 NbtTreeNode 递归组件渲染树，支持展开/折叠。
 * 后端手动实现 NBT 解析器（gzip 解压 + 大端二进制解析），无需 nightly crate。
 */
import { ref } from 'vue'
import {
  CubeIcon,
  BoltIcon,
  DocumentIcon,
} from '@heroicons/vue/24/outline'
import Button from '@/components/common/Button.vue'
import Input from '@/components/common/Input.vue'
import { toastError } from '@/utils/toast'
import { nbtParse } from '@/utils/api/tools'
import type { NbtNode } from '@/utils/api/tools'
import NbtTreeNode from '@/views/tools/data/NbtTreeNode.vue'

const filePath = ref('')
const parsing = ref(false)
const root = ref<NbtNode | null>(null)
const expandedSet = ref<Set<string>>(new Set())

function nodeKey(node: NbtNode, path: string): string {
  return path + '/' + node.name + ':' + node.tag_type
}

function toggleExpand(key: string) {
  const next = new Set(expandedSet.value)
  if (next.has(key)) next.delete(key)
  else next.add(key)
  expandedSet.value = next
}

async function doParse() {
  if (!filePath.value.trim()) return
  parsing.value = true
  root.value = null
  expandedSet.value = new Set()
  try {
    const res = await nbtParse(filePath.value.trim())
    root.value = res.root
    // 默认展开根节点
    expandedSet.value = new Set([nodeKey(res.root, '')])
  } catch (e) {
    toastError('解析失败: ' + (e instanceof Error ? e.message : String(e)))
  } finally {
    parsing.value = false
  }
}

function countNodes(node: NbtNode): number {
  let count = 1
  for (const child of node.children) count += countNodes(child)
  return count
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <CubeIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">NBT 数据查看</h3>
      <span v-if="root" class="ml-auto text-xs text-gray-400">{{ countNodes(root) }} 个节点</span>
    </div>
    <div class="px-5 pb-5 space-y-3">
      <p class="text-xs text-gray-500">
        解析 NBT 文件（level.dat、playerdata/*.dat 等），以树形结构展示。支持 gzip 压缩格式。
      </p>

      <!-- 路径输入 -->
      <div class="flex items-end gap-3">
        <div class="flex-1">
          <label class="mb-1 block text-xs font-medium text-gray-700">NBT 文件路径</label>
          <Input
            v-model="filePath"
            placeholder="如 .minecraft/saves/MyWorld/level.dat"
            clearable
          />
        </div>
        <Button type="primary" :loading="parsing" :disabled="!filePath.trim()" @click="doParse">
          <template #icon><BoltIcon class="h-4 w-4" /></template>
          {{ parsing ? '解析中...' : '解析' }}
        </Button>
      </div>

      <!-- 树形展示 -->
      <div v-if="root" class="max-h-[500px] overflow-y-auto rounded-lg border border-gray-200 p-3">
        <NbtTreeNode
          :node="root"
          :path="''"
          :expanded-set="expandedSet"
          @toggle="toggleExpand"
        />
      </div>

      <!-- 空状态 -->
      <div
        v-else
        class="flex flex-col items-center justify-center py-8 text-gray-400"
      >
        <DocumentIcon class="h-8 w-8 mb-2 text-gray-300" />
        <span class="text-xs">输入 NBT 文件路径后点击解析</span>
      </div>
    </div>
  </section>
</template>
