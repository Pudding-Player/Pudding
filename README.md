<div align="center">

<img src="assets/brand/avatar.png" width="88" alt="Pudding" />

# Pudding

用自然语言驱动游戏脚本工具的 Windows 桌面助手。

[![License: GPL v3](https://img.shields.io/badge/License-GPL--3.0--only-blue.svg)](LICENSE) ![Platform](https://img.shields.io/badge/Platform-Windows-0078D6.svg) ![Language](https://img.shields.io/badge/Rust%20%2B%20TypeScript-orange.svg)

[官网](https://sleepy-doll.restless-nh3.com) · [快速开始](#快速开始) · [文档](docs/README.md) · [贡献指南](CONTRIBUTING.md)

</div>

## 快速开始

1. 从[官网](https://sleepy-doll.restless-nh3.com)下载安装包并安装；缺少 WebView2 时安装器会自动补装。
2. 启动 Pudding，在设置中添加模型。
3. 在 BetterGI 页面连接本机的 BetterGI。
4. 使用说明见[官网指南](https://sleepy-doll.restless-nh3.com/docs/guide)。

## 它能做什么

用自然语言提出目标，Pudding 负责检索资料、配置任务、调用工具和核对结果；游戏操作由接入的脚本工具执行。

| 能力 | 说明 |
| --- | --- |
| 自然语言组织操作 | 在对话中描述目标，助手规划步骤、逐项执行并核对结果 |
| 接入工具 | 通过桥接入游戏脚本工具，支持五种模型协议、技能与插件；通用 Agent 运行时与适配层分离 |
| 快捷与定时任务 | 把常用流程保存为快捷任务，手动或按计划执行，无需再次调用模型 |
| 执行记录和操作审批 | 每次执行留有记录可回看，敏感操作先经确认再提交 |

## 适配的脚本工具

| 工具 | 游戏 | 桥源码 | 状态 |
| --- | --- | --- | --- |
| [BetterGI](https://github.com/babalae/better-genshin-impact) | 原神 | [Pudding-BGI-Bridge](https://github.com/Pudding-Player/Pudding-BGI-Bridge) | 已适配 |
| [三月七小助手](https://github.com/moesnow/March7thAssistant) | 崩坏：星穹铁道 | [Pudding-March7th-Bridge](https://github.com/Pudding-Player/Pudding-March7th-Bridge) | 暂时停用，不随安装包分发 |
| [ZZZ一条龙](https://github.com/OneDragon-Anything/ZenlessZoneZero-OneDragon) | 绝区零 | [Pudding-ZZZ-Bridge](https://github.com/Pudding-Player/Pudding-ZZZ-Bridge) | 暂时停用，不随安装包分发 |
| [MaaEnd](https://github.com/MaaEnd/MaaEnd) | 明日方舟：终末地 | [Pudding-MAAEnd-Bridge](https://github.com/Pudding-Player/Pudding-MAAEnd-Bridge) | 暂时停用，不随安装包分发 |

各工具是独立项目，由其原项目维护；桥不会自动启动工具或游戏。

<details>
<summary><b>从源码构建</b></summary>

Windows 构建需要 Rust stable、Node.js 22.18+、.NET 8 SDK 和 Visual Studio 2022 C++ Build Tools；主程序运行需要 WebView2 Runtime。

```powershell
npm run bridges:fetch
npm ci
npm run dist
```

完整构建在 `dist/` 生成当前版本安装包、SHA-256 校验文件和本地构建清单。`npm run desktop` 在 `target/dev/desktop/` 组装并启动开发程序，用户数据不进入交付目录。目录职责见 [AGENTS.md](AGENTS.md)，仓库拆分与完整构建链路见[仓库与构建链路](docs/repositories.md)。

</details>

## 文档

- [文档导航](docs/README.md) —— 配置与数据、运行内核、扩展开发、快捷任务、发布与更新
- [仓库与构建链路](docs/repositories.md) —— 桥仓锁定、本地开发环境与正式构建
- [BGI 桥说明](https://github.com/Pudding-Player/Pudding-BGI-Bridge/blob/main/bgi-bridge/README.md) · [宿主契约](docs/bgi/host-contracts.md) · [配置恢复](docs/bgi/config-recovery.md)
- [贡献指南](CONTRIBUTING.md) · [安全说明](SECURITY.md)

## 许可

本项目以 [GNU GPL v3.0（GPL-3.0-only）](LICENSE) 发布，Copyright © 2026 Pudding contributors。BetterGI 等宿主工具、第三方依赖与外部资源是独立项目，分别遵循其原许可与资源使用条件。
