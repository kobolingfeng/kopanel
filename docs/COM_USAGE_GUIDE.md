# Windows COM 使用指南

本项目中多处使用了 Windows COM (Component Object Model) 接口。本文档记录正确的 COM 使用方式，避免资源泄漏和线程模型冲突问题。

## 基本原则

1. **每次 `CoInitializeEx` 必须配对 `CoUninitialize`**
2. **COM 初始化是线程级别的**，不同线程需要各自初始化
3. **线程模型不能混用**：同一线程不能先用 `COINIT_APARTMENTTHREADED` 再用 `COINIT_MULTITHREADED`
4. **某些 COM 类需要对应进程已运行**（如 `UIHostNoLaunch` 需要 TabTip.exe 已启动）

## 正确的代码模板

```rust
use windows::Win32::System::Com::{CoInitializeEx, CoUninitialize, COINIT_APARTMENTTHREADED};

fn use_com_api() -> Result<(), String> {
    unsafe {
        // 1. 初始化 COM，记录是否需要清理
        let co_init_result = CoInitializeEx(None, COINIT_APARTMENTTHREADED);
        let need_uninit = co_init_result.is_ok();
        
        // 2. 使用闭包包装业务逻辑，确保无论成功失败都能清理
        let result = (|| {
            // ... COM 操作 ...
            // 创建 COM 对象
            // 调用接口方法
            // 释放 COM 对象 (调用 Release)
            Ok(())
        })();
        
        // 3. 清理 COM（必须！）
        if need_uninit {
            CoUninitialize();
        }
        
        result
    }
}
```

## 线程模型选择

| 模型 | 常量 | 适用场景 |
|------|------|----------|
| 单线程套间 (STA) | `COINIT_APARTMENTTHREADED` | UI 相关、大多数 COM 组件 |
| 多线程套间 (MTA) | `COINIT_MULTITHREADED` | 后台任务、无 UI 交互 |

**推荐**：如果不确定，使用 `COINIT_APARTMENTTHREADED`，兼容性更好。

## 常见错误码

| HRESULT | 含义 | 解决方案 |
|---------|------|----------|
| `0x80040154` | REGDB_E_CLASSNOTREG - 类未注册 | 对应进程可能未运行，或 COM 组件未安装 |
| `0x8001010E` | RPC_E_WRONG_THREAD - 错误的线程 | 检查线程模型是否匹配 |
| `0x80010106` | RPC_E_CHANGED_MODE - 模式已更改 | 线程已用不同模式初始化过 COM |
| `S_FALSE` (1) | 已初始化 | 正常情况，该线程之前已调用过 CoInitializeEx |

## 项目中的 COM 使用位置

### 1. TabTip 触摸键盘切换
- 文件：`keyboard_redirect.rs`, `panel.rs`
- COM 类：`UIHostNoLaunch` → `ITipInvocation`
- 注意：**必须先确保 TabTip.exe 进程已运行**，否则 `CoCreateInstance` 会返回 `0x80040154`

```rust
// 正确做法：先检查进程是否运行
if !is_tabtip_running() {
    open_tabtip_process();
    thread::sleep(Duration::from_millis(300));
    return; // 进程启动后会自动显示键盘
}
// TabTip 已运行，再使用 COM toggle
toggle_tabtip_com();
```

### 2. 任务计划创建
- 文件：`main.rs`
- COM 类：`TaskScheduler` → `ITaskService`
- 使用 `COINIT_MULTITHREADED` 因为是后台操作

## 调试技巧

1. **打印 HRESULT**：`println!("COM error: 0x{:X}", hr);`
2. **检查返回值**：`CoInitializeEx` 返回 `S_OK` (0) 或 `S_FALSE` (1) 都是成功
3. **确认进程状态**：使用 `tasklist` 检查依赖的进程是否运行

## 参考资料

- [Microsoft COM Documentation](https://docs.microsoft.com/en-us/windows/win32/com/component-object-model--com--portal)
- [windows-rs crate](https://github.com/microsoft/windows-rs)
