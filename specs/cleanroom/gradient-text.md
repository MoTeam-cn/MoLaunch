# 规格：渐变文字工具

依据：Minecraft 聊天颜色码公开文档、各插件公开语法、线性插值定义。本规格不含任何第三方实现细节。

## 1. 输入

```
document : 行数组；每行是若干 run；每个 run = { text, formats[] }
formats  : bold | italic | underlined | strikethrough | obfuscated（顺序固定，按此顺序输出）
colors   : 颜色停靠点数组，每项为 #RGB 或 #RRGGBB
options  : { control: '&' | '§', simplifyGradient: boolean }
```

## 2. 颜色分配

1. 把每个 run 按 Unicode 码点切分（`Array.from`，代理对不拆）。
2. 统计「可见字符数」= 不属于 `\s` 的字符个数。
3. 用色标做**等距线性插值**得到 N 个颜色，N = 可见字符数：
   - 色标数 = 0 → 不产生颜色
   - 色标数 = 1 或 N = 1 → 全部使用该色标
   - 否则：第 i 个颜色对应位置 `t = i / (N-1)`，在色标序列上按段线性插值，逐通道 `round`
4. 按出现顺序给可见字符上色；空白字符无色、不消耗色标。
5. 行与行之间插入换行单元（无色、无格式）。

## 3. 颜色归一化

- 接受 `#RGB` / `#RRGGBB`（可省略 `#`），统一输出 `#RRGGBB` 大写
- 从自由文本提取色标：先按出现顺序取所有 `#RGB|#RRGGBB`，再按出现顺序取所有 `rgb()/rgba()` 三元组，合并后去重保序

## 4. 阴影色（原版渲染规则）

原版 Minecraft 以「同色相、同饱和度、亮度 = 通道最大值 ÷ 4」绘制阴影：
`shadow = HSL(hue(rgb), sat(rgb), max(rgb)/255/4)`，转回 RGB 后取整。输入为空或非法时返回 `#000000`。
（例：`#FF0000` → `#800000`；`#00FF00` → `#008000`；`#808080` → `#202020`）

## 5. 输出格式

逐字符格式：每个字符前写颜色前缀，字符后不写后缀；格式标记按 §6 包裹。

| id | 颜色前缀 | 颜色后缀 | 备注 |
|---|---|---|---|
| vanilla | `<ctl>#RRGGBB` | — | `ctl` 取 options.control |
| vanilla-compatible | `<ctl>x` + 6×(`<ctl>` + 位) | — | 1.16 原版十六进制写法 |
| standard | `#RRGGBB` | — | |
| cmi | `{#RRGGBB}` | — | |
| minimessage | `<#RRGGBB>` | — | |
| minedown | `&#RRGGBB&` | — | |
| trchat | `&{#RRGGBB}` | — | |
| taboolib | `&{#RRGGBB}` | — | 与 trchat 语法相同 |
| chat-colors | `[#RRGGBB]` | — | |
| motd | `\\u00A7x` + 6×(\\u00A7 + 位) | — | 以转义文本形式输出 |
| bbcode | `[color=#RRGGBB]` | `[/color]` | |
| terraria | `[c/RRGGBB:` | `]` | 颜色不带 # |

整段格式（把同色连续字符合成一段）：

| id | 语法 |
|---|---|
| minimessage-gradient | `<gradient:#c1:#c2…>文本</gradient>` |
| taboolib-gradient | `[文本](gradient=#c1,#c2…;格式字母…)`，格式字母 b/i/u/s/o |
| rosegarden-gradient | `<g:#c1:#c2…>文本` |

- 分段规则：格式集合变化或遇到无色字符时切段；换行处结束当前段。
- `simplifyGradient = true` 且段内颜色多于 1 个时，只保留首尾两个色标。

整文档格式：

| id | 语法 |
|---|---|
| json | 每个字符一个组件：`{"text":"X","color":"#RRGGBB","bold":true,…}` 的 JSON 数组 |
| snbt | `[{text:'X',color:'#RRGGBB',bold:true}]`，单引号转义 `\\` 与 `'` |
| csv | 表头 `color,char,bold,italic,underlined,strikethrough,obfuscated`；换行字符记作 `\\n`；含 `",` 或换行的字段加引号 |
| html | 每行一个 `<p>`；空格转 `&nbsp;`；颜色包 `<span style="color: #RRGGBB;">`；其余字符做 HTML 转义 |

## 6. 格式标记

| 格式 | 原版码（vanilla / motd） | 成对标记（minimessage / html） | minedown | chat-colors | bbcode | cmi/trchat/taboolib |
|---|---|---|---|---|---|---|
| bold | l | `<b></b>` | `**` | `**` | `[b][/b]` | `&l` |
| italic | o | `<i></i>` | `##` | `*` | `[i][/i]` | `&o` |
| underlined | n | `<u></u>` | `__` | `__` | `[u][/u]` | `&n` |
| strikethrough | m | `<st></st>` | `~~` | `~~` | `[s][/s]` | `&m` |
| obfuscated | k | `<obf></obf>` | `??` | 无 | 无 | `&k` |

- 原版码写法：`<ctl>` + 字母，紧跟颜色前缀之后，逐字符重复。
- 成对标记：按 formats 顺序由内向外包裹（第一个格式在最内层）。

## 8. 本地状态

- localStorage 键：`molaunch.lab.gradient-text.v1`
- 保存字段：`{ version: 1, document, colors, adapterId, control, simplifyGradient, presets[] }`
- 读取时逐字段校验：非法颜色丢弃；未知 `adapterId` 回退 `vanilla`；`control` 非 `&`/`§` 时回退 `§`；预设无色标则丢弃并重命名
- 兼容旧字段名：`vanillaCharacter` → `control`，`simplifyGradients` → `simplifyGradient`
- 预设导出只保留 `{ name, colors }`，便于跨设备迁移

## 9. 文档操作

- 纯文本 ⇄ 文档：按 `\n` 切行，一行一个 run
- 归一化：丢弃 `text` 非字符串的 run；`formats` 去重并按 §6 表顺序排列；空文档补一行空 run
- 默认文档：单行 `MoLaunch`；默认色标 `#A855F7 → #22C55E`

## 7. 验收用例

| 输入 | 期望 |
|---|---|
| 单色 `#0F0F0F` + 文本 `A`，vanilla，ctl=`&` | `&#0F0F0FA` |
| 双色 `#000000→#FFFFFF` + `AB`，vanilla，ctl=`§` | `§#000000A§#FFFFFFB` |
| `Mo 🦎` + `#111111→#222222` | M=#111111、o=#1A1A1A、空格无色、🦎=#222222 |
| 三色 `#000000,#808080,#FFFFFF` + `ABC`，minimessage-gradient，简化开 | `<gradient:#000000:#FFFFFF>ABC</gradient>` |
| 两行 `A`(bold,italic) / `B`，vanilla | `&#111111&l&oA\n&#222222B` |
| 同上，json | `[{"text":"A","color":"#111111","bold":true,"italic":true},{"text":"\\n"},{"text":"B","color":"#222222"}]` |
| `#FF0000` 阴影色 | `#800000` |
