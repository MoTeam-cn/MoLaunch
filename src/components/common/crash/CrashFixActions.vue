<script setup lang="ts">
/**
 * 崩溃弹窗的「一键修复」动作按钮组
 *
 * 动作由后端崩溃分析给出（FixAction）：禁用出问题的 Mod、打开 mods 目录、跳转到设置页。
 * 无动作时不渲染任何内容。
 */
import { ref, defineAsyncComponent } from 'vue'
import { useRouter } from 'vue-router'
import { toggleMod, openModsDir } from '@/utils/api/personalization/mods'
import { toastError, toastSuccess, toastInfo } from '@/utils/toast'
import type { CrashAction } from '@/types/version'

const Button = defineAsyncComponent(() => import('@/components/common/Button.vue'))

const props = defineProps<{
  actions: CrashAction[]
  versionId: string | null
}>()

const emit = defineEmits<{ close: [] }>()

const router = useRouter()
const running = ref<string | null>(null)

function actionKey(action: CrashAction): string {
  return action.kind === 'disable_mod' ? `${action.kind}:${action.file_name}` : action.kind
}

async function run(action: CrashAction) {
  if (action.kind === 'open_route') {
    emit('close')
    router.push(action.route)
    return
  }
  if (!props.versionId) {
    toastInfo('未找到对应的版本，请在版本设置中手动处理')
    return
  }
  running.value = actionKey(action)
  try {
    if (action.kind === 'disable_mod') {
      await toggleMod(props.versionId, action.file_name, false)
      toastSuccess(`已禁用 ${action.file_name}，可以重新启动游戏`)
    } else {
      await openModsDir(props.versionId)
    }
  } catch (error) {
    toastError(`操作失败：${String(error)}`)
  } finally {
    running.value = null
  }
}
</script>

<template>
  <div v-if="actions.length" class="flex flex-wrap items-center gap-2">
    <Button
      v-for="action in actions"
      :key="actionKey(action)"
      type="primary"
      size="small"
      :loading="running === actionKey(action)"
      @click="run(action)"
    >
      {{ action.label }}
    </Button>
  </div>
</template>
