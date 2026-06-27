# PCL2 Toast 提示设计规范

## 概述

PCL2 的 Toast 提示是显示在启动器窗口左下角的浮动通知条，用于向用户反馈操作结果、错误信息等简短文本。支持三种类型（蓝色信息、绿色成功、红色错误），带有流畅的弹性进入动画和自动消失机制。

---

## 1. 控件结构

```
StackPanel (PanHint, 容器，左下角定位)
  └── Border (单条 Toast)
        └── TextBlock (文本)
```

- `PanHint` 是一个 `StackPanel`，垂直堆叠多条 Toast
- 每条 Toast 是一个 `Border` + `TextBlock` 的组合
- 最大同时显示 **20** 条，超出后新提示直接忽略

---

## 2. 容器样式 (Border)

| 属性 | 值 | 说明 |
|------|-----|------|
| Height | `26px` | 固定高度 |
| CornerRadius | `0, 6px, 6px, 0` | 只有右上和右下有圆角，左侧贴边 |
| Margin | `-70, 0, 20, 0` | 初始水平偏移，配合动画从左侧滑入 |
| HorizontalAlignment | `Left` | 左对齐 |
| Background | 线性渐变 (90°) | 从左到右的水平渐变 |
| Opacity | `0` → `1` | 初始透明，动画渐显 |

---

## 3. 文本样式 (TextBlock)

| 属性 | 值 |
|------|-----|
| FontSize | `13px` |
| Foreground | `#FFFFFF` (白色) |
| Margin | `33px, 5px, 8px, 5px` (左侧 33px 预留图标空间) |
| TextTrimming | `CharacterEllipsis` (超出部分显示 `...`) |
| TextWrapping | 不换行，单行显示 |

---

## 4. 颜色方案

渐变初始混合比例：**30% 目标色 + 70% 白色**，动画过程中渐变至 **100% 目标色**。

Alpha 值：`215 / 255 ≈ 0.84`（半透明）

### 蓝色 (Blue) — 普通信息

| 位置 | RGBA 值 | Hex (不透明) |
|------|---------|-------------|
| 起始色 (左) | `rgba(37, 155, 252, 0.84)` | `#259BFC` |
| 终止色 (右) | `rgba(10, 142, 252, 0.84)` | `#0A8EFC` |

### 绿色 (Green) — 成功提示

| 位置 | RGBA 值 | Hex (不透明) |
|------|---------|-------------|
| 起始色 (左) | `rgba(33, 177, 33, 0.84)` | `#21B121` |
| 终止色 (右) | `rgba(29, 160, 29, 0.84)` | `#1DA01D` |

### 红色 (Red) — 错误提示

| 位置 | RGBA 值 | Hex (不透明) |
|------|---------|-------------|
| 起始色 (左) | `rgba(255, 53, 11, 0.84)` | `#FF350B` |
| 终止色 (右) | `rgba(255, 43, 0, 0.84)` | `#FF2B00` |

---

## 5. 动画系统

### 5.1 进入动画

Toast 从左侧滑入窗口，同时淡入显示。

| 属性变化 | 时长 | 延迟 | 缓动函数 | 说明 |
|---------|------|------|---------|------|
| X 位移 | `400ms` | `0ms` | `EaseOutElastic (Weak)` | 从 -70px 滑到 0px，带弹性回弹 |
| X 微调 | `200ms` | `0ms` | `EaseOutFluent` | +30px → +20px 的微调 |
| 透明度 | `100ms` | `0ms` | `Linear` | 从 0 → 1 |
| 颜色渐变 | `250ms` | `100ms` | `Linear` | 从白色混合 → 目标色 |
| 高度 (多条) | `150ms` | `0ms` | `EaseOutFluent` | 从 0px → 26px |

> **首条 Toast**：高度直接设为 26px，无高度动画。
> **后续 Toast**：先播放高度展开动画 (0 → 26px)。

### 5.2 显示时长

Toast 在屏幕上停留的时间由以下公式决定：

