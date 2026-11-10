# 主仓插件

本目录维护 windows-scheduler、web-search、example-weather 等通用插件。UI 继续留在主仓 monorepo。

四套宿主桥独立维护于 `bridges/<ID>/`，各仓包含 plugins/<ID> 的清单、技能与桥源码。主仓通过 scripts/bridges.lock.json 固定提交，构建时自动获取、编译并按清单组装；三套附加桥默认停用。

开发装载所需资源由主仓构建入口生成至 target/dev/plugins，安装包使用 target/setup/staging，不向源码 plugins 目录回写产物。详见[仓库与构建链路](../docs/repositories.md)。
