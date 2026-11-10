# Pudding

用于游戏脚本工具的本地桌面助手，目前已适配 BetterGI（BGI）。用户用自然语言提出目标，助手负责检索资料、配置任务、调用工具和核对结果；游戏操作由接入的脚本工具执行。

支持五种模型协议、技能与插件、操作审批、执行记录、取消与恢复，以及无需再次调用模型的快捷任务。通用 Agent 运行时与 BGI 适配层分开。

## 构建与运行

Windows 构建需要 Rust stable、Node.js 22.18+、.NET 8 SDK 和 Visual Studio 2022 C++ Build Tools。主程序运行需要 WebView2 Runtime；安装器使用 Slint 原生界面，可在缺少 WebView2 时启动并补装运行时。

```powershell
npm run bridges:fetch
npm ci
npm run dist
```

完整构建在 `dist/` 生成当前版本安装包、SHA-256 校验文件和本地构建清单。组包暂存与内嵌载荷位于 `target/setup/`，构建结束后自动清理。`npm run desktop` 在 `target/dev/desktop/` 组装并启动开发程序，用户数据不进入交付目录。目录规范见 [AGENTS.md](AGENTS.md)。

Web 与安卓组件库继续作为仓库内部 monorepo 模块维护。四套桥独立维护，构建入口自动获取锁定源码并组装，详见[仓库与构建链路](docs/repositories.md)。Moduloom 预览工具是本地可选依赖，正常安装和 GitHub 构建无需它；需要预览时按[开发文档](docs/development.md#浏览器预览)单独安装。

安装后启动 Pudding，在设置中添加模型，并在 BetterGI 页面连接工具。程序启动时请求管理员权限。默认安装目录为固定 D 盘上的 `D:\Pudding`；没有固定 D 盘时使用 `%LOCALAPPDATA%\Programs\Pudding`。

配置、模型密钥、会话和日志位于安装目录的 `user/`。覆盖构建、安装与更新保留该目录，卸载删除程序及用户数据。从旧名 Sleepy Doll 的安装升级时，首次启动会把旧目录的配置与数据复制到新目录，源目录保持不动。

## 工程结构

| 路径 | 内容 |
| --- | --- |
| `src/` | Rust 桌面壳、Agent 运行时、安装引擎与 Slint 安装界面 |
| `web/` | React 主程序与托盘界面 |
| `packages/` | 共用 Web 组件与 Android 模块 |
| `android/` | Android 工程与构建入口 |
| `frame/` | Loom 组件清单、渲染与主题适配 |
| `plugins/` | 领域技能与工具接入清单 |
| `bridges/` | 四个独立桥仓的本地检出，构建入口按锁定提交自动获取 |
| `scripts/` | 构建、发布、更新验证与缓存清理 |
| `tests/` | 界面、桌面与通用运行时验证 |
| `cloudflare/` | 分发服务 |
| `assets/` | 品牌资产与 Windows 资源文件 |
| `docs/` | 维护文档 |
| `target/` | 构建缓存、生成报告与验证数据，不进 Git |
| `dist/` | 最终软件交付物及其校验文件、清单，不进 Git |

宣传片制作区 `video/` 与本机适配资料 `test/` 由维护者在本地保留，不进入仓库。

## 文档

- [完整文档导航](docs/README.md)
- [贡献指南](CONTRIBUTING.md)与[安全报告](SECURITY.md)
- [开发与验证](docs/development.md)
- [配置与数据](docs/configuration.md)
- [运行内核与调用链路](docs/internals.md)
- [技能与插件](docs/extensions.md)
- [快捷任务](docs/quick-tasks.md)
- [发布与更新](docs/distribution.md)
- [BGI 桥](bridges/bgi/bgi-bridge/README.md)、[宿主契约](docs/bgi/host-contracts.md)、[配置恢复](docs/bgi/config-recovery.md)

## 许可

[MIT](LICENSE)。BetterGI 是独立项目，其许可与资源使用条件分别遵守原项目要求。
