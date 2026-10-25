# 验收记录

状态：开发中，尚未达到正式发布门槛。

| 项目 | 方法/环境 | 当前结果 |
|---|---|---|
| 需求与源码 | 本地完整需求、实际 git clone | PASS；两个 SHA 见 architecture.md |
| 操作系统 | 注册表 CurrentBuild/DisplayVersion/UBR | Windows 11 26H2 26300.9711 x64 |
| GitHub 身份 | gh 官方设备授权、api user | Richard-Nido / Yuan Haoran；已获使用空仓库授权 |
| 原生桌面工具 | computer-use @oai/sky list_apps | PASS；旧 CLI 路径缺失，目录链接指向应用管理的完整 CLI 后恢复 |
| 玻璃动态背景与折射 | 实际显示 examples/glass_background 动态网格，主面板截图；没有使用内置壁纸 | 已观察悬浮模式实时背景、模糊和边缘位移；无自身反馈。仍需补齐拖动、浅色、桌面视觉验收 |
| 边缘/悬浮/桌面 | 单 HWND 4591886，菜单切 Desktop，JSON Mode=Desktop | 悬浮可见可编辑；真实桌面宿主已切入。Win+D 与桌面输入等待人工结果，未判通过 |
| 中文 IME/候选窗 | 实际按 n、i、Return；行内 n、i、Space、Esc；独立原生编辑控件 | PASS 当前缩放：候选窗出现；组合 Enter 未新增任务，第二次 Enter 才新增；Space 选出“你”；Esc 恢复原文。证据 images/p0-ime-inline.jpg |
| Unicode 待办新增 | 原生输入“中文待办 · LightDo 🙂”，Enter，读取隔离数据 | PASS；首尾空白去除，顶部新增，输入清空且保留焦点 |
| 自动化测试 | cargo test --lib | 13 PASS，7 个真实桌面测试最初默认忽略；后续将捕获相关测试单独隔离 |
| 原生捕获集成测试 | scene_first_frame_holds_the_windows / scene_shows / wanted_once_one_is_taken_another_is_read | 首帧和场景测试 PASS；连续帧测试首次 FAIL、重试 PASS，稳定性仍待排查，不能抹去失败 |
| 安装/升级/卸载 | 最终安装包 | NOT TESTED |
| 正式 Release | 标签 + Actions + 资产下载 | NOT RELEASED |

没有将源码推断、编译成功或单元测试写成桌面验收通过。
