# 通过 GitHub 发布目录

[English](GITHUB-PUBLISHING.md) | 简体中文

App Hub 仓库管理员批准目录的确切摘要。GitHub Actions 使用短期身份
证书为这些字节签名，并把签名记录到公开透明日志中。管理员不需要
保存额外的 Hub 私钥。这是 GitHub 管理的制品签名，不是 SSH 签名，
也不是 commit 上的绿色 Verified 标记。

**交付状态：** 本分支已实现工作流和原生验证器。启用新渠道之前，
还必须验证正式工作流、生产环境保护和更新后的 OctoSense 发布版。
本文不表示迁移已经完成。

## 谁能批准发布

只有当前仓库的 **admin** 可以在 `main` 上手动启动或重新运行
[`publish-catalog.yml`](../.github/workflows/publish-catalog.yml)。工作流
检查原发起者的数字身份，以及原发起者和重跑者当前是否都是管理员。
拥有写权限的协作者或普通应用发布者不能通过此工作流签署目录。

启用发布前，为 GitHub 的 `app-hub-catalog` 环境设置：仅允许从
`main` 部署、要求 App Hub 管理员审核、禁止绕过保护。管理员成员
变化时，同步更新审核人。用仓库审核规则保护工作流和发布工具的修改。
原生验证器要求签名证书中包含正确的环境声明；修改工作流并删除环境
不能绕过此检查。

批准的对象是完整 commit、候选名称和准备好的 SHA-256，不是应用
名称或可变分支。管理员仍需完成[常规准入审核](SUBMITTING.zh-CN.md#8-审核检查什么)。

## 准备待审核候选

把候选保存在 `main` 上可审核的 commit 中：

```text
catalog-candidates/<name>/catalog.json
catalog-candidates/<name>/artifacts/<app>-<version>.bundle/...
catalog-candidates/<name>/artifacts/<app>-<version>.bundle.pack.json
catalog-candidates/<name>/index/<app>-<version>.json
```

候选 `catalog.json` 是未签名载荷：schema 为 1，不带 `key` 或
`signature`，sequence 比已验证的基准恰好大 1，发布日期为当天
UTC 日期。保留全部历史条目。已有条目只能从 offered 改为带理由的
withdrawn；不能删除历史、替换既有字节或重新上架已撤回版本。
仅刷新有效期的更新不需要新增制品文件。

对于新版本，附上审核时的原样应用包、对应 pack 和 index。
`hub catalog-prepare` 重新运行准入和发布者连续性检查，比较每个
派生条目和打包文件。工作流仅把候选解压为数据，不运行其中的脚本
或构建命令。

[准备原生工作区](FIRST-APP.zh-CN.md#1-准备工具和应用仓库)后：

```sh
cargo build --locked --release -p octosense-app-hub
# catalog-v2.json 存在后，改用它作为基准。
target/release/hub catalog-prepare --base catalog.json \
  --candidate catalog-candidates/<name>/catalog.json \
  --artifact-root catalog-candidates/<name> \
  --out /tmp/catalog-v2.payload.json
```

记录 JSON 回执中的 `payload_sha256` 和候选完整的 40 位 commit。
批准摘要之前，审核生成的载荷及所有新增文件。基准改变或 UTC 日期
跨天时重新准备。命令本身有原生集成测试；正式派发在另行记录前
仍属于未验证。

## 签名、验证和发布

打开 **Actions → Publish catalog → Run workflow**，选择 `main`，
填写候选名称、完整 commit 和确切的 `payload_sha256`。先使用
`dry_run: true`，再用管理员账号批准受保护环境。

第一个任务验证管理员身份、构建可信工具、验证候选，并把准备包绑定
到审核的摘要。受环境保护的任务使用 GitHub OIDC 和 Sigstore 签署
`catalog-v2.payload.json`，随后用原生验证器验证返回的证明。
试运行把验证后的 `catalog-v2.json` 信封和回执保留为工作流制品，
不改变公开目录。

消费者验收该证明后，以 `dry_run: false` 运行同一审核事务。工作流
只提交信封及新增的准入制品和 index。非强制 push 必须从已检查的
确切 main commit 向前推进。遇到并发修改时停止发布，不自动 rebase
或重试。再次派发需要对照新基准审核。旧 `catalog.json` 保持原样。

## OctoSense 验证什么

原生验证器检查载荷的确切 SHA-256、证书链、证书透明证明、已签名
日志条目及包含证明、GitHub 发行者、App Hub 仓库和所有者数字 ID、
工作流路径、`main` 引用、手动启动事件、GitHub 托管 runner 和受保护
环境。签名声明必须描述确切的目录对象和获准工作流。网络镜像无法用
其他仓库或工作流替换它。

发布前验收时，在启动宿主前设置 `OCTOSENSE_HUB_CATALOG=github-v2`。
生产证明和消费者验收通过之前，默认仍为旧格式。应用库一旦缓存 v2，
移除环境变量仍保持 v2；显式请求 `legacy` 会因降级而被拒绝。

宿主在拉取之前选定 v2 客户端模式。它拒绝旧格式或畸形文档，
不自动回退。缓存必须原子保存整个信封及证明，并在重启后保留最大
已接受序号。相同序号但内容不同也会被拒绝。现有 14 天有效期限制
仍约束新安装。

## 迁移与应用发布者

旧发布版只理解信任锚签署的 `catalog.json`，需要更新宿主才能读取
`catalog-v2.json`。直接替换旧文件会破坏旧客户端。迁移时保留独立
的渠道缓存。

本改动替换的是 **Hub 目录签名密钥**，不替换既有应用发布者签名，
也不转移已发布应用的所有权。[连续性规则](PUBLISHING.zh-CN.md#签名)
仍然有效。基于 GitHub 的应用发布者身份流程是独立工作；不能因为
管理员可批准目录，就删除已登记的发布者密钥。

## 验证

运行 `python3 -m unittest discover -s tools -p test_catalog_publish.py`，
检查管理员授权、输入和路径拒绝、准备包的确切绑定，以及真实 Git
并发更新拒绝。PR 工作流运行这些测试时没有签名或写权限。
原生证明、防篡改、准入、防回退和缓存测试位于 `crates/app-hub`。
正式证明和设备验收结果应与这些自动检查分开记录。
