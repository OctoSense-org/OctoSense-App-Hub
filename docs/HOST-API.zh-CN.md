# 宿主 API 兼容性

[English](HOST-API.md) | 简体中文

1.6 合约增加声明和发现机制，不开放任意 Rust 函数或操作系统调用。应用需要兼容的宿主构建。在合约和宿主发布前，应使用经过审阅的匹配源码版本；旧安装包不会自动获得新服务。

应用声明 `runtime` 能力后，可用 `{}` 调用 `runtime.list`，或用 `{"method":"location.get"}` 调用 `runtime.describe`。结果区分方法描述和 `runtime_features`，例如 `app_tools.dispatch@1`。运行时 ABI 不能通过 `host.request` 调用。元数据包含参数模式、ABI 版本、平台支持和 Agent 调用范围，只列出已注册描述，不能把它当成所有旧服务的完整清单。结果不含账户数据或凭据。

存在 API、完成配置和取得授权是三件事。`configured: null` 表示未知，`authorization: "checked-on-call"` 表示每次执行仍会检查权限。配置情况应查看对应服务的状态或账户方法。声明依赖不能代替 OS 授权或提供商注册。

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

这是清单片段。必需的 ABI 主版本必须完全匹配，不是宿主最低版本号。可选方法缺失时，应用需要可用的替代路径。`host-api-v1` 还要求 `app_policy.device_consent@1` 运行时策略 ABI；普通 Makepad 宿主不能只因为认识这些字段就声称支持。
`backend-api-v1` 要求 `auth.backend.request@1`，`script-tools-v1` 要求 `app_tools.dispatch@1`。使用清单的新 API 或 backend 字段时，应在 `host-api-v1` 之外声明对应标记。App Hub 拒绝不兼容的安装，启动时再次检查已安装版本。目录中新版本需要更高 API 时，仍兼容的旧安装版本可以继续打开。

可选的 `backend` 块包含公开 OAuth 客户端信息和同源的具名业务操作，要求 `auth`、账户存储及 `backend-api-v1`。宿主只从已准入、签名验证的应用包解析声明，返回不透明连接句柄，拒绝重定向和含令牌的结果，并在原生界面审阅写入。后台读取不能批准写入。端点或安装生命周期变化会使后端连接失效。

脚本工具在完整应用已有的 VM 和存储沙箱中执行签名源码的 `app_tool(name, call_id)`，不加载原生库或 Wasm。应用关闭时返回 `app_not_running`。详见[脚本工具 ABI](PUBLISHING.zh-CN.md)和[开发指南](https://github.com/OctoSense-org/OctoScript-App-Design-Flow/blob/main/docs/HOST-API-V1.zh-CN.md)。

平台限制仍须明确：首批设备服务支持 Android 和 macOS 权限；`location.get` 只返回 Android 上次已知位置，时效未知。Linux/Windows 内嵌 `WebReader` 尚不支持。字体修复为宿主提供的随包字体选择正确资源加载器，并保留中文后备字体；准入同时检查字面量和单层 token 引用的字体。这些改动不代表 Linux/Windows、手机、真实提供商或模型验收已经完成。
