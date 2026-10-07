# 宿主 API 兼容性

[English](HOST-API.md) | 简体中文

1.6 契约让应用可以声明自己需要的宿主 API，并查询宿主实现了哪些 API。它增加的是声明和发现机制，而不是访问权限：应用只能调用宿主中已编译的 Rust 服务，每次调用仍要经过应用自己的授权。同一版契约还让应用包可以声明自己的后端（[登录自己的后端](PUBLISHING.zh-CN.md#登录自己的后端)），并运行自己的 Agent 工具（[脚本工具执行](PUBLISHING.zh-CN.md#脚本工具执行script-tools-v1)）。

契约 1.6.0 已发布到 crates.io，但目前还没有任何已发布的 OctoSense 版本实现这些 API。在此之前，请用对应源码版本构建的宿主测试。旧宿主不会获得这些服务，并会拒绝要求这些服务的应用。

## 声明应用需要什么

下面的清单片段要求 `runtime.list`，并在宿主提供 `location.get` 时使用它：

```json
{
  "requires": ["host-api-v1"],
  "capabilities": ["runtime", "location"],
  "host_api": {
    "required": {"runtime.list": 1},
    "optional": {"location.get": 1}
  }
}
```

| 字段或标记 | 含义 |
| --- | --- |
| `host_api.required` | 应用缺了就无法运行的方法，每个都写明确切的 ABI 主版本。版本 2 不能满足对版本 1 的要求；这个版本也不是宿主的发布版本号。 |
| `host_api.optional` | 宿主提供时应用才会使用的方法。请为每个方法准备替代路径。 |
| `host-api-v1` | 使用 `host_api` 时必须声明。它还会让相机、麦克风和位置改由每个应用单独授权，因此宿主必须实现 `app_policy.device_consent@1` 运行时 ABI。普通 Makepad 运行器即使能解析这些字段，也不能声称支持它。 |
| `backend-api-v1` | 使用 `backend` 时必须声明。宿主必须实现 `auth.backend.request@1`。 |
| `script-tools-v1` | 使用 `implemented_by: "app"` 的工具时必须声明。宿主必须实现 `app_tools.dispatch@1` 运行时 ABI。 |

商店在安装应用时检查这些要求，之后每次打开应用时再检查一次。宿主无法满足的应用，商店会拒绝，报 `app <id> needs a host implementing <method>@1` 或 `this host does not implement required APIs: <method>@<version>`。签名目录中的新版本即使需要更新的 API，也不会影响宿主仍能运行的已安装版本。

## 查询宿主实现了什么

应用声明 `runtime` 能力后，可以用 `{}` 调用 `runtime.list` 列出所有已描述的方法，或用 `{"method":"location.get"}` 调用 `runtime.describe` 查询单个方法：

- 方法描述包含输入和输出 schema、ABI 版本、所需能力、支持的平台，以及 Agent 能否调用它。
- `runtime_features` 列出运行时 ABI，例如 `app_tools.dispatch@1`。`host.request` 不能调用运行时 ABI。
- 只有注册了描述的方法才会出现。一些较早的服务没有描述，所以这份列表并不完整。
- 结果不含账户数据或凭据。

API 可用不等于已经配置，也不等于已经授权。`configured: null` 表示宿主并不知道配置情况，`authorization: "checked-on-call"` 表示每次调用仍会检查各自的授权。要了解更多，请调用相应服务的状态或账户方法。声明依赖既不会开启操作系统权限，也不会添加提供商注册信息。

## 限制

- **Agent 工具。** `host-service` 工具可以用 `host_method` 映射到 `auth.backend.me`、`auth.backend.request`、`runtime.list`、`runtime.describe`、三个 `*.permission.status` 方法或 `location.get`，最低风险为 `read`，并且需要该方法所属的能力和 `private_data: true`（[把工具映射到共享服务](PUBLISHING.zh-CN.md#把工具映射到共享服务host_method)）。Agent 的调用从不弹出提示，所以 `auth.backend.request` 只执行已声明的 `GET` 操作，宿主会在发出任何 HTTP 请求之前拒绝写操作；写操作仍须在前台应用中发起，并在宿主的原生审阅界面上确认。权限申请和撤销、账户管理和面板控制都没有工具别名。准入不能代替宿主对账户、授权和平台的检查。
- **后端。** 宿主只从已准入的签名应用包读取 `backend` 块。它返回不透明的连接句柄，拒绝重定向和含有令牌的应答，并在原生界面中让用户确认每一次写操作。后台读取无法批准写操作。修改端点、更新应用或撤回应用，都会结束应用的后端连接。
- **脚本工具。** 签名的 `app_tool(name, call_id)` 处理函数在已打开的完整应用自己的 VM 和存储隔离目录中运行。它不加载任何原生库或 Wasm；应用关闭时返回 `app_not_running`。
- **平台。** 设备权限方法支持 Android 和 macOS。`location.get` 返回 Android 上次已知的位置，时效未知。Linux 和 Windows 不支持内嵌 `WebReader`。
- **`card-host`。** 三个标记所需的 API，它一个也没有实现，因此会拒绝要求这些标记的应用（[在本地运行应用包](DEVELOPMENT.zh-CN.md#在本地运行应用包card-host)）。
- **未验证：** Linux、Windows、手机、真实提供商和真实模型的验收。

具体调用方法见 Design Flow 的[发现并使用宿主 API](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-API-V1.zh-CN.md)。
