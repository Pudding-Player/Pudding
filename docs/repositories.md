# 仓库与构建链路

Pudding 主仓保留 `src/`、`web/`、`android/`、`packages/ui/`、`packages/ui-android/`、通用插件及组装入口。UI 继续使用 npm workspace 和 Gradle 项目引用，不从 Pudding-UI 仓拉取源码。该远端空仓暂不使用。

桥仓各自独立维护，主仓通过 [桥版本锁定文件](../scripts/bridges.lock.json) 记录完整提交 SHA、本机检出位置、产品文件清单和是否随安装包分发。不使用 submodule，也不把桥源码提交回主仓。

| 仓库 | 本地唯一源码位置 | 内容 |
| --- | --- | --- |
| Pudding | 工程根目录 | 主程序、UI monorepo、通用插件、公共协议生成器与安装组装 |
| Pudding-BGI-Bridge | `bridges/bgi/` | `bgi-bridge/`、`plugins/bgi/` |
| Pudding-March7th-Bridge | `bridges/march7th/` | `plugins/march7th/` |
| Pudding-ZZZ-Bridge | `bridges/zzz/` | `plugins/zzz/` |
| Pudding-MAAEnd-Bridge | `bridges/maaend/` | `plugins/maaend/` |

## 获取与本地开发

```powershell
git clone https://github.com/Pudding-Player/Pudding.git
cd Pudding
npm run bridges:fetch
npm ci
npm run dist
```

只需手动克隆主仓。`npm run dist` 也会先检查并获取缺少的桥仓；没有 node_modules 时先执行 npm ci。各桥目录是固定的正式检出，不是缓存、临时副本或 submodule。日常开发的构建使用现存源码，保留未提交修改，不自动更新现有桥分支。显式运行 `bridges:fetch` 会将干净检出切换到锁定提交；有修改且需要切换时拒绝，不执行 reset 或清理。

`npm run bridges:status` 汇总四个桥的实际提交、锁定提交与未提交修改。`npm run bridges:build` 编译启用的生产桥并生成 `target/bridge/bridge-build.json`，记录源码指纹和产物校验。组包前再次核对，拒绝源码或二进制在构建后发生变化。

主仓的 `npm run backend` 会准备 `target/dev/plugins/` 中的产品插件资源；前端开发仍运行 `npm run dev`。普通开发不自动启动桥、工具或游戏。桥仓单独克隆后可按各自 README 构建，生成文件已随仓提交。

## 更新依赖与正式构建

先在对应桥仓提交、推送到 main，再在主仓执行 `npm run bridges:lock`，审阅并提交锁定文件。锁定命令只接受已推送到桥仓 main 的干净提交，不将主仓工作文件复制到远端。

GitHub Actions 与显式 `--locked` 构建使用锁定 SHA；拒绝未提交的桥源码。主仓只跟踪四个仓库的明确提交，不在发布时取分支最新值。私有桥仓需要在主仓配置仅能读取这些仓库的 Actions secret `BRIDGE_READ_TOKEN`；脚本只在 Git 子进程环境中使用，不写入 URL、文件或凭据配置。公开桥仓不需要该 secret。

构建顺序：获取桥源码 → 编译启用的桥 → 构建主仓 Web → Rust 编译并嵌入 target/ui → 按清单组装 target/setup/staging → 生成载荷和安装包。当前仅 BGI 桥随安装包分发，插件位于 plugins/bgi，原生组件位于 bridge/。三月七、ZZZ、MAAEnd 在锁定清单中设置 enabled=false、bundled=false，暂不获取、构建、加载或分发；恢复时重新启用对应条目。

主仓 `packages/bridge-disclosure/` 是公共协议生成输入的唯一维护位置。生成器把协议与各桥清单编译成桥仓内的 generated 头文件；更新需在对应桥仓分别提交。桥仓生产构建只使用已提交的生成结果，不向主仓反向获取代码。

构建、暂存和最终输出继续遵循 [目录规范](../AGENTS.md)。临时资源只在登记的 target 子目录内；本地配置、令牌、用户数据、宿主工具和开发夹具不进入安装包。
