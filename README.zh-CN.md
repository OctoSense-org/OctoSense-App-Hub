# OctoSense app hub

[English](README.md) | 简体中文

这里是为 OctoSense 发布的应用索引、每个 OctoSense 商店都会读取的签名目录、App Hub 为每个已准入应用包保存的副本，以及运行 hub 和商店的代码。发布者的应用源码留在各自仓库中；本仓库也包含商店 UI、运行宿主、模板和测试夹具。

| 想找 | 仓库 |
| --- | --- |
| 如何开发应用：快速上手、脚本 API、脚本应用模板、设计流程、示例 | [OctoScript-App-Design-Flow](https://github.com/OctoSense-org/OctoScript-App-Design-Flow) |
| AppCard 助手运行时（在 Shell 中需 `--features app-appcard` 才启用） | [OctoSense `apps/appcard`](https://github.com/OctoSense-org/OctoSense/tree/main/apps/appcard) |
| 第一方系统应用（新闻、相册、地图、相机、邮件、AI providers）及其宿主服务（`mail`、`llm`） | [OctoSense `apps/`](https://github.com/OctoSense-org/OctoSense/tree/main/apps) |
| L0 解析器/检查器及其 Makepad 转换/渲染层 | [Octoscript](https://github.com/OctoSense-org/Octoscript) 与 [Octoscript-Makepad](https://github.com/OctoSense-org/OctoScript-Makepad) |
| 应用包格式、准入检查、签名、提交与商店 | 本仓库 |

**要开发应用？** 请从 [OctoSense-org 主页](https://github.com/OctoSense-org)的“请按顺序先阅读”列表开始（先读 OctoScript-App-Design-Flow 的 `AGENTS.md`，再读其 `docs/QUICKSTART.md`）。你只需要把本仓库作为兄弟目录克隆下来，用于构建 `hub` 和 `card-host`，以及提交应用（在这里开 issue，见[提交](docs/PUBLISHING.md#submitting)）。不要修改 `catalog.json`、`index/` 或 `artifacts/`。

| 路径 | 说明 |
| --- | --- |
| `catalog.json` | 签名目录。商店在展示任何内容之前，先用下方的信任锚验证它。 |
| `index/<app>-<version>.json` | 每个应用版本一条已准入条目：清单、发布者、源码位置与状态。维护者在 `hub publish` 后从目录导出已准入条目；尚无已发布应用时不存在。 |
| `artifacts/<app>-<version>.bundle/` | App Hub 保存的应用包副本，与审核时的字节完全一致。由 `hub publish` 生成。 |
| `artifacts/<app>-<version>.bundle.pack.json` | 打成单个文件的同一应用包，商店下载的就是它。 |
| `docs/FIRST-APP.md` | 第一个应用（卡片应用或脚本应用）的完整演练：编写、打包、运行、截图、验证与提交。 |
| `docs/PUBLISHING.md` | 应用包、商店信息、能力、宿主服务、准入规则、签名与提交的完整规范。 |
| `docs/ICONS.md` | 规范图标的归属、导出约束与视觉审查。 |
| `docs/DEVELOPMENT.md` | 应用编写指南所在的仓库、交付路径，以及 `card-host` 及其远程控制路由。 |
| `templates/app/` | 卡片应用仓库脚手架，包含元数据、示例图标和链接好的 Agent 指引。 |
| `crates/app-contract` | 应用契约，即 crates.io 上的 `octosense-app-contract`（OctoSense ADR 0005）：清单、应用获得的策略、包完整性校验与运行包所需的内容。应用和宿主按版本依赖它；在 `1.x` 内只做增量变更（[README](crates/app-contract/README.md)）。 |
| `crates/app-policy` | 签名清单与商店信息、准入，以及解析为隔离环境设置和 Agent 会话配置（ADR 0002）；应用自带的 Agent（`tools.json`、`AGENT.md`、技能）加载为 `AgentBundle`；原生模块的工具清单共用同一套 `tools.json` 解析与检查（`ToolManifest::load`）。并重新导出应用契约。 |
| `crates/app-hub` | 索引、签名目录、准入检查、Agent 扫描、设备端客户端和 `hub` 命令（ADR 0003）。 |
| `crates/appstore` | 作为 OctoSense 模块的商店；把已安装应用作为独立客户端运行的 `card` 模块；系统应用（`os.` 前缀 id）；以及宿主服务及其面板。 |
| `crates/appstore-app` | 作为独立应用的商店（`appstore`）。 |
| `crates/card-host` | 隔离运行单个应用包（卡片应用或脚本应用）的参考宿主（`card-host`）。 |
| `crates/card-studio` | 在隐藏的 `card-host --remote` 中按速览卡片、手机和桌面尺寸渲染卡片，运行测量检查，并准备视觉评审请求（`card-studio`，ADR 0002 第 7 节）。 |
| `skills/card-studio` | 基于 `card-studio` 的 octos 技能：`card_render`、`card_critique_payload`。 |
| `crates/app-host` | 单窗口宿主，可把任意 OctoSense AppModule 作为独立应用运行。 |
| `crates/app-hub-app` | 每个 OctoSense Shell 都会链接的集成：原生商店模块、`card` 运行模块、由 `OCTOSENSE_SYSTEM_APPS` 指定的系统应用、已安装应用和图标（[README](crates/app-hub-app/README.md)）。 |

这些 crate 基于固定版本的 OctoSense Makepad 与 Octoscript 分支构建，和启动器工作区一样，从同级检出目录（`../makepad`、`../octoscript-makepad`、`../octoscript`）解析依赖。`cargo test --workspace` 以无界面方式运行策略、准入检查、签名和商店测试；`cargo run -p octosense-app-hub --bin hub` 是发布工具。

## 代码导读

先看[代码导读中的宿主选择与启动命令（英文）](docs/CODE-WALKTHROUGH.md#2-run-the-right-host)，
再追踪应用包如何进入 UI，以及宿主服务的应答如何回到回调函数。“总结已保存的笔记”
这一请求串起应用 Agent 的对话、账户存储与工具授权。导读区分原生 `AppModule`、
Splash 脚本和 L0 卡片，并把 crate 索引放在最后。仓库协作规则见 [AGENTS.md](AGENTS.md)。

系统应用申请跨应用代理工具时，需要宿主明确提供接纳范围。Shell 在准备应用前，
按应用 id 调用 `system::set_agent_tool_offer`。仅 manifest 请求的名称可以获接纳；
工具所有者的共享声明、调用者授权与实际执行器仍由 Shell 分别检查。提供工具范围
不会启动代理，也不授予脚本 UI 原始宿主服务权限；商店应用的默认规则不变。

主机账户和审核面板采用模态输入：文字、键盘、输入法、剪贴板和指针抬起事件
只交给可见的主机面板，定时器及服务回调仍能到达应用。运行器在执行应用控件前
保存自身卡片和面板引用，应用不能通过重复控件 ID 替换主机界面。
`services::is_sheet_input_event` 为集成前台 Glance 主机提供相同输入边界。

## 信任锚

商店信任这个锚，并沿着它的证书找到为目录签名的工作密钥。轮换工作密钥不需要发布新版商店。

```
6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11
```

## 让商店指向这里

商店构建默认读取本 hub 并信任上述锚。下面的环境变量可以覆盖它们，用于镜像或开发用的 hub；写全之后，默认值如下：

```sh
OCTOSENSE_HUB=https://raw.githubusercontent.com/OctoSense-org/OctoSense-App-Hub/main/ \
OCTOSENSE_HUB_ANCHOR=6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11 \
appstore
```

## 发布应用

应用分为卡片应用（`page.card`）和脚本应用（`main.splash`）。从[开发你的第一个 Hub 应用](docs/FIRST-APP.md)开始。完整规范见[发布](docs/PUBLISHING.md)，图标素材见[图标](docs/ICONS.md)；[开发指南导航](docs/DEVELOPMENT.zh-CN.md)链接了其他仓库中的编写与测试指南。

为应用包打戳，截图，重新打戳，运行 `hub check` 和 `hub scan`，为清单签名，然后在本仓库开一个 issue 提交（[提交](docs/PUBLISHING.md#submitting)）。目前还没有发布用的 Action，也没有独立的索引仓库：由维护者对你所打 tag 的那个提交的确切字节运行 `hub publish`，并提交签名后的目录。用 `hub withdraw` 撤回某个版本，每个商店在下次拉取时都会遵守；`hub remove` 用于删除本不该发布的条目。

## 应用

| 应用 | 版本 | 分类 | 运行平台 | 发布者 | 允许的权限 | 状态 |
| --- | --- | --- | --- | --- | --- | --- |
| [GitHub Notes](https://github.com/ymote/octosense-github-notes) | 0.1.0 | 效率 | macOS | ymote | 本地草稿；宿主管理的 GitHub 登录、提交审核及可选外壳 Ask | 预览 |
| [Inbox Assistant](https://github.com/ymote/octosense-inbox-assistant) | 0.1.0 | 效率 | macOS | ymote | Gmail、本地草稿、已配置模型，以及授权后的 Glance/助手处理 | 预览 |
| [Google Calendar](https://github.com/ymote/octosense-google-calendar) | 0.1.0 | 效率 | macOS | ymote | Google Calendar、本地草稿、授权后的聊天与 Glance | 预览 |

以上为 **macOS 开发者预览**，在目录序列 7 中准入。在 Apple Silicon Mac 上安装
[OctoSense 桌面版 0.1.0-beta.2](https://github.com/OctoSense-org/OctoSense/releases/tag/desktop-v0.1.0-beta.2)，
然后在 App Hub 搜索上述名称，选择 Get、查看权限，再选择 Install → Open。
[准入与验收记录](reviews/connected-apps-0.1.0/README.zh-CN.md)列出发布者的精确提交和测试过的运行时。
旧目录格式的解析已测试；尚未验证旧桌面版本能否运行这三款应用。

提供方登录需要宿主 OAuth 应用注册，配置保存在应用包之外；参阅
[关联账户说明](https://github.com/OctoSense-org/OctoSense/blob/desktop-v0.1.0-beta.2/crates/oauth-service/README.zh-CN.md)。
真实 GitHub/Google 登录和远程写入尚未验证。Calendar 助手仅提供建议，不能预约。
无需 OctoSense 云端账户。

曾用于跑通发布流程的相机卡片已于 2026 年 9 月 20 日移除：相机是随 Shell 出厂的系统应用（与新闻、相册一样），不是商店应用。它的仓库仍保留在 [ymote/camera-card](https://github.com/ymote/camera-card)，作为可发布应用包的完整示例。

应用契约 1.5 新增四项独立的宿主服务能力：`auth` 管理 GitHub/Google
连接，`github` 访问仓库，`gcalendar` 访问 Google Calendar，`gmail` 访问 Gmail。
OAuth 令牌由宿主保存；应用只取得绑定自身身份与授权范围的连接句柄。
能力声明不会注册服务；独立的 `card-host` 不提供这些服务。样例登录需要
安装 OctoSense 的宿主实现并配置提供商注册信息，无需 OctoSense 账号。

普通应用可在 `tools.json` 中用 `"implemented_by":"host-service"` 和
`"host_method":"github.read"` 将自身命名空间的工具绑定到已审核的共享服务。
准入检查 `SHARED_HOST_METHODS` 白名单、声明的服务权限、私密数据披露和最低风险；
执行时宿主仍需检查实际授予的权限。省略该字段时保持原有分发行为。
登录、宿主 sheet、直接保存和发送不能通过别名调用。别名通过准入不等于服务
已注册，也不等于用户批准外部操作。

目录中的工具摘要省略 `host_method`，以便旧版商店仍可读取目录；权限和风险信息
保持不变，该摘要不用于执行分发。签名应用包保留完整工具文件，安装及启动仍须
验证准确的包摘要和发布者签名，之后才加载工具绑定。

Calendar 的脚本 UI 另需应用契约 1.4 的 `calendar` 能力。OctoSense 注册日历服务，
每次调用均校验所属应用身份。声明能力本身不会创建服务、连接日历账号或授予 Agent 工具。

应用契约 1.5 的 `photos` 和 `youtube` 能力允许请求 OctoSense 提供的应用自有媒体服务，
不会授予 Android 相册、YouTube 账号或其他代理工具的访问权。跨应用工具仍需
所有者声明、调用方明确授权、宿主准入和执行器。
