# WebView 前端定时器与硬件控制的坑位记录

本项目在应对 TDP 锁定延迟问题时，踩到了一次典型的 WebView/Chromium 定时器节流坑。这里简单记录经验，避免以后在类似场景再踩一次。

## 背景

- 前端：Svelte + Tauri WebView（Edge/Chromium 内核）。
- 功能：TDP 滑条 / 预设 调整 CPU 功耗上限，通过 Tauri `invoke("set_tdp_auto")` 调用后端 Rust，后端再通过 LhmHelper 管道设置 TDP，并有 TDP 锁定服务每 3 秒重写一次。
- 为了避免拖动滑条时频繁调用后端，前端在 `applyTdp` 里加了一层防抖：
  - `setTimeout(300)` 里再调用 `invoke("set_tdp_auto")`。

**现象：**

- 程序刚启动后的 1–2 分钟内，TDP 调整响应极快（几百毫秒内生效）；
- 运行几分钟后，在游戏里用手柄调 TDP：
  - UI 数值会立即更新；
  - 但硬件功耗 / 行为依旧停留在旧值上；
  - 过了一段时间（几十秒甚至一两分钟）才突然跳到新值；
- 用户直观感受是「TDP 锁死旧值」或「TDP 非常久才生效」。

## 根因分析

### 1. WebView 后台定时器节流

Chromium/Edge 对**后台标签页 / 隐藏窗口**会做激进定时器节流：

- 前台可见时：`setTimeout(fn, 300)` 基本就是几百毫秒级；
- 后台 / 不可见时间较长后：
  - 先把所有定时器统一节流到秒级；
  - 进入 intensive throttling 后，**每分钟才允许执行一次回调**。

Tauri 的 WebView 直接继承了这套行为。

在本项目中：

- 面板绝大多数时间是**隐藏的**（玩家在游戏里，只通过手柄快捷键调 TDP）；
- TDP 的真实后端调用被挂在：

  ```ts
  tdpDebounceTimer = window.setTimeout(() => {
    invoke("set_tdp_auto", { tdp: value });
  }, 300);
  ```

- 当 WebView 被视为后台窗口时，这个 `setTimeout(300)` 实际上会被拖到**几十秒甚至更久**，极端时是一分钟一次；
- 因此：
  - UI 里的 TDP 数字是同步更新的（立即改变 `finalTdp/tdpWatts`）；
  - 但真正的 `set_tdp_auto` 被延迟到「下一次定时器被放行」时才执行；
  - 时间越久，节流越激进，于是玩家感受到「越玩越久，TDP 越难改动」。

### 2. 为什么看起来像「锁死」

后端 Rust 里有 TDP 锁定服务：

- `set_tdp_auto` 会：
  - 更新 `TDP_LOCKED_VALUE`；
  - 立即应用一次 TDP；
  - 启动后台线程 `tdp_service_loop`，每秒检测功耗、每 3 秒强制重写一次。

理论上，只要 `set_tdp_auto` 被调用，新值就能立刻覆盖旧锁定值，并由后台线程不断重写。

**问题在于：** 当 `set_tdp_auto` 迟迟没被调用，新值连 `TDP_LOCKED_VALUE` 都没写进去，后台服务自然只能继续「忠实锁住旧值」。

这就是为什么：

- 刚打开面板时（前台），TDP 调整非常灵敏；
- 关上面板，跑游戏一段时间后，TDP 看起来「完全不变」，直到很久以后才突然跳一下。

## 教训：哪些场景不要依赖前端定时器

在 Tauri / WebView 类应用里，**凡是满足以下条件之一的逻辑，都不应该把“真实动作”放在 `setTimeout/setInterval` 里**：

1. 窗口/面板可能长期隐藏、最小化或在后台运行；
2. 定时器回调里会：
   - 调用 Tauri `invoke(...)` 去**改硬件 / 动系统设置**；
   - 写配置文件 / 状态到磁盘；
   - 做与用户体验强相关的「必须及时」操作。

`setTimeout` 可以用来做纯 UI 行为，例如：

