# 当前接续点

目标仍是完整 LightDo、本机真实验收、Windows x64 安装包及公开正式 Release。用户已授权本项目源码推送和正式发布，已确认使用 Richard-Nido/LightDo 空公开独立仓库。不需要重新询问产品需求或发布权限。

首先读 `Prompt/LightDo_Codex_Development_Prompt.md`、`docs/verification.md`、`docs/architecture.md`。上游 SHA 和许可见 THIRD_PARTY_NOTICES.md。

## 当前阻塞

Windows 会话经 WTS 只读诊断确认锁定（lockFlag=0）。不要在锁定桌面上注入输入，不要把黑色窗口捕获判为视觉通过。用户已被请求唤醒并解锁桌面。更早的 SetParent 旧方案 Win+D 手工问题已经作废。

当前生产候选代码使用顶层窗口 + 桌面 Z 序维护，不能恢复被否决的桌面子窗采样方案。子窗排除实验的真实结果是 FAIL，两组采样亮度都约 0.21176487；该实验不是产品验收通过项。

## 解锁后的顺序

1. 运行 `scripts/check.ps1`、`scripts/build.ps1`；只使用与当前提交一致的二进制。
2. 用 `LIGHTDO_DATA_DIR` 指向本项目的 `local-evidence/test-data`，启动 `target/x86_64-pc-windows-msvc/release/lightdo.exe`。已有合成测试任务，不要混入用户的实际数据。
3. 已有动态测试背景是 `examples/glass_background.rs`；检查实例是否存在后再决定启动，避免堆积窗口。该背景只作为外部真实窗口测试，不是产品的固定内置背景。
4. 重新完成 P0：真实背景折射/拖动/浅深色；同一 HWND 的边缘/悬浮/桌面；原生中文 IME 与 Enter 时序。早期图片只证明早期构建，不代表新版已通过。
5. 使用 computer-use 的 @oai/sky 操作真实窗口。Win+D 和锁屏等受该技能限制的步骤由用户手动操作，记录实际反馈；不要使用替代输入通道规避技能限制。
6. 完成完整功能、DPI、设置、恢复、资源占用、安装/覆盖/卸载验收，持续更新 verification.md。正常退出使用托盘或本应用 `--shutdown`，安装器不会强杀进程。
7. 当前 `docs/release-acceptance.json` 为 pending。只有实际完成本机验收后才将项目置为 true，再运行 `scripts/finalize-acceptance.ps1` 生成稳定源码指纹，随后使用 `scripts/release.ps1 -Publish`。跟踪 Actions，下载正式 Release EXE 校验并安装复核。

## 数据与工具

- 正式数据 `%APPDATA%/LightDo/state.json`；隔离开发数据在 `local-evidence`，被 Git 忽略。
- 原始 QuietDo 数据不得改写。导入只复制、显式确认和按 ID 去重。
- Rust 1.99.1 MSVC、VS Build Tools 17.14.42、SDK 10.0.26100、NSIS 3.12 已可用。
- 本地 GitHub CLI 在 `.tools/gh/bin/gh.exe`，账号已官方登录 Richard-Nido；不读取或打印 Token。
- `--diagnose` 只返回本应用窗口和会话状态，不包含任务正文、用户名或截图。
- 开发安装包位于 dist；源码未验收前不要把它标为正式可用版。
