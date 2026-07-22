<script setup lang="ts">
/**
 * 教程子页签（占位）
 */
import Card from '@/components/common/Card.vue'
import { BookOpenIcon } from '@heroicons/vue/24/outline'
</script>

<template>
  <Card>
    <template #title>
      <div class="flex items-center gap-2">
        <BookOpenIcon class="h-4 w-4 text-gray-500" />
        <span class="text-sm font-semibold text-gray-800">使用教程</span>
      </div>
    </template>

    <div class="py-12 text-center">
      <BookOpenIcon class="mx-auto mb-3 h-12 w-12 text-gray-200" />
      <p class="text-[13px] font-medium text-gray-400">教程编写中</p>
      <p class="mt-1 text-[12px] text-gray-400">敬请期待</p>
    </div>
  </Card>
</template>
