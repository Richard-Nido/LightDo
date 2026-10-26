# 当前接续点

目标仍是完整 LightDo、本机真实验收、Windows x64 安装包及公开正式 Release。用户已授权本项目源码推送和正式发布，已确认使用 Richard-Nido/LightDo 空公开独立仓库。不需要重新询问产品需求或发布权限。

首先读 `Prompt/LightDo_Codex_Development_Prompt.md`、`docs/verification.md`、`docs/architecture.md`。上游 SHA 和许可见 THIRD_PARTY_NOTICES.md。

## 当前接续状态

旧实例首次隐藏提示已由用户关闭，正常退出后已替换为新实现，无需再次要求关闭旧提示。后台优化与界面调整已在标准 target/x86_64-pc-windows-msvc/release/lightdo.exe 运行；不要再用旧 target/candidate 覆盖它。优化后隐藏实测 64.93 秒，CPU 0.0012%、GPU 0%、平均工作集 95.26 MiB，前后 visible=false、visibleOwnedPopup=false。见 performance.md 的二进制哈希和场景边界。可见静态/动态、切换泄漏仍待复测。

后续用户界面要求已实现：菱形按钮恢复（悬浮置顶、边缘固定、桌面禁用）；40 DIP 轻色玻璃输入框与居中占位字；空列表仅保留 14 DIP「在上方写下第一件事」；标题三图标与高亮块共用居中几何；正常状态不显示左下角模式文字。实际悬浮置顶切换、草稿跨模式/拖动、中文与 Emoji 新增、鼠标选区已观察。新编辑表面的拼音测试发现占位字重叠，已在组合开始清除并交还原生绘制；候选显示正常且组合 Enter 未新增任务。最终完整 IME/不同 DPI 复测仍需补齐。20 项常规测试通过，dist 安装包仍为旧开发构建，最终打包尚待完成。

一次亮度元数据读回优化被撤回，生产源码维持原采样密度；原生首帧集成用例在当前全屏置顶背景下恢复原实现后也 FAIL。local-evidence/capture-regression.json 保留四次失败结果，相关实验补丁也仅保存在 local-evidence；需要排查原因，不能抹去失败或擅自放宽断言。重新通过本机图形验证前不可正式发布。

用户已解锁 Windows，WTS 已复核 lockFlag=1、locked=false。用户针对新版 71b5f56 桌面 Win+D 往返、添加/编辑、层级反馈「一切正常！」，菜单 F10/方向键/Esc 与设置模式选择反馈「可以实现」，对应项目已通过。更早旧子窗口反馈没有混用。

用户后续明确要求完全移除 QuietDo 导入，见 product-decisions.md，此要求覆盖初始 Prompt 的导入条款。当前构建包含 26b770d 的行为修正；浅深色主菜单已实际确认无导入。恢复的菱形依模式表示固定或置顶，覆盖此前隐藏按钮的决定。隔离测试数据保留，不读取或改写 QuietDo 数据。

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
