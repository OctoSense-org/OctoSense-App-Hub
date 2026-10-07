# App Hub 发布参考

[English](PUBLISHING.md) | 简体中文

| 任务 | 参阅 |
| --- | --- |
| 分步提交应用 | [向 App Hub 提交应用](SUBMITTING.zh-CN.md) |
| 开发并检查第一个应用 | [开发你的第一个 Hub 应用](FIRST-APP.zh-CN.md) |
| 搭建工作区，构建 `hub` 和 `card-host`，运行应用并截图 | 应用开发工具集 Design Flow（OctoScript-App-Design-Flow）的[快速上手](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.md) |
| 在脚本中调用能力或宿主服务 | Design Flow 的[能力](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/CAPABILITIES.md)和[脚本 API](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/SCRIPT-API.md) |
| 查询哪个 Shell 提供哪个宿主服务 | Design Flow 的[宿主服务](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-SERVICES.md) |
| 准备图标 | [应用图标与随包素材](ICONS.zh-CN.md) |

准入检查决定应用包能否进入 Hub。用 `hub check` 自行运行准入检查：每违反一条规则，它都会报告一条拒绝或警告。通过准入检查，并不能说明应用能正常渲染、图标在小尺寸下依然清晰，或商店信息属实。这些由审核人员检查（[审核检查什么](SUBMITTING.zh-CN.md#8-审核检查什么)）。

## 应用是什么

应用包是一个由文本和素材组成的目录。运行应用包的是**宿主**：OctoSense Shell（桌面版或手机版应用）或 `card-host`。每个应用都运行在自己的**隔离环境**（一个独立的沙箱化脚本运行环境）中，并受其**策略**约束：只能使用清单请求、并经宿主授予的内容。应用包不含原生代码。需要新原生代码的应用，要改为随 Shell 版本发布（[选择合适的交付路径](DEVELOPMENT.zh-CN.md#选择合适的交付路径)）。

入口文件决定应用的类型：

| 类型 | 入口 | 运行形式 |
| --- | --- | --- |
| 脚本应用 | 应用包根目录下的 `main.splash` | 用 Splash（Makepad 的 UI 脚本语言）编写的程序，有自己的状态、处理函数、存储和请求。 |
| 卡片应用 | 应用包根目录下的 `page.card` | 用 L0（OctoSense 的声明式卡片语言）编写的卡片，由宿主渲染为控件。卡片不含任何逻辑。 |

同时含有两个入口文件的应用包算作脚本应用。

脚本应用的应用包中有这些文件：

```text
my-app/
  manifest.json      应用是什么、可以做什么（必需）
  listing.json       商店展示的应用信息（必需）
  main.splash        程序本身（必需）
  assets/            图标和其他本地素材（图标必需）
  screenshots/       商店信息引用的截图，至少一个 PNG 或 SVG 文件（必需）
```

以 Design Flow 的 [`templates/script-app/`](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/tree/main/templates/script-app) 为起点，按照它的[脚本应用流程](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/flows/script-app/FLOW.md)和[脚本 API](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/SCRIPT-API.md) 开发脚本应用。

引用应用包自带的素材时，使用 `{{assets}}` 占位符，例如 `http_resource("{{assets}}/assets/logo.png")`。宿主会把占位符替换为一个本地回环地址，该地址只提供这个应用包的内容。

第一方[系统应用](https://github.com/OctoSense-org/OctoSense/tree/main/apps)（`apps/<name>/bundle/`）是完整的示例。它们的 `os.` ID 是保留的，所以复制出来的应用需要换用自己的 ID。

卡片应用的应用包中有这些文件：

```text
my-app/
  manifest.json      应用是什么、可以做什么（必需）
  listing.json       商店展示的应用信息（必需）
  page.card          L0 卡片，即应用的界面（必需）
  page.data.json     绑定到卡片中的数据（可选）
  kit/               渲染卡片的控件 kit（必需）
  assets/            图标和其他本地素材（图标必需）
  screenshots/       商店信息引用的截图，至少一个 PNG 或 SVG 文件（必需）
```

用 Design Flow 的[图像到卡片流程](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/flows/image-to-card/FLOW.md)生成卡片、卡片数据和 kit。[L0 规范](https://github.com/OctoSense-org/OctoSense/blob/main/apps/appcard/a2app-l0/framework/l0.md)定义了这门卡片语言。

两类应用都可以在 `manifest.json` 旁附带自己的 Agent：`tools.json`、`AGENT.md` 和 `skills/<name>/`（[应用的 Agent 与工具](#应用的-agent-与工具)）。

提交的只有应用包。开发者说明、源码与工具、密钥、测试数据和审核包都要放在应用包之外（[安排仓库结构](SUBMITTING.zh-CN.md#1-安排仓库结构)）。

## 准入检查的规则

`hub check` 分两个阶段运行：先检查结构限制，再检查其余所有规则。

### 结构限制

违反结构限制的应用包不会得到报告。`hub check` 只输出一行 `hub: <reason>`，退出码为 1。加上 `--json` 时，它把同一原因报告为一条拒绝，检查项名称为 `bundle-invalid`。

| 限制 | 输出的原因 |
| --- | --- |
| 文件总大小最多 8 MiB（8,388,608 字节），不计 `manifest.json` | `the bundle exceeds the size limit` |
| 文件和目录最多 2048 个 | `the bundle exceeds the file count limit` |
| 目录嵌套最多 32 层 | `the bundle exceeds the directory depth limit` |
| 应用包根目录下有 `manifest.json`，最大 64 KiB | `<bundle>/manifest.json: No such file or directory (os error 2)`、`manifest.json exceeds the size limit` |
| 只能有普通文件和目录，不能有符号链接 | `<path>: only regular files and directories are allowed` |
| 路径可移植：名称为 UTF-8，不含 `\` 或 `:`，也没有空段、`.` 段或 `..` 段 | `not a portable bundle path: "<path>"` |
| 清单可以解析：没有未知字段或取值，`schema` 为 1，`requires` 中的每个特性都是已知的 | `manifest is not valid: …`、`manifest schema <n> is not 1`、`app <id> needs a newer host: …` |

### 检查结果

其余规则每违反一条，报告中就多一行**检查结果**，注明所属的检查项：`[refused] <check> (<path>): <detail>` 或 `[warning] <check>: <detail>`。只有问题出在单个文件或属性上时，才会出现 `(<path>)`。拒绝会终止准入，警告不会。

| 检查项 | 拒绝条件 | 警告条件 |
| --- | --- | --- |
| `digest` | `integrity.bundle_blake3` 与应用包不符。每次改动之后都要运行 `hub stamp`。 | |
| `publisher-signature` | 用 `--publisher-key` 给出的密钥或签名目录中记录的密钥验证签名失败；清单已签名，却没有提供密钥（`publisher key "<id>" is not registered with this hub`）；清单未签名，且没有加 `--allow-unsigned`。 | 清单未签名，且加了 `--allow-unsigned`。 |
| `identity` | ID 以 `os.` 开头；ID 本身或其最后一段是保留名称（[ID 与保留名称](#id-与保留名称)）。 | |
| `contents` | 文件的扩展名不是 `.card`、`.json`、`.l0`、`.octoscript`、`.splash`、`.svg`、`.png`、`.jpg`、`.jpeg`、`.webp`、`.ttf`、`.otf`、`.txt` 或 `.md` 之一。准入检查同样拒绝没有扩展名的文件（例如 `.DS_Store` 和 `LICENSE`）。 | |
| `contents-invalid`（文本和图片） | 文本文件（`.splash`、`.card`、`.json`、`.l0`、`.octoscript`、`.txt` 或 `.md`）超过 1 MiB 或不是 UTF-8；JSON 无法解析；PNG、JPEG 或 WebP 无法解码，或单边超过 4096 像素；商店信息中的图标不是正方形，或者是超过 1 MiB 或单边超过 1024 像素的位图。 | |
| `contents-invalid`（SVG） | SVG 无法解析；既没有数值形式的 `width` 和 `height`，也没有 `viewBox`；单边超过 4096 像素；或含有脚本、`foreignObject` 或 `on…` 事件属性。SVG 的样式（`style` 属性或 `<style>` 块）导入样式表、使用转义或注释，或让 `url()` 指向同一文件内 `#fragment` 以外的任何位置。 | |
| `entry` | 应用包中既没有 `main.splash` 也没有 `page.card`；`page.card` 不是有效的 L0；`page.data.json` 不是 JSON；应用包中既没有 `kit/native/<theme>/kit.json`，卡片所需的 OctoScript kit 模块也不齐全。 | |
| `resource-invalid` | 卡片对图片或字体的引用，或 SVG 的 `href`，指向应用包中没有的文件。检查结果会给出对应的 JSON 指针。见[字体](#字体)。 | |
| `assets` | 除 `manifest.json`、`listing.json` 和 Agent 文件外，某个 `.card`、`.json`、`.l0`、`.octoscript`、`.txt` 或 `.md` 文件含有 `http://`、`https://`、`file://` 或 `../`，包括随包附带的 README 或字体许可证中的 URL。`.splash` 文件或 Agent 文件含有 `http://`、`file://`、`../`，或不在 `network.hosts` 中的 `https://` 主机（应用请求了 `images` 或 `web` 时，允许任何公开主机）。 | |
| `secrets` | `.card`、`.l0`、`.octoscript` 或 `.splash` 文件声明了 `is_password: true`，或把 `TextInputContentType` 设为 `Password`、`NewPassword` 或 `OneTimeCode`。 | |
| `storage` | | 没有 `storage` 能力时，某个 `.splash` 文件调用了 `fs.*`，或应用请求了 `camera`。此时 `grants:` 行显示 `storage none`。 |
| `listing` | 缺少 `listing.json`，或它违反了[商店信息](#商店信息)中的规则；商店信息未指定截图或未指定图标；指定的截图或图标不在应用包中。 | |
| `tools`、`agent`、`skills` | `tools.json`、`AGENT.md` 或某个技能不符合 [Agent 文件的规则](#agent-文件的规则)。 | 某个工具是破坏性工具（风险为 `destructive`）或对外工具（带有 `outward`），每次调用都要等待批准；某个工具写了 `confirm: "app"`；应用声明了工具，但 `agent.model.needs` 中没有 `tool_calling`；后台 Agent 带有破坏性工具。 |
| `policy` | 某项能力未知；ID 违反了 [ID 与保留名称](#id-与保留名称)中的规则；版本为空；某个主机不是纯主机名，或者列出了主机却没有请求 `net`（[网络主机](#网络主机)）；`research` 范围或某个 `agent` 字段违反了相应规则；`storage.cache_max_bytes` 为 0。 | |
| `version`（使用 `--catalog` 时） | 签名目录中已有该应用的这个版本。 | |
| `continuity`（使用 `--catalog` 时） | 违反了[签名](#签名)中的发布者规则，或签名目录的历史记录自相矛盾。 | |

### 字体

卡片用 `font_src` 指定字体，写在 `page.data.json` 的布局项（placements）中，或原生 kit 的组件样式中。它的值必须是应用包中的字体文件，或者准入检查唯一允许的内置字体 `makepad_widgets:resources/Inter.ttf`。其他内置字体一律拒绝：

```text
[refused] resource-invalid (kit/native/light/kit.json/components/detail/style/font_src): not a portable bundle path: "makepad_widgets:resources/LXGWWenKaiRegular.ttf"
```

尚不支持：其他内置字体（[#75](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/75)）。对于 Inter 没有覆盖的文字（例如中文），请把 `.ttf` 或 `.otf` 文件打包进应用包；该文件计入 8 MiB 上限。准入检查不解码字体文件，所以请在 `card-host` 中测试字体。

## 清单

```json
{
  "schema": 1,
  "id": "com.example.forecast",
  "version": "1.0.0",
  "name": "Forecast",
  "integrity": { "bundle_blake3": "<written by hub stamp>" },
  "capabilities": ["storage", "net"],
  "network": { "hosts": ["api.open-meteo.com"] },
  "storage": { "max_bytes": 1048576 },
  "compute": { "instruction_budget": 5000000, "memory_bytes": 33554432 }
}
```

| 字段 | 含义 | 规则 |
| --- | --- | --- |
| `schema` | 清单的语法版本。 | `1`。 |
| `id` | 应用的身份。应用的存储文件夹以它命名，它的最后一段是应用工具的命名空间。 | 见 [ID 与保留名称](#id-与保留名称)。每个版本都保持不变。 |
| `version` | 本次发布的版本。 | 不能为空。每次发布都使用新值；Hub 从不替换已发布的版本。 |
| `name` | 用户看到的名称。 | |
| `integrity.bundle_blake3` | 应用包摘要。 | 由 `hub stamp` 写入。不要手工修改。 |
| `integrity.signature` | `{key_id, value}`，即发布者对清单的签名。 | 由 `hub sign-manifest` 写入（[签名](#签名)）。 |
| `capabilities` | 应用可以使用的能力。 | 封闭列表中的名称（[能力](#能力)）。 |
| `network.hosts` | 应用可以访问的主机。 | 纯主机名，且必须同时请求 `net`（[网络主机](#网络主机)）。 |
| `storage` | `max_bytes`、`accounts`、`agent_workspace`、`cache_max_bytes`。 | 见[存储与配额](#存储与配额)。 |
| `compute` | `instruction_budget`、`memory_bytes`。 | 会限制在宿主的上限以内。 |
| `agent` | 应用自己的 Agent。 | 可选（[清单中的 `agent`](#清单中的-agent)）。 |
| `research` | `research` 和 `crawl` 的范围。 | 请求了 `research` 或 `crawl` 时必需；两者都没请求时，出现此字段即拒绝（[research 范围](#research-范围)）。 |
| `requires` | 应用需要的宿主特性。 | 每一项都必须是宿主已知的特性；已知特性有 `palpo-admin-v1`、`host-api-v1`、`backend-api-v1` 和 `script-tools-v1`。后三项还要求宿主实现对应接口，见[宿主 API 兼容性](HOST-API.zh-CN.md)。 |
| `host_api` | 可选的必需/可选 API 精确版本映射。 | 要求 `host-api-v1`，安装和启动时检查必需的实现。 |
| `backend` | 可选的公开后端登录及具名业务操作声明。 | 要求 `backend-api-v1`、`auth` 和账户存储，不得包含凭据。 |
| `schema_minor` | 清单用到了 schema 1 的哪些新增内容。 | 省略此字段。 |

其他字段一律拒绝。运行 `hub sign-manifest` 之后，你省略的可选字段也会出现在清单中，值为 `null`，例如 `"agent": null`。这些字段不改变任何行为。

只请求应用所需的最小权限。清单没有请求的，宿主一律不授予；安装前，商店会用通俗的话向用户展示每一项请求。

### 能力

准入检查能识别 103 个能力名称：下表中的 25 个，以及[精确的服务名](#精确的服务名octosmatrixpalpo)一节中的 78 个。其他名称一律拒绝：

```text
[refused] policy: app com.example.forecast requests unknown capability "model.image"
```

能力让应用可以发出请求，但并不提供响应这些请求的服务。响应请求的是**宿主服务**：OctoSense Shell 中的代码，负责完成应用自己无权做的事。**目前由谁提供**一列说明 OctoSense 桌面版 0.1.0-beta.2 上由谁响应。

| 能力 | 授予的权限 | 商店显示的文字 | 目前由谁提供 |
| --- | --- | --- | --- |
| `storage` | 应用自己的存储文件夹：`fs.*`、相机拍摄的内容，以及控件读取的本地文件。没有它，所有 `fs.*` 调用都会失败。 | Keep its own data on this device | 运行时，所有宿主都提供 |
| `net` | 向 `network.hosts` 中的主机发出请求，不能访问其他主机。 | Reach only: *主机列表* | 运行时，所有宿主都提供 |
| `images` | 显示任何公开 https 主机上的图片，不限于 `network.hosts`。 | Show pictures from any website | 运行时 |
| `web` | 在系统网页视图中打开任何公开 https 页面；网页视图没有任何回到应用的通道。 | Open web pages in a browser view | 运行时 |
| `location` | 设备的位置。 | Use your location | 运行时，限具备该功能的设备 |
| `camera` | 相机。拍摄的内容保存在应用的存储中，所以应用还需要 `storage`。 | Use the camera | 运行时，限具备该功能的设备 |
| `microphone` | 相机录像时的声音。 | Use the microphone | 运行时，限具备该功能的设备 |
| `library` | 把拍摄的内容提供给系统相册，其他应用也能看到。 | Save to your photo library, where other apps can see it | 运行时，限具备该功能的设备 |
| `clipboard` | 剪贴板。 | Use the clipboard | 尚不支持：没有 API 使用它 |
| `prompt` | 应用向用户提出的问题。 | Ask you questions | 尚不支持：没有宿主读取它。应用 Agent 用 `ask_user_question` 提问。 |
| `ledger.read` | 读取共享账本。 | Read your shared data | 尚不支持：没有 `ledger` 服务 |
| `mail` | 通过宿主的 `mail` 服务收发邮件，使用用户在宿主[面板](#面板应用从不收集机密信息)上登录的账户。 | Read and send mail from accounts you sign in to on the device | OctoSense |
| `auth` | 连接应用自己的 GitHub 或 Google 账户。仅在 OctoSense `main` 上，还能登录应用自己的后端（[登录自己的后端](#登录自己的后端)）。 | Connect and disconnect its own GitHub or Google accounts through the host | OctoSense，需要宿主上有 OAuth 客户端注册信息（[已连接账户](#已连接账户)） |
| `github` | 读取仓库；每次 Markdown commit 都要等用户确认。 | Read authorized repositories and ask you to review Markdown commits | 同 `auth` |
| `gcalendar` | 读取 Google 日历；每次修改日程都要等用户确认。 | Read authorized Google calendars and ask you to review event changes | 同 `auth` |
| `gmail` | 读取 Gmail 并保存回复草稿；每次发送都要等用户确认。与 `mail` 相互独立。 | Read authorized Gmail messages, keep reply drafts and request native send review | 同 `auth` |
| `calendar` | 日历应用的本地日程存储和界面。不是 Google 日历。 | Read and manage local events through the device's Calendar service | 仅系统应用 `os.calendar`；商店应用请用 `gcalendar` |
| `llm` | 通过 `llm` 服务管理设备的 AI 提供商。 | Manage the assistant's AI providers, whose keys stay with the device | 仅系统应用 |
| `news` | 设备从订阅源和主题订阅源收集的条目。 | Read news the device collects from its feeds and topics | 仅系统应用 |
| `photos` | 相册应用自己的图库和合集。 | Read Photos's own library and publish collections | 仅系统应用：OctoSense 只向 `os.photos` 提供 `photos.notify` |
| `youtube` | YouTube 搜索和音乐推荐。 | Search YouTube and manage music recommendations | 仅系统应用：OctoSense 只向 `os.youtube` 提供 `youtube.notify` |
| `glance` | 向速览栏发布速览卡片（`glance.publish`、`glance.withdraw`、`glance.list`）。宿主会检查卡片，为卡片设置上限和有效期；卡片只能打开它自己的应用。 | Show cards on your glance screen | OctoSense |
| `model` | `model.complete` 和 `model.budget`，受每个应用的每日预算限制。`model.complete` 接受一个模型类别（`fast` 或 `strong`）和一个 JSON Schema。不支持图片、音频、视频或向量嵌入调用。 | Send what you give it to the AI provider you configured, within a daily budget | OctoSense |
| `research` | 通过系统工具箱搜索，不超出清单的 research 范围（[research 范围](#research-范围)）。每次搜索都由宿主执行。 | Search *范围允许的内容* | 仅系统应用，且只在手机版构建中 |
| `crawl` | 通过系统工具箱抓取网站，深度和页数不超过范围中的 `max_depth` 和 `max_pages`，并遵守其中的域名列表。覆盖面比 `research` 更广。 | Crawl websites, *范围的限制*, which reaches more than searching | 同 `research` |

任何能力都不隐含其他能力。尚不支持：面向商店应用的 `photos` 和 `youtube` 服务。脚本如何调用各项能力，见 Design Flow 的[能力](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/CAPABILITIES.md)文档。

源码：`crates/app-contract/src/manifest.rs` 中的 `KNOWN_CAPABILITIES`。

### 精确的服务名：`octos.*`、`matrix.*`、`palpo.*`

这 78 个名称各是一项独立的能力，按全名精确匹配。`octos.` 或 `matrix.` 这样的前缀属于未知能力。仅通过准入检查还不够：每次调用时，宿主还会检查自己是否提供这个名称、应用的策略是否包含它，以及用户是否授权了它。

| 组 | 名称 | 授予的权限 | 目前由谁提供 |
| --- | --- | --- | --- |
| `octos.*` | 4 个：`octos.session.open`、`octos.session.history`、`octos.turn.start`、`octos.turn.interrupt` | 与应用自己的 Agent 对话，对话由 OctoSense 运行的 Agent 内核 octos 承载：打开对话、读取对话历史、开始一轮、停止应用发起的那一轮。应用从不指定提供商、模型或密钥。 | OctoSense，前提是用户允许了该应用的 Agent。在此之前，调用会得到 `Waiting for the person to allow this app's agent (OctoSense asks the first time)`。 |
| `matrix.*` | 45 个，例如 `matrix.read_messages`、`matrix.room_members`、`matrix.send_message` | 每个名称对应一项 Matrix 操作，使用用户当前的账户，限于用户允许的房间。 | 没有任何 OctoSense 宿主服务提供它们。未验证：OctoSense 随附的原生应用 Rinx 会向导入其中的应用包提供这些名称。 |
| `palpo.*` | 29 个，例如 `palpo.projects.list`、`palpo.inbox.decide` | 每个名称对应一项 Palpo 管理操作。 | 尚不支持：OctoSense 中没有任何组件提供它们。 |

源码：商店为每个名称显示的文字位于 `crates/app-policy/src/services.rs` 和 `crates/app-contract/src/palpo.rs`。

### ID 与保留名称

ID 由 1 到 64 个 `[a-z0-9.-]` 字符组成，不能以 `.` 开头，也不能包含 `..`。准入检查还会拒绝：

- **所有以 `os.` 开头的 ID。** 这些 ID 属于随设备发布的系统应用，任何商店都不会安装它们。
- **本身是保留名称、或最后一段是保留名称的 ID。** 宿主以 ID 为键管理应用的存储和用户同意记录，以命名空间为键管理应用的工具，因此 `com.example.notes` 会顶替原生 Notes 应用。保留名称共 23 个：

| 保留原因 | 名称 |
| --- | --- |
| OctoSense 以原生应用形式随附的应用 | `apphub`、`appcard`、`browser`、`calculator`、`clock`、`notes`、`octoscode`、`reference`、`reminders`、`rinx`、`sheets`、`task`、`terminal`、`weather` |
| Shell 自身使用的身份名称，或 Shell 拥有的工具命名空间 | `agents`、`card`、`dev`、`octos`、`os`、`shell`、`system`、`toolbox`、`workflow` |

```text
[refused] identity: app id "com.example.notes" ends in "notes", which is reserved: its tools would be notes.*, a native app's or the host's
```

系统应用自己的命名空间不在保留之列：`com.example.news` 是允许的。附带 `tools.json` 的应用，命名空间必须符合 `[a-z0-9_]{1,24}`：`com.example.mynotes` 可以声明工具，`com.example.my-notes` 则不能。

源码：`crates/app-contract/src/manifest.rs` 中的 `RESERVED_NAMES`。

### 网络主机

除 `images` 和 `web` 外，应用只有请求了 `net` 并给出确切的主机列表，才能访问网络。运行时在每一条离开隔离环境的路径上强制执行这份列表：网络模块、素材加载和数据获取。`net` 配上空列表，什么也访问不了。主机按名称精确匹配：列出 `example.com` 并不允许访问 `api.example.com`。请写纯主机名，例如 `api.example.com`，不带协议、路径、端口或通配符：

```text
[refused] policy: host "https://api.open-meteo.com/v1" must be a bare host name, with no scheme or path
```

### 存储与配额

| 字段 | 含义 |
| --- | --- |
| `storage.max_bytes` | 应用请求的存储空间，单位为字节。 |
| `storage.accounts` | 设为 `true` 时，每个账户各有自己的数据文件夹和 Agent。默认只有一个 `device` 文件夹。 |
| `storage.agent_workspace` | `"account"`（默认）：Agent 读取所属账户的文件夹。`"none"`：Agent 不读取任何文件，只通过自己的工具工作。 |
| `storage.cache_max_bytes` | 应用为 `cache/` 请求的空间，单位为字节；必须大于 0。 |
| `compute.instruction_budget` | 每次会话累计可执行的脚本指令数。 |
| `compute.memory_bytes` | 隔离环境的堆大小。 |

配额只是请求。宿主会把每项配额限制在对应的上限（即宿主允许的最大值）以内；未填写的配额直接取上限。`hub check` 输出的 `grants:` 行显示限制之后应用实际获得的存储空间。

| 项目 | 上限 |
| --- | --- |
| 存储 | 16 MiB |
| 每次会话的指令数 | 20,000,000 |
| 堆 | 64 MiB |

### research 范围

`research` 和 `crawl` 共用一个范围，即清单顶层的 `research` 对象。准入检查按 octos 自己的规则检查它，宿主则把同一段 JSON 交给系统工具箱。

```json
{
  "capabilities": ["research", "crawl"],
  "research": {
    "langs": ["en", "zh"],
    "regions": ["US", "CN"],
    "domains_allow": [],
    "domains_deny": ["example-spam.com"],
    "max_age_days": 7,
    "categories": ["news"],
    "max_results": 20,
    "max_depth": 2,
    "max_pages": 20
  }
}
```

| 字段 | 含义 | 规则 |
| --- | --- | --- |
| `langs` | 应用可以搜索的语言，采用 BCP-47 标签。 | 语言标签，例如 `en`、`zh-CN` 或 `zh-Hant`；`zh_cn` 会转换为 `zh-CN`。 |
| `regions` | ISO 3166-1 alpha-2 地区代码。 | 两个字母；会转为大写。 |
| `domains_allow` | 只允许这些域名及其子域名。 | 纯域名：`example.com`、`.example.com` 或 `*.example.com`；不带协议、路径或端口。 |
| `domains_deny` | 始终排除这些域名。 | 同上。 |
| `max_age_days` | 允许的最旧内容，以距今天数表示。 | 整数天数。 |
| `categories` | 元搜索类别。 | `news`、`general`、`science`、`it` 或 `social`。 |
| `max_results` | 每次搜索最多返回的结果数。 | 大于 0；默认 20。 |
| `max_depth`、`max_pages` | `crawl` 的限制：单次抓取的链接深度和页数。 | 请求 `crawl` 时两者都必须大于 0；不请求时两者都为 0 或省略。 |

列表为空或省略，或者省略 `max_age_days`，都表示不设限制。`{}` 只保留默认值：每次搜索 20 条结果，不抓取。未知字段一律拒绝。

商店会用通俗的话展示这个范围。例如，`research` 的范围为 `{"langs":["en","zh"],"categories":["news"],"max_age_days":7}` 时，商店的隐私概要（[商店信息](#商店信息)）显示“Searches news in English and Chinese, from the last 7 days”。

## 应用的 Agent 与工具

应用可以自带 Agent，由清单中的 `agent` 字段声明：工具写在 `tools.json` 中，指令写在 `AGENT.md` 中，只含数据的技能放在 `skills/` 下。每个文件都在应用包摘要的覆盖范围内，因此运行的 Agent 就是审核过的那一个。商店把这个 Agent 称为应用的助手（assistant）。

### 清单中的 `agent`

```json
"agent": {
  "profile": "workspace-write-never-ask",
  "tools": [],
  "max_iterations": 8,
  "token_budget": 120000,
  "model": {
    "needs": ["tool_calling", "long_context"],
    "tier": "standard",
    "per_task": { "triage": { "needs": ["tool_calling"], "tier": "fast" } }
  },
  "background": true,
  "triggers": { "schedule": ["0 7 * * *", "0 19 * * *"], "events": ["news.items.new"] },
  "instructions": "AGENT.md",
  "skills": ["news-digest"]
}
```

| 字段 | 含义 | 拒绝条件 |
| --- | --- | --- |
| `profile` | Agent 会话的权限配置：`read-only`（每次写入前都先询问）、`workspace-write`（可读写自己的工作区，其他操作都先询问）或 `workspace-write-never-ask`（同上，但其他操作不经询问直接拒绝）。没有任何权限配置能授予完全访问权限。 | 其他任何值（`manifest is not valid`）。 |
| `tools` | 通用宿主工具（`ledger.read`、`ledger.write`、`net.fetch`、`storage.read`、`storage.write` 和 `card.render`）和一个内核工具 `ask_user_question`。应用自己的工具来自 `tools.json`；Agent 只获得这两类工具。 | 除 `ask_user_question` 以外的内核工具，或宿主不提供的名称。 |
| `max_iterations`、`token_budget` | 每次请求最多可用的模型迭代次数和 token 数，分别以 8 次迭代和 200,000 个 token 为上限。 | |
| `model` | Agent 对模型的要求，而不是提供商或模型名称。`needs` 列出以下任意几项：`tool_calling`、`vision`、`long_context`、`reasoning`、`structured_output` 和 `multilingual`；`tier` 取 `fast`、`standard` 或 `strong`（默认 `standard`）；`local_only` 用于不得离开用户设备的数据（作用于整个应用，单个任务不能放宽）；`per_task` 为 `AGENT.md` 中引用的具名任务单独设置要求。 | `needs` 或 `tier` 的值未知（`manifest is not valid`）；`per_task` 超过 8 项；任务名不符合 `[a-z_]{1,32}`。 |
| `background` | 申请在应用关闭时运行。由用户按应用逐个授予。 | 设为 `true` 却没有触发器。 |
| `triggers.schedule` | 五段式 cron 表达式，按本地时间计。 | 不是由数字和 `* , / -` 组成的五个字段；超过 16 项。 |
| `triggers.events` | 应用自己的宿主服务事件，位于应用的命名空间内（`news.items.new`）。 | 事件不在应用的命名空间内；超过 16 项。 |
| `instructions` | Agent 的指令，按惯例写在 `AGENT.md` 中。 | 不是应用包内的 `.md` 路径。 |
| `skills` | 要加载的技能，以 `skills/` 下的目录名指定。 | 超过 16 个技能，或同一技能列出两次。 |

Agent 的工作区就是应用自己的存储文件夹，它也只能访问应用获准访问的主机。Agent 得到的权限绝不会超过应用本身。

### 带 `tools.json` 的应用都有 Agent

只要应用带有 `tools.json`，即使没有 `agent` 字段，OctoSense 也会为它提供“Ask &lt;app&gt;”。`hub check` 对这样的应用仍会输出 `agent none`，因为 `grants:` 行只反映清单中的 `agent` 字段。商店的隐私概要如何描述这类应用，取决于构建该商店所用的 App Hub 版本：

| 商店 | 有 `tools.json`、没有 `agent` 时的隐私概要 |
| --- | --- |
| 基于 App Hub `main` 构建的商店 | “Offers the host's Ask assistant for its admitted tools, only after you consent. Your conversation and tool results may be sent to your configured AI provider.”和“No app-declared background assistant or automatic triggers.” |
| OctoSense 桌面版 0.1.0-beta.2 中的商店 | “Runs no assistant.” |

每个 OctoSense 构建显示的，都是它锁定的 App Hub 版本给出的概要。既没有 `agent` 也没有 `tools.json` 的应用，在所有商店中都显示“Runs no assistant.”；声明了 `agent` 的应用，显示的则是“Runs an assistant limited to this app's own data.”这条概要。

参考应用 GitHub Notes 正好体现了这一区别。0.1.0 版带了三个工具，却写着 `"agent": null`：`hub check` 输出 `agent none`，OctoSense 桌面版 0.1.0-beta.2 的商店显示“Runs no assistant.”。0.1.1 版声明了 `read-only` 的 `agent` 字段：`hub check` 输出 `agent read-only`，每个商店都显示“Runs an assistant limited to this app's own data.”。

带了 `tools.json`，就同时声明 `agent`，并在隐私政策中说明这个 Agent。如果应用不应有 Agent，就不要带 `tools.json`。

### `tools.json`：应用的工具

以下摘自[新闻示例](../crates/app-policy/tests/fixtures/news-agent)，有删节：

```json
{
  "schema": 1,
  "tools": [
    {
      "name": "news.list",
      "description": "List collected stories, newest first, optionally for one topic or since a time.",
      "input_schema": {
        "type": "object",
        "properties": {
          "topic": { "type": "string" },
          "since": { "type": "string", "format": "date-time" },
          "limit": { "type": "integer", "minimum": 1, "maximum": 100 }
        }
      },
      "output_schema": {
        "type": "object",
        "properties": { "items": { "type": "array", "items": { "type": "object" } } },
        "required": ["items"]
      },
      "risk": "read",
      "background": true,
      "shareable": true,
      "private_data": false,
      "implemented_by": "host-service"
    },
    { "name": "news.read",         "risk": "read", "…": "…" },
    { "name": "news.topics.get",   "risk": "read", "…": "…" },
    { "name": "news.topics.set",   "risk": "act",  "…": "…" },
    { "name": "news.digest.write", "risk": "act",  "…": "…" }
  ]
}
```

| 字段 | 含义 |
| --- | --- |
| `name` | `<namespace>.<tool>`。命名空间是应用 ID 的最后一段：`os.news` 和 `dev.example.news` 的命名空间都是 `news`。每一段都由 `[a-z0-9_]` 组成。OctoSense 的工具代理（tool broker）负责注册和分派应用工具，它把命名空间之后的部分写成代理名称，用下划线代替点（`news.topics.get` 变为 `topics_get`）。准入检查拒绝超过 32 个字符的代理名称。 |
| `description` | 写给模型的说明：工具做什么、何时使用。长度为 1 到 1024 个字符。 |
| `input_schema`、`output_schema` | JSON Schema，只能使用以下关键字：`type`、`title`、`description`、`properties`、`required`、`items`、`enum`、`const`、`default`、`minimum`、`maximum`、`minLength`、`maxLength`、`minItems`、`maxItems`、`additionalProperties`、`format` 和 `pattern`。不支持 `$ref`，不支持 `anyOf`、`oneOf` 或 `allOf`，也不支持条件关键字。最大 8 KiB，最多嵌套 8 层。输入为对象。 |
| `risk` | `read`（只读取）、`act`（修改应用自身的状态）或 `destructive`（发送、发帖、分享、购买、删除，即任何超出应用本身的操作）。必填。也接受工具代理的写法 `Read`、`Act` 和 `Destructive`。 |
| `background` | 工具可以在并非由用户发起的一轮对话中运行。默认 `false`。 |
| `shareable` | 可以授权给应用自身 Agent 以外的调用方，例如 OctoSense 的系统 Agent（覆盖整台设备，用户直接与它对话）和其他应用的 Agent。默认 `false`。 |
| `private_data` | 结果中含有用户的私人数据。`local_only` 应用的可共享工具必须设为 `false`；带 `host_method` 的工具必须设为 `true`。 |
| `implemented_by` | `host-service`：由宿主服务运行。`app`：由应用自己的脚本运行。必填。 |
| `host_method` | 工具映射到的已审核共享服务方法（[把工具映射到共享服务](#把工具映射到共享服务host_method)）。可选。 |
| `outward` | 如果 `act` 工具的调用会触及设备之外（发送、发帖、分享），就设置此项。这样每次调用都会像破坏性（`destructive`）调用一样，等待用户批准。默认 `false`；准入检查拒绝在 `read` 工具上设置它。 |
| `auto_approvable` | 常设规则（例如“一小时内允许”）可以批准调用。默认 `true`。对于删除、付款、账户或安全设置的变更，以及向设备之外分享，请设为 `false`，让用户逐次当场批准。 |
| `confirm` | 执行破坏性或对外的调用之前，由谁询问用户：`host`（默认，即宿主的批准流程）或 `app`（应用自己的确认界面）。只有应用自己实现的工具才能使用 `app`。 |

### 把工具映射到共享服务：`host_method`

`host_method` 把工具路由到共享宿主服务中一个已审核的方法，写作 `family.method`，其中 family（服务族）就是服务本身，例如 `github.read` 中的 `github`。工具在应用的命名空间中保留自己的名称，调用以应用的身份运行。参考应用 [GitHub Notes](SUBMITTING.zh-CN.md#三个参考应用) 把它的 `githubnotes.read` 工具映射到 `github.read`：

```json
{
  "name": "githubnotes.read",
  "risk": "read",
  "private_data": true,
  "implemented_by": "host-service",
  "host_method": "github.read",
  "…": "…"
}
```

`host_method` 必须满足以下每一条规则，否则准入检查会拒绝：

| 规则 | 拒绝消息 |
| --- | --- |
| 工具声明了 `implemented_by: "host-service"`。 | `host_method is only valid for implemented_by "host-service"` |
| 取值为 `family.method` 形式，各段由 `[a-z0-9_]` 组成，不超过 96 字节。 | `host_method must be family.method with nonempty [a-z0-9_] segments, at most 96 bytes` |
| 任何一段都不是 `sheet`。 | `host_method cannot target a host sheet or approve an action` |
| 该方法列在下表中。 | `host_method "<m>" is not in the reviewed shared-service tool contract` |
| 工具的 `risk` 不低于该方法要求的最低风险。 | `host_method "<m>" requires at least <risk> risk` |
| 工具声明了 `"private_data": true`。 | `shared-service tools must declare private_data: true` |
| 清单把该方法所属的服务族声明为能力，或者声明了这个方法本身。 | `host_method "<m>" requires the declared "<family>" service capability` |

| 服务族 | 最低风险为 `read` 的方法 | 最低风险为 `act` 的方法 |
| --- | --- | --- |
| `github` | `github.repositories`、`github.files`、`github.read` | |
| `gcalendar` | `gcalendar.calendars`、`gcalendar.sync`、`gcalendar.refresh`、`gcalendar.cached`、`gcalendar.get`、`gcalendar.prepare` | |
| `gmail` | `gmail.labels`、`gmail.messages`、`gmail.message`、`gmail.draft.get`、`gmail.event.status` | `gmail.draft.open`、`gmail.draft.edit`、`gmail.event.decide` |
| `glance` | `glance.list` | `glance.publish`、`glance.withdraw` |

提供商写入、登录、确认和批准都没有 `host_method`：这些操作由用户在应用自己的界面上发起。

对于映射到 `glance.publish` 的工具，让 `input_schema` 只接受 `template` 加 `initial`，或 L0 `source` 加 `data`。绝不要接受 `script`。OctoSense 桌面版 0.1.0-beta.2 按应用自身的策略发布 Agent 提交的脚本卡片，因此模型写出的 `script` 会作为你的应用运行。OctoSense `main`（尚未进入任何发布版）会拒绝这类卡片，报错 `Agents cannot publish executable Splash; choose an admitted template with initial data, or L0 source`；它也拒绝 L1 的 `source`。

源码：`crates/app-policy/src/agent.rs` 中的 `SHARED_HOST_METHODS`。

### 由谁确认调用

`risk` 和 `outward` 决定调用是否需要用户参与；`confirm` 决定由谁的界面询问用户。`read` 调用，以及没有设置 `outward` 的 `act` 调用，无需用户参与即可运行。破坏性或对外的调用只有在用户批准后才会运行，用户可以当场批准，也可以通过常设规则批准：

| `risk: "destructive"`（或 `outward: true`）时的 `confirm` | 用户在场 | 用户不在场 |
| --- | --- | --- |
| `confirm: "host"`（默认） | 由宿主的批准流程询问用户。 | 应用的对话中出现一条批准请求。 |
| `confirm: "app"` | 应用自己的确认界面是唯一的确认环节（例如 Rinx 的 `send_message`），宿主不会再次询问。 | 应用的对话中出现一条批准请求。 |

同一次调用绝不会询问用户两次。破坏性工具仍然可以设置 `background: true`：准入检查会记录一条警告，而该工具只有在获得批准后才会运行。如果工具既不是破坏性的，也不是对外的，`confirm: "app"` 就不会确认任何东西，准入检查也会对此发出警告。

### `AGENT.md` 与技能

`AGENT.md` 写的是 Agent 的角色和指令：每个触发器触发后做什么、应用数据中哪些内容重要、评判输出的标准，以及它的记忆规则。

技能是一个 octos 技能目录，包含 `SKILL.md` 和 `manifest.json`。对商店应用来说，技能只含数据：技能的 `manifest.json` 包含 `name`（即目录名）、`version`、`description`、`uses`（它调用的工具，每一项都必须是应用自己的工具或列在 `agent.tools` 中），以及可选的 `prompts.include`。

```json
{
  "name": "news-digest",
  "version": "1.0.0",
  "description": "Write a cited morning or evening digest from collected stories.",
  "uses": ["news.list", "news.read", "news.digest.write"]
}
```

### Agent 文件的规则

出现以下情况时，准入检查会在 `tools`、`agent` 或 `skills` 检查项下拒绝：

| 文件 | 拒绝条件 |
| --- | --- |
| `tools.json` | 文件超过 64 KiB，或没有声明任何工具，或声明的工具超过 64 个；某个工具不在应用的命名空间内；命名空间不符合 `[a-z0-9_]{1,24}`；名称重复；两个名称对应同一个代理名称；代理名称超过 32 个字符；工具缺少 `name`、`description`、`input_schema`、`output_schema`、`risk` 或 `implemented_by`；含有未知字段；schema 使用了上文 `input_schema` 一行没有列出的关键字；由宿主服务实现的工具设置了 `confirm: "app"`；`read` 工具设置了 `outward`；`local_only` 应用的可共享工具缺少 `"private_data": false`；`host_method` 违反其规则。 |
| `AGENT.md` | `agent.instructions` 没有指向它；文本为空、超过 32 KiB、不是 UTF-8、含有控制字符、以 `#!` 开头，或包含 `<script`、`<iframe`、`<object`、`<embed`、`javascript:`、`vbscript:` 或 `data:text/html`。 |
| `skills/<name>/` | `agent.skills` 没有列出它；技能的清单声明了可执行字段（`tools`、`binaries`、`sha256`、`mcp_servers`、`hooks`、`hardware_lifecycle`、`tool_discovery`、`actions` 或 `make_type`）；目录中有 `.md`、`.json`、`.txt` 以外的文件，或有符号链接；清单中的 `name` 与目录名不同；`uses` 中有一项既不是应用自己的工具，也不在 `agent.tools` 中。 |
| `AGENT.md` 或 `skills/` | 清单中没有 `agent` 字段。 |

### 商店显示的内容

商店根据清单和 `tools.json` 生成类似下面的说明，在安装前与其他权限并列显示：

- “Run an assistant for this app, only after you allow it”（`agent`）
- “Its assistant can use these app tools: news.list, news.read”（声明了 `agent` 的应用在 `tools.json` 中的工具）
- “Its assistant requests these additional tools: net.fetch”（`agent.tools` 中除 `ask_user_question` 以外的工具）
- “Its assistant may work while the app is closed, on a schedule and when new data arrives; only if you allow it, and you can turn it off.”（`background`）
- “Can ask to mail.send: nothing of this runs until you approve it.”（由宿主确认的破坏性或对外工具）
- “You approve every call of pay.transfer yourself: no standing rule can.”（`auto_approvable: false`）
- “Offers news.list to other assistants you allow.”（`shareable`）

OctoSense 桌面版 0.1.0-beta.2 的商店用一行说明代替前三行：“Run an assistant for this app (&lt;tools&gt;), inside this app's own data only”，其中 &lt;tools&gt; 只列出 `agent.tools`。因此，工具全部写在 `tools.json` 中的应用会显示“Run an assistant for this app (no tools), inside this app's own data only”。

### OctoSense 目前的支持情况

在 OctoSense 桌面版 0.1.0-beta.2 上，各部分的情况如下：

| 组成部分 | 现状 |
| --- | --- |
| 与 Agent 对话 | 用户允许应用的 Agent 之后，在 Shell 的“Ask &lt;app&gt;”对话栏中进行。OctoSense 会在首次使用时询问。`card-host` 不运行 Agent。 |
| `implemented_by: "host-service"` 的工具 | 以应用的身份，在工具命名空间对应的宿主服务上运行；设置了 `host_method` 时，则在该方法所属的宿主服务上运行。清单必须授予该服务族。工具调用从不弹出面板。 |
| `implemented_by: "app"` 的工具 | 提供 `app_tools.dispatch@1` 的宿主在已打开的完整应用中执行签名处理函数。清单声明 `requires: ["script-tools-v1"]`；应用关闭时返回 `app_not_running`。旧宿主仍拒绝此类调用。 |
| `AGENT.md` 和技能 | 作为指引，在每一轮对话中加载。它们不授予任何工具。 |
| `agent.tools` | `ask_user_question` 可用。尚不支持：`ledger.read`、`ledger.write`、`net.fetch`、`storage.read`、`storage.write` 和 `card.render` 的执行器。 |
| `background` 和 `triggers.events` | 只支持 `<namespace>.new_message` 事件，且仅限获得 `gmail` 和 `auth` 授权、设置了 `background: true` 的应用，并须在用户允许其 Agent 之后。 |
| `triggers.schedule` 和其他事件 | 尚不支持。 |
| `agent.model` | 尚不支持：OctoSense 会忽略它。 |
| Agent 发布的速览卡片 | 应用可以发布的任何卡片，包括 `script` 卡片。 |

OctoSense `main`（尚未进入任何发布版）改变了三点：

- 发布速览卡片的工具调用（直接调用 `glance.publish`，或经由 `host_method`）只接受带 `initial` 对象的模板，或 L0 `source`。它拒绝 `script` 和 L1 源码，返回的错误类型为 `unsafe_card_source`。
- 批准 GitHub 或 Google 日历的保存须亲手点按（见[已连接账户](#已连接账户)）。
- 宿主只保留从 30 天前到 366 天后的 Google 日历日程。

## 商店信息

`listing.json` 是用户安装前在商店中看到的内容。它与应用包一同接受审核，并随签名目录分发，因此审核人员读到的内容就是商店显示的内容。商店信息旁边显示的权限来自清单，而不是商店信息，所以商店信息无法淡化应用的实际行为。

| 字段 | 规则 |
| --- | --- |
| `schema` | `1`。 |
| `subtitle` | 最多 80 个字符。 |
| `description` | 不能为空；最多 4000 个字符。 |
| `category` | 取以下值之一：`productivity`、`utilities`、`photo-video`、`news`、`weather`、`travel`、`finance`、`health`、`education`、`entertainment`、`games`、`social`、`shopping`、`lifestyle` 或 `developer`。 |
| `keywords` | 最多 10 个。 |
| `screenshots` | 1 到 8 个路径，指向应用包内的 PNG 或 SVG 文件。请使用应用运行时的真实截图（[截图](SUBMITTING.zh-CN.md#4-截图)）。 |
| `icon` | 应用包内一个正方形 PNG 或 SVG 文件的路径（[应用图标与随包素材](ICONS.zh-CN.md)）。PNG 图标不超过 1 MiB，单边不超过 1024 像素。启动器中请使用同一个图标。 |
| `platforms` | 至少一项，取自 `android`、`ios`、`macos`、`windows`、`linux`、`openharmony` 和 `web`。只列出你测试过的平台。 |
| `publisher.name` | 不能为空。 |
| `publisher.support` | URL 或电子邮件地址。 |
| `publisher.privacy_policy_url` | 以 `https://` 开头的 URL。 |
| `release_notes` | 本版本的变更内容。 |
| `age_rating` | `all`、`12+`、`16+` 或 `18+`。 |
| `license` | 源码开放时，填写 SPDX 标识符。 |

`subtitle`、`keywords`、`release_notes` 和 `license` 是可选字段，其他字段都必须填写。未知字段一律拒绝。商店信息是签名版本的一部分：要修改其中的文字，请发布新版本。

商店还会显示**隐私概要**。隐私概要根据清单生成；在基于 App Hub `main` 构建的商店中，还会参考审核过的工具。它说明应用存储什么、联系哪些主机、使用哪些设备功能，以及是否运行 Agent（[带 `tools.json` 的应用都有 Agent](#带-toolsjson-的应用都有-agent)）。不要在描述中复述隐私概要，而是把清单写对。不过，你仍可以像 GitHub Notes 0.1.1 那样，在描述中说明应用 Agent 会发送哪些数据。

## 宿主服务与面板

脚本应用从不持有凭据。需要凭据的工作，由它调用宿主服务来完成：

```splash
host.request("mail.list", {…}, fn(r){ … })
```

除非应用的策略授予了相应的服务族（`mail.*` 对应 `mail`）或确切的服务名，否则隔离环境会拒绝调用。获准的调用会交给宿主为该服务族注册的服务。服务完成工作后返回数据，绝不返回凭据或连接。如果某个服务族没有任何服务响应，对它的调用会立即失败，返回 `no service answers "<family>" on this device`；`card-host` 不注册任何服务。哪个 Shell 提供哪个服务族，见 Design Flow 的[宿主服务](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-SERVICES.md)。

### 面板：应用从不收集机密信息

只应由用户本人提供的输入，例如密码或账户授权，都放在**面板**上：面板是宿主绘制在应用上层的界面，运行在独立的隔离环境中，不受任何应用的策略约束。只有服务能打开面板。接收机密信息的服务方法都位于 `<family>.sheet.` 之下，并且只接受来自该面板的调用。应用自己的密码字段在运行时不接收任何输入，准入检查也会拒绝声明了密码字段的应用包。

面板显示期间，文本、按键、输入法输入、剪贴板和指针释放事件只发给面板；计时器和服务回复仍会送达应用。基于 App Hub `main` 构建的宿主会自己保存对面板的引用，因此应用中名为 `sheet` 的控件无法隐藏或替换面板。

参见 Design Flow 的 [Mail 完整示例](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-SERVICES.md#mail-the-worked-example)。

### 已连接账户

使用 GitHub 或 Google 的应用要声明 `auth`，再加上它用到的提供商能力：`github`、`gcalendar` 或 `gmail`。用户在宿主面板上登录，应用拿到的是连接句柄，绝不是令牌。写操作要经过宿主确认。请设置 `storage.accounts: true`，让每个账户各自保存数据。哪些宿主支持这类应用，见[开始之前](SUBMITTING.zh-CN.md#开始之前)；具体如何调用，见 Design Flow 的[使用已连接账户](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/CAPABILITIES.md#use-a-connected-account)。

宿主执行哪些规则，取决于它的构建：

| | OctoSense 桌面版 0.1.0-beta.2 | OctoSense `main`（尚未进入任何发布版） |
| --- | --- | --- |
| 批准 Gmail 发送 | 在原生的 Approve & Send 控件上亲手点按 | 相同 |
| 批准 GitHub 或 Google 日历的保存 | 在宿主的确认面板上批准，不检查是否为亲手点按 | 在原生的 Approve & Save 控件上亲手点按；脚本和 Agent 的请求无法批准 |
| 宿主保留的 Google 日历日程 | 日历的全部历史 | 从今天前 30 天到今天后 366 天（按 UTC 计），重复日程会展开 |

如果只想识别用户身份、不读取其数据，就只声明 `auth`，并且仅请求身份类 scope：GitHub 用 `read:user`，Google 用 `openid`、`email` 和 `profile`。这样 `auth.connect` 返回的连接带有提供商的 `subject` 和用于显示的 `label`，不授予对仓库、邮件或日历的任何访问权限。

登录需要宿主中有该提供商的 OAuth 注册信息：

| 宿主 | 注册信息来源 | 没有注册信息时 `auth.connect` 的报错 |
| --- | --- | --- |
| OctoSense 桌面版 0.1.0-beta.2 | 宿主的 `oauth/clients.json`，该发布版不附带这个文件 | `OAuth is not configured. Add provider registrations in the host's oauth/clients.json` |
| OctoSense `main`（尚未进入任何发布版） | 发行方在构建时编入；宿主上的 `oauth/clients.json` 会替换它们 | `GitHub sign-in is unavailable in this build. Check for an OctoSense update or contact its distributor.`，或 Google 的同类消息 |

### 登录自己的后端

OctoSense `main` 可以让应用登录它自己的后端，但目前还没有任何发布版包含这项功能。声明 `auth`，并设置 `storage.accounts: true`。先用 `{"provider":"backend","scopes":["app.session"]}` 调用 `auth.connect`，再用返回的连接句柄调用 `auth.backend.me`，获取后端验证过的身份（`sub` 和 `label`）。用户在后端自己的网页上注册或登录。在 macOS 和 Android 9 及以上版本中，宿主会在自己的网页视图中显示这个页面，该视图不会离开你的登录来源。在桌面端，如果要改用系统浏览器，就在 `auth.connect` 的参数中加入 `"presentation":"browser"`；如果该页面会把用户转到 GitHub 或其他提供商，就必须这样做。Windows 和 Linux 使用浏览器（未经验证）。iOS 不支持后端登录。

应用不能做的事：

- 登记自己的后端。宿主从自己的 `oauth/backends.json` 读取每个应用的后端登记信息，这个文件由设备的运维人员写入。应用包无法提供登记信息，因此已发布的应用只能在运维人员登记了其后端的设备上登录（[#16](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/16)）。
- 用这个会话调用自己的后端。应用拿到的是经过验证的身份，绝不是后端的令牌，因此这个会话不能为其他任何请求授权。
- 自行收集密码，或复用宿主的 GitHub 或 Google 令牌。

协议的后端部分见 OctoSense 的[开发者后端接口约定](https://github.com/OctoSense-org/OctoSense/blob/main/crates/oauth-service/README.zh-CN.md#开发者后端接口约定)。

### 宿主服务调用的限制

每次调用都恰好得到一个应答：服务返回的数据，或者应用可以据此处理的错误。

| 限制项 | 取值 | 不满足时应用收到 |
| --- | --- | --- |
| 应答时间 | 60 秒，除非服务要求更长时间；服务的面板显示期间暂停计时 | `the host service timed out` |
| 每个应用等待中的调用数 | 32 | `too many host requests are waiting; try again when some have answered` |
| 参数 | 1 MiB 以内的有效 JSON | `the request's arguments exceed 1 MiB`，或 `the request's arguments are not valid JSON: …` |
| 应答 | 4 MiB | `the service's answer exceeds 4 MiB` |

应用关闭时，它等待中的调用随之结束，迟到的应答也不会送达应用的下一次会话。

### 面板可以出现在哪里

服务只能在前台应用之上弹出面板。对于以主屏幕磁贴形式显示的应用，以及来自 Agent 的工具调用，宿主会把 `may_prompt` 设为 `false`，并拒绝弹出任何面板，返回 `this surface cannot raise a prompt; open the app to continue`。`prompt` 能力也改变不了这一点。

## 命令

`hub` 运行的就是准入检查本身的代码，Hub 依据的也正是它的报告。按 Design Flow 的[快速上手](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.md)构建它。运行 `hub`、`hub help`，或在任何命令后加上 `--help` 或 `-h`，都会输出用法；这些命令都不会读取应用包，也不会写入文件。未知命令会失败，并输出 ``hub: unknown command "<name>"; run `hub help` for usage``。

| 命令 | 作用 |
| --- | --- |
| `hub stamp <bundle>` | 用准入检查的解析器解析 `manifest.json`，然后把应用包摘要写入 `integrity.bundle_blake3` 并输出。准入检查无法读取的清单，它会拒绝处理。 |
| `hub check <bundle> [--allow-unsigned] [--publisher-key <id>=<hex>] [--catalog <file> [--anchor <hex>]] [--json]` | 准入检查本身。输出 `PASSED` 或 `REFUSED`、每个检查结果，以及应用将获得的授权。有拒绝时退出码为 1。`--json` 以 JSON 输出报告（`schema`、`stage`、`passed`、`app_id`、`version`、`digest`、`findings` 和 `resources`）。 |
| `hub scan <bundle> [--publisher-key <id>=<hex>] [--catalog <file> [--anchor <hex>]] [--packet <out.json>] [--reviewer <cmd>]` | 先运行准入检查，再写出审核包，还可以把审核包交给一条审核命令。准入检查拒绝的应用包不会进入扫描。 |
| `hub keygen <key-file>` | 新建一个密钥文件，以十六进制写入签名密钥，并输出公钥。如果路径已存在（包括符号链接），则拒绝执行。在 macOS 和 Linux 上，该文件只有你自己能读取（权限模式 `0600`）。 |
| `hub pubkey <key-file>` | 输出密钥的公钥。 |
| `hub sign-manifest <bundle> --key <key-file> --key-id <publisher-id>` | 为清单签名，签名覆盖其中的摘要。 |
| `hub verify <catalog> --anchor <hex>` | 用信任锚验证签名目录。 |

`--catalog <file>` 会对照一份已发布的签名目录（例如本仓库的 `catalog.json`），增加 `version` 和 `continuity` 两项检查，并使用签名目录中记录的发布者密钥。如果该文件不存在，`hub check` 会跳过这两项检查，而且不给出警告。签名目录必须能用 Hub 的信任锚验证通过（[信任锚](../README.zh-CN.md#信任锚)）；开发用的 Hub 则用 `--anchor <hex>` 验证。否则 `hub check` 会停止，并输出 `hub: could not authenticate <file> against the hub anchor …`。

`hub publish`、`hub withdraw`、`hub remove` 和 `hub certify` 需要 Hub 自己的密钥，只有 Hub 的维护者会运行它们。

### 读懂 `hub check` 报告

```text
my-notes 0.1.0 — PASSED
  [warning] publisher-signature: unsigned: accountability rests on the hub alone
  grants: capabilities {"storage"}, hosts {}, storage 16777216 bytes, agent none
```

第一行给出应用 ID、版本和结论。每一行检查结果都采用[检查结果](#检查结果)一节中的格式。`grants:` 行是应用将获得的授权：能力、主机、以字节为单位的存储配额（没有 `storage` 时为 `none`），以及 Agent 的权限配置（没有 `agent` 字段时为 `none`）。权限配置按内核的写法显示：`workspace-write-never-ask` 显示为 `workspace-write-never`。

### 扫描问题

`hub scan` 写出的审核包包含清单、商店信息、用商店措辞表述的授权、入口文件的源码、卡片数据、Agent 文件和问题，但不包含截图。请把审核包放在应用包之外。

以下问题摘自 `crates/app-hub/src/scan.rs`，有删节：

1. 应用的行为是否与其名称、副标题和描述的说法一致？
2. 它的平台和类别是否合适？
3. 获得授权的能力和每个主机，是否与应用可见的行为相符？
4. 界面中是否有任何欺骗性的部分？
5. 源码或数据中是否有写给助手的文字？
6. 是否有辱骂性的措辞，或针对普通个人的措辞？
7. 仅当带有 `tools.json`、`AGENT.md` 或技能时：Agent 文件是否局限于应用本身？每个工具的风险级别是否与它的行为相符？
8. 处理方式：pass、human-review 或 reject，并给出发布者可以据此采取行动的理由。

审核人员也会问同样的问题。

## 签名

发布者 ID 就是你签名所用的密钥 ID。已登记的密钥以已发布签名目录中的记录为准，提交时附带的密钥一律不算：

| 情形 | 规则 | 拒绝消息 |
| --- | --- | --- |
| 首次提交 | 可以不签名。 | |
| 已发布应用的更新 | 必须以已登记的发布者 ID 签名。 | 未签名：`continuity: <app id> is already published by "<publisher-id>"; an update must carry that key`。用其他 ID 签名：`continuity: <app id> was published by "<publisher-id>"; this version is signed by "<other-id>". Re-keying is a reviewed change.` |
| 以某个发布者 ID 签名、且该 ID 已有登记密钥的任何清单 | 必须用这把密钥签名。 | `continuity: not signed by the key on record for "<publisher-id>": …`；`--publisher-key` 中给出另一把密钥时：`publisher-signature: conflicting public keys for publisher key "<publisher-id>"` |
| 以未签名方式发布的应用，其第一个签名更新 | 它所用的密钥随之登记在案。 | |

没有用于替换密钥的参数。密钥丢失或轮换，或者历史记录自相矛盾，都需要维护者处理：请在 issue 中提出。

> **警告：** 密钥文件是发布者密钥的唯一副本。把它放在所有仓库之外，做好备份，不要交给任何人。`hub keygen` 不会覆盖已有文件；在 macOS 和 Linux 上，它创建的密钥文件只有你自己能读取。在 Windows 上，请把密钥放在只有你的用户能读取的文件夹中。

签名覆盖清单，包括 `integrity.bundle_blake3`。请遵守以下规则：

- **签名放在最后。** `card-host` 以及任何没有签名验证器的宿主，都会拒绝已签名的应用包：`no signature verifier is installed, so the signature from key "<id>" cannot be checked`。先在未签名的应用包上截图和测试，再写入摘要并签名。
- **先写入摘要，再签名。** 为尚未写入摘要的清单签名，签下的是错误的摘要。
- **任何改动之后，都要重新写入摘要并签名。** 改动 `manifest.json` 以外的任何文件，`hub check` 都会拒绝，报 `digest: the bundle hashes to …, the manifest claims …`。只重新运行 `hub stamp` 的话，`hub check` 仍会拒绝，这次报 `publisher-signature: the signature from key "<id>" does not match the manifest`。直接改动 `manifest.json` 本身，会立即得到第二种拒绝。请依次重新运行 `hub stamp`、`hub sign-manifest` 和 `hub check --publisher-key`。
- **用密钥检查签名后的字节。** 除非 `--catalog` 指定的签名目录已经记录了你的密钥，否则不带 `--publisher-key <id>=<hex>` 时，`hub check` 会拒绝已签名的应用包，即使加了 `--allow-unsigned` 也一样。`hub scan` 没有 `--allow-unsigned` 参数：它直接接受未签名的应用包，而已签名的应用包同样需要提供密钥。
- **保持字节原样。** 摘要覆盖除 `manifest.json` 以外的每个文件：文件的路径、长度和字节；在所有平台上，路径各段之间都用 `/` 分隔。换行符转换会改变摘要，所以不要让 Git 转换应用包（[安排仓库结构](SUBMITTING.zh-CN.md#1-安排仓库结构)）。

## 提交

通过开 issue 提交，绝不要开修改 `catalog.json`、`index/` 或 `artifacts/` 的 pull request（[向 App Hub 提交应用](SUBMITTING.zh-CN.md)）。

## 发布之后

- **版本。** 已安装的应用运行的是已安装的那个版本，并沿用该版本的授权。较新的版本是用户可以选择安装的更新；在用户安装之前，打开的仍是已安装的版本。
- **完整性。** 宿主把已安装的应用包存放在应用的存储之外，因此应用无法写入它。每次启动时，宿主都会对照签名目录检查应用包：清单、摘要和发布者签名。应用包一旦不再一致，宿主就会拒绝运行它，直到用户重新安装该应用。
- **撤回。** 每台设备下次拉取签名目录时，已撤回的版本就会停止运行，其他版本照常运行（[提交之后](SUBMITTING.zh-CN.md#9-提交之后)）。

### 脚本工具执行（`script-tools-v1`）

通用运行器和 OctoSense 中继属于需要集成的改动；请使用公布
`app_tools.dispatch@1` 的宿主版本。清单必须包含
`"requires": ["script-tools-v1"]`。在 `tools.json` 中声明名称和 JSON schema，
设置 `"implemented_by": "app"`，再在应用签名包的 Splash 入口源码中实现固定钩子：

```text
fn app_tool(name, call_id) {
    let request = mod.app_tools.request(call_id)
    if name == "notes.read" {
        mod.app_tools.complete(call_id, {text: fs.read("note.txt")})
    } else {
        mod.app_tools.fail(call_id, "Unknown tool")
    }
}
```

`request.args` 是已验证的工具参数。`request.context` 包含宿主填写的 `app`、
`account`、`caller` 和 `call_id`；脚本不能选择这些身份字段。结果必须符合工具的
`output_schema`。输入和结果各限 1 MiB。`pattern` 和 `format` 仅作说明，
与宿主中继既有的 JSON Schema 子集一致。

钩子在 UI 线程运行，与应用界面共享**同一个活动 Splash VM 和存储沙箱**。
也可在稍后的 `host.request` 回调中，使用相同令牌调用 `complete` 或 `fail`。
异步处理函数可用 `mod.app_tools.active(call_id)` 检查调用是否仍有效。
另一个应用或宿主面板即使知道令牌，也不能读取参数或完成该调用。

仅经过准入验证的完整应用运行器持有工具。Glance 副本不会注册另一个所有者；
多个完整应用实例同时争用工具时会被拒绝。关闭应用、切换宿主连接的账号、取消
或超过期限都会使待处理调用失效；迟到或重复结果会被丢弃。期限最长为 60 秒，
每个应用最多 16 个待处理调用，整个进程最多 128 个。取消不会撤销已经执行的
操作。VM 的指令和内存限制仍然生效；其他线程不能抢占正在执行的同步钩子。

首版 ABI **不会**启动已关闭的应用，也不会启动第二个后台 VM。
`background: true` 不改变此限制。应用可见不代表工具可以弹出宿主权限面板。
请使用默认的宿主确认方式；此 ABI 不实现 `confirm: "app"` 的真实人工批准证明。
访问宿主 API 仍需应用已获得的授权。

运行器测试执行真实 Splash 处理函数，覆盖共享 UI/存储状态、schema 错误、
生命周期、取消、账号切换、堆隔离、禁止工具弹出权限面板和指令限制。
新 ABI 的手机和真实模型验收在集成宿主实际执行前均为**未验证**。
