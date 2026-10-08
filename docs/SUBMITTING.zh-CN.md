# 向 App Hub 提交应用

[English](SUBMITTING.md) | 简体中文

先开一个 [App Hub 投稿 issue](#7-开提交-issue) 发起发布请求。Release 尚未准备好时也可以开 issue：填写应用 ID、计划版本、公开仓库、所需能力和当前测试状态，之后在同一个 issue 中补齐证据。

首次投稿和常规更新都通过应用仓库的 GitHub 工作流准备、生成证明并打包发布。开发者无需单独管理发布者私钥，也不需要仓库签名 Secret。维护者核验精确的 Release pack 及其 GitHub 证明后，由管理员发布到经过认证的目录。创建标签或 GitHub Release 不等于 Hub 批准。安装需要支持 `publisher-github-v1` 的宿主；兼容的 OctoSense 宿主发布版仍待完成。

目录第 10 版的三个参考应用是历史示例。其中的 Ed25519 密钥和已签名源码布局只是可选的旧协议兼容路径，不是 GitHub 发布者应用的必做步骤。准入、能力、清单和签名规则见[发布参考](PUBLISHING.zh-CN.md)。

```text
投稿 issue（可以先开）→ 仓库、清单、商店信息和截图
→ 测试开发源码 → GitHub 标签工作流 → 验证 Release pack
→ 在 issue 中补齐证据 → 维护者审核 → 目录准入
```

## 开始之前

宿主负责运行应用包：开发时是 `card-host`，用户安装后是 OctoSense Shell（桌面版或手机版）。宿主服务（例如 `github`、`model`）是 Shell 中的代码，应用通过 `host.request` 调用。

### 准备工具

| 工具 | 来源 | 用途 |
| --- | --- | --- |
| `hub` | App Hub `main` | 为开发源码写入摘要并检查、扫描，再准备、验证和打包带 GitHub 证明的发布。审核人员运行的是同一份代码。 |
| `card-host` | App Hub `main` | 运行未签名的应用包，驱动它并截图。 |
| `tools/octo` | [Design Flow](https://github.com/OctoSense-org/OctoScript-App-Design-Flow) | 创建、运行应用并截图。它封装了 `card-host` 和 `hub`。 |
| OctoSense 桌面版 | 面向 macOS（Apple 芯片）的 [0.1.0-beta.2](https://github.com/OctoSense-org/OctoSense/releases/tag/desktop-v0.1.0-beta.2) 发布版 | 运行使用宿主服务（包括[连接账户](PUBLISHING.zh-CN.md#已连接账户)）的应用。 |

按 [QUICKSTART §1](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.zh-CN.md#1-前置条件) 准备好工作区，然后从最新的 `main` 构建 `hub` 和 `card-host`：

```sh
cd ~/octosense-ws/OctoSense-App-Hub
git pull
cargo build --release -p octosense-app-hub --bin hub
cargo build --release -p octosense-card-host --bin card-host
export PATH="$PWD/target/release:$PATH"
hub
```

不带参数运行 `hub` 会输出用法，第一行是 `hub — the OctoSense app hub command (ADR 0003)`。任何命令加上 `--help` 或 `-h`（例如 `hub check --help`）也会输出用法。如果 `hub check --help` 输出的是 `hub: No such file or directory (os error 2)`，说明你的 `hub` 比 `main` 旧：拉取最新代码后重新构建。

如果构建失败并报 `no variant … TextInputStateQuery`，请看 [`card-host` 构建失败](DEVELOPMENT.zh-CN.md#card-host-构建失败)。

旧版或打过补丁的 `hub` 可能放行一些应用包，而审核人员用的构建会拒绝它们。请使用未打补丁的 `main` 构建，并在提交之前不要改动这个检出目录。记录精确工具版本，使用经过认证的 `catalog-v2.json` 检查发布者连续性；`catalog.json` 是明确选择的旧协议目录（[第 6 步](#6-冻结并验证发布)）。

### 应用能在哪里运行

| 宿主 | 能运行 | 不能 |
| --- | --- | --- |
| `card-host` | 单个未签名的应用包，并提供远程控制接口，用于驱动和截图 | 提供用于发现宿主 API 的 `runtime` 以外的任何宿主服务：其他每个 `host.request` 都会失败，返回 `no service answers "<family>" on this device`。它也拒绝已签名的应用包，以及要求 `host-api-v1`、`backend-api-v1` 或 `script-tools-v1` 的应用。 |
| desktop-v0.1.0-beta.2（macOS，Apple 芯片） | 已安装的应用，包括使用 `auth`、`github`、`gmail` 和 `gcalendar` 的应用 | 在宿主的 `oauth/clients.json` 中配好 OAuth 注册之前，登录 GitHub 或 Google（[配置方法](https://github.com/OctoSense-org/OctoSense/blob/desktop-v0.1.0-beta.2/crates/oauth-service/README.md)）；该发布版不附带任何注册信息。让应用登录它自己的后端。安装请求 `wasm` 的应用：它的商店会以 `unknown capability "wasm"` 拒绝。未在该发布版上验证：真实的提供商登录。 |
| desktop-v0.1.0-beta.1 和 home-v0.1.0-beta.1（目前唯一发布的手机版本） | 所用能力都在旧版应用契约之内的商店应用 | 安装请求 `auth`、`github`、`gmail`、`gcalendar`、`calendar`、`photos`、`youtube`、`wasm` 或 `palpo.*` 的应用。它们的商店会拒绝这类应用，例如报 `unknown capability "auth"`。 |

OctoSense `main` 与 desktop-v0.1.0-beta.2 有以下不同，这些改动尚未进入任何发布版。

- 发行方可以在构建时编入 GitHub 和 Google 注册信息。你自己从源码构建的版本不带注册信息，需要自行添加，例如写入 `oauth/clients.json`（见发布参考的[已连接账户](PUBLISHING.zh-CN.md#已连接账户)一节）。
- 在 macOS 和 Android 上，应用可以登录清单声明的后端，并调用清单列出的后端操作（见发布参考的[登录自己的后端](PUBLISHING.zh-CN.md#登录自己的后端)一节）。
- 声明了 `host-api-v1` 的应用可以在 macOS 和 Android 上使用设备权限方法（见[宿主 API 兼容性](HOST-API.zh-CN.md)）。
- 批准 GitHub 或 Google 日历的保存时，须在宿主的确认面板上亲手点按；批准 Gmail 发送在两个版本上都有这个要求。
- 应用 Agent 调用 `glance.publish` 时，宿主拒绝可执行的 Splash（`script`）和 L1 卡片源码。
- 宿主只保留从 30 天前到 366 天后的 Google 日历日程，而不是日历的全部历史。

想在提交前把应用装进 Shell 试用，可以先把它发布到本地签名目录（[演练步骤](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/PUBLISHING.zh-CN.md#4-在本地演练商店流程)）。演练只在从源码构建的 Shell 上验证过。

### 确认你的平台

- **macOS（Apple 芯片）**：历史参考应用命令已验证；新的 GitHub 发布者工作流以第 5 步单独列出的源码与发布状态为准。
- **Windows**：尚未在当前 `main` 上验证。开放中的 issue [#41](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/41) 记录了在较早版本上完成的 Windows 11 原生构建和运行，由社区成员而非维护者验证。用 `python tools/octo` 运行 Design Flow 的工具，它能找到 `hub.exe` 和 `card-host.exe`。commit 应用包之前，先加上[第 1 步](#1-安排仓库结构)的 `.gitattributes`；最后一次检查要在全新克隆上做（[第 6 步](#6-冻结并验证发布)）。
- **Linux**：未经验证。另有报告称，在软件渲染（llvmpipe、WSL）下截取画面（`/g`）会超时，这一点同样未经验证。

### 保护发布者密钥

**仅用于可选的旧协议兼容。** 使用 GitHub 发布的首次投稿和更新均跳过本节，无需创建开发者签名密钥。

本节只适用于历史 Ed25519 发布流程。新的 GitHub 发布者应用无需开发者签名密钥，请走[第 5 步](#5-生成最终字节)中的新路径。契约 1.8.0 已发布；兼容的 OctoSense 宿主发布版仍待完成，已有宿主发布版尚不能消费新证明。

发布者 ID 是你在签名目录中的名字；发布者密钥为你发布的每个版本签名。一旦有一个由这把密钥签名的版本进入签名目录，Hub 就只接受同一把密钥签名的后续版本。没有任何命令能替换丢失的密钥；万一丢失，请在 issue 中联系审核人员。

> **警告：** 密钥文件是发布者密钥的唯一副本。把它放在所有仓库之外，做好备份，不要交给任何人。`hub keygen` 不会覆盖已有文件；在 macOS 和 Linux 上，它创建的密钥文件只有你自己能读取。在 Windows 上，请把密钥放在只有你的用户能读取的文件夹中。

## 三个参考应用

签名目录第 10 版收录了发布者 `ymote` 的三个 macOS 开发者预览版，每个应用有两个版本：首个版本 0.1.0 和后续的 0.1.1，后者针对 [0.1.0 的教训](#010-的教训)中的三个问题做了改进。每个应用都有自己的公开仓库，标签为 `v0.1.0` 和 `v0.1.1`。下表描述的是 0.1.1：

| | [GitHub Notes](https://github.com/ymote/octosense-github-notes) | [Inbox Assistant](https://github.com/ymote/octosense-inbox-assistant) | [Google Calendar](https://github.com/ymote/octosense-google-calendar) |
| --- | --- | --- | --- |
| 应用 ID | `org.octosense.samples.githubnotes` | `org.octosense.samples.inbox` | `org.octosense.samples.googlecalendar` |
| 演示内容 | 把 Markdown 草稿保存为经用户确认的 GitHub commit | 读取 Gmail、共用一份回复草稿、在后台分拣新邮件 | 日程视图，含日程草稿、经用户确认的保存和提供建议的应用 Agent |
| 能力 | `storage`、`auth`、`github` | `storage`、`auth`、`gmail`、`model`、`glance`、`octos.session.open`、`octos.turn.start` | `storage`、`auth`、`gcalendar`、`glance`、`octos.session.open`、`octos.turn.start` |
| 应用 Agent | `read-only` 权限配置，只在前台运行；3 个 `read` 工具 | `read-only` 权限配置，`background: true`，由 `inbox.new_message` 触发；1 个技能；9 个工具（4 个 `read`，5 个 `act`） | `read-only` 权限配置，只提供建议；4 个 `read` 工具 |
| 速览卡片 | 无 | 模板文件 `glance-workspace.splash`，以 `template` 和 `initial` 发布 | `main.splash` 中的 L0 卡片，以 `source` 和 `data` 发布 |
| 受保护的写操作 | 通过 `github.review_save` 创建 commit，在宿主的确认面板上批准 | 通过 `gmail.draft.review` 发送，须在宿主的审阅界面上亲手点按批准按钮 | 通过 `gcalendar.review_save` 保存，在宿主的确认面板上批准 |

应用 Agent 是 OctoSense 为单个应用运行的 AI Agent，使用该应用的工具。`read-only` 是 Agent 自身会话的权限配置：Agent 每次写入工作区（即应用的存储）之前都要先征得同意。它不约束 `tools.json` 中的工具：这些工具的调用是否要等用户确认，由各自的 `risk` 级别决定。Inbox 的 5 个 `act` 工具会直接修改回复草稿、记录分拣结果、发布速览卡片，无需询问。

Design Flow 的[连接账户参考应用 README](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/examples/connected-apps/README.zh-CN.md) 介绍了每个应用的工作方式。其中 GitHub Notes 和 Inbox 的副本包含与 0.1.1 相同的修复；Google Calendar 的副本不显示日期范围。

### 0.1.0 的教训

每个应用的 0.1.0 版都通过了审核，但各有一个问题，0.1.1 版对此做了改进。你的应用应从一开始就避开这些问题：

- **卡片工具绝不接受 `script`。** Inbox 0.1.0 的后台工具 `inbox.notify` 映射到 `glance.publish`，除了 `template`，还接受最大 16 KiB 的 `script` 卡片。OctoSense 桌面版 0.1.0-beta.2 会按应用自身的策略运行 Agent 发布的 `script` 卡片，因此一轮遭到提示注入的后台对话就可能发布任意 Splash 代码。Inbox 0.1.1 只接受已准入的模板加 `initial` 数据；OctoSense `main`（尚未进入任何发布版）会拒绝 Agent 发布的 `script` 卡片。只接受 `template` 加 `initial`，或 L0 `source` 加 `data`。
- **带了工具，就声明 `agent`。** GitHub Notes 0.1.0 在 `tools.json` 中带了三个工具，却写着 `"agent": null`。Hub 照样把这些工具作为应用 Agent 准入，OctoSense 也照样提供这个应用 Agent，但准入检查的 `grants:` 行输出的是 `agent none`，OctoSense 桌面版 0.1.0-beta.2 的商店显示的也是“Runs no assistant.”。GitHub Notes 0.1.1 声明了 `read-only` 的 `agent` 字段并附带 `AGENT.md`，于是准入检查输出 `agent read-only`，每个商店都会显示这个应用 Agent（见发布参考的[带 `tools.json` 的应用都有 Agent](PUBLISHING.zh-CN.md#带-toolsjson-的应用都有-agent)一节）。在隐私政策中说明应用 Agent。如果应用不应有 Agent，就不要带 `tools.json`。
- **显示日期范围。** Google Calendar 0.1.0 列出日历的全部历史，最早的日程排在最前面：OctoSense 桌面版 0.1.0-beta.2 会同步所有日程，Agent 的 `cached` 工具也会把它们全部返回。Google Calendar 0.1.1 在宿主报告日期范围时显示“past 30 days / next 366 days”，没有报告时显示“date range unavailable”，也不再把范围之外的日程说成已删除。OctoSense `main`（尚未进入任何发布版）只同步这个范围。显示一段日期范围，并只把选中的日程交给 Agent，而不是整个缓存。

0.1.1 中还留着一个问题：Inbox 的后台工具可以改写回复草稿，包括 `to` 地址；改了收件人的回复，只有宿主的审阅界面（发送须亲手点按）能拦住。让后台工具只做读取，以及需要用户确认的写操作。

## 1. 安排仓库结构

Hub 只准入 `bundle/`。准入检查不读取仓库中的其他内容，但审核人员会打开你的隐私政策和支持页面。使用 GitHub 发布时，把 `.github/workflows/publish-app.yml` 与可编辑的 `bundle/` 一起 commit；无需 `publisher.json`。历史参考应用还附带了下面这些文件，审核人员都用到了。以 `v0.1.0` 的 GitHub Notes 为例：

```text
octosense-github-notes/
  bundle/          提交的内容：清单、商店信息、代码、工具、素材、截图
  PRIVACY.md       privacy_policy_url 指向的页面
  SUPPORT.md       如何报告问题
  README.md        应用做什么，以及如何验证发布
  publisher.json   历史 Ed25519 发布者 ID、算法和公钥
  review/          签名后应用包的准入检查输出（GATE.txt）和扫描问题的回答（ANSWERS.md）
  LICENSE, NOTICE
  .gitignore       把密钥、build/ 和 .local-state/ 挡在 Git 之外
```

开发 README、密钥、审核包和 `.local-state/` 放在 `bundle/` 之外。素材要求保留的许可证和来源说明应以 `.txt` 或 `.md` 随包提供；这些文档中的链接不会授予网络权限。准入检查会按规则检查应用包中的每个文件，并计入 8 MiB 的大小上限。`tools/octo new` 生成的 `.gitignore` 已经排除了 `build/`、`.local-state/` 和 `*.key`。

添加一个 `.gitattributes` 文件，让 Git 原样保存和检出应用包的字节。参考应用没有这个文件，所以检出时只要设置了 `core.autocrlf=true`（Windows 上很常见），Git 就会把它们的文本文件转换成 CRLF 换行，连摘要也对不上。

```sh
cd ~/apps/my-app
printf 'bundle/** -text\n' >> .gitattributes
git check-attr text -- bundle/manifest.json
```

成功时输出 `bundle/manifest.json: text: unset`。如果应用包已经 commit 过，运行 `git add --renormalize bundle`，再重做[第 5 步](#5-生成最终字节)。

Git 从仓库根目录开始匹配这个模式，所以 `bundle/**` 只覆盖根目录下的应用包。如果应用包在更深的目录中，就写出它的实际路径，例如 `apps/my-app/bundle/** -text`；也可以写 `**/bundle/** -text`，覆盖任意层级的 `bundle/`。然后对这个应用包的 `manifest.json` 运行 `git check-attr`，对它的目录运行 `git add --renormalize`。输出 `text: unspecified` 说明这个模式没有覆盖应用包，Git 仍可能转换其中的文件。

## 2. 写对清单

编辑 `bundle/manifest.json`。下面是 GitHub Notes 0.1.0 在写入摘要和签名之前的清单：

```json
{
  "schema": 1,
  "id": "org.octosense.samples.githubnotes",
  "version": "0.1.0",
  "name": "GitHub Notes",
  "integrity": { "bundle_blake3": "" },
  "capabilities": ["storage", "auth", "github"],
  "storage": { "accounts": true, "max_bytes": 4194304 }
}
```

- **`id`** 永久不变。只能用 `[a-z0-9.-]`，最长 64 个字符。准入检查拒绝以 `os.` 开头的 ID。ID 的最后一段会成为工具的命名空间（`githubnotes.*`），不能是保留名称；[常见拒绝原因](#常见拒绝原因及修复)中 `identity` 一行列出了全部 23 个。`tools/octo new` 也会拒绝这些名称。
- **`version`** 每次提交都要用新值。标签是 `v` 加上这个版本号。
- **`capabilities`** 只列出界面确实用到、而且有宿主提供服务的能力（见 [Hub 目前做不到的事](#hub-目前做不到的事)）。商店会在安装前把每一项展示给用户。
- **`integrity`** 存放应用包摘要，由 `hub stamp` 计算并写入（即“写入摘要”）。不要手工修改。

然后写入应用包摘要：

```sh
hub stamp bundle
```

成功时输出应用包摘要：对 `bundle/` 中除 `manifest.json` 以外的所有文件计算的 BLAKE3，共 64 个十六进制字符。如果 `hub stamp` 输出 `hub: manifest is not valid: unknown field …`，说明清单中有契约未定义的字段；消息里列出了所有合法字段。`hub stamp` 不检查能力和商店信息，这些由第 5 步的 `hub check` 负责。

## 3. 撰写商店信息

`bundle/listing.json` 是商店在安装前展示给用户的内容。发布者签名通过应用包摘要覆盖了它，所以以后哪怕改一个字，也要发布新版本。不要写任何会过时的内容。

| 字段 | 规则 | 参考应用 |
| --- | --- | --- |
| `subtitle` | 最多 80 个字符 | `"Markdown drafts with reviewed GitHub saves"` |
| `description` | 最多 4000 个字符 | 写明哪些是虚构数据、哪些未经验证 |
| `category` | [发布参考](PUBLISHING.zh-CN.md#商店信息)列出的 15 个类别之一 | `productivity` |
| `keywords` | 最多 10 个 | 各 4 个 |
| `screenshots` | 应用包内的 1 到 8 个 PNG 或 SVG 文件 | 2 到 4 张真实的 PNG 截图 |
| `icon` | 应用包内的正方形 SVG 或 PNG（[图标](ICONS.zh-CN.md)） | `assets/icon.svg` |
| `platforms` | 只写实际运行过的平台 | `["macos"]` |
| `publisher` | `name`、`support`（URL 或电子邮件地址），以及以 `https://` 开头的 `privacy_policy_url` | 仓库的 Issues 页面和 `PRIVACY.md` |
| `age_rating` | `all`、`12+`、`16+` 或 `18+` | `all` |

把模板商店信息中的占位内容全部换掉（`example.com` 链接和“Replace with …”文字），并把 `platforms` 设为你实际运行过的平台。准入检查接受占位内容，`tools/octo check` 则会把它们标出来。

提交之前先写好隐私政策。说明哪些数据会离开设备、去往哪里：通过宿主服务访问的提供商 API、发给用户所选模型提供商的文本、速览卡片，以及应用是否带有应用 Agent。GitHub Notes 发布 0.1.0 之后修订过隐私政策，因为它的 `tools.json` 带来了一个政策中没有提到的应用 Agent。

签名之前，先检查商店信息中的链接；标签推送之后再发现 URL 有误，就只能发布新版本来修正。先公开隐私政策页面，再逐个请求 `publisher` 字段中的 URL：

```sh
python3 -c 'import json; p = json.load(open("bundle/listing.json"))["publisher"]; print(p["privacy_policy_url"]); print(p["support"])' \
  | grep -E '^https?://' \
  | while read -r url; do echo "$(curl -sL -o /dev/null -w '%{http_code}' "$url") $url"; done
```

成功时每个 URL 都输出 `200`。GitHub Notes：

```text
200 https://github.com/ymote/octosense-github-notes/blob/main/PRIVACY.md
200 https://github.com/ymote/octosense-github-notes/issues
```

`support` 是电子邮件地址时，这条命令会跳过它。`-L` 会跟随重定向，所以放在你自己域名上的隐私政策页面也能报告最终状态。输出 `404` 表示页面尚未公开、仓库是私有的，或者地址拼错了。

## 4. 截图

商店信息里，只有截图能让审核人员对照运行中的应用核实。在 `card-host` 中运行未签名的应用包，通过远程控制接口把应用驱动到要展示的每个状态（路由见 [QUICKSTART](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.zh-CN.md)），逐一截图。在 Design Flow 的检出目录中运行：

```sh
tools/octo run ~/apps/my-app/bundle --port 8141 --detach
tools/octo shot 8141 ~/apps/my-app/bundle/screenshots/01-main.png
curl -s 127.0.0.1:8141/quit
```

成功时输出：

```text
wrote …/bundle/screenshots/01-main.png (824x1784, 32539 bytes). Look at it before you ship it.
{"ok":1}
```

使用前逐张打开 PNG 查看。不要发布错误画面、空白的首帧或效果图。如果应用一直在播放动画，`shot` 仍会保存最后一帧，并在输出中注明 `(still changing after 2s, e.g. an animation; this is the last frame)`；请检查这一帧。其他截图问题见 [QUICKSTART 的故障排查](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/QUICKSTART.zh-CN.md#故障排查)。

未验证，仅限 Linux：如果在软件渲染（llvmpipe、WSL）下 `shot` 超时，请在启动 `tools/octo run` 或 `card-host` 之前设置 `MAKEPAD_WRITE_FRAMEBUFFER_PNG=<file>`，Makepad 的 OpenGL 后端每次绘制窗口时都会把窗口画面保存到这个 PNG 文件；在 macOS 上，`shot` 仍是经过验证的截图方式。

需要宿主服务或原生控件（例如 GitHub Notes 的 Markdown 编辑器）的界面，在 `card-host` 中渲染不出来。这样的界面，请在提供该服务或控件的 Shell 中用虚构数据截图；否则，截下它的不可用状态，就像 Google Calendar 的 `03-host-required.png` 那样。在描述中注明哪些截图用了虚构数据。

`hub scan` 生成的审核包不含截图。审核人员会在应用包和你的 issue 中查看截图。

## 5. 生成最终字节

首次投稿和更新都使用 GitHub 发布者来源证明。契约 1.8.0 新增 `publisher-github-v1`。两个真实标签推送发布及原生 Store 安装／更新／启动检查已通过（[证据与限制](PUBLISHING.zh-CN.md#github-发布者来源证明)）；兼容宿主发布版仍待完成。

1. 如果尚未开投稿 issue，现在就可以开。Release 证据可以稍后补充；未完成的检查标为待完成，不要编造通过结果。
2. 测试未签名的开发副本并截取真实界面。运行准入检查和 `hub scan bundle --allow-unsigned --packet build/review.json`，其中 `build/` 位于应用包外。逐题回答审核包中的问题：共七题，带 `tools.json`、`AGENT.md` 或技能时为八题。注明每个回答依据的文件及未测试的行为。
3. 用 Design Flow 的 `tools/octo publish-github <app-directory>` 安装并审核 `.github/workflows/publish-app.yml`（`tools/octo new` 也会提供）。此路径不需要 `keygen`、`sign-manifest`、`--publisher-key` 或仓库签名 Secret。原生命令见 [GitHub 发布者来源证明](PUBLISHING.zh-CN.md#github-发布者来源证明)。
4. Commit 测试过的开发源码、截图、商店信息、隐私政策／支持页面、`.gitattributes` 和工作流。每次发布使用新的语义版本和精确的 `v<manifest.version>` 标签。工作流生成带证明的清单及最终 Release pack；不要把生成的字节覆盖提交到开发源码中。

常规更新保留相同的仓库名、不可变仓库／所有者 ID 和工作流身份，并提高语义版本。更新同样无需发布者私钥。历史 Ed25519 应用身份继续使用原协议；这不是转移应用归属的流程。

### 可选的历史 Ed25519 路径

只有明确选择兼容已有旧协议包或旧协议时才走这条路径。密钥管理与命令见[签名](PUBLISHING.zh-CN.md#签名)。带日期的 [0.1.0 准入记录](../reviews/connected-apps-0.1.0/admission.json)和 [0.1.1 记录](admissions/connected-apps-0.1.1/README.zh-CN.md)保持原样。它们是历史证据，不是要求新开发者创建密钥的步骤。不要给封存的发布包重新写入摘要；从未签名开发副本开始，发布一个新版本。

## 6. 冻结并验证发布

标签标识测试过的开发源码和工作流。Release pack 包含最终带证明的清单；仅检查源码克隆并不能验证这个 pack。

1. 审核测试过的 commit 后，推送新的 `v<version>` 标签。记录 `git rev-parse "v0.1.0^{commit}"`（换成你的版本），并用 `git ls-remote origin 'refs/tags/v0.1.0*'` 确认。附注标签的 `^{}` 行标识 commit。不要移动、删除或重建已推送的标签；修正需要更高版本和新标签。
2. 等待 GitHub 工作流成功。保存工作流运行 URL、精确 commit、Release URL、`app.bundle.pack.json` 和 `release-receipt.json`。记录中的 `pack_sha256` 必须与下载的 pack 匹配，例如在 macOS/Linux 上运行 `shasum -a 256 app.bundle.pack.json`。
3. 验证下载的最终字节，不要修改它们：

   ```sh
   hub publisher-unpack app.bundle.pack.json --out review-bundle
   hub publisher-verify review-bundle --catalog /path/to/catalog-v2.json
   git -C ~/octosense-ws/OctoSense-App-Hub rev-parse HEAD
   ```

   `review-bundle` 必须是尚不存在的目录。使用当前经过认证的目录，且拟提交版本尚未准入；这会同时核对发布者连续性和版本递增。保留完整验证输出及精确工具版本。拒绝结果是需要修复的问题，不能靠删除证明或添加 `--allow-unsigned` 绕过。
4. 在已有的投稿 issue 中补充这些产物及测试证据。审核人员会验证源码 commit 和下载的 pack。Release 工作流成功并不等于安装、投稿或批准应用。

只有可选的旧协议包才需要按[签名](PUBLISHING.zh-CN.md#签名)一节，用登记的公钥检查已签名源码标签的全新克隆。GitHub 证明发布不需要这种密钥检查。

## 7. 开提交 issue

使用 [Submit an app 表单](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/new?template=submit-app.yml)，每个计划版本开一个 issue，标题为 `Submit <app id> <version>`。**第 1–6 步尚未完成时也可以发起请求。** 准备好后，在同一个 issue 中补充发布证据；待完成字段不代表获得批准。

| 表单字段 | 何时提供什么 |
| --- | --- |
| Release status（发布状态） | 现在：证据待补充、GitHub 发布已准备好，或可选的旧协议包 |
| App id / Version / Repository URL（ID／版本／仓库） | 现在：清单 ID、计划版本及公开 GitHub 仓库 |
| Requested capabilities and app behavior（能力与行为） | 现在：应用用途及每项能力的理由 |
| What is not verified（未验证项） | 现在及每次更新后：未完成检查、宿主／平台／提供商限制 |
| Tag / Full commit SHA / Bundle path（标签／commit／路径） | 准入前：不可变源码标签及 commit、开发应用包路径 |
| GitHub workflow run URL（工作流运行） | 准入前：精确发布对应的成功标签推送运行 |
| Release and pack URL / Release pack SHA256（下载与摘要） | 准入前：最终 pack、receipt 及匹配的下载摘要 |
| Bundle BLAKE3 digest（应用包摘要） | 准入前：带证明的发布清单中的摘要 |
| Privacy policy URL / Support contact（隐私／支持） | 准入前：商店信息中的有效链接或支持邮箱 |
| Platforms tested / App Hub revision the gate ran on（平台／工具版本） | 准入前：精确宿主／工具版本和已测试行为 |
| Gate and publisher verification output（准入与证明验证） | 准入前：完整开发准入检查及下载 pack 的验证输出 |
| Scan answers / Screenshots（扫描回答／截图） | 准入前：附源码依据的完整回答及真实原生截图 |
| Optional legacy authentication（可选旧协议认证） | 只用于明确选择的历史 Ed25519 包；GitHub 首次发布和更新均留空 |
| Confirmations（确认项） | 无机密、推送后标签不可变、如实列出待完成证据 |

不要开修改 `catalog.json`、`catalog-v2.json`、`index/` 或 `artifacts/` 的 PR。审核后的目录发布是管理员的独立[受保护 GitHub 工作流](GITHUB-PUBLISHING.zh-CN.md)。

## 8. 审核检查什么

审核人员在准入前核验 GitHub 身份、精确源码 commit、完整发布证明、pack 摘要、准入结果、隐私与界面证据。受保护的目录工作流会再次核对这些字节，不会运行投稿代码。历史参考应用 0.1.0 版的准入记录 [`reviews/connected-apps-0.1.0/admission.json`](../reviews/connected-apps-0.1.0/admission.json) 列出了对每个应用确认过的内容：

| 检查项 | 0.1.0 记录中的字段 | 自己怎么查 |
| --- | --- | --- |
| 标签解析到所声明的 commit | `tag_verified` | `git ls-remote`（第 6 步） |
| 隐私政策和支持地址返回 HTTP 200 | `public_privacy_and_support_http` | 第 3 步的链接检查 |
| release 下载文件与哈希值一致 | `release_download_hashes_match` | 发布了 release 时：`shasum -a 256 -c SHA256SUMS` |
| 下载的应用包上，发布者签名验证通过 | `downloaded_publisher_signature_verified` | 历史 Ed25519 全新克隆检查；GitHub 发布按第 6 步验证下载的 pack |
| 下载的应用包通过准入检查 | `downloaded_gate_output` | 同上 |
| 应用包摘要与 issue 中的一致 | `bundle_digest` | `integrity.bundle_blake3` |

当前版本 0.1.1 的记录见 [`docs/admissions/connected-apps-0.1.1`](../docs/admissions/connected-apps-0.1.1/README.zh-CN.md)。这份记录还保存了每个应用签名后的准入检查输出和源码审核结论，并记录了一项测试：用商店的代码安装每个应用，并从 0.1.0 升级。

审核人员会对照应用包逐题核对你的扫描回答：商店信息中的声明、平台与类别、最小授权、欺骗性界面、写给 AI Agent 的指令性文字、辱骂性措辞，以及每个工具的范围和风险。首次提交一定会等待人工审核。审核人员不承诺审核时间。

历史参考发布使用 `hub publish` 和 Ed25519 目录。当前 GitHub 发布通过[管理员目录工作流](GITHUB-PUBLISHING.zh-CN.md)准入精确审核字节并推进经过认证的目录。开发者无需获得目录签名密钥，也不应修改目录文件。

准入并不证明应用能与真实的提供商正常配合。0.1.0 的记录写明了 `"live_provider_login_and_remote_effects_verified": false`，0.1.1 的记录也不证明原生界面、提供商流量或亲手点按批准。请在商店信息中说明你尚未验证的内容。

## 9. 提交之后

- 审核人员关闭 issue 时，会注明应用所在的签名目录版本号，或者列出需要修复的问题。在 issue 中回答提问，但不要改动标签所指的内容。
- **更新就是新版本加新 issue。** 要修复问题或发布改动：提高 `version` 并开一个链接旧 issue 的新 issue，可以先于 Release。重做第 4 到 6 步（界面有变化时才需要第 4 步），然后在新 issue 中补充该发布证据。不要在评论中发布新版本。Hub 从不替换已发布的版本。参考应用的 0.1.1 版就是作为新 issue 提交的：[#130](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/130)、[#131](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/131) 和 [#132](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/132)。
- GitHub 更新保留已登记的仓库／所有者／工作流身份，提高语义版本并运行同一标签工作流，无需单独的开发者私钥。只有历史 Ed25519 包继续使用登记的密钥（[可选旧协议说明](#保护发布者密钥)）。
- 审核人员可以发布附带理由的目录撤回记录。各商店下次拉取签名目录时，会停止运行该版本已安装的副本；其他版本不受影响。如需撤回，请开一个 issue，写明应用 ID、版本和要向用户展示的理由。撤回的版本号不能再用，修复请以新版本发布。

## 常见拒绝原因及修复

`hub check` 每个检查结果输出一行：`[refused|warning] <check> (<file or property>): <detail>`。准入检查无法读取的应用包不会得到报告，只有一行 `hub: …`。完整规则见[发布参考](PUBLISHING.zh-CN.md#准入检查的规则)。

| 检查项或消息 | 原因 | 修复 |
| --- | --- | --- |
| `digest: the bundle hashes to …, the manifest claims …` | 写入摘要之后字节变了：有改动、在 commit 之后才写入摘要（`tools/octo check` 会重新写入）、Git 检出产生了 CRLF 换行，或者用了比 `main` 旧的 Windows 版 `hub`（它用 `\` 拼接路径来计算摘要）。 | 从 `main` 构建 `hub`，加上[第 1 步](#1-安排仓库结构)的 `.gitattributes`，然后为开发源码重新写入摘要并生成新的 GitHub 证明版本，验证下载的 pack；不要给封存发布重新写入摘要。 |
| `publisher-signature: publisher key "<id>" is not registered with this hub` | 仅可选的旧 Ed25519 路径：检查或扫描已签名包时没有提供密钥；加 `--allow-unsigned` 或通过 `tools/octo check` 运行也一样。 | 传入 `--publisher-key <publisher-id>=<hex public key>`。 |
| `publisher-signature: the signature from key "<id>" does not match the manifest` | 仅可选的旧 Ed25519 路径：签名字节发生变化。 | 从未签名开发源码重建，用登记的密钥签一个新版本。 |
| `continuity: not signed by the key on record for "<id>"`，或 `continuity: … is already published by "<id>"; an update must carry that key` | 仅可选的旧 Ed25519 路径：该发布者 ID 已登记了另一把密钥（要么是别人的 ID，要么是你的旧密钥），或者更新没有签名。 | 首个应用换一个没人用过的 ID；每次更新都用已登记的密钥签名。 |
| `version: version … is already published; publish a new version` | 该版本已在签名目录中。 | 提高 `version`，使用新标签并验证工作流生成的新 Release pack（第 5、6 步）。 |
| `policy: app … requests unknown capability "<name>"` | 名称不在契约中（`contacts`、`model.image`，或 `octos.` 这样的前缀），或者你的 `hub` 太旧，还不认识这个能力。 | 使用[发布参考](PUBLISHING.zh-CN.md#清单)中的确切名称，并从 `main` 重新构建 `hub`。 |
| `hub: manifest is not valid: unknown field …`，没有报告 | `hub stamp` 和 `hub check` 无法解析清单。 | 删除该字段，或改用正确的字段名。消息中列出了合法字段。 |
| `contents: <file> has extension "…", which a bundle may not hold` | `.DS_Store`、`LICENSE`，或其他扩展名不在允许范围内的文件。 | 删除它，或移出 `bundle/`。 |
| `contents-invalid (<file>): cannot decode the image: …` | 图片损坏、其他格式改名成了 `.png`，或者单边超过 4096 像素。 | 重新截图或导出。 |
| `resource-invalid (…/font_src): not a portable bundle path: "makepad_widgets:resources/…"` | 引用了允许列表之外的内置字体。 | 使用 Inter、LXGW WenKai Regular 或 Bold 的准确资源名，或随包提供有许可的字体子集。完整名称和旧版宿主限制见[字体](PUBLISHING.zh-CN.md#字体)。 |
| `assets: <file> contains https://…` | 卡片数据含有外部 URL，或脚本、Agent 指令引用未声明的主机。 | 随包提供素材；为脚本声明所需的主机及 `net` 权限。普通 `.txt`、`.md` 文档链接不会触发此检查，也不授予网络权限。 |
| `identity: … is under os.`，或 `identity: app id "…" ends in "…", which is reserved` | ID 以 `os.` 开头，或者 ID 本身或其最后一段是以下名称之一：`agents` `apphub` `appcard` `browser` `calculator` `card` `clock` `dev` `notes` `octos` `octoscode` `os` `reference` `reminders` `rinx` `sheets` `shell` `system` `task` `terminal` `toolbox` `weather` `workflow`。 | 在首次发布之前换一个 ID。 |
| `listing: listing has more than 10 keywords`、`… more than 8 screenshots` 或 `listing platform "…" is not one of […]` | 商店信息超出上限，或名称拼错。 | 精简列表，或使用消息中给出的名称。 |
| `listing: screenshots/01-main.png is named by the listing but is not in the bundle` | 文件不存在。 | 截图（[第 4 步](#4-截图)），或修正路径。 |
| `hub: the bundle exceeds the size limit`，没有报告（`--json` 显示 `bundle-invalid`） | 除 `manifest.json` 以外的文件超过 8 MiB（8,388,608 字节）。 | 压缩图片，对字体做子集化。 |
| `entry (main.splash): the bundle has no entry: …` | 应用包根目录下既没有 `main.splash`，也没有 `page.card`。 | 把入口文件放在 `bundle/` 的根目录。 |
| `secrets: <file> declares is_password:true: …` | 声明了密码或一次性验证码字段。 | 删除它。登录由宿主服务负责。 |
| `agent: AGENT.md is in the bundle but agent.instructions does not name it` | 有 Agent 文件，却没有对应的 `agent` 声明。 | 在 `agent` 中声明 `"instructions": "AGENT.md"`，或删除该文件。 |
| `card-host: refused: no signature verifier is installed, so the signature from key "<id>" cannot be checked` | `card-host` 只运行未签名的应用包。 | 用未签名的副本运行和截图，最后再签名（[第 5 步](#5-生成最终字节)）。 |

## Hub 目前做不到的事

| 需求 | 现状 | 替代做法 |
| --- | --- | --- |
| 登录你自己的后端 | 只有 macOS 和 Android 上的 OctoSense `main` 支持，尚未进入任何发布版。在清单中声明后端之后，应用只能调用自己声明的后端操作，每次写操作都要等用户在宿主上确认（见发布参考的[登录自己的后端](PUBLISHING.zh-CN.md#登录自己的后端)一节）。 | 在已发布的版本上，用只验证身份的登录来识别用户：`auth` 搭配 GitHub 的 `read:user`，或 Google 的 `openid`、`email` 和 `profile`。需要提供商数据时，再加上 `github`、`gmail` 或 `gcalendar`。 |
| 在应用中保存 API 密钥或令牌 | 不支持。准入检查只拒绝密码和一次性验证码字段，因此发现不了输入到普通字段或存放在存储中的密钥。 | 不要附带任何密钥。生成文本请用 `model`，它调用的是用户自己的 AI 提供商。 |
| 生成图片、音频、视频或向量嵌入 | 尚不支持。`model` 只提供 `model.complete` 和 `model.budget`；`model.image`、`model.audio`、`model.video` 和 `model.embeddings` 都是未知能力（[#85](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/85)–[#88](https://github.com/OctoSense-org/OctoSense-App-Hub/issues/88)）。 | 用 `model.complete` 生成文本。 |
| 使用 `llm`、`news`、`calendar`、`prompt`、`ledger.read`、`clipboard` 或 `palpo.*` | 准入检查接受它们，但没有宿主向商店应用提供这些服务。`llm` 和 `news` 只响应 `os.*` 应用，`calendar` 只响应 `os.calendar`，其余的没有任何宿主处理。 | 不要请求它们。访问 Google 日历请用 `gcalendar`。 |
| 用 Agent 工具运行应用自身的逻辑 | 尚未进入任何发布版。OctoSense `main` 会在完整应用打开期间运行 `implemented_by: "app"` 的工具；应用关闭时返回 `app_not_running`。OctoSense 桌面版 0.1.0-beta.2 拒绝这类工具，返回 `app_tool_unavailable`。没有 `host_method` 的 `host-service` 工具会调用以应用的命名空间命名的服务，而命名空间不是能力，所以调用失败，返回 `not_granted`。 | 要调用共享服务，用 `host_method` 把工具映射到 `github`、`gcalendar`、`gmail` 或 `glance` 的某个方法（见发布参考的[把工具映射到共享服务](PUBLISHING.zh-CN.md#把工具映射到共享服务host_method)一节）。要运行应用自身的逻辑，在清单中声明 `requires: ["script-tools-v1"]`，并实现 `app_tool` 钩子（见发布参考的[脚本工具执行](PUBLISHING.zh-CN.md#脚本工具执行script-tools-v1)一节）。请在基于 `main` 构建的 OctoSense Shell 中测试。 |
| 在应用中附带原生 Rust 代码 | 商店应用包不能携带原生代码。准入检查会拒绝原生库，原生代码需要随 Shell 新版本发布（[交付路径](DEVELOPMENT.zh-CN.md#选择合适的交付路径)）。 | 要做纯计算，把 Rust 代码编译成 `fns/` 中的 WebAssembly 模块，并请求 `wasm` 能力（见发布参考的[能力](PUBLISHING.zh-CN.md#能力)一节）。只有启用 `wasm-lab` 特性的 OctoSense 构建会运行它，目前还没有任何发布版启用这项特性。具体做法，以及设备 API、网络和文件各走哪条路，见 Design Flow 的[运行自己的 Rust 代码](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/RUST.zh-CN.md)。 |
| 提交系统应用（`os.*`）或原生应用 | 这里没有提交途径。系统应用随 Shell 一起发布，原生代码需要随 Shell 新版本发布（[交付路径](DEVELOPMENT.zh-CN.md#选择合适的交付路径)）。 | 用自己的 ID 做一个商店应用。 |
| 在手机上安装 `auth` 应用 | 目前没有任何已发布的手机版本能做到。 | 在 desktop-v0.1.0-beta.2 上测试。 |
| 在卡片 kit 中引用 Makepad 内置的 CJK 字体 | 当前 Hub 和锁定运行时支持 Regular 与 Bold 的准确资源名。 | 名称、打包字体及旧版宿主限制见[字体](PUBLISHING.zh-CN.md#字体)。Mac 上已验证 `card-host` 原生显示；尚未在每种 Shell 和平台上验证。 |

哪个 Shell 提供哪项宿主服务，见 Design Flow 的 [HOST-SERVICES.zh-CN.md](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-SERVICES.zh-CN.md)。
