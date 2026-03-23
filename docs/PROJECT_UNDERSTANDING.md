# KoPanel 项目理解文档

**创建时间**: 2026-01-23 08:09 UTC
**当前状态**: 初步理解完成

---

## 1. 项目概述

**项目名称**: KoPanel (kopanel)
**技术栈**: Tauri 2 + Svelte 5 + Rust
**目标平台**: Windows (GPD 掌机控制面板)
**版本**: 0.9.150

### 核心定位
这是一个专为 GPD Windows 掌机设计的系统控制面板，提供：
- TDP (功耗) 控制
- 触摸键盘管理
- 游戏手柄映射/陀螺仪
- 性能优化
- OSD 覆盖显示
- 游戏库管理
- 修改器/外挂集成

---

## 2. 架构分析

### 2.1 前端 (Svelte 5)
- **入口**: `src/main.ts` → `src/App.svelte`
- **多页面**:
  - `index.html`: 主面板 UI
  - `osd.html`: OSD 覆盖层
- **组件**: `src/components/`
- **样式**: `src/styles.css` (159KB)

### 2.2 后端 (Rust/Tauri)
**核心模块** (src-tauri/src/):
```
硬件控制:
- ec.rs (59KB) - EC 嵌入式控制器
- tdp.rs (101KB) - TDP 功耗管理 [最大模块]
- device.rs - 设备检测
- gpu.rs - GPU 管理
- adlx.rs (43KB) - AMD GPU 接口

输入设备:
- gamepad.rs (82KB) - 手柄管理
- gyro.rs (86KB) - 陀螺仪支持
- gilrs_gamepad.rs - 跨平台手柄 (DS4/DS5/Switch Pro)
- vibration.rs (64KB) - 震动反馈
- vigem.rs - ViGEm 虚拟手柄驱动
- custom_keys.rs (38KB) - 自定义按键
- kbm_mapper.rs (52KB) - 键鼠映射

UI/显示:
- panel.rs (58KB) - 主面板控制
- osd.rs (31KB) - OSD 覆盖
- display.rs (33KB) - 显示/HDR 管理
- keyboard_redirect.rs - 触摸键盘

游戏相关:
- games.rs (44KB) - 游戏检测
- trainer.rs (26KB) - 修改器集成
- game_library/ - 游戏库管理
- auto_profile.rs - 自动配置切换
- auto_tdp.rs - 自动功耗调整

性能监控:
- rtss.rs (76KB) - RivaTuner OSD
- lhm_bridge.rs (71KB) - LibreHardwareMonitor 桥接
- optimization.rs (43KB) - 系统优化
- lossless_scaling.rs - Lossless Scaling 集成

系统集成:
- main.rs (77KB) - Tauri 主程序
- power.rs - 电源管理
- media.rs - 媒体控制
- toast.rs - 通知
- widget_server.rs - Xbox Game Bar Widget 通信
```

---

## 3. 关键技术特征

### 3.1 COM 使用
**文档**: `COM_USAGE_GUIDE.md`
- TabTip 触摸键盘 (UIHostNoLaunch)
- 任务计划程序 (ITaskService)
- 传感器 API (陀螺仪)
- 严格的 `CoInitializeEx`/`CoUninitialize` 配对

### 3.2 Windows 特权操作
- TDP 控制需要管理员权限
- PawnIO/RyzenSMU 驱动（TDP/EC 控制）
- HIDHide 设备隐藏
- 任务计划自启动

### 3.3 开发工具
- **构建**: `build.bat`, `build-fast.bat`, `build-release.bat`
- **清理**: `scripts/clean-locked.ps1` (处理锁定文件)
- **测试配置**: `[profile.fast-test]` in Cargo.toml
- **调试**: `console` feature 启用控制台

### 3.4 第三方集成
- **ViGEm**: 虚拟手柄驱动
- **RTSS**: RivaTuner 帧率显示
- **LibreHardwareMonitor**: 硬件监控
- **ADLX**: AMD GPU SDK
- **gilrs**: 跨平台手柄 (支持非 XInput)
- **WMI**: 5-10倍快于 wmic.exe

---

## 4. 已知设计注意事项

### 4.1 定时器陷阱
**文档**: `TIMER_PITFALLS.md`
(需进一步查看内容)

### 4.2 主题开发
**文档**: `THEME_DEVELOPMENT_NOTES.md`
(需进一步查看内容)

### 4.3 线程模型
- 多线程 Tokio 运行时
- COM 操作需要正确的线程模型 (STA/MTA)
- 全局状态使用 `lazy_static` + `Mutex`

---

## 5. 编译优化

### Release 配置
```toml
panic = "abort"
codegen-units = 1      # 单线程编译
lto = "fat"            # 完整 LTO
opt-level = 3          # 最高性能
strip = true
```

### Fast Test 配置
```toml
[profile.fast-test]
lto = false
opt-level = 0
codegen-units = 256
incremental = true
```

---

## 6. 下一步行动

当前理解程度: **40%** (架构清晰，细节待深入)

**待探索**:
1. `TIMER_PITFALLS.md` 内容
2. `THEME_DEVELOPMENT_NOTES.md` 内容
3. 前端组件结构 (`src/components/`)
4. 游戏库实现 (`game_library/`)
5. 具体的 EC 通信协议
6. 构建/发布流程细节

**可能问题点**:
- COM 资源泄漏 (已有规范文档)
- 多线程同步问题
- 权限提升处理
- 驱动依赖检测

---

## 7. 快速参考

### 开发命令
```powershell
npm run dev              # Vite 开发服务器
npm run tauri:dev        # Tauri 开发模式 (含清理)
npm run build            # 前端构建
npm run tauri:build      # 完整打包

# 后端
cargo build --profile fast-test   # 快速测试构建
cargo run                          # 直接运行
```

### 关键路径
- 前端入口: `src/App.svelte`
- 后端入口: `src-tauri/src/main.rs`
- 配置: `src-tauri/tauri.conf.json`
- 图标: `icons/`

---

**状态**: ✅ 已建立基础理解，等待具体任务指令
