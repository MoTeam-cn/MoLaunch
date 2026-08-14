<script setup lang="ts">
/**
 * 指令生成 - 召唤实体
 *
 * 配置实体、坐标（支持 ~ 相对坐标）、自定义名称与数量，
 * 生成 /summon 指令。
 */
import { computed, ref } from 'vue'
import { SparklesIcon } from '@heroicons/vue/24/outline'
import Input from '@/components/common/Input.vue'
import Button from '@/components/common/Button.vue'
import Select from '@/components/common/Select.vue'
import { copyToClipboard } from '@/utils/clipboard'
import { ENTITIES } from './data'
import { COLOR_OPTIONS, buildSummonCommand } from './generator'

const entityId = ref(ENTITIES[0].id)
const x = ref('~')
const y = ref('~')
const z = ref('~')
const name = ref('')
const nameColor = ref('white')
const count = ref(1)

const entityOptions = computed(() => ENTITIES.map((e) => ({ label: `${e.name}（${e.id}）`, value: e.id })))

const command = computed(() =>
  buildSummonCommand({
    entityId: entityId.value,
    x: x.value,
    y: y.value,
    z: z.value,
    name: name.value,
    nameColor: nameColor.value,
    count: count.value,
  }),
)

async function copyCommand() {
  await copyToClipboard(command.value, { toast: true })
}
</script>

<template>
  <section class="rounded-lg border border-gray-300 bg-white">
    <div class="flex items-center gap-2 px-5 pt-5 pb-3">
      <SparklesIcon class="h-5 w-5 text-gray-700" />
      <h3 class="text-sm font-semibold text-gray-900">召唤实体（/summon）</h3>
    </div>

    <div class="px-5 pb-5 space-y-4">
      <!-- 实体选择 -->
      <div>
        <div class="text-xs font-medium text-gray-500 mb-2">实体</div>
        <Select v-model="entityId" :options="entityOptions" />
      </div>

      <!-- 坐标 -->
      <div>
        <div class="text-xs font-medium text-gray-500 mb-2">坐标（支持 ~ 相对坐标）</div>
        <div class="grid grid-cols-3 gap-2">
          <Input v-model="x" placeholder="X" size="small" />
          <Input v-model="y" placeholder="Y" size="small" />
          <Input v-model="z" placeholder="Z" size="small" />
        </div>
      </div>

      <!-- 数量 + 名称 -->
      <div class="grid grid-cols-[80px_1fr] gap-4 items-end">
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">数量</div>
          <Input v-model.number="count" type="number" min="1" size="small" />
        </div>
        <div>
          <div class="text-xs font-medium text-gray-500 mb-2">自定义名称（可选）</div>
          <Input v-model="name" placeholder="如：守卫者" size="small" />
        </div>
      </div>

      <!-- 颜色 -->
      <div v-if="name.trim()">
        <div class="text-xs font-medium text-gray-500 mb-2">名称颜色</div>
        <Select v-model="nameColor" :options="COLOR_OPTIONS" />
      </div>

      <!-- 指令结果 -->
      <div class="border-t border-gray-100 pt-4">
        <div class="text-xs font-medium text-gray-500 mb-2">生成指令</div>
        <div class="rounded-lg bg-gray-50 px-3 py-2.5 font-mono text-xs break-all text-gray-700 min-h-[2.5rem]">
          {{ command }}
        </div>
        <div class="mt-2 flex justify-end">
          <Button size="small" @click="copyCommand">
            复制指令
          </Button>
        </div>
      </div>
    </div>
  </section>
</template>
