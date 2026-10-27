/**
 * 最小 PNG 解码（测试用）
 *
 * 只支持 8 位深、非隔行的真彩与真彩透明，够读纹理图集。
 * 输出统一成 RGBA，方便测试直接按像素断言。
 */
import { inflateSync } from 'node:zlib'

export interface DecodedPng {
  width: number
  height: number
  data: Uint8Array
}

export function decodePng(buffer: Buffer): DecodedPng {
  if (buffer.readUInt32BE(0) !== 0x89504e47) throw new Error('不是 PNG')
  let offset = 8
  let width = 0
  let height = 0
  let colorType = 0
  const idat: Buffer[] = []

  while (offset < buffer.length) {
    const length = buffer.readUInt32BE(offset)
    const type = buffer.toString('ascii', offset + 4, offset + 8)
    const body = buffer.subarray(offset + 8, offset + 8 + length)
    if (type === 'IHDR') {
      width = body.readUInt32BE(0)
      height = body.readUInt32BE(4)
      if (body[8] !== 8) throw new Error('只支持 8 位深')
      if (body[12] !== 0) throw new Error('不支持隔行扫描')
      colorType = body[9]
    } else if (type === 'IDAT') {
      idat.push(body)
    } else if (type === 'IEND') {
      break
    }
    offset += 12 + length
  }

  const channels = colorType === 6 ? 4 : colorType === 2 ? 3 : 0
  if (channels === 0) throw new Error(`不支持的颜色类型 ${colorType}`)

  const raw = inflateSync(Buffer.concat(idat))
  const stride = width * channels
  const pixels = new Uint8Array(width * height * channels)

  // 逐行反滤波：每行开头一个滤波器字节
  for (let y = 0; y < height; y++) {
    const filter = raw[y * (stride + 1)]
    const src = y * (stride + 1) + 1
    const dst = y * stride
    const prev = dst - stride
    for (let x = 0; x < stride; x++) {
      const value = raw[src + x]
      const left = x >= channels ? pixels[dst + x - channels] : 0
      const up = y > 0 ? pixels[prev + x] : 0
      const upLeft = y > 0 && x >= channels ? pixels[prev + x - channels] : 0
      let out = value
      if (filter === 1) out = value + left
      else if (filter === 2) out = value + up
      else if (filter === 3) out = value + ((left + up) >> 1)
      else if (filter === 4) {
        const p = left + up - upLeft
        const pa = Math.abs(p - left)
        const pb = Math.abs(p - up)
        const pc = Math.abs(p - upLeft)
        out = value + (pa <= pb && pa <= pc ? left : pb <= pc ? up : upLeft)
      }
      pixels[dst + x] = out & 0xff
    }
  }

  return { width, height, data: toRgba(pixels, width, height, channels) }
}

function toRgba(pixels: Uint8Array, width: number, height: number, channels: number): Uint8Array {
  if (channels === 4) return pixels
  const out = new Uint8Array(width * height * 4)
  for (let i = 0; i < width * height; i++) {
    out[i * 4] = pixels[i * 3]
    out[i * 4 + 1] = pixels[i * 3 + 1]
    out[i * 4 + 2] = pixels[i * 3 + 2]
    out[i * 4 + 3] = 255
  }
  return out
}
