# 致谢与历史说明

## Axolotl Launcher

2026 年 8 月引入的三个功能——**合成配方生成器**、**渐变文字生成器**、**AI 模组翻译**——
在最初实现时参考并使用了 [Axolotl Launcher](https://github.com/Axolotl-Launcher/Axolotl)
（GPL-3.0）的代码。这一点在此如实记录，不做抹除。

期间还发生过一件事需要更正：提交 `e45fd33`「清理配方生成器注释中的第三方来源表述」
把源码里原本写明的来源说明删掉了。**这是错误的处理方式**，我们为此致歉——
正确的做法是在此保留来源说明并解决授权问题，而不是删掉痕迹。

### 署名位置

| 位置 | 说明 |
|---|---|
| [README.md](README.md) 的「鸣谢 → 特别感谢」 | 公开仓库首页可见 |
| 应用内「设置 → 关于 → 特别鸣谢」 | `src-tauri/resources/about/acknowledgements.txt` |
| 应用内「设置 → 关于 → 许可与版权声明」 | `src-tauri/resources/about/licenses.txt` |
| 本文件 | 完整历史说明与处理过程 |

## Plain Craft Launcher 2 (PCL2)

项目开发初期参考并移植过 [PCL2](https://github.com/Meloong-Git/PCL)（《PCL 分发有限许可》）的部分实现，
涉及启动流程、崩溃分析与若干界面说明文案。该许可只授予「原样副本」的免费分发权，不授予修改与派生权，
所以这部分只能靠重写解决，署名不能替代。

处理情况：崩溃分析器已按「采集 → 检测 → 评分」重构、魔数改为运行时随机生成、注释中的方法名引用清除、
界面说明文案按自有措辞重写（记录见 `CHANGELOG.md` 的 2026-08-05 与 2026-10-25 两条）。署名位置同上表。

按该指南的划分，本项目现状属「轻度使用」：早期参考的内容已重写，剩下的只是术语与文件格式兼容，
而启动、下载、Mod 管理是同类软件的必需功能，不构成「参考大量内容重实现」。轻度对应的两条义务是
署名、不与 PCL 混淆或暗示关联，均已满足——README 与应用内致谢均列名，且两处都写明无隶属关系。
另外，非正版账号累计启动若干次后启动器会提示购买正版。
## 当前处理

本项目的分发许可证保持不变。对上述受影响的功能，处理方式是**按公开规范独立重写**，
过程记录在 [`specs/cleanroom/`](specs/cleanroom/)：

| 功能 | 状态 | 规格 |
|---|---|---|
| 渐变文字生成器 | 已按规格重写 | [`gradient-text.md`](specs/cleanroom/gradient-text.md) |
| 合成配方生成器 | 已按规格重写 | [`recipe-generator.md`](specs/cleanroom/recipe-generator.md) |
| AI 模组翻译 | 已按规格重写 | [`mod-translation.md`](specs/cleanroom/mod-translation.md) |

规格只依据公开资料撰写：JVM 规范（JVMS 第 4 章）、ZIP 容器格式、Minecraft 数据包配方文档、
各聊天插件公开语法、以及原版界面截图上的实测坐标。规格中不包含任何第三方实现的内部细节。

重写覆盖范围（`e2081f8d` 之后的提交）：`src-tauri/src/mod_translation` 全部 58 个文件、
`src/utils/gradient-text` 全部 11 个文件、`src/utils/recipe-generator` 与配方工作台共 11 个文件，
含各自测试。工作区内部文件名与续传身份也已改为本项目自有前缀，仓库中不再出现历史前缀字符串。

## 相关讨论

版权异议与处理进展见 issue #17：https://github.com/MoTeam-cn/MoLaunch/issues/17

## 其他致谢

其他第三方资源与工具的致谢见 [README.md](README.md) 的「鸣谢」章节与
[`src-tauri/resources/about/acknowledgements.txt`](src-tauri/resources/about/acknowledgements.txt)。
