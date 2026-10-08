# OctoSense App Hub

[English](README.md) | 简体中文

App Hub 为 OctoSense 发布应用。本仓库包含：

- 每个 OctoSense 商店都会读取的签名目录；
- App Hub 为每个已准入应用包保存的副本；
- 准入检查（每个应用包都必须通过）、签名、商店和参考宿主的代码。

每个应用的源码留在发布者自己的仓库中。

| 想找 | 仓库 |
| --- | --- |
| 如何开发应用：快速上手、脚本 API、脚本应用模板、设计流程、示例 | [OctoScript-App-Design-Flow](https://github.com/OctoSense-org/OctoScript-App-Design-Flow) |
| AppCard 助手运行时（Shell 中需 `--features app-appcard` 才启用） | [OctoSense `apps/appcard`](https://github.com/OctoSense-org/OctoSense/tree/main/apps/appcard) |
| 第一方系统应用（AI 提供方、日历、相机、邮件、地图、新闻、相册、YouTube）及其宿主服务（`llm`、`model`、`calendar`、`mail`、`news`） | [OctoSense `apps/`](https://github.com/OctoSense-org/OctoSense/tree/main/apps) |
| L0 解析器和检查器，以及 Makepad 转换层和渲染层 | [OctoScript](https://github.com/OctoSense-org/OctoScript) 与 [OctoScript-Makepad](https://github.com/OctoSense-org/OctoScript-Makepad) |
| 应用包格式、准入检查、签名、提交与商店 | 本仓库 |

**要开发应用？** 请从 [OctoSense-org 主页](https://github.com/OctoSense-org)上的“先读这些”列表开始：先读 OctoScript-App-Design-Flow（下称 Design Flow）的 `AGENTS.md`，再读其 `docs/QUICKSTART.md`。接着按照[开发你的第一个 Hub 应用](docs/FIRST-APP.zh-CN.md)和[应用图标与随包素材](docs/ICONS.zh-CN.md)操作。把本仓库克隆到 Design Flow 检出目录旁边，用来构建 `hub` 和 `card-host`。不要修改 `catalog.json`、`index/` 或 `artifacts/`。

**要提交应用？** 请按照[向 App Hub 提交应用](docs/SUBMITTING.zh-CN.md)操作：在本仓库开一个 issue，提交一个已签名的应用包；它必须位于打了 tag 的 commit 上。目录中的[三个参考应用](docs/SUBMITTING.zh-CN.md#三个参考应用)（GitHub Notes、Inbox Assistant 和 Google Calendar）完整通过了准入，它们的仓库展示了一次完整的提交。规则、能力和字段请查阅[发布参考](docs/PUBLISHING.zh-CN.md)。

**要使用宿主 API？** [宿主 API 兼容性](docs/HOST-API.zh-CN.md)说明应用如何声明所需的宿主 API，以及如何查询宿主实现了哪些 API。

## 仓库结构

| 路径 | 说明 |
| --- | --- |
| `catalog.json` | 旧格式签名目录。商店在展示任何内容之前，先用下方的信任锚验证它。 |
| `index/<app>-<version>.json` | 每个应用版本对应一条已准入条目：清单、发布者、源码位置与状态。维护者在 `hub publish` 之后从目录导出。 |
| `artifacts/<app>-<version>.bundle/` | App Hub 保存的应用包副本，与审核时的字节完全一致。由 `hub publish` 生成。 |
| `artifacts/<app>-<version>.bundle.pack.json` | 打成单个文件的同一应用包，商店下载的就是它。 |
| `docs/FIRST-APP.md` | 第一个应用（卡片应用或脚本应用）的分步教程：创建、运行、截图与检查。 |
| `docs/SUBMITTING.md` | 提交流程的分步说明：仓库、清单、商店信息、截图、签名、发布、issue 与审核。 |
| `docs/PUBLISHING.md` | 参考文档：准入规则；能力及其提供方；清单、商店信息和工具的字段；宿主服务；`hub` 命令；签名。 |
| `docs/GITHUB-PUBLISHING.md` | 管理员授权的 GitHub 签名、确切候选审核及 v2 迁移。 |
| `docs/HOST-API.md` | 声明应用需要的宿主 API、用 `runtime` 发现宿主 API，以及目前各宿主实现了哪些 API。 |
| `docs/ICONS.md` | 规范图标的归属、导出约束与视觉评审。 |
| `docs/DEVELOPMENT.md` | 指南导航、交付路径、`card-host` 及其远程控制路由，以及 `card-studio`。 |
| `templates/app/` | 卡片应用仓库脚手架，包含元数据、示例图标和链接好的 Agent 指引。 |
| `crates/app-contract` | 应用契约 `octosense-app-contract`（[OctoSense ADR 0005](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/0005-app-contract.md)）：清单、应用获得的策略、应用包完整性校验与运行应用包所需的内容。在 `1.x` 内只做增量变更（[README](crates/app-contract/README.md)）。本分支正在准备 1.8.0；crates.io 当前提供 1.7.1（[crates.io 上的版本](crates/app-contract/README.md#versions-on-cratesio)，英文）。 |
| `crates/app-policy` | 签名清单与商店信息、准入，以及解析为隔离环境设置和 Agent 会话配置（[OctoSense Home ADR 0002](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/home/0002-agentic-app-security-model.md)）；应用自带的 Agent（`tools.json`、`AGENT.md`、技能）加载为 `AgentBundle`；原生模块的工具清单共用同一套 `tools.json` 解析与检查（`ToolManifest::load`）。它还重新导出应用契约。 |
| `crates/app-hub` | 索引、签名目录、准入检查、Agent 扫描、设备端客户端和 `hub` 命令（[OctoSense Home ADR 0003](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/home/0003-app-hub-and-store.md)）。 |
| `crates/appstore` | 作为 OctoSense 模块的商店；把已安装应用作为独立客户端运行的 `card` 模块；系统应用（`os.` 前缀 id）；以及宿主服务及其面板。 |
| `crates/appstore-app` | 作为独立应用的商店（`appstore`）。 |
| `crates/card-host` | 参考的隔离宿主：运行单个应用包，可以是卡片应用，也可以是脚本应用（`card-host`）。 |
| `crates/card-studio` | 一个工具：在隐藏的 `card-host --remote` 中按速览卡片、手机和桌面尺寸渲染卡片，运行测量检查，并准备视觉评审（`card-studio`，[OctoSense ADR 0002 第 7 节](https://github.com/OctoSense-org/OctoSense/blob/main/docs/adr/0002-event-driven-app-agents.md#7-cards-l0-grounded-rendered-and-critiqued-before-publishing)）。 |
| `crates/app-host` | 单窗口宿主，可把任意 OctoSense `AppModule` 作为独立应用运行。 |
| `crates/app-hub-app` | 每个 OctoSense Shell 都会链接的集成：原生商店模块、`card` 运行模块、由 `OCTOSENSE_SYSTEM_APPS` 指定的系统应用、已安装应用和图标（[README](crates/app-hub-app/README.md)）。 |
| `skills/card-studio` | 基于 `card-studio` 的 octos 技能：`card_render`、`card_critique_payload`。 |

## 构建与测试

这些 crate 基于固定版本的 OctoSense Makepad 复刻、OctoScript-Makepad 和 OctoScript 构建，从同级检出目录（`../makepad`、`../octoscript-makepad`、`../octoscript`）解析依赖；[开发你的第一个 Hub 应用](docs/FIRST-APP.zh-CN.md#1-准备工具和应用仓库)会准备好这些检出目录。构建两个开发工具：

```sh
cargo build --release -p octosense-card-host -p octosense-app-hub
```

不做 release 构建时，可以直接从源码运行 `hub`：`cargo run -p octosense-app-hub --bin hub`。[运行合适的宿主](docs/CODE-WALKTHROUGH.md#2-run-the-right-host)（英文）列出了各个包的测试。[原生工具 CI](.github/workflows/native-tools.yml) 在 macOS、Windows 和 Linux 上运行这条构建命令和这些测试，所用的同级检出目录没有打 OctoSense 的运行时补丁。原生字体渲染检查在 macOS 图形会话中运行；其他任务覆盖构建和测试。

只构建你需要的包。如果一次构建包含按默认特性编译的商店相关包（例如 `cargo test --workspace`），`../makepad` 还必须打上 OctoSense 的运行时补丁。如果构建失败并出现 `no variant … TextInputStateQuery`，请参阅 [`card-host` 构建失败](docs/DEVELOPMENT.zh-CN.md#card-host-构建失败)。

商店界面在桌面和手机布局中使用同一套控件。要预览它，先构建它的示例，再为它指定一个单独的应用数据目录并运行：

```sh
cargo build --locked --release -p octosense-app-hub-app --example preview
OCTOSENSE_PREVIEW_SIZE=1200x860 OCTOSENSE_APP_DATA="$(mktemp -d)" target/release/examples/preview
```

`OCTOSENSE_PREVIEW_SIZE` 设定窗口尺寸：桌面用 `1200x860`，手机尺寸用默认的 `406x820`。手机尺寸的窗口并不等于在手机上测试。做自动化检查时，再设置 `MAKEPAD_HIDE_WINDOWS=1` 和 `MAKEPAD_REMOTE=<port>`。预览会按默认特性构建商店，因此同样需要上述运行时补丁。在预览中安装应用，仍要经过商店的同意和准入检查。

## 代码导读

先看[运行合适的宿主](docs/CODE-WALKTHROUGH.md#2-run-the-right-host)（英文）。导读接着追踪应用包如何进入 UI、宿主服务的请求如何回到回调函数，以及一次查询已保存笔记的应用 Agent 请求。仓库协作规则见 [AGENTS.md](AGENTS.md)。

## 宿主目前提供什么

准入检查接受 105 个能力名称，但只有宿主提供了对应的服务，能力才会生效。[能力](docs/PUBLISHING.zh-CN.md#能力)一节列出了每项能力目前由谁提供。

- 除了用于发现宿主 API 的 `runtime`，`card-host` 不提供任何宿主服务，也不运行 Agent。
- OctoSense 向每个获得授权的应用提供 `mail`、`model` 和 `glance`。
- 宿主配置好 OAuth 客户端注册信息后，OctoSense 桌面版 0.1.0-beta.2 提供已连接账户相关的能力（`auth`、`github`、`gcalendar`、`gmail`）。令牌留在宿主中，应用只拿到连接句柄。
- OctoSense `main`（尚未进入任何发布版本）还能在 macOS 和 Android 上通过 `auth` 让应用登录清单声明的后端，并执行清单列出的后端操作。每次写操作都要等用户确认（[登录应用自己的后端](docs/PUBLISHING.zh-CN.md#登录自己的后端)）。
- OctoSense `main` 还在 macOS 和 Android 上向声明了 `host-api-v1` 和相应能力的应用提供设备权限方法：`camera.permission.*`、`microphone.permission.*` 和 `location.permission.*`，各含 `status`、`request` 和 `revoke`。在 Android 上，它还提供 `location.get`。只有在前台的应用才能请求权限，并由用户在宿主面板上批准，如果操作系统要求，还要在系统提示中确认（[宿主 API 兼容性](docs/HOST-API.zh-CN.md)）。
- OctoSense `main` 还要求亲手点按才能批准 GitHub 或 Google 日历的保存，拒绝应用 Agent 发布可执行 Splash（`script`）卡片，并且只保留从 30 天前到 366 天后的 Google 日历日程（尚未进入任何发布版本）。OctoSense 桌面版 0.1.0-beta.2 不具备其中任何一项。
- OctoSense 只向自己的系统应用提供 `calendar`、`llm` 和 `news`。对于 `photos` 和 `youtube`，它只向 `os.photos` 和 `os.youtube` 提供一个 `notify` 方法。尚未提供：面向商店应用的媒体服务。
- 目前没有任何发布版本提供 `wasm`（应用 `fns/*.wasm` 中自带的函数，契约 1.7.0）。OctoSense 只在启用 `wasm-lab` 特性的构建中运行它们。
- OctoSense 运行应用 Agent 已获授权的 `implemented_by: "host-service"` 工具，包括用 `host_method` 映射到已审核共享服务的工具，并把 `AGENT.md` 和技能作为指引加载（[应用的 Agent 与工具](docs/PUBLISHING.zh-CN.md#应用的-agent-与工具)）。
- 目前还没有任何 OctoSense 发布版本运行 `implemented_by: "app"` 的工具：桌面版 0.1.0-beta.2 会拒绝它们，返回 `app_tool_unavailable`。OctoSense `main` 提供 `app_tools.dispatch@1`，会在已打开的应用中运行这类工具，前提是应用声明了 `requires: ["script-tools-v1"]`；应用关闭时返回 `app_not_running`（[脚本工具执行](docs/PUBLISHING.zh-CN.md#脚本工具执行script-tools-v1)）。

## 信任锚

旧版商店信任这个锚，并沿着它的证书找到为 `catalog.json` 签名的工作密钥。轮换工作密钥不需要发布新版商店。

```text
6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11
```

## 让商店读取其他 Hub

商店构建默认读取本 App Hub，并信任上述锚。如需读取镜像，把 `OCTOSENSE_HUB` 设为镜像目录或基础 URL；用自己的锚签名的开发用 Hub 还需要设置 `OCTOSENSE_HUB_ANCHOR`。下面的命令把这两个变量都设为默认值：

```sh
OCTOSENSE_HUB=https://raw.githubusercontent.com/OctoSense-org/OctoSense-App-Hub/main/ \
OCTOSENSE_HUB_ANCHOR=6000284a069ba7cada2925094074e8e0baae07e25d1b7fc31f396c993f363e11 \
appstore
```

## 维护者如何发布应用

你提交应用（[向 App Hub 提交应用](docs/SUBMITTING.zh-CN.md)）之后，维护者审核你打了 tag 的 commit 中的原样字节。[GitHub 管理员发布](docs/GITHUB-PUBLISHING.zh-CN.md)为新的 v2 目录签名，无需额外的 Hub 私钥；正式启用和兼容宿主发布仍待完成。现有 `hub publish` 路径继续生成旧的信任锚签名目录。

维护者用 `hub withdraw` 撤回某个版本，每个商店下次拉取目录时都会撤下该版本。`hub remove` 用于删除本不该发布的条目。

## 应用

| 应用 | 版本 | 分类 | 运行平台 | 发布者 | 用到的功能 | 状态 |
| --- | --- | --- | --- | --- | --- | --- |
| [GitHub Notes](https://github.com/ymote/octosense-github-notes) | 0.1.1 | 效率 | macOS | ymote | 本地草稿、通过宿主登录 GitHub、经你确认的 commit，以及 Shell 的 Ask 对话栏中可选的应用 Agent | 预览 |
| [Inbox Assistant](https://github.com/ymote/octosense-inbox-assistant) | 0.1.1 | 效率 | macOS | ymote | Gmail、本地草稿、已配置的模型，以及经你同意的速览卡片与 Agent 处理 | 预览 |
| [Google Calendar](https://github.com/ymote/octosense-google-calendar) | 0.1.1 | 效率 | macOS | ymote | Google Calendar、本地草稿，以及经你同意的应用 Agent 聊天与速览卡片 | 预览 |

以上为 **macOS 开发者预览**。第 10 版目录提供每个应用的 0.1.1 版，并保留其 0.1.0 条目。请在搭载 Apple 芯片的 Mac 上通过 [OctoSense 桌面版 0.1.0-beta.2](https://github.com/OctoSense-org/OctoSense/releases/tag/desktop-v0.1.0-beta.2) 的商店安装。OctoSense 桌面版 0.1.0-beta.1 和 Home 0.1.0-beta.1 会列出这些应用，但无法安装：它们的商店把 `auth` 当作未知能力拒绝。目前还没有任何手机版本能安装它们。[0.1.1 准入记录](docs/admissions/connected-apps-0.1.1/README.zh-CN.md)列出了发布者的 commit 和已验证的内容。[0.1.0 记录](reviews/connected-apps-0.1.0/README.zh-CN.md)还包括在桌面版 0.1.0-beta.2 发布版中进行的商店测试。

登录 GitHub 或 Google 需要宿主上有 OAuth 客户端注册信息。OctoSense 桌面版 0.1.0-beta.2 只从宿主的 `oauth/clients.json` 读取注册信息，而该版本没有附带任何注册信息（[已连接账户与 App Hub 示例](https://github.com/OctoSense-org/OctoSense/blob/desktop-v0.1.0-beta.2/crates/oauth-service/README.zh-CN.md)）。OctoSense `main` 会在构建时编入发行方提供的注册信息，可选的 `oauth/clients.json` 会整体替换它们；这项改动尚未进入任何发布版本。

尚未在发布版本上验证：真实登录和远程写入。在开发构建上，仅验证身份的登录已在 macOS 上通过，Google Calendar 也完成了一次连接并保存了一个日程；GitHub commit 和 Gmail 发送仍未验证（[当前交付边界](https://github.com/OctoSense-org/OctoSense/blob/main/crates/oauth-service/README.zh-CN.md#当前交付边界)）。

Calendar 应用 Agent 只提供建议，不会创建日程。这三个应用都不需要 OctoSense 云端账户。
