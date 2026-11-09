# 插件目录

本目录存放随产品分发的正式插件：清单（`.pudding-plugin/plugin.json`）、技能文档与原生桥。运行时由主程序按配置的插件目录动态发现，`src/` 对具体插件零静态引用。

## 两类插件

- **纯清单插件**：`web-search`、`windows-scheduler`、`example-weather`，只有清单与技能文档。
- **适配插件**：`bgi`（随 BetterGI 桥工作）、`march7th`、`zzz`、`maaend`。后三者的 `bridge/` 是注入或附加到第三方工具的本机 C++ 代码，各自包含 `build.cmd`、注入器/桥源码与伪宿主测试。

## 适配插件是可整体搬出的单元

`march7th`、`zzz`、`maaend` 三个适配插件计划在维护者组织下拆分为独立仓库。拆分边界如下，搬出时不需要改动主程序：

- 每个插件目录自包含：桥源码、构建入口、测试与技能文档互不依赖其他插件。
- 与宿主的唯一契约是 **localhost HTTP + `X-Pudding-Token` 请求头**（端口与端点见各自清单），没有任何进程内 API 或路径耦合。
- `bridge/build.cmd` 自带独立布局守卫：上三级没有仓库标志（`Cargo.toml`）时改为在脚本自身目录构建输出，搬出后零修改可用。
- 主仓库侧的唯一耦合点是 `scripts/package-adapters.ps1` 的显式组装清单；拆分后从该清单移除对应条目，由新仓库自行交付。

## 交付

适配插件不随默认安装包安装；独立交付目录由 `scripts/package-adapters.ps1` 按明确清单组装为 `dist/plugins/<插件 ID>/`，并附文件校验。包内不含第三方工具本体、用户配置、凭据或测试程序。
