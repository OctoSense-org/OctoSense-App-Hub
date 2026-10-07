# 关联应用预览版准入

[English](README.md) | 简体中文

目录序列 **7** 收录 ymote 发布的三款独立安装的 macOS 开发者预览应用。
用户要求完成运行时交付和目录收录，已授权发布。独立评审子代理审查了应用包、
权限、隐私说明和兼容性修复。本记录区分代理审查与用户发布授权，不宣称已有
独立人工完成真实提供方验收。

| 应用 | 发布者提交 | 提交工单 |
| --- | --- | --- |
| GitHub Notes | `5f0c4c6b13bddae87a4945b2b76ca2a416793f14` | [#121](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/121) |
| Inbox Assistant | `28dd24a39a7e9668b877dd84d9577d10b3f87363` | [#122](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/122) |
| Google Calendar | `c0783291a528689008d7213f1b1163c9c15ac874` | [#123](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/123) |

[admission.json](admission.json)记录包摘要、源码标签、公开发布文件验证和目录标识。
三款最终签名应用包均通过 `hub check`；`hub publish` 再次执行准入检查，复制
原始文件并使用现有已认证工作密钥签署目录。每个工件文件均按 SHA-256 与发布者
源码比对。索引条目是目录准入条目的精确副本，相邻 pack 文件包含相同应用包。
未收录签名私钥、提供方凭据、私人配置或原始运行日志；公共信任锚保持不变。

准入前评审覆盖可见功能、平台与分类、最小必要权限、误导性界面、嵌入的助手指令、
冒犯性或私人内容、工具风险及路由。结论限于已披露的 macOS 开发者预览范围。
涉及私人提供方数据的工具默认不可向其他应用分享。Notes 没有写入工具，Inbox
没有发送工具，Calendar 没有预约工具。账户管理和外部操作确认由宿主管理。
发布者隐私政策与修正后的商店摘要说明了通过提供方及模型服务产生的网络访问。

兼容性评审发现：旧目录读取器会拒绝展示元数据中的 `host_method`。修复只在
目录工具摘要中省略该字段，原始签名 `tools.json` 保持逐字节一致，执行时仍读取
其中映射。回归测试验证旧字段结构解析、规范签名数据、安装后的映射不变，以及
篡改和禁止路由被拒绝。这是旧版数据格式测试，不是运行旧桌面二进制的证明。

这些记录证明准入和完整性，不证明真实 GitHub/Google 登录、提供方写入、物理
点击确认、Android Google 登录、Windows/Linux 体验或长期运行无内存增长。
历史原生/模拟提供方及模型证据在发布者仓库中保留原始源码摘要。

## 官方商店验收

[公开 HTTP 检查](official-http.json)验证了全部 21 个目录、索引和工件地址，
以及默认信任锚和未改动的发布者签名应用包。

[macOS Apple Silicon 预览版](https://github.com/OctoSense-org/OctoSense/releases/tag/desktop-v0.1.0-beta.2)
对应源码 `84e3438b7ee25b43ba945bfe063db594a71e61ba`。普通打包应用在全新的隔离
配置中，从官方目录搜索、安装并打开三款应用，没有替换目录或信任锚。
[九项功能检查和六项独立视觉审查](native-store/receipt.json)通过，包括完整进程
重启后本地草稿的精确恢复。记录绑定运行时、可执行文件、安装包、内核和应用包
摘要。候选版本的原始记录保持不变，发布不会改写测试历史。

| 应用 | 恢复状态与原始截图 |
| --- | --- |
| GitHub Notes | [Unicode Markdown 与渲染预览](native-store/notes-restored-preview.png) |
| Inbox Assistant | [在 Reply/Chat 间保留的第 2 版虚构回复](native-store/inbox-restored-reply.png) |
| Google Calendar | [本地事件草稿](native-store/calendar-restored-draft.png)及[时区](native-store/calendar-restored-timezone.png) |

[商店](native-store/official-store.png)和[已安装应用库](native-store/installed-library.png)
也保留原始像素，所有图片均未编辑。

验收保留了 **14 次只读截图错误**、零输入错误或重放，以及一次因自动化使用滚动
完成前的坐标而停止、恢复的记录。修正后的驱动先观察新画面，再计算点击位置。
这证明上述本地功能，不是无中断的仪器测试、延迟结果或无故障 UX 长测。
本次未连接 OAuth、读取真实提供方数据、调用模型、远程写入或测试物理发送/保存确认。
