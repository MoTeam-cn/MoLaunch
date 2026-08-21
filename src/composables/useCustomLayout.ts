/**
 * 自定义布局配置 composable
 *
 * 布局格式/来源配置读写（防抖同步 pluginStore）、内联编辑器 draft、URL 输入防抖与缓存刷新；
 * 返回全部模板所需状态与动作。
 */
import { ref, computed, onMounted } from 'vue'
import { usePluginStore } from '@/stores/plugins'
import { writeTextFile } from '@/utils/api/system'
import { readLayoutSample } from '@/utils/api/plugins'
import { pickSavePath } from '@/utils/fileDialog'
import { toastInfo, toastSuccess, toastError, toastWarning } from '@/utils/toast'
import { showConfirm } from '@/utils/modal'
import { formatDateTime } from '@/utils/format'

/** 自定义布局配置 composable（格式 / 来源 / 内联编辑器 / URL 加载 / 示例导出） */
export function useCustomLayout() {
  const pluginStore = usePluginStore()

  /** 自定义布局配置（从 store 读取） */
  const customConfig = computed(() => pluginStore.customLayoutConfig)

  /** 内联编辑器占位文本（根据格式返回示例） */
  const inlinePlaceholder = computed(() => {
    switch (customConfig.value.format) {
      case 'json':
        return '{\n  "title": "我的面板",\n  "sections": [\n    { "type": "text", "content": "Hello" }\n  ]\n}'
      case 'xml':
        return '<panel title="我的面板">\n  <text>Hello</text>\n</panel>'
      case 'html':
      default:
        return '<div>\n  <h3>我的面板</h3>\n  <p>Hello</p>\n</div>'
    }
  })

  /** 布局格式选项 */
  const formatOptions = [
    { label: 'JSON（结构化布局）', value: 'json' },
    { label: 'HTML（直接渲染）', value: 'html' },
    { label: 'XML（结构化布局）', value: 'xml' },
  ]

  /** 布局来源选项 */
  const sourceOptions = [
    { label: '内联（直接编辑）', value: 'inline' },
    { label: 'URL（远程加载）', value: 'url' },
  ]

  /** JSON/XML 内联内容编辑器（本地 ref，防抖同步到 store） */
  const inlineContentDraft = ref('')
  let inlineSyncTimer: ReturnType<typeof setTimeout> | null = null

  /** 初始化内联内容 draft */
  function initInlineDraft() {
    if (customConfig.value.source === 'inline') {
      inlineContentDraft.value = customConfig.value.inlineContent
    }
  }

  /** 内联内容变更（防抖 500ms 同步到 store） */
  function onInlineContentChange() {
    if (inlineSyncTimer) clearTimeout(inlineSyncTimer)
    inlineSyncTimer = setTimeout(async () => {
      await pluginStore.setCustomLayoutConfig({ inlineContent: inlineContentDraft.value })
    }, 500)
  }

  /** URL 刷新中 */
  const urlRefreshing = ref(false)

  /** 刷新 URL 缓存 */
  async function onRefreshUrl() {
    if (urlRefreshing.value) return
    if (!customConfig.value.url) {
      toastError('请先填写 URL 地址')
      return
    }
    urlRefreshing.value = true
    try {
      toastInfo('正在刷新布局缓存...')
      await pluginStore.refreshCustomLayoutCache()
      toastSuccess('布局缓存已刷新')
    } catch (e) {
      toastError(String(e))
    } finally {
      urlRefreshing.value = false
    }
  }

  /** 切换布局格式 */
  async function onFormatChange(value: string | number) {
    await pluginStore.setCustomLayoutConfig({ format: String(value) as 'json' | 'html' | 'xml' })
  }

  /** 切换布局来源 */
  async function onSourceChange(value: string | number) {
    const source = String(value) as 'inline' | 'url'
    await pluginStore.setCustomLayoutConfig({ source })
    if (source === 'inline') {
      initInlineDraft()
    }
  }

  /** URL 输入防抖同步 */
  let urlSyncTimer: ReturnType<typeof setTimeout> | null = null
  function onUrlInput(value: string | number) {
    if (urlSyncTimer) clearTimeout(urlSyncTimer)
    urlSyncTimer = setTimeout(async () => {
      await pluginStore.setCustomLayoutConfig({ url: String(value) })
    }, 500)
  }

  /** 缓存时间格式化 */
  const cachedTimeText = computed(() => {
    if (!customConfig.value.cachedAt) return '未缓存'
    return formatDateTime(customConfig.value.cachedAt, { invalidValue: '未缓存' })
  })

  /** 根据当前格式从后端读取示例布局并导出 */
  async function onExportSampleLayout() {
    const format = customConfig.value.format
    const ext = format
    const defaultName = `layout-sample.${ext}`
    try {
      const content = await readLayoutSample(format)
      const savePath = await pickSavePath({
        title: '保存示例布局文件',
        defaultPath: defaultName,
        filters: [{ name: `${ext.toUpperCase()} 文件`, extensions: [ext] }],
      })
      if (!savePath) return
      await writeTextFile(savePath, content)
      toastSuccess(`示例布局已导出至：${savePath}`)
    } catch (e) {
      toastError('导出示例失败：' + e)
    }
  }

  /**
   * 填入示例模板到内联编辑器
   *
   * 直接从后端读取当前格式的示例布局内容，填入内联编辑器并同步到 store，
   * 省去用户「导出文件 → 打开文件 → 复制内容 → 粘贴到编辑器」的繁琐流程。
   *
   * 保护逻辑：
   * - 来源为 URL 时提示先切换到内联模式（URL 模式下内联编辑器不可见）
   * - 内联编辑器已有内容时弹窗确认避免覆盖
   */
  const fillingTemplate = ref(false)
  async function onFillTemplate() {
    if (customConfig.value.source !== 'inline') {
      toastWarning('请先切换内容来源为「内联」模式')
      return
    }
    if (inlineContentDraft.value.trim()) {
      const confirmed = await new Promise<boolean>((resolve) => {
        showConfirm(
          '覆盖现有内容',
          '内联编辑器中已有内容，填入模板将覆盖现有内容，是否继续？',
          () => resolve(true),
          () => resolve(false),
        )
      })
      if (!confirmed) return
    }

    fillingTemplate.value = true
    try {
      const content = await readLayoutSample(customConfig.value.format)
      inlineContentDraft.value = content
      if (inlineSyncTimer) clearTimeout(inlineSyncTimer)
      await pluginStore.setCustomLayoutConfig({ inlineContent: content })
      toastSuccess('已填入示例模板')
    } catch (e) {
      toastError('填入示例失败：' + e)
    } finally {
      fillingTemplate.value = false
    }
  }

  onMounted(() => {
    initInlineDraft()
  })

  return {
    customConfig,
    inlinePlaceholder,
    formatOptions,
    sourceOptions,
    inlineContentDraft,
    onInlineContentChange,
    urlRefreshing,
    onRefreshUrl,
    onFormatChange,
    onSourceChange,
    onUrlInput,
    cachedTimeText,
    onExportSampleLayout,
    fillingTemplate,
    onFillTemplate,
  }
}