- 延迟隐藏 toast；
- 控制按钮点击节奏（但真正业务逻辑仍然立即执行，只是按钮短暂禁用）。

但它**不适合作为“是否真正调用后端”的唯一门槛**。

## 推荐实践

### 1. 关键操作：前端立即调用后端，防抖放在后端

原则：

> 硬件相关 / 系统相关 / 关键状态写入，尽量让「是否执行」和「什么时候执行」都由后端控制。

实现方式：

- 前端事件：直接 `invoke("xxx")`，不做或少做时间控制；
- 后端：在命令实现里自己做限频 / 防抖，例如：

  ```rust
  static LAST_TDP_APPLY_MS: AtomicU64 = AtomicU64::new(0);
  const MIN_INTERVAL_MS: u64 = 100; // 例：100ms 内只接受一次

  #[tauri::command]
  async fn set_tdp_auto(app: AppHandle, tdp: i32) -> CommandResult {
      use std::time::{SystemTime, UNIX_EPOCH};

      let now = SystemTime::now().duration_since(UNIX_EPOCH)
          .unwrap_or_default().as_millis() as u64;
      let last = LAST_TDP_APPLY_MS.load(Ordering::Relaxed);

      if now.saturating_sub(last) < MIN_INTERVAL_MS {
          // 仅更新内存状态，不立即打硬件，或者直接丢弃本次请求
          return CommandResult::ok();
      }

      LAST_TDP_APPLY_MS.store(now, Ordering::Relaxed);

      // 真正应用 TDP 的逻辑...
  }
  ```

好处：

- 不受 WebView 前台/后台影响；
- 所有节奏由 Rust 端掌控，更可预测。

### 2. 条件防抖：只在面板可见时使用轻微防抖

若确实需要减轻某类操作负载（例如频繁拖动某个 UI 滑条），可以：

- 仅在面板**可见**时，用浅一点的 `setTimeout` 防抖（100–200ms）；
- 面板隐藏 / 后台时，事件就直接 `invoke`，不再走 JS 防抖链路。

伪代码：

```ts
if (panelIsOpen) {
  // 前台：允许 UI 防抖
  if (timer) clearTimeout(timer);
  timer = setTimeout(() => invoke("set_xxx", { value }), 150);
} else {
  // 后台：直接调用后端
  invoke("set_xxx", { value });
}
```

### 3. JS 定时器只用于纯前端效果

推荐仅用 `setTimeout/setInterval` 做这些事情：

- 动画节奏（不影响逻辑正确性）；
- 提示/气泡的自动消失时间；
- 键盘导航 / 手柄导航的长按连发（但真正的「一次动作」仍然由事件本身触发）。

一旦涉及「是否真的去 `invoke` 做某个关键操作」，宁愿：

- **不防抖**，把限频放在后端；
- 或者只做前端可见状态下的轻量防抖。

## 本仓库中已采取的修复措施

针对 TDP 控制，已进行如下调整：

- 原先 `applyTdp` 中通过 `setTimeout(300)` 防抖后再 `invoke("set_tdp_auto")` 的逻辑已移除；
- 现实现为：
  - 用户每次调整 TDP：
    - 立即更新前端 UI 状态；
    - 立即调用 `set_tdp_auto` 或 `clear_tdp`；
    - 立即保存相关配置；
- 防抖 / 频率控制交由后端 TDP 锁定服务和硬件调用超时机制处理。

验证结果：

- 在面板隐藏、长时间游玩游戏的场景下，多次测试表明：
  - TDP 调整在任意时刻都能快速生效；
  - 不再出现「前 1–2 分钟正常、之后要等一两分钟才生效」的现象。

## 总结

**一句话版：**

> 在 Tauri/Chromium WebView 里，不要把关键 `invoke` 放在 `setTimeout` 里，尤其当窗口可能长期隐藏时；
> 把「什么时候真正执行」交给后端（Rust）会更可靠。

以后如果需要对硬件/系统状态做防抖或限频，优先考虑：

1. 后端维护节奏；
2. 前端只做 UI 层的小修饰；
3. JS 定时器只用于「就算被节流也不会出大问题」的场景。