```
显示时长 = (800 + clamp(文本长度, 5, 23) × 180) × AniSpeed
```

- `文本长度`：字符数，限制在 5~23 之间
- `AniSpeed`：全局动画速度倍率（默认为 1）

**示例**：
- 5 字符：`(800 + 5 × 180) × 1 = 1700ms`
- 10 字符：`(800 + 10 × 180) × 1 = 2600ms`
- 23 字符：`(800 + 23 × 180) × 1 = 4940ms`

### 5.3 退出动画

显示时长结束后，Toast 向左滑出并消失。

| 属性变化 | 时长 | 延迟 | 缓动函数 | 说明 |
|---------|------|------|---------|------|
| X 位移 | `200ms` | = 显示时长 | `EaseInFluent` | 从 0px → -50px |
| 透明度 | `150ms` | = 显示时长 | `EaseInFluent` | 从 1 → 0 |
| 高度 | `100ms` | 等 X/Opacity 完成后 | `EaseOutFluent` | 从 26px → 0px |
| 移除控件 | — | 高度完成后 | — | 从 StackPanel 中移除 |

### 5.4 重复提示闪烁动画

当相同文本的 Toast 再次出现时，不新增控件，而是对已有控件执行"闪烁"动画：

```
时序（各 50ms，共 200ms）：
  X: 当前位置 → -12px → -8px → +8px → -8px

同时（250ms）：
  颜色：白色混合 → 目标色

然后重新启动退出计时。
```

---

## 6. PanHint 容器定位

```xml
<StackPanel x:Name="PanHint"
    IsHitTestVisible="False"
    UseLayoutRounding="True"
    SnapsToDevicePixels="True"
    HorizontalAlignment="Left"
    VerticalAlignment="Bottom"
    Margin="0,0,0,20"
    Grid.RowSpan="2" />
```

| 属性 | 值 | 说明 |
|------|-----|------|
| 位置 | 左下角 | `HorizontalAlignment="Left"` + `VerticalAlignment="Bottom"` |
| 底部间距 | `20px` | `Margin="0,0,0,20"` |
| 交互 | 不可点击 | `IsHitTestVisible="False"` |
| 最大数量 | `20` 条 | 超出后新提示被忽略 |

---

## 7. 日志系统集成

Toast 与日志系统通过 `LogBehavior` 枚举关联：

| LogBehavior | 行为 |
|-------------|------|
| `None` | 不触发任何提示 |
| `ToastIfDebug` | 仅调试模式下弹出蓝色 Toast |
| `Toast` | 弹出红色 Toast + 上报遥测 |
| `Alert` | 弹窗 (MyMsgBox) |
| `AlertThenFeedback` | 弹窗 + 询问是否反馈 |
| `AlertThenCrash` | 弹窗 + 崩溃退出 |

调用方式：

```vb
' 直接调用
Hint("操作成功", HintType.Blue)

' 通过日志系统触发
Logger.Error(ex, "加载失败", LogBehavior.Toast)
```

---

## 8. CSS 实现参考

