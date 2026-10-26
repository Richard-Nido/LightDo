# 验收记录

状态：开发中，尚未达到正式发布门槛。

用户要求进一步确认并尽量降低后台占用。优化前实测和候选状态见 [performance.md](performance.md)。目前已得到真实 CPU、工作集、GPU 引擎与共享显存计数；优化后的结果仍待实际复测。新版首帧捕获集成复测在全屏置顶测试背景环境下出现 FAIL，恢复原采样密度也失败，原因尚未定位；没有扩大历史通过结果。

此前锁屏阻塞已解除：用户回复桌面已解锁，WTS 复核 `lockFlag=1, locked=false`。当前 GetDpiForWindow=144（150%）。2026-10-26 用户针对 71b5f56 新版桌面模式的 Win+D 往返、添加/编辑与普通应用层级反馈「一切正常！」，该项目记为用户手测 PASS；更早旧子窗口方案的反馈仍仅作历史记录。

本轮菜单/设置归并和实际画面见 [menu-redesign.md](menu-redesign.md)。新增中文与 Emoji 任务、首尾空白清理、输入清空且保留焦点、JSON 保存、浅深色菜单和主题切换、子菜单禁用状态、F10 打开与 Esc 关闭已观察。用户手测菜单方向键与设置模式选择反馈「可以实现」，对应项目 PASS。按用户最新要求已完整移除 QuietDo 导入，实际浅深色菜单已检查，原有轻待数据的废弃字段兼容测试通过。

