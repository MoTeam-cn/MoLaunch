/**
 * 统一 HTTP 请求客户端（axios）
 *
 * 前端所有 JSON / 文本请求统一走本模块，禁止直接使用原生 fetch（除 wasm 二进制加载、
 * PNG blob 解码等需要 Response 流的场景）。
 * 注意：WebView 内 axios 走 XMLHttpRequest，浏览器强制忽略自定义 User-Agent 头，
 * 需要自定义 UA 的请求必须走后端 reqwest 转发，前端无法实现。
 */
import axios from 'axios'

/** 统一请求实例：默认 15s 超时 */
const http = axios.create({
  timeout: 15_000,
})

/** GET 请求并解析 JSON */
export async function getJson<T>(url: string): Promise<T> {
  const res = await http.get<T>(url)
  return res.data
}

/** POST 表单（application/x-www-form-urlencoded）并解析 JSON */
export async function postForm<T>(url: string, body: Record<string, string>): Promise<T> {
  const params = new URLSearchParams(body)
  const res = await http.post<T>(url, params)
  return res.data
}

/** GET 请求返回纯文本（如 .txt 资源清单） */
export async function getText(url: string): Promise<string> {
  const res = await http.get<string>(url, { responseType: 'text' })
  return res.data
}

export default http
