/**
 * DataChannel 帧加密（阶段三子任务 8 安全加强）
 *
 * 房主创建房间 / 加入方加入房间时由后端下发 32 字节 AES-256 密钥（Base64Url 编码），
 * 前端在 DataChannel 收发前对完整协议帧（含头部）做 AES-GCM 加解密。
 *
 * # 帧格式
 *
 * 加密后的帧布局：
 * ```
 * +----------------+---------------------------+
 * | IV (12 bytes)  | ciphertext + tag (N+16)   |
 * +----------------+---------------------------+
 * ```
 *
 * - IV：每次加密随机生成 12 字节，明文发送（AES-GCM 标准做法）
 * - ciphertext + tag：AES-GCM 输出（包含 16 字节认证标签）
 *
 * # 密钥来源
 *
 * 后端 `rooms.room_key` 字段（Base64Url 无填充编码的 32 字节 AES-256 密钥）。
 * 空字符串表示服务器未启用加密（兼容旧服务器），此时 `importRoomKey` 返回 null，
 * 调用方应跳过加解密，直接透传原始帧。
 *
 * # 性能
 *
 * Web Crypto API（`crypto.subtle`）基于原生实现，单帧加解密耗时通常 < 0.1ms。
 * 协议帧典型大小 1500 字节（IP 包 + 7 字节帧头），加密后约 1535 字节，
 * 远低于 DataChannel 16KB 单消息上限。
 *
 * @example 房主侧使用
 * const key = await importRoomKey(store.roomState.roomKey)
 * hostMesh.setRoomKey(key)
 *
 * @example 手动加解密
 * const key = await importRoomKey(base64Key)
 * if (key) {
 *   const encrypted = await encryptFrame(plaintext, key)
 *   // ... send encrypted via DataChannel
 *   const decrypted = await decryptFrame(encrypted, key)
 *   if (decrypted) decode(decrypted)
 * }
 */

/** AES-GCM IV 长度（12 字节，NIST 推荐值） */
const IV_LENGTH = 12

/** AES-256 密钥字节数 */
const KEY_LENGTH = 32

/**
 * 从 Base64Url 字符串导入 AES-GCM 密钥
 *
 * 接受的格式：
 * - 标准 Base64Url 无填充（后端默认，如 `vCfP8sVN8LwT3jKB_XxT8P9k3yJ7mQ2NpQ4R5sT8uVw`）
 * - 标准 Base64（兼容手写测试密钥）
 *
 * @param base64Key Base64Url 编码的 32 字节 AES 密钥；空字符串返回 null
 * @returns CryptoKey（可用于 encrypt/decrypt）；输入为空或格式非法时返回 null
 */
export async function importRoomKey(base64Key: string): Promise<CryptoKey | null> {
  if (!base64Key) return null
  if (typeof crypto === 'undefined' || !crypto.subtle) {
    console.warn('[Online][crypto] 当前环境不支持 Web Crypto API，跳过加密')
    return null
  }
  try {
    // Base64Url → bytes（兼容标准 Base64）
    const raw = base64UrlToBytes(base64Key)
    if (raw.length !== KEY_LENGTH) {
      console.warn(`[Online][crypto] 密钥长度非法：期望 ${KEY_LENGTH} 字节，实际 ${raw.length}，跳过加密`)
      return null
    }
    return await crypto.subtle.importKey(
      'raw',
      raw,
      { name: 'AES-GCM', length: 256 },
      false,
      ['encrypt', 'decrypt'],
    )
  } catch (e) {
    console.warn('[Online][crypto] 导入密钥失败，跳过加密:', e)
    return null
  }
}

/**
 * 加密协议帧
 *
 * 生成随机 12 字节 IV → AES-GCM 加密 → 返回 `IV || ciphertext+tag`。
 * 同一明文每次加密结果不同（IV 随机），保证语义安全。
 *
 * @param plaintext 原始协议帧（含头部）
 * @param key AES-GCM 密钥
 * @returns 加密后的字节序列（IV + ciphertext + tag）
 */
export async function encryptFrame(
  plaintext: ArrayBuffer,
  key: CryptoKey,
): Promise<ArrayBuffer> {
  const iv = crypto.getRandomValues(new Uint8Array(IV_LENGTH))
  const ciphertext = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv },
    key,
    plaintext,
  )
  // 拼接 IV + ciphertext+tag
  const result = new Uint8Array(IV_LENGTH + ciphertext.byteLength)
  result.set(iv, 0)
  result.set(new Uint8Array(ciphertext), IV_LENGTH)
  return result.buffer
}

/**
 * 解密协议帧
 *
 * 输入应为 `encryptFrame` 输出格式（IV + ciphertext+tag）。
 * 解密失败（密钥不匹配 / 数据篡改 / 长度不足）返回 null，调用方应静默丢弃。
 *
 * @param encrypted DataChannel 收到的加密帧
 * @param key AES-GCM 密钥
 * @returns 解密后的原始协议帧；失败返回 null
 */
export async function decryptFrame(
  encrypted: ArrayBuffer,
  key: CryptoKey,
): Promise<ArrayBuffer | null> {
  if (encrypted.byteLength < IV_LENGTH + 16) return null // 至少 IV + 16 字节 tag
  const data = new Uint8Array(encrypted)
  const iv = data.slice(0, IV_LENGTH)
  const ciphertext = data.slice(IV_LENGTH)
  try {
    return await crypto.subtle.decrypt(
      { name: 'AES-GCM', iv },
      key,
      ciphertext,
    )
  } catch {
    // GCM 认证失败：数据被篡改或密钥不匹配，静默丢弃
    return null
  }
}

/**
 * Base64Url / Base64 字符串 → Uint8Array
 *
 * 兼容带/不带填充的 Base64Url 与标准 Base64。
 */
function base64UrlToBytes(input: string): Uint8Array {
  // Base64Url → Base64
  let b64 = input.replace(/-/g, '+').replace(/_/g, '/')
  // 补齐 padding
  const pad = b64.length % 4
  if (pad) b64 += '='.repeat(4 - pad)
  const binary = atob(b64)
  const bytes = new Uint8Array(binary.length)
  for (let i = 0; i < binary.length; i++) {
    bytes[i] = binary.charCodeAt(i)
  }
  return bytes
}