```css
/* ========== 容器 ========== */
.toast-container {
  position: fixed;
  left: 0;
  bottom: 20px;
  display: flex;
  flex-direction: column;
  gap: 0;
  pointer-events: none;
  z-index: 9999;
}

/* ========== 单条 Toast ========== */
.toast {
  height: 26px;
  border-radius: 0 6px 6px 0;
  opacity: 0;
  margin-left: -70px;
  margin-right: 20px;
  transform: translateX(-70px);
  animation: toast-slide-in 0.4s cubic-bezier(0.2, 0.8, 0.3, 1.1) forwards;
  overflow: hidden;
}

/* 颜色类型 */
.toast--blue {
  background: linear-gradient(90deg, rgba(37,155,252,0.84), rgba(10,142,252,0.84));
}
.toast--green {
  background: linear-gradient(90deg, rgba(33,177,33,0.84), rgba(29,160,29,0.84));
}
.toast--red {
  background: linear-gradient(90deg, rgba(255,53,11,0.84), rgba(255,43,0,0.84));
}

/* 文本 */
.toast-text {
  color: #fff;
  font-size: 13px;
  line-height: 26px;
  padding: 0 8px 0 33px;
  white-space: nowrap;
  overflow: hidden;
  text-overflow: ellipsis;
  margin: 0;
}

/* ========== 进入动画 ========== */
@keyframes toast-slide-in {
  0%   { transform: translateX(-70px); opacity: 0; }
  60%  { transform: translateX(4px); opacity: 1; }
  80%  { transform: translateX(-2px); }
  100% { transform: translateX(0); opacity: 1; }
}

/* ========== 退出动画 ========== */
.toast--hiding {
  animation:
    toast-slide-out 0.2s ease-in forwards,
    toast-fade-out 0.15s ease-in forwards,
    toast-collapse 0.1s ease-out 0.15s forwards;
}

@keyframes toast-slide-out {
  to { transform: translateX(-50px); }
}
@keyframes toast-fade-out {
  to { opacity: 0; }
}
@keyframes toast-collapse {
  to { height: 0; margin-bottom: 0; }
}

/* ========== 闪烁动画 (重复提示) ========== */
@keyframes toast-shake {
  0%   { transform: translateX(0); }
  25%  { transform: translateX(-12px); }
  50%  { transform: translateX(-8px); }
  75%  { transform: translateX(8px); }
  100% { transform: translateX(-8px); }
}

.toast--shake {
  animation: toast-shake 0.2s ease-in-out;
}
```

---

## 9. JavaScript 实现参考

```javascript
class ToastManager {
  constructor(container, options = {}) {
    this.container = container;
    this.maxCount = 20;
    this.baseDuration = 800;
    this.charDuration = 180;
    this.speed = options.speed ?? 1;
  }

  show(text, type = 'blue') {
    if (this.container.children.length >= this.maxCount) return;

    // 检查重复
    const existing = this.findDuplicate(text);
    if (existing) {
      this.shake(existing, type);
      return;
    }

    // 创建元素
    const el = document.createElement('div');
    el.className = `toast toast--${type}`;
    el.innerHTML = `<span class="toast-text">${this.escapeHtml(text)}</span>`;
    el.dataset.text = text;
    this.container.appendChild(el);

    // 自动退出
    const duration = this.calcDuration(text);
    setTimeout(() => this.dismiss(el), duration);
  }

  calcDuration(text) {
    const len = Math.min(Math.max(text.length, 5), 23);
    return (this.baseDuration + len * this.charDuration) * this.speed;
  }

  dismiss(el) {
    el.classList.add('toast--hiding');
    el.addEventListener('animationend', () => el.remove(), { once: true });
  }

  findDuplicate(text) {
    return [...this.container.children].find(
      el => el.dataset.text === text && !el.classList.contains('toast--hiding')
    );
  }

  shake(el, type) {
    el.className = `toast toast--${type} toast--shake`;
    const duration = this.calcDuration(el.dataset.text);
    setTimeout(() => this.dismiss(el), duration);
  }

  escapeHtml(str) {
    const div = document.createElement('div');
    div.textContent = str;
    return div.innerHTML;
  }
}
```

---

## 10. 设计要点总结

1. **左下角定位**：Toast 固定在窗口左下角，不遮挡主要内容
2. **弹性进入**：使用 `EaseOutElastic` 缓动，有轻微回弹效果，吸引注意力
3. **自动消失**：显示时长与文本长度成正比，短文本快速消失，长文本多留阅读时间
4. **半透明背景**：Alpha ≈ 0.84，不完全遮挡底层内容
5. **右侧圆角**：左侧贴边无缝，只有右侧圆角，视觉上像从边缘滑出
6. **去重机制**：相同文本不重复堆叠，而是闪烁提示
7. **最大数量限制**：最多 20 条，防止界面被淹没
