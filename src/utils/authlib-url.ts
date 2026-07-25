/**
 * yggdrasil 服务器 URL 规范化工具
 *
 * 用户在登录页输入的服务器地址可能是多种形式：
 *   - `littleskin.cn`
 *   - `https://littleskin.cn`
 *   - `https://littleskin.cn/api/yggdrasil`
 *   - `littleskin.cn/api/yggdrasil`
 *   - 带尾部斜杠 `littleskin.cn/`
 *   - 皮肤站页面地址 `https://littleskin.cn/user` / `https://littleskin.cn/index`
 *
 * 本工具按以下规则自动补全为后端可识别的 yggdrasil API 根地址：
 *   1. 去除首尾空白和首尾斜杠
 *   2. 若无协议头（http:// 或 https://），自动补 https://
 *   3. 若路径以 /api/yggdrasil 结尾，直接使用
 *   4. 若路径为空或为皮肤站页面路径（/user、/index、/register、/login 等），
 *      去掉原路径，替换为 /api/yggdrasil
 *   5. 其它非标准路径保留用户输入（可能是自建服务器）
 *
 * 与 PCL2 行为一致：用户只需输入域名或皮肤站任意页面地址，自动识别并补全 API 路径。
 */

/** yggdrasil 协议约定的标准 API 路径后缀 */
const YGGDRASIL_API_PATH = '/api/yggdrasil'

/**
 * 皮肤站常见页面路径（用户可能误粘贴这些地址）
 *
 * 遇到这些路径时，视为皮肤站页面而非 yggdrasil API 路径，
 * 自动去掉并替换为 /api/yggdrasil。
 *
 * 使用 Array 而非 Set：避免 tsc 在低 target 下迭代 Set 报 TS2802。
 */
const SKIN_SITE_PAGE_PATHS: string[] = [
  '/',
  '/index',
  '/index.php',
  '/index.html',
  '/user',
  '/users',
  '/login',
  '/signin',
  '/register',
  '/signup',
  '/dashboard',
  '/skin',
  '/skins',
  '/cape',
  '/capes',
  '/profile',
  '/profiles',
  '/settings',
  '/about',
  '/help',
  '/faq',
  '/home',
]

/**
 * 判断字符串是否以 http:// 或 https:// 开头（不区分大小写）
 */
function hasProtocol(url: string): boolean {
  return /^https?:\/\//i.test(url)
}

/**
 * 判断路径是否为皮肤站页面路径（需要被替换为 /api/yggdrasil）
 *
 * 规则：
 * - 空路径或 "/" → 是
 * - 路径在 SKIN_SITE_PAGE_PATHS 列表中 → 是
 * - 路径以这些页面路径开头（如 /user/profile、/skin/edit）→ 是
 */
function isSkinSitePagePath(path: string): boolean {
  if (!path || path === '/') return true

  // 标准化：去尾部斜杠，转小写
  const normalized = path.replace(/\/+$/, '').toLowerCase()

  // 精确匹配
  if (SKIN_SITE_PAGE_PATHS.includes(normalized)) return true

  // 前缀匹配（如 /user/123、/skin/edit）
  for (const pagePath of SKIN_SITE_PAGE_PATHS) {
    if (pagePath === '/' || pagePath === '') continue
    if (normalized.startsWith(pagePath + '/') || normalized === pagePath) {
      return true
    }
  }

  return false
}

/**
 * 规范化 yggdrasil 服务器 URL
 *
 * @param input 用户输入的原始地址
 * @returns 规范化后的完整 URL（如 `https://littleskin.cn/api/yggdrasil`）
 */
export function normalizeAuthlibServerUrl(input: string): string {
  // 1. 去除首尾空白
  let url = input.trim()

  // 2. 去除首尾斜杠（保留中间路径）
  url = url.replace(/^\/+|\/+$/g, '')

  // 3. 空字符串直接返回
  if (!url) return ''

  // 4. 补全协议头
  if (!hasProtocol(url)) {
    url = 'https://' + url
  }

  // 5. 解析 URL，判断路径部分
  //    使用 URL 构造器解析（此时协议已补全）
  try {
    const parsed = new URL(url)
    const path = parsed.pathname

    // 路径已含 /api/yggdrasil → 直接返回（去尾部斜杠）
    if (path.endsWith(YGGDRASIL_API_PATH)) {
      return url.replace(/\/+$/, '')
    }

    // 无路径、或为皮肤站页面路径 → 替换为 /api/yggdrasil
    if (isSkinSitePagePath(path)) {
      parsed.pathname = YGGDRASIL_API_PATH
      parsed.search = ''
      parsed.hash = ''
      return parsed.toString().replace(/\/+$/, '')
    }

    // 其它非标准路径 → 保留用户输入（可能是自建服务器或非标准 yggdrasil 路径）
    // 但去除可能的尾部斜杠
    return url.replace(/\/+$/, '')
  } catch {
    // URL 解析失败（极端情况），返回原始输入（去尾部斜杠）
    return url.replace(/\/+$/, '')
  }
}

/**
 * 判断输入是否需要自动补全或替换路径
 *
 * 用于前端实时提示：当用户输入的地址不含 /api/yggdrasil 路径时，
 * 可以在 UI 上提示"将自动补全为 xxx"，避免用户疑惑。
 *
 * @param input 用户输入的原始地址
 * @returns true 表示会自动补全 /api/yggdrasil
 */
export function willAutoCompletePath(input: string): boolean {
  const url = input.trim().replace(/^\/+|\/+$/g, '')
  if (!url) return false

  try {
    const parsed = new URL(hasProtocol(url) ? url : 'https://' + url)
    const path = parsed.pathname
    return !path || path === '/' || !path.endsWith(YGGDRASIL_API_PATH)
  } catch {
    return false
  }
}