| 项目 | 方法/环境 | 当前结果 |
|---|---|---|
| 需求与源码 | 本地完整需求、实际 git clone | PASS；两个 SHA 见 architecture.md |
| 操作系统 | 注册表 CurrentBuild/DisplayVersion/UBR | Windows 11 26H2 26300.9711 x64 |
| GitHub 身份 | gh 官方设备授权、api user | Richard-Nido / Yuan Haoran；已获使用空仓库授权 |
| 原生桌面工具 | computer-use @oai/sky list_apps | PASS；旧 CLI 路径缺失，目录链接指向应用管理的完整 CLI 后恢复 |
| 玻璃动态背景与折射 | 实际显示 examples/glass_background 动态网格，主面板截图；没有使用内置壁纸 | 已观察悬浮模式实时背景、模糊和边缘位移；无自身反馈。仍需补齐拖动、浅色、桌面视觉验收 |
| 边缘/悬浮/桌面 | 单 HWND 菜单切换；新版桌面由用户 Win+D 手测 | 旧 SetParent 已否决；新版桌面可见/可编辑/普通应用层级 PASS（用户反馈）；边缘和悬浮拖动手势等完整场景仍待补齐 |
| 中文 IME/候选窗 | 实际按 n、i、Return；行内 n、i、Space、Esc；独立原生编辑控件 | PASS 当前缩放：候选窗出现；组合 Enter 未新增任务，第二次 Enter 才新增；Space 选出“你”；Esc 恢复原文。证据 images/p0-ime-inline.jpg |
| Unicode 待办新增 | 原生输入“中文待办 · LightDo 🙂”，Enter，读取隔离数据 | PASS；首尾空白去除，顶部新增，输入清空且保留焦点 |
| 自动化测试 | cargo test --locked --lib | 20 PASS，9 个真实桌面测试/实验明确忽略；覆盖模式偏好、反向展开、隐藏/暂停计时器生命周期，没有把忽略项写成通过 |
| 原生捕获集成测试 | scene_first_frame_holds_the_windows / scene_shows / wanted_once_one_is_taken_another_is_read | 首帧和场景测试 PASS；连续帧测试首次 FAIL、重试 PASS，稳定性仍待排查，不能抹去失败 |
| 连续帧补充复测 | 同一测试独立运行 5 次 | 5/5 PASS；不抹去首次 FAIL，且不等同于全部视觉验收 |
| 桌面子窗口排除探索 | 白宿主/红子窗，对比设置排除标志前后 | FAIL：两组亮度均 0.21176487，不能排除自身，生产实现未采用该路径 |
| CI 基线 | GitHub Actions run 40217914615，提交 0b88a36 | PASS：Windows 检查、测试、x64 编译；仅适用于该基线提交，不代表当前新增功能验收 |
| 新版 CI 首次运行 | run 40222550769，提交 900f90e | FAIL：NSIS 下载内容未通过固定 SHA-256 校验，流程正确停止，未执行该下载。已增加多个官方镜像、HTTPS curl 和固定哈希校验的顺序回退，等待新 CI 结果 |
| 新版 CI 修复后 | [run 40223078172](https://github.com/Richard-Nido/LightDo/actions/runs/40223078172)，提交 d551e7e | PASS：格式、全目标检查、16 项常规测试、x64 构建、固定哈希 NSIS 获取、安装包生成及静默安装生命周期测试 |
| 最新文档提交 CI | [run 40223515215](https://github.com/Richard-Nido/LightDo/actions/runs/40223515215)，提交 df7ab92 | PASS：文档和诊断入口随主分支检查完成；没有改变 pending 的桌面验收状态 |
| 最新设置修正 CI | [run 40224724729](https://github.com/Richard-Nido/LightDo/actions/runs/40224724729)，提交 130d4d0 | PASS：设置模式选择通知修正后，完整检查、构建和安装冒烟通过 |
| 最新模型约束 CI | [run 40225266894](https://github.com/Richard-Nido/LightDo/actions/runs/40225266894)，提交 4e4800d | PASS：持久化模式/置顶/固定展开/快捷键约束及新增测试通过；旧 run 40225240144 的失败来自此前未修正的测试夹具 |
| 最新文档 CI | [run 40225675890](https://github.com/Richard-Nido/LightDo/actions/runs/40225675890)，提交 da11dcb | PASS：发布指纹脚本与接续文档检查、构建和安装冒烟通过 |
| 正式发布门槛 | scripts/release.ps1 -CheckOnly，干净工作区 | PASS（正确阻止发布）：本机视觉/模式/安装运行验收仍 pending，脚本返回失败，不创建标签或 Release |
| 安装器运行中保护 | 开发 EXE 静默安装，隔离测试应用运行中 | PASS：返回 1618，没有安装程序文件，没有强杀进程 |
| 开发 EXE 生成 | NSIS 3.12、SHA-256 校验的编译器、/WX | 已生成 0.1.0 开发安装包和许可 ZIP，未完成安装/升级/卸载整体验收 |
| 静默安装文件级冒烟 | scripts/test-installer.ps1，空的本应用用户目录与隔离安装路径 | PASS：安装、payload 哈希、发布者/卸载引号、同版本覆盖、默认卸载保留合成标记、开始菜单清理；未观察安装 GUI、未测试跨版本升级和主动删除数据选择 |
| 安装/升级/卸载 | 最终安装包 | NOT TESTED |
| 正式 Release | 标签 + Actions + 资产下载 | NOT RELEASED |

没有将源码推断、编译成功或单元测试写成桌面验收通过。

## 必须补齐的实际验收

- 新版浅色、深色、动态背景、圆角折射、拖动/缩放连续刷新、无自身递归与闪烁。
- 新版中文候选窗、Enter 时序、行内 Enter/失焦/Esc、选区、复制粘贴、长文本与键盘删除。
- 左右边缘防误触、延迟返回、编辑阻止收回、固定展开、锁定、置顶、全屏避让、快捷键冲突。
- 新版同一面板三种模式、拖出/重新停靠、桌面图标交互、Explorer 重启；Win+D 往返已获用户实测通过。
- 锁屏后解锁、睡眠恢复、100/125/150/200% 实际 DPI、多屏主屏切换和断开后的找回。几何单元测试仅覆盖比例计算，不能冒充这些实测。
- 设置窗口的滑块、主题、自启、快捷键和恢复默认，异常保存提示。
- 完整安装、首次启动、升级保留数据、卸载保留/删除数据、自启卸载清理。
- 资源占用、启动时间、切换泄漏、从最终 Release 下载后校验并安装；干净 Sandbox/虚拟机尚未执行。

测试硬件：Intel Core i9-13900H、Intel UHD Graphics + RTX 4070 Laptop GPU。最终资源占用尚未测量，不引用上游宣传数字。图片 images/p0-ime-inline.jpg 是早期实际基线运行证据，只能证明当时的 IME 与背景效果，不能作为最新提交的整体验收截图。

当前移除数据导入后的开发安装包为 0.1.0，360180 字节，未签名；SHA-256 为 `5fd32b86a5ad8bcc8cb3b6a13d1e1d5ab274e969dc81a970fd3a04023ac46e21`。程序 PE Machine=0x8664，资源版本 0.1.0，CompanyName=Yuan Haoran (Richard-Nido)。构建保持静态 CRT 与 Windows 系统 DLL 依赖。此开发包尚未完成安装后全交互验收，不能称为正式交付版。历史开发包已被本次构建替代。
