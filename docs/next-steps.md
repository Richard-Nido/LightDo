# 当前接续点

目标是完整 LightDo、本机真实验收、Windows x64 安装包及公开正式 Release。用户授权源码推送和正式发布，确认使用 Richard-Nido/LightDo 空公开独立仓库，并要求最终交付后清理全部非必要中间文件。不要重复询问产品需求或发布权限。

## 已确认的最新状态

- 需求权威：Prompt/LightDo_Codex_Development_Prompt.md；后续明确修订见 product-decisions.md。按用户要求完全删除 QuietDo 导入；来源和 MIT 许可保留。
- 菱形恢复为悬浮置顶/边缘固定快捷入口，桌面禁用；40 DIP 轻色玻璃输入框、居中占位字、14 DIP 单行空态；标题三图标按可见笔画居中；左下角正常状态不显示模式文字。
- 原生捕获首帧失败已定位到测试红窗仍被 DWM cloak。夹具固定 Z 序、等待红窗真正可见后，原捕获实现连续 3 轮各 12/12 纯红亮度 21。未采用会话重建、DwmFlush 或丢首帧的实验方案。历史失败和实验仅存在 local-evidence。
- 本轮实际动态网格、标题拖动、150% 原生输入框 n/i 候选、Space 选择“你”、Enter 新增通过，截图 docs/images/lightdo-glass-dark-current.jpg 仅含合成任务和受控背景。行内双击进入已观察，提交/取消/失焦仍须补测。
- 启动时保存的置顶偏好未应用已修正，首次启动调用统一 show(false)，诊断 topmost=true、foreground=false。不要把两个互相竞争的置顶测试窗口的 Z 序当成窗口已抢焦点。
- 用户对新版 71b5f56 桌面 Win+D 往返、添加/编辑与层级反馈“一切正常！”，菜单 F10/方向键/Esc 和设置模式选择反馈“可以实现”。旧 SetParent 子窗口方案被否决，不能恢复；当前顶层工具窗维护真实桌面 Z 序。
- 隐藏优化已运行：64.93 秒 CPU 0.0012%、GPU 0%、平均工作集 95.26 MiB，前后 visible=false、visibleOwnedPopup=false。原始二进制哈希和边界见 performance.md；可见场景和切换泄漏仍待。
- 20 项常规测试通过；9 个原生桌面测试/实验不在常规 CI。7eaad6c 的 CI 40286234271 已成功。
- 最新开发安装包 0.1.0 已重建（363199 字节），静默安装、payload 哈希、同版本覆盖、默认保留合成标记、快捷方式清理通过。GUI、跨版本升级、主动删除选择及从正式 Release 下载复核仍未完成。

## 下一步

1. 对当前标准 release 二进制完成新输入表面的行内 Enter/Esc/失焦、组合 Enter、复制粘贴与选区、长文本、完成/恢复/删除、重启持久化。
2. 受控背景完成浅深色、缩放、动态背景和无自身递归；补齐边缘左右触发、防误触、编辑/菜单阻止收回、拖出/重新停靠、锁定和快捷键冲突。
3. 实际 DPI 100/125/150/200%、Explorer 重启、锁屏/睡眠恢复、主屏变化和安全找回；Win+D 已手测，不要重复套用旧子窗口结果。
4. 用 computer-use 的 @oai/sky 操作本应用。Windows 键受技能禁止，必要系统操作由用户手动，不能用替代注入通道绕过。不得自动操作 Codex/ChatGPT 界面。
5. 完成设置、自启、异常保存、资源占用、安装/跨版本升级/卸载验收，再准备正式版本。docs/release-acceptance.json 仍 pending；只能把真实通过项置 true，提交记录后 finalize-acceptance，再 release -Publish。
6. 跟踪正式标签 Actions，下载 EXE 对照 SHA-256、安装运行复核。完成后清理 target、工具/参考缓存、废弃实验和临时测试数据，保留源码、正式 dist 安装包、许可及必要验收结论；检查工作目录整洁。不要清理项目外数据或用户自己的任务。

## 数据与工具

正式数据 %APPDATA%/LightDo/state.json；本轮始终使用 LIGHTDO_DATA_DIR 指向 local-evidence/test-data。不要混用正式数据或读取 QuietDo 数据。当前运行可能保留合成测试任务，请正常退出，不强杀 LightDo。标准二进制在 target/x86_64-pc-windows-msvc/release/lightdo.exe，旧 target/candidate 已过时，禁止覆盖。

Rust 1.99.1 MSVC、VS Build Tools 17.14.42、SDK 10.0.26100、NSIS 3.12。GitHub CLI 在 .tools/gh/bin/gh.exe，已官方登录 Richard-Nido，不读取或打印 Token。--diagnose 不输出任务正文或截图。