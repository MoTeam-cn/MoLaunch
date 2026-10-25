# 规格：合成配方生成器

依据：Minecraft 数据包「配方（Recipe）」公开文档与各版本变更记录、原版工作台界面截图。
目标：把界面上的槽位填写内容，转换成对应游戏版本可用的配方 JSON，并给出文件路径与预览。

## 1. 槽位模型

| 槽位 | 含义 |
|---|---|
| `crafting.1` … `crafting.9` | 3×3 合成格，行优先 |
| `crafting.result` | 合成产出 |
| `cooking.ingredient` / `cooking.result` | 熔炼系（熔炉/高炉/烟熏/营火）投入与产出 |
| `stonecutter.ingredient` / `stonecutter.result` | 切石机 |
| `smithing.template` / `base` / `addition` / `result` | 锻造台 |

槽位值有四种：内置物品、内置标签、自定义物品、自定义标签；每种都带数量（默认 1）。

## 2. 工作台面板几何

- 画布 696×292，面板为原版界面截图。
- 3×3 合成格：原点 (117, 64)，槽位 64×64，步进 71（= 64 + 7 间隔）；第 r 行第 c 列（0 起）左上角 = (117 + 71c, 64 + 71r)。
- 其余槽位坐标由截图量取（单位 px，左上/右下）：

| 面板 | 槽位 | 盒子 |
|---|---|---|
| 工作台 | result | (476,121)-(571,215) |
| 熔炉系 | ingredient / result | (220,64)-(283,127) / (444,120)-(539,215) |
| 营火 | ingredient / result | (142,132)-(203,195) / (512,132)-(575,195) |
| 切石机 | ingredient / result | (76,129)-(139,191) / (552,112)-(647,207) |
| 锻造台 | template / base / addition / result | (28,188)-(92,252) / (99,188)-(163,252) / (170,188)-(234,252) / (388,188)-(451,251) |

界面按盒子渲染可点击热区与图标，图标尺寸 = min(盒宽, 盒高) / 696 × 舞台宽度 × 0.9。

## 3. 版本矩阵

| 能力 | 引入版本 |
|---|---|
| 配方类型 `crafting_shaped` / `crafting_shapeless` / `smelting` / `blasting` / `smoking` / `campfire_cooking` / `stonecutter` / `smithing` / `smithing_trim` / `smithing_transform` | 按各版本公开变更记录 |

生成时按目标版本决定：

1. **类型名**：1.14 起写命名空间（`minecraft:crafting_shaped`），1.13 及以前写裸名。
2. **物品引用**：1.13 起为字符串 `"minecraft:iron_ingot"`；1.12 为对象 `{"item": "iron_ingot", "data": 0}`，并在 result 中单列 `data`。
3. **配方结果**：1.12 为 `{"item": …, "data": …, "count": n}`；1.13+ 为 `{"item": …, "count": n}`；1.20 起的 `cooking` 系用对象形式，之前用字符串。
4. **标签引用**：1.13 起可写 `"#minecraft:planks"`；1.12 不支持标签（校验期直接拒绝）。
5. **数量**：1.21.2 起 ingredient 支持 `count` 字段，之前不支持（忽略并给出提示）。
6. **锻造**：1.21.5 起支持 `pattern` 字段；`smithing_trim` 必须提供 pattern。
7. **目录**：1.21 之前写 `data/<ns>/recipes/`，1.21 起写 `data/<ns>/recipe/`。
8. **pack.mcmeta**：按版本写 `pack_format` 数字；新版本支持 `[min, max]` 区间。

## 4. 生成规则

- 有形状合成：按填了内容的槽位求最小包围盒，逐行输出 pattern（每行一个字符），并给出 `key` 映射；`keepWhitespace` 为真时保留空列/空行。
- 无形状合成：输出扁平 ingredient 列表。
- 2×2 模式：禁用 3、6、7、8、9 号槽位，界面用屏障图标占位。
- 名称：`[a-z0-9_./-]+`，默认 `my_recipe`；同时生成 `data/<ns>/recipe(s)/<name>.json`。
- 自定义标签：写入 `data/<ns>/tags/item(s)/<name>.json`（目录名同样随版本变化）。

## 5. 校验

| 情况 | 结果 |
|---|---|
| 缺少 ingredient 或 result | 报错 |
| result 槽位放标签 | 报错 |
| 1.12 使用标签 | 报错 |
| `smithing_trim` 在支持 pattern 的版本上缺 pattern | 报错 |
| 1.21.2 之前的版本填了 ingredient 数量 | 忽略数量并提示 |

错误信息面向中文用户，直接指出哪个槽位有问题。

## 6. 验收用例

1. 1.20 + 有形状配方：pattern 与 key 正确、result 为对象、物品为字符串。
2. 1.21.2 + ingredient 数量：数量写进 ingredient。
3. 1.12 + 有形状配方：物品为 `{item, data}`、类型为裸名、result 含 data。
4. 1.21 + 无形状：ingredient 为扁平数组。
5. `smithing_trim` 在 1.21.5：写出 pattern 且不含 result。
6. 目录名随版本切换 `recipes/` ↔ `recipe/`。
7. 2×2 模式：3/6/7/8/9 号槽位被禁用。
8. 每种配方类型的槽位盒与 §2 表格逐像素一致，且互不重叠、不越出面板。
