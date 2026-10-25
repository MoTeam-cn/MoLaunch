# 致谢与历史说明

## Axolotl Launcher

2026 年 8 月引入的三个功能——**合成配方生成器**、**渐变文字生成器**、**AI 模组翻译**——
在最初实现时参考并使用了 [Axolotl Launcher](https://github.com/Axolotl-Launcher/Axolotl)
（GPL-3.0）的代码。这一点在此如实记录，不做抹除。

期间还发生过一件事需要更正：提交 `e45fd33`「清理配方生成器注释中的第三方来源表述」
把源码里原本写明的来源说明删掉了。**这是错误的处理方式**，我们为此致歉——
正确的做法是在此保留来源说明并解决授权问题，而不是删掉痕迹。

## 当前处理

本项目的分发许可证保持不变。对上述受影响的功能，处理方式是**按公开规范独立重写**，
过程记录在 [`specs/cleanroom/`](specs/cleanroom/)：

| 功能 | 状态 |
|---|---|
| 渐变文字生成器 | 已按规格重写（`specs/cleanroom/gradient-text.md`） |
| 合成配方生成器 | 重写进行中（`specs/cleanroom/recipe-generator.md`） |
| AI 模组翻译 | 重写进行中（`specs/cleanroom/mod-translation.md`） |

规格只依据公开资料撰写：JVM 规范（JVMS 第 4 章）、ZIP 容器格式、Minecraft 数据包配方文档、
各聊天插件公开语法、以及原版界面截图上的实测坐标。规格中不包含任何第三方实现的内部细节。

## 相关讨论

版权异议与处理进展见 issue #17：https://github.com/MoTeam-cn/MoLaunch/issues/17

## 其他致谢

（此处后续补充其他第三方资源与工具的致谢。）
