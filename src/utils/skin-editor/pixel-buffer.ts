/**
 * 皮肤像素缓冲
 *
 * 以 RGBA 字节数组为唯一数据源，提供画笔、直线、油漆桶与差分撤销栈。
 * 纯逻辑实现（不依赖 DOM），可在单元测试中直接使用。
 */

export interface Rgba {
  r: number
  g: number
  b: number
  a: number
}

/** 撤销栈深度上限（差分记录，非整图快照） */
const UNDO_LIMIT = 50

type PixelValue = [number, number, number, number]
type PixelDiff = Map<number, PixelValue>

export class PixelBuffer {
  readonly width: number
  readonly height: number

  private readonly data: Uint8ClampedArray
  private strokeDiff: PixelDiff = new Map()
  private strokeOpen = false
  private readonly undoStack: PixelDiff[] = []
  private readonly redoStack: PixelDiff[] = []

  constructor(width: number, height: number, data?: Uint8ClampedArray) {
    this.width = width
    this.height = height
    this.data = new Uint8ClampedArray(width * height * 4)
    if (data) {
      this.data.set(data.subarray(0, this.data.length))
    }
  }

  /** 全透明空白皮肤 */
  static blank(size: number): PixelBuffer {
    return new PixelBuffer(size, size)
  }

  private index(x: number, y: number): number {
    return (y * this.width + x) * 4
  }

  private inside(x: number, y: number): boolean {
    return x >= 0 && y >= 0 && x < this.width && y < this.height
  }

  getPixel(x: number, y: number): Rgba {
    if (!this.inside(x, y)) return { r: 0, g: 0, b: 0, a: 0 }
    const i = this.index(x, y)
    return { r: this.data[i], g: this.data[i + 1], b: this.data[i + 2], a: this.data[i + 3] }
  }

  /** 导出副本（供渲染或编码） */
  toRgba(): Uint8ClampedArray {
    return new Uint8ClampedArray(this.data)
  }

  /** 开始一次笔画（连续拖动合并为一步撤销） */
  beginStroke(): void {
    this.strokeOpen = true
    this.strokeDiff = new Map()
  }

  /** 结束笔画；有实际改动时入撤销栈，返回是否产生改动 */
  endStroke(): boolean {
    this.strokeOpen = false
    if (this.strokeDiff.size === 0) return false
    this.undoStack.push(this.strokeDiff)
    if (this.undoStack.length > UNDO_LIMIT) this.undoStack.shift()
    this.redoStack.length = 0
    this.strokeDiff = new Map()
    return true
  }

  /** 写单个像素，返回是否发生改动 */
  setPixel(x: number, y: number, color: Rgba): boolean {
    if (!this.inside(x, y)) return false
    const i = this.index(x, y)
    const a = Math.round(color.a)
    if (
      this.data[i] === Math.round(color.r) &&
      this.data[i + 1] === Math.round(color.g) &&
      this.data[i + 2] === Math.round(color.b) &&
      this.data[i + 3] === a
    ) {
      return false
    }
    if (!this.strokeOpen) this.beginStroke()
    if (!this.strokeDiff.has(i)) {
      this.strokeDiff.set(i, [this.data[i], this.data[i + 1], this.data[i + 2], this.data[i + 3]])
    }
    this.data[i] = color.r
    this.data[i + 1] = color.g
    this.data[i + 2] = color.b
    this.data[i + 3] = a
    return true
  }

  /** 以方形笔刷绘制，返回改动像素数 */
  paint(x: number, y: number, color: Rgba, brushSize = 1): number {
    const half = Math.floor((brushSize - 1) / 2)
    let changed = 0
    for (let dy = 0; dy < brushSize; dy++) {
      for (let dx = 0; dx < brushSize; dx++) {
        if (this.setPixel(x - half + dx, y - half + dy, color)) changed++
      }
    }
    return changed
  }

  /** 画直线（Bresenham），返回改动像素数 */
  line(x0: number, y0: number, x1: number, y1: number, color: Rgba, brushSize = 1): number {
    let changed = 0
    let x = x0
    let y = y0
    const dx = Math.abs(x1 - x0)
    const dy = Math.abs(y1 - y0)
    const sx = x0 < x1 ? 1 : -1
    const sy = y0 < y1 ? 1 : -1
    let err = dx - dy
    for (;;) {
      changed += this.paint(x, y, color, brushSize)
      if (x === x1 && y === y1) break
      const e2 = 2 * err
      if (e2 > -dy) {
        err -= dy
        x += sx
      }
      if (e2 < dx) {
        err += dx
        y += sy
      }
    }
    return changed
  }

  /** 油漆桶（四连通，按起始像素颜色扩散），返回填充像素数 */
  floodFill(x: number, y: number, color: Rgba): number {
    if (!this.inside(x, y)) return 0
    const target = this.getPixel(x, y)
    if (
      target.r === Math.round(color.r) &&
      target.g === Math.round(color.g) &&
      target.b === Math.round(color.b) &&
      target.a === Math.round(color.a)
    ) {
      return 0
    }
    if (!this.strokeOpen) this.beginStroke()
    const stack: Array<[number, number]> = [[x, y]]
    let filled = 0
    while (stack.length > 0) {
      const next = stack.pop()
      if (!next) break
      const [cx, cy] = next
      if (!this.inside(cx, cy)) continue
      const current = this.getPixel(cx, cy)
      if (
        current.r !== target.r ||
        current.g !== target.g ||
        current.b !== target.b ||
        current.a !== target.a
      ) {
        continue
      }
      if (this.setPixel(cx, cy, color)) filled++
      stack.push([cx + 1, cy], [cx - 1, cy], [cx, cy + 1], [cx, cy - 1])
    }
    return filled
  }

  /** 清空为全透明（可撤销） */
  clear(): void {
    this.beginStroke()
    for (let y = 0; y < this.height; y++) {
      for (let x = 0; x < this.width; x++) {
        this.setPixel(x, y, { r: 0, g: 0, b: 0, a: 0 })
      }
    }
    this.endStroke()
  }

  canUndo(): boolean {
    return this.undoStack.length > 0
  }

  canRedo(): boolean {
    return this.redoStack.length > 0
  }

  undo(): boolean {
    const diff = this.undoStack.pop()
    if (!diff) return false
    const revert: PixelDiff = new Map()
    diff.forEach((value, index) => {
      revert.set(index, [this.data[index], this.data[index + 1], this.data[index + 2], this.data[index + 3]])
      this.data[index] = value[0]
      this.data[index + 1] = value[1]
      this.data[index + 2] = value[2]
      this.data[index + 3] = value[3]
    })
    this.redoStack.push(revert)
    return true
  }

  redo(): boolean {
    const diff = this.redoStack.pop()
    if (!diff) return false
    const restore: PixelDiff = new Map()
    diff.forEach((value, index) => {
      restore.set(index, [this.data[index], this.data[index + 1], this.data[index + 2], this.data[index + 3]])
      this.data[index] = value[0]
      this.data[index + 1] = value[1]
      this.data[index + 2] = value[2]
      this.data[index + 3] = value[3]
    })
    this.undoStack.push(restore)
    return true
  }
}
