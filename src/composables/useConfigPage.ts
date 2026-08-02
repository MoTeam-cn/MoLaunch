/**
 * 配置页 composable：抽象 Settings 页「加载配置 + 防抖保存 + loaded 守卫」三件套
 *
 * onLoad 回调自定义赋值，自动处理 loaded 守卫与 await nextTick()（避免加载值触发 watch 保存），
 * 自动创建 useDebouncedSave 绑定 applyConfig；onMounted 自动首次加载，reload() 手动刷新。
 */
import { ref, onMounted, nextTick } from 'vue'
import type { Ref } from 'vue'
import * as tauri from '@/utils/tauri'
import type { ConfigPatch, ConfigSnapshot } from '@/utils/api/config'
import { useDebouncedSave } from '@/composables/useDebouncedSave'
import { safeCall } from '@/utils/async'
import { toastError } from '@/utils/toast'

interface UseConfigPageOptions {
  /** 防抖延迟（ms），默认 800 */
  delay?: number
  /** 错误日志前缀（用于 console.error），默认 'save config' */
  errorLabel?: string
  /** 加载配置后的回调（用于自定义赋值逻辑），支持异步 */
  onLoad?: (cfg: ConfigSnapshot) => void | Promise<void>
  /** 加载失败的回调（默认 console.error） */
  onLoadError?: (e: unknown) => void
}

interface UseConfigPageReturn {
  /** 是否加载完毕（onMounted 的 nextTick 后才为 true） */
  loaded: Ref<boolean>
  /** 标记字段已改变，并防抖触发保存 */
  markDirty: (key: keyof ConfigPatch, value: unknown) => void
  /** 立即 flush，返回未保存的 patch */
  flushSave: () => ConfigPatch
  /** 防抖触发 flush */
  scheduleSave: () => void
  /** 是否有未保存的改动 */
  isDirty: () => boolean
  /** 手动重新加载配置（也会重置 loaded 并触发 nextTick） */
  reload: () => Promise<void>
}

/**
 * 抽象 Settings 页的配置加载 + 防抖保存 + loaded 守卫三件套
 */
export function useConfigPage(options: UseConfigPageOptions = {}): UseConfigPageReturn {
  const {
    delay = 800,
    errorLabel = 'save config',
    onLoad,
    onLoadError,
  } = options

  const loaded = ref(false)

  // 防抖保存：内部统一绑定 applyConfig + 错误日志
  const { markDirty, flushSave, scheduleSave, isDirty } = useDebouncedSave(
    'patch',
    async (patch: ConfigPatch) => {
      await safeCall(() => tauri.applyConfig(patch), errorLabel, () => toastError('配置保存失败'))
    },
    delay,
  )

  // 加载配置：拉取全量快照 → 调用 onLoad 赋值 → nextTick → loaded=true
  async function reload(): Promise<void> {
    await safeCall(async () => {
      const cfg = await tauri.getConfigMap()
      if (onLoad) {
        await onLoad(cfg)
      }
    }, 'load config', (e) => {
      toastError('配置加载失败')
      onLoadError?.(e)
    })
    // 等待 watch 回调执行完毕（避免加载值被误判为用户改动触发保存）
    await nextTick()
    loaded.value = true
  }

  // onMounted 自动触发首次加载
  onMounted(() => {
    void reload()
  })

  // markDirty 内置 loaded 守卫：
  // onLoad 赋值时 loaded 仍为 false，此时 watch 触发不应标记 dirty，否则
  // 切换页面（组件卸载触发 flushSave）时会发出无意义的 apply_config。
  function guardedMarkDirty(key: keyof ConfigPatch, value: unknown): void {
    if (!loaded.value) return
    markDirty(key, value)
  }

  return {
    loaded,
    markDirty: guardedMarkDirty,
    flushSave,
    scheduleSave,
    isDirty,
    reload,
  }
}
