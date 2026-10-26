# 本机资源占用实测

2026-10-26；Windows 11 26H2 26300.9711，i9-13900H，20 个逻辑处理器，16 GB 内存，Intel UHD Graphics / RTX 4070 Laptop GPU，150% 缩放。

以下是开发构建测量，不是完整产品最终验收结果。采样工具为本机进程累计 CPU 时间、工作集和 Windows GPU 性能计数器。CPU 按 20 个逻辑处理器归一化；GPU 为轻待进程每次采样中使用率最高的引擎，未包含测试背景进程。

| 场景 | 时长 | CPU 平均 | 内存工作集平均 | 私有提交 | GPU 引擎平均 / 峰值 |
|---|---:|---:|---:|---:|---:|
| 悬浮置顶、外部动态网格，出现黑帧的初轮 | CPU 30.42 s；GPU 30 样本 | 1.13% | 115.05 MiB | 91.30 MiB | 7.42% / 9.03% |
| 隐藏到托盘，首次提示状态尚待关闭 | 33.79 s；GPU 31 样本 | 0.097% | 102.92 MiB | 88.37 MiB | 0% / 0% |
| 优化后隐藏到托盘，无阻塞弹窗 | 64.93 s；GPU 61 样本 | 0.0012% | 95.26 MiB | 71.52 MiB | 0% / 0% |

隐藏样本的共享 GPU 内存计数器为 107.53 MiB，独立显存计数器为 0 MiB。工作集、提交量和驱动内存反映不同的资源指标，分别报告。隐藏样本尚不代表正常确认首次提示后的长期闲置状态。

优化后样本于 2026-10-26 15:09 完成，测量前后均为 visible=false、visibleOwnedPopup=false、桌面未锁定。CPU 累计耗时 15.625 ms；结束工作集 95.21 MiB，句柄 513，共享 GPU 内存平均 93.21 MiB，独立显存为 0 MiB。对应二进制 SHA-256 为 `dfcde1c246de829a841e89814de935cf404ceb384aeba73c48aaeea7d42bfcca`。它是包含标题按钮和紧凑输入框的本机开发构建；不能把不同弹窗状态的前后样本当作严格配对实验，也不推广为可见玻璃的占用或跨设备保证。

原始数值保存在本机被 Git 忽略的 local-evidence/perf-*.json，包含二进制 SHA-256 与采样前后窗口可见状态，不包含任务正文或背景像素。

## 当前优化及验证状态

- 隐藏到托盘、最小化、锁屏和挂起时停止不必要的定时器；边缘自动收起仍保留呼出检测。生命周期策略已通过单元测试。
- 背景捕获入口增加隐藏状态检查，避免已排队的帧通知在隐藏后重建采集。
- 进入长期闲置时释放场景、背景和镜头纹理，清理 Direct2D 缓存，并调用 DXGI Trim 归还闲置驱动缓存；恢复后的图形与输入仍需实际复测。Trim 仅用于闲置阶段，依据 [Microsoft 的 IDXGIDevice3::Trim 文档](https://learn.microsoft.com/en-us/windows/win32/api/dxgi1_3/nf-dxgi1_3-idxgidevice3-trim)。
- 首次隐藏改为非阻塞托盘提示。实际点击关闭后主窗口隐藏、无可见拥有弹窗；隐藏采样完成后再次启动可恢复同一面板并获得输入焦点。
- 20 项常规测试通过。隐藏实测已补齐；可见静态/动态场景、反复切换后的占用和恢复图形稳定性仍需继续验收。

曾尝试降低亮度元数据的 CPU 读回密度，实际玻璃纹理仍保持全分辨率；但受控首帧捕获用例失败，恢复旧密度也失败。该优化和实验性测试修改均未采用。全屏置顶测试背景环境下的捕获集成失败尚未定位，未将其写成通过。

## 复测入口

```powershell
.\scripts\measure-runtime.ps1 -Name tray-after -Seconds 60 -IncludeGpu -ExpectedState Hidden
.\scripts\measure-runtime.ps1 -Name static-after -Seconds 30 -IncludeGpu -ExpectedState Visible
.\scripts\measure-runtime.ps1 -Name live-after -Seconds 30 -IncludeGpu -ExpectedState Visible
```

每次先通过实际操作建立对应场景，测量中保持场景稳定。脚本记录真实时长和窗口状态；计数器不可用时显示不可用，不把缺失值填成 0。不使用强制清空进程工作集的方式制造低内存数字。
