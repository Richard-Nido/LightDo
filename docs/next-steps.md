# 已完成交付与后续维护

正式v1.0.0已公开，源码标签提交c2c476d9f1d2bfdca00f71faf33af19a0705b1b6。标签工作流40447543051的build/publish均成功，Release为公开正式版，6项资产已下载并核对GitHub摘要和校验文件。发布记录见[release-validation.json](release-validation.json)。

公开下载包已安装在D:\LightDo，用户确认玻璃、中文输入、任务与设置正常。当前6条正式任务保留，停靠范围保留用户值8、停靠侧保留用户后续选择Left；新用户默认Right、范围16。任务正文只留本机。

人工结束隔离进程后，12条已保存的合成任务完整恢复。结束回调未被观测，不将其扩大为写入中崩溃测试。隔离实例已正常退出，临时目录已删除，正式6条重新启动且原文件SHA不变。

清理脚本已实际删除target、.tools、.reference、local-evidence、dist/stage和旧0.1.0资产。工作目录保留源码/Git、正式文档/图片及dist当前EXE、许可ZIP、SHA256SUMS这3项交付物；D盘应用与正式数据未受影响，无遗留保护目录。已核对安装包与公开资产摘要一致。

后续修改按[开发与发布](releasing.md)：修改源码及版本说明，使用Prepare准备1.0.1，完成实际验收后finalize，再Publish触发新标签自动构建和公开发布。已公开的1.0.0不改写；main中的后续提交仅补充本次交付记录。不要把历史构建性能或截图扩大为新版本的直接实测。

GitHub CLI现位于%LOCALAPPDATA%/Programs/GitHub CLI/gh.exe，登录与Git凭据助手已保留；项目缓存清理后仍可推送。新终端读取更新后的用户PATH。用户要求避免Computer Use，后续优先文件、命令行与诊断，必须的桌面交互再给简短人工清单。

单物理显示器、单一Windows构建、未签名等限制见[验收记录](verification.md)，各场景资源数据见[performance.md](performance.md)。
