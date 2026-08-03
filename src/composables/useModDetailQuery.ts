/**
 * Mod 详情查询 composable：详情按钮三级 fallback
 *
 * 1. project 已预加载 → 直接弹窗 2. 并发请求 CF+MR（Promise.any）3. 本地信息弹窗；
 * 另封装「前往百科」按钮（mcmod 直链 → 搜索页回退）。
 */
import { ref, type Ref } from 'vue'
import { getProjectDetail, getMcmodUrl } from '@/utils/api/community'
import { showInfo } from '@/utils/modal'
import { formatBytes } from '@/utils/format'
import { open as openUrl } from '@tauri-apps/plugin-shell'
import { toastError } from '@/utils/toast'
import { modTitle, loaderVisual, stripModVersion } from '@/utils/mod-display'
import type { ResourceProject } from '@/types/community'
import type { ModInfo } from '@/utils/tauri'

export function useModDetailQuery() {
  // Mod 详情弹窗（关联到 CF/MR 平台工程时使用）
  const detailVisible = ref(false)
  const detailProject = ref<ResourceProject | null>(null)
  /** 当前正在加载详情的 mod file_name（用于按钮 spinner + 防止重复点击同一 mod） */
  const detailLoadingFor = ref<string | null>(null)

  /** 显示本地 Mod 信息弹窗（无法关联到 CF/MR 平台时使用） */
  function showLocalModInfo(mod: ModInfo) {
    const lines: string[] = []
    if (mod.description) {
      lines.push(mod.description)
      lines.push('')
    }
    lines.push(`文件：${mod.file_name}（${formatBytes(mod.size)}）`)
    if (mod.version) lines.push(`版本：${mod.version}`)
    if (mod.translated_name) lines.push(`译名：${mod.translated_name}`)
    if (mod.loader_type !== 'unknown') lines.push(`加载器：${loaderVisual(mod.loader_type).label}`)
    showInfo(modTitle(mod, 0), lines.join('\n'))
  }

  /**
   * 详情按钮：
   *
   * 核心设计：**详情按钮本身不发任何网络请求**，只判断 `mod.project` 是否已被预加载填充。
   *
   * 三级 fallback：
   * 1. **零延迟路径（最优）**：`mod.project` 已被 `preload_mods_detail_cmd` 后台预加载填充
   *    → 直接弹 ResourceDetail（工程已就绪分支）
   * 2. **并发 fallback**：预加载尚未完成（用户点太快）或预加载失败
   *    → 并发请求 CF + MR（`Promise.any`），谁先成功用谁
   * 3. **本地信息**：无 slug 或两个平台都查不到
   *    → 弹本地信息弹窗 + "百科搜索"按钮（本地兜底分支）
   *
   * 防呆：detailLoadingFor 记录当前加载中的 mod file_name，
   * 按钮显示 spinner 并禁用同 mod 的重复点击。
   *
   * 参数 mods / isPreloadDone 由父组件传入 refs，便于在预加载等待循环中
   * 重新读取 mods 数组中最新的 mod 状态（slug / project 可能已被预加载事件填充）。
   */
  async function handleShowInfo(mod: ModInfo, mods: Ref<ModInfo[]>, isPreloadDone: Ref<boolean>) {
    // 防呆：同一 mod 正在加载中，忽略重复点击
    if (detailLoadingFor.value === mod.file_name) return

    // 1. 零延迟路径：预加载已就绪，直接弹窗
    if (mod.project) {
      detailProject.value = mod.project
      detailVisible.value = true
      return
    }

    // 2. 无 slug：预加载可能还没读到 jar 元数据，等一小段时间再判断
    //    如果预加载已完成且仍无 slug，说明 jar 内没有 metadata，立即走本地信息弹窗
    if (!mod.slug) {
      // 预加载已完成 → slug 不会再来了，立即走本地信息弹窗
      if (isPreloadDone.value) {
        showLocalModInfo(mod)
        return
      }
      // 预加载未完成 → 等待最多 3 秒（每 100ms 检查一次 slug 或 project 是否就绪）
      detailLoadingFor.value = mod.file_name
      try {
        for (let i = 0; i < 30; i++) {
          await new Promise(r => setTimeout(r, 100))
          const current = mods.value.find(m => m.file_name === mod.file_name)
          if (current?.slug) {
            mod = current
            break
          }
          // 如果预加载期间 project 就绪了，直接弹窗
          if (current?.project) {
            detailProject.value = current.project
            detailVisible.value = true
            return
          }
          // 预加载已完成 → slug 不会再来了，跳出等待
          if (isPreloadDone.value) break
        }
      } finally {
        detailLoadingFor.value = null
      }
      // 等待后仍无 slug，走本地信息弹窗
      if (!mod.slug) {
        showLocalModInfo(mod)
        return
      }
    }

    // 3. 有 slug 但 project 未就绪：并发请求 CF + MR
    detailLoadingFor.value = mod.file_name
    try {
      const project = await Promise.any([
        getProjectDetail('CurseForge', mod.slug, 'Mod').catch(e => {
          console.debug('[ModTab] CF 详情查询失败:', e)
          throw e
        }),
        getProjectDetail('Modrinth', mod.slug, 'Mod').catch(e => {
          console.debug('[ModTab] MR 详情查询失败:', e)
          throw e
        }),
      ])
      detailProject.value = project
      detailVisible.value = true
    } catch (e) {
      // Promise.any 在所有 promise 都 reject 时抛 AggregateError
      console.debug('[ModTab] CF/MR 详情查询均失败，回退本地信息:', e)
      showLocalModInfo(mod)
    } finally {
      detailLoadingFor.value = null
    }
  }

  /**
   * 前往百科按钮：
   * - 优先通过 slug 查 mcmod.cn 直链（先 CF 后 MR，因为 mcmod 数据库中 CF 收录更全）
   * - 查不到直链时打开 mcmod.cn 搜索页，关键字优先用译名，其次用文件名去扩展名+版本号
   *
   * 搜索 URL 格式：https://search.mcmod.cn/s?key=<keyword>
   * 关键字必须去除版本号等参数（如 "AI-Improvements-1.20-0.5.2" → "AI-Improvements"），
   * 否则百科搜索匹配不到结果。
   */
  async function handleOpenWiki(mod: ModInfo) {
    // 有 slug：尝试 CF → MR 查 mcmod.cn 直链
    if (mod.slug) {
      try {
        let url = await getMcmodUrl('CurseForge', mod.slug)
        if (!url) url = await getMcmodUrl('Modrinth', mod.slug)
        if (url) {
          await openUrl(url)
          return
        }
      } catch (e) {
        console.debug('[ModTab] 查 mcmod 直链失败，回退搜索页:', e)
      }
    }
    // 回退：打开 mcmod.cn 搜索页（注意：URL 是 search.mcmod.cn/s?key=，不是 www.mcmod.cn/search?key=）
    const keyword = stripModVersion(mod.translated_name || mod.file_name)
    const searchUrl = `https://search.mcmod.cn/s?key=${encodeURIComponent(keyword)}`
    try {
      await openUrl(searchUrl)
    } catch {
      toastError('打开百科失败')
    }
  }

  return { detailVisible, detailProject, detailLoadingFor, handleShowInfo, handleOpenWiki }
}
