/**
 * authlib-injector 外置登录（yggdrasil 协议）API 封装
 *
 * 对应后端 `commands::auth::authlib` 模块的 11 个命令（已聚合为 `meta_manager` IPC 入口）：
 * - `authlib_fetch_server_meta`：获取 yggdrasil 服务器元数据（服务器名/注册链接）
 * - `authlib_login`：账号密码登录，返回单角色成功或多角色待选
 * - `authlib_select_profile`：多角色场景下选定 profile 完成登录
 * - `switch_authlib_account`：切换到已保存的 authlib 账号（三步降级）
 * - `get_authlib_accounts`：获取已保存的 authlib 账号列表
 * - `remove_authlib_account`：删除指定 authlib 账号
 * - `authlib_get_skin_info`：查询外置账号皮肤/披风信息（含 uploadableTextures）
 * - `authlib_upload_skin`：上传皮肤 PNG（multipart，由后端构造）
 * - `authlib_delete_skin`：删除皮肤
 * - `authlib_upload_cape`：上传披风 PNG
 * - `authlib_delete_cape`：删除披风
 *
 * 字段名约定：
 * - 请求参数（params）使用 camelCase（后端 Params 结构体使用
 *   `#[serde(rename_all = "camelCase")]` 反序列化）。
 * - 响应数据使用 snake_case（后端 Serialize derive 默认按字段名输出，
 *   与 `AuthlibAccountInfo` / `AuthlibSkinCapeInfo` 等类型一致）。
 */

import { metaManager, META_ACTIONS } from '@/utils/api/meta-manager'
import type {
  AuthlibAccountInfo,
  AuthlibLoginResult,
  AuthlibProfile,
  AuthlibServerMeta,
  AuthlibSkinCapeInfo,
  LocalAuthResult,
} from '@/types/auth'

/**
 * 获取 yggdrasil 服务器元数据
 *
 * 用户在登录页输入 server_url 后调用，用于显示服务器名/注册链接。
 * 失败时抛出错误（调用方提示用户检查地址或网络）。
 */
export async function authlibFetchServerMeta(serverUrl: string): Promise<AuthlibServerMeta> {
  return await metaManager<AuthlibServerMeta>(META_ACTIONS.AUTHLIB_FETCH_SERVER_META, { serverUrl })
}

/**
 * 账号密码登录
 *
 * - 单角色或服务器已选定 → 返回 `status: 'success'`，含可直接使用的 LocalAuthResult
 * - 多角色 → 返回 `status: 'need_select'`，前端需弹窗让用户选择 profile
 *           再调用 `authlibSelectProfile` 完成登录
 */
export async function authlibLogin(
  serverUrl: string,
  username: string,
  password: string,
): Promise<AuthlibLoginResult> {
  return await metaManager<AuthlibLoginResult>(META_ACTIONS.AUTHLIB_LOGIN, {
    serverUrl,
    username,
    password,
  })
}

/**
 * 多角色场景下选定 profile 完成登录
 *
 * 前端拿到 `need_select` 后弹窗让用户选择 profile，选定后调用此命令。
 * 内部调用 yggdrasil `/authserver/refresh` 指定 selected_profile，
 * 成功后持久化账号并设为当前用户。
 */
export async function authlibSelectProfile(profile: AuthlibProfile): Promise<LocalAuthResult> {
  return await metaManager<LocalAuthResult>(META_ACTIONS.AUTHLIB_SELECT_PROFILE, { profile })
}

/**
 * 切换到已保存的 authlib 账号（三步降级：validate → refresh → 用密码重登）
 *
 * 仅传 server_url + uuid，内部从持久化存储读取账号信息。
 * 任何一步成功即返回 LocalAuthResult，全部失败则抛出错误。
 */
export async function switchAuthlibAccount(
  serverUrl: string,
  uuid: string,
): Promise<LocalAuthResult> {
  return await metaManager<LocalAuthResult>(META_ACTIONS.SWITCH_AUTHLIB_ACCOUNT, {
    serverUrl,
    uuid,
  })
}

/**
 * 获取已保存的 authlib 账号列表
 *
 * 返回的列表不含敏感字段（password/access_token/client_token 已过滤）。
 */
export async function getAuthlibAccounts(): Promise<AuthlibAccountInfo[]> {
  return await metaManager<AuthlibAccountInfo[]>(META_ACTIONS.GET_AUTHLIB_ACCOUNTS)
}

/**
 * 删除指定 authlib 账号
 *
 * 按 server_url + uuid 唯一定位（同一服务器可有多个角色账号）。
 */
export async function removeAuthlibAccount(serverUrl: string, uuid: string): Promise<void> {
  return await metaManager<void>(META_ACTIONS.REMOVE_AUTHLIB_ACCOUNT, { serverUrl, uuid })
}

// ============================================================
// yggdrasil 皮肤管理（5 个端点封装，参考 yggdrasil-api-analysis.md 4.3/4.4 节）
// ============================================================

/**
 * 查询外置账号的皮肤/披风信息
 *
 * 返回 `AuthlibSkinCapeInfo`，含：
 * - `skin_url` / `skin_model`：当前皮肤 URL 与模型（default / slim）
 * - `cape_url`：当前披风 URL（无披风时为 null）
 * - `uploadable_textures`：可上传材质类型（"skin" / "cape" / "skin,cape" / 空串）
 *
 * 前端据 uploadable_textures 动态显示上传按钮（空串表示服务器不允许上传）。
 */
export async function authlibGetSkinInfo(
  serverUrl: string,
  uuid: string,
): Promise<AuthlibSkinCapeInfo> {
  return await metaManager<AuthlibSkinCapeInfo>(META_ACTIONS.AUTHLIB_GET_SKIN_INFO, { serverUrl, uuid })
}

/**
 * 上传皮肤
 *
 * 与微软 `uploadSkin` / 离线 `saveCustomSkin` 一致：传入本地文件路径，
 * 后端读取并校验 PNG（避免前端引入 `@tauri-apps/plugin-fs` 依赖）。
 *
 * @param serverUrl yggdrasil API 根地址
 * @param uuid 角色 UUID
 * @param filePath PNG 文件本地路径（由 pickFile 返回）
 * @param model "slim"（Alex）或 "default"（Steve）
 */
export async function authlibUploadSkin(
  serverUrl: string,
  uuid: string,
  filePath: string,
  model: 'slim' | 'default',
): Promise<void> {
  return await metaManager<void>(META_ACTIONS.AUTHLIB_UPLOAD_SKIN, {
    serverUrl,
    uuid,
    filePath,
    model,
  })
}

/**
 * 删除皮肤
 */
export async function authlibDeleteSkin(serverUrl: string, uuid: string): Promise<void> {
  return await metaManager<void>(META_ACTIONS.AUTHLIB_DELETE_SKIN, { serverUrl, uuid })
}

/**
 * 上传披风
 *
 * @param serverUrl yggdrasil API 根地址
 * @param uuid 角色 UUID
 * @param filePath PNG 文件本地路径
 */
export async function authlibUploadCape(
  serverUrl: string,
  uuid: string,
  filePath: string,
): Promise<void> {
  return await metaManager<void>(META_ACTIONS.AUTHLIB_UPLOAD_CAPE, {
    serverUrl,
    uuid,
    filePath,
  })
}

/**
 * 删除披风
 */
export async function authlibDeleteCape(serverUrl: string, uuid: string): Promise<void> {
  return await metaManager<void>(META_ACTIONS.AUTHLIB_DELETE_CAPE, { serverUrl, uuid })
}
