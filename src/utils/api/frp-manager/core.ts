/** Frp 管理 IPC 核心入口与 action 常量。 */
import { invoke } from '@tauri-apps/api/core'

/** Frp 管理 action 名称常量。 */
export const FRP_ACTIONS = {
  LIST_PROVIDERS: 'list_providers',
  ENSURE_FRPC: 'ensure_frpc',
  LIST_TUNNELS: 'list_tunnels',
  CREATE_TUNNEL: 'create_tunnel',
  UPDATE_TUNNEL: 'update_tunnel',
  DELETE_TUNNEL: 'delete_tunnel',
  IMPORT_FRPC_CONFIG: 'import_frpc_config',
  START_TUNNEL: 'start_tunnel',
  STOP_TUNNEL: 'stop_tunnel',
  GET_TUNNEL_STATUS: 'get_tunnel_status',
  INSTALL_PROVIDER_FROM_DIR: 'install_provider_from_dir',
  INSTALL_PROVIDER_FROM_ZIP: 'install_provider_from_zip',
  INSTALL_PROVIDER_FROM_URL: 'install_provider_from_url',
  UNINSTALL_PROVIDER: 'uninstall_provider',
  ENABLE_PROVIDER: 'enable_provider',
  DISABLE_PROVIDER: 'disable_provider',
  LIST_LOG_FILES: 'list_log_files',
  READ_LOG_FILE: 'read_log_file',
  CLEAR_LOG_FILE: 'clear_log_file',
  LIST_PUBLIC_SERVERS: 'list_public_servers',
  ALLOCATE_PUBLIC_SERVER: 'allocate_public_server',
  RELEASE_PUBLIC_SERVER: 'release_public_server',
  KEEPALIVE_PUBLIC_SERVER: 'keepalive_public_server',
  GET_AUTH_STATUS: 'get_auth_status',
  START_OAUTH2: 'start_oauth2',
  START_DEVICE_CODE: 'start_device_code',
  POLL_DEVICE_CODE: 'poll_device_code',
  REFRESH_TOKEN: 'refresh_token',
  REVOKE_AUTH: 'revoke_auth',
  SAVE_API_KEY: 'save_api_key',
  FETCH_TUNNELS: 'fetch_tunnels',
  DETECT_PACKAGE_TYPE: 'detect_package_type',
} as const

/** Frp 管理统一调用入口。 */
export async function frpManager<T = unknown>(action: string, params?: unknown): Promise<T> {
  return invoke<T>('frp_manager', { req: { action, params: params ?? null } })
}
