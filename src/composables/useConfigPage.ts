/**
 * 配置页 composable - 抽象 Settings 页的"加载配置 + 防抖保存 + loaded 守卫"三件套
 *
 * 重复模式（消除前在 4 个文件中出现）：
 * 1. `const loaded = ref(false)`
 * 2. `useDebouncedSave('patch', async (patch) => { try { await tauri.applyConfig(patch) } catch (e) { console.error('Failed to save xxx:', e) } }, delay)`
 * 3. `onMounted(async () => { try { const cfg = await tauri.getConfigMap(); ...赋值... } catch (e) { console.error(...) } await nextTick(); loaded.value = true })`
 *
 * 使用方式：
 * ```ts
 * const { loaded, markDirty, reload } = useConfigPage({
 *   delay: 500,
 *   errorLabel: 'save launch settings',
 *   onLoad: async (cfg) => {
 *     gameDir.value = cfg.gameDir || '.minecraft'
 *     memoryMode.value = cfg.memoryMode === 'custom' ? 'custom' : 'auto'
 *     // 可执行额外的异步初始化（如 getSystemMemory）
 *   }
 * })
 *
 * // 每个页面的 watch 不同，仍需手动绑定
 * watch(isolationMode, (v) => markDirty('isolationMode', v))
 * ```
 *
 * 设计说明：
 * - `onLoad` 回调允许调用方自定义赋值逻辑（各页面字段不同）
 * - 自动处理 `loaded` 守卫和 `await nextTick()`（避免加载值触发 watch 保存）
 * - 自动创建 `useDebouncedSave` 并绑定到 `applyConfig`
 * - 自动处理错误日志（统一前缀）
 * - onMounted 自动触发首次加载，也提供 `reload()` 支持手动刷新
 */
import { ref, onMounted, nextTick } from 'vue'
import type { Ref } from 'vue'
import * as tauri from '@/utils/tauri'
import type { ConfigPatch, ConfigSnapshot } from '@/utils/api/config'
import { useDebouncedSave } from '@/composables/useDebouncedSave'

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
      try {
        await tauri.applyConfig(patch)
      } catch (e) {
        console.error(`Failed to ${errorLabel}:`, e)
      }
    },
    delay,
  )

  // 加载配置：拉取全量快照 → 调用 onLoad 赋值 → nextTick → loaded=true
  async function reload(): Promise<void> {
    try {
      const cfg = await tauri.getConfigMap()
      if (onLoad) {
        await onLoad(cfg)
      }
    } catch (e) {
      if (onLoadError) {
        onLoadError(e)
      } else {
        console.error(`Failed to load config:`, e)
      }
    }
    // 等待 watch 回调执行完毕（避免加载值被误判为用户改动触发保存）
    await nextTick()
    loaded.value = true
  }

  // onMounted 自动触发首次加载
  onMounted(() => {
    void reload()
  })

  return {
    loaded,
    markDirty,
    flushSave,
    scheduleSave,
    isDirty,
    reload,
  }
}
