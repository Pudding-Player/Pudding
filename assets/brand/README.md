# Pudding 品牌资产来源

当前品牌方向（2026-11-08 用户批准）：

- **软件图标**：桑多涅 Q 版头像（`avatar.png`，1254×1254，透明底，userApproved: true）。此前的 P 字母图形已废弃。本文件为全尺寸唯一源图；`packages/ui/src/brand/avatar.png`（256px）与 `src/setup/brand-avatar.png`（512px）是从它导出的使用副本，`assets/pudding.ico`、`assets/pudding-setup.ico`（16–256px 多尺寸）由它生成。
- **字标**：`packages/ui/src/brand/wordmark.svg`，不等高 Pudding 字标（P 与小写共用基线），奶油/焦糖双色。插画风字标改稿仍在评审中，未经用户批准前不替换本文件。
- **配色**：奶油 `#FFF6E4 → #F2DFBC`，焦糖 `#FF8A2B → #F56C13` / `#D09B4D → #BD863E`，强调橙沿用 `#FF7A20`。

头像与字标的完整评审记录（manifest.json、selected-concept.json、插画风字标草稿与大图）保存在本地 `video/output/pudding-brand/`，不入库。修改品牌方向时先更新该目录的批准记录，再同步本目录与使用副本。
