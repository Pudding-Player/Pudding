# Pudding 品牌资产来源

当前品牌方向（2026-11-08 用户批准）：

- **软件图标**：桑多涅 Q 版头像（`avatar.png`，864×828，透明底，userApproved: true）。按可见边缘裁去透明留白，仅保留 2px 抗锯齿余量。此前的 P 字母图形已废弃。本文件为全尺寸唯一源图；`packages/ui/src/brand/avatar.png`（最长边 256px）与 `src/setup/brand-avatar.png`（最长边 512px）是从它导出的使用副本，`assets/pudding.ico`、`assets/pudding-setup.ico`（16–256px 多尺寸）由它居中适配正方形后生成。
- **字标**：插画风 Pudding 全名字标（源图 `video/output/pudding-brand/pudding-wordmark-illustrated.png`，2172×724，本地保留），用户 2026-11-09 确认采用；使用副本为 `packages/ui/src/brand/wordmark.png`（裁去透明留白后等比缩放至 256px 高）。2026-11-06 的几何版 `wordmark.svg` 已停用。
- **配色**：奶油 `#FFF6E4 → #F2DFBC`，焦糖 `#FF8A2B → #F56C13` / `#D09B4D → #BD863E`，强调橙沿用 `#FF7A20`。

头像与字标的完整评审记录（manifest.json、selected-concept.json、插画风字标草稿与大图）保存在本地 `video/output/pudding-brand/`，不入库。修改品牌方向时先更新该目录的批准记录，再同步本目录与使用副本。
