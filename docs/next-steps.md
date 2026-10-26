# 当前接续点

目标仍是完整 LightDo、本机真实验收、Windows x64 安装包及公开正式 Release。用户已授权本项目源码推送和正式发布，已确认使用 Richard-Nido/LightDo 空公开独立仓库。不需要重新询问产品需求或发布权限。

首先读 `Prompt/LightDo_Codex_Development_Prompt.md`、`docs/verification.md`、`docs/architecture.md`。上游 SHA 和许可见 THIRD_PARTY_NOTICES.md。

## 当前接续状态

用户最新要求确认后台占用并尽量轻量化。见 performance.md：优化前隐藏样本 CPU 0.097%、GPU 0%、工作集 102.92 MiB；该样本的首次提示状态尚待关闭，不能当成正常闲置最终验收。旧运行实例因首次隐藏的阻塞提示无法由窗口工具定位，用户已被异步请求点“确定”。不要强杀。性能优化候选在 target/candidate/x86_64-pc-windows-msvc/release/lightdo.exe；正常退出旧实例后再替换并实际测 tray-after/static-after/live-after。候选已把首次提示改为非阻塞托盘通知，加入隐藏/锁定/最小化停计时器、隐藏捕获防重建和闲置 GPU Trim，20 项常规测试通过。

一次亮度元数据读回优化被撤回，生产源码维持原采样密度；原生首帧集成用例在当前全屏置顶背景下恢复原实现后也 FAIL。local-evidence/capture-regression.json 保留四次失败结果，相关实验补丁也仅保存在 local-evidence；需要排查原因，不能抹去失败或擅自放宽断言。重新通过本机图形验证前不可正式发布。

用户已解锁 Windows，WTS 已复核 lockFlag=1、locked=false。用户针对新版 71b5f56 桌面 Win+D 往返、添加/编辑、层级反馈「一切正常！」，菜单 F10/方向键/Esc 与设置模式选择反馈「可以实现」，对应项目已通过。更早旧子窗口反馈没有混用。

用户后续明确要求完全移除 QuietDo 导入，见 product-decisions.md，此要求覆盖初始 Prompt 的导入条款。当前标准 release 路径已重建并启动移除导入的新版本，包含 26b770d 的行为修正。测试数据仅将模式设为 Floating 以继续自动测试，其余任务保留；旧导入偏好字段已在正常设置保存时被移除。浅深色主菜单已实际确认无导入、非边缘模式无固定展开按钮。19 项常规测试通过，安装包已重建。

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
- 只使用轻待的数据目录，导入功能已按用户要求完全移除；不读取或改写 QuietDo 数据。
- Rust 1.99.1 MSVC、VS Build Tools 17.14.42、SDK 10.0.26100、NSIS 3.12 已可用。
- 本地 GitHub CLI 在 `.tools/gh/bin/gh.exe`，账号已官方登录 Richard-Nido；不读取或打印 Token。
- `--diagnose` 只返回本应用窗口和会话状态，不包含任务正文、用户名或截图。
- 开发安装包位于 dist；源码未验收前不要把它标为正式可用版。
