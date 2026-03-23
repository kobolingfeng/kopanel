# WARP.md

This file provides guidance to WARP (warp.dev) when working with code in this repository.

## Project Overview

**KoPanel** is a handheld gaming device control panel built with Tauri 2 + Svelte 5. It provides TDP control, fan management, display settings, gamepad configuration, OSD overlay, and game library management for Windows handheld gaming PCs (GPD, ASUS ROG Ally, Lenovo Legion Go, etc.).

## Build & Development Commands

```bash
# Development (cleans locked files + starts dev server)
npm run tauri:dev

# Production build (cleans locked files + builds installer)
npm run tauri:build

# Quick development (without cleanup, faster but may fail if files are locked)
npm run tauri dev

# Clean locked files only (fast, recommended before build)
npm run clean:locked

# Full cache clean (slow - 10-30 min rebuild, use only when build errors persist)
npm run clean:cache
```

### Build Scripts
- `scripts/dev.bat` - Development startup (recommended for daily use)
- `scripts/build.bat` - Production packaging with interactive cleanup prompt
- `scripts/clean-cache.bat` - Full cleanup when encountering build errors

### Rust Fast Test Profile
```bash
cargo build --profile fast-test  # Faster build for testing (no LTO, incremental)
```

## Architecture

### Frontend (`src/`)
- **Framework**: Svelte 5 with TypeScript
- **Entry Points**: 
  - `index.html` → `src/App.svelte` (main control panel)
  - `osd.html` → OSD overlay window
  - `src/components/BigScreen.svelte` (fullscreen mode)
- **State Management**: Svelte 5 runes ($state, $derived)
- **Navigation**: DOM-based gamepad/keyboard navigation (`src/lib/dom-navigation.ts`)
- **i18n**: Multi-language support (`src/lib/i18n.ts`) with Chinese/English

Key frontend files:
- `src/lib/types.ts` - TypeScript type definitions shared with backend
- `src/lib/constants.ts` - UI constants, presets, theme definitions
- `src/lib/api.ts` - Tauri IPC wrapper functions

### Backend (`src-tauri/src/`)
- **Framework**: Tauri 2 with Rust
- **Entry Point**: `main.rs` - App initialization, global shortcuts, tray icon
- **Module Organization**: One file per feature domain

Key Rust modules:
| Module | Purpose |
|--------|---------|
| `tdp.rs` | TDP/power control via PawnIO/RyzenSMU (AMD) and LHM/PawnIO IntelMSR (Intel) |
| `gpu.rs` | GPU clock control, iGPU detection |
| `adlx.rs` | AMD ADLX integration (RSR, Anti-Lag, AFMF, etc.) |
| `display.rs` | Brightness, resolution, refresh rate |
| `power.rs` | Battery status, power plans |
| `ec.rs` | EC RAM fan control for various handhelds |
| `gamepad.rs` | XInput state, button bindings |
| `gilrs_gamepad.rs` | Non-XInput gamepads (DS4, DS5, NS Pro) |
| `gyro.rs` | IMU/gyroscope input handling |
| `vigem.rs` | Virtual gamepad via ViGEm |
| `hidhide.rs` | HidHide device isolation |
| `vibration.rs` | XInput hook for input isolation |
| `rtss.rs` | RivaTuner OSD integration |
| `osd.rs` | Built-in OSD overlay |
| `lhm_bridge.rs` | LibreHardwareMonitor integration |
| `sysinfo.rs` | System metrics (CPU/GPU/RAM/temps) |
| `games.rs` | Game library management |
| `game_library.rs` | Multi-platform game import (Steam, Epic, EA, Xbox, GOG) |
| `trainer.rs` | Game trainer/cheat management |
| `auto_profile.rs` | Automatic game profile switching |
| `presets.rs` | Performance preset management |
| `global_config.rs` | AC/Battery power state configurations |
| `settings.rs` | App settings persistence |
| `panel.rs` | Window positioning, show/hide logic |
| `custom_keys.rs` | Handheld-specific HID buttons (L4/R4, etc.) |
| `keyboard_hook.rs` | Global keyboard shortcut handling |
| `touch_gesture.rs` | Touch gesture detection |
| `updater.rs` | App auto-update |

### Third-Party Tools (`src-tauri/tool/`)
Bundled native tools for hardware control:
- `AMD/` - ADLX wrapper DLL
- `Intel/` - Intel GPU tools
- `rtss/` - RivaTuner Statistics Server (OSD)
- `lhm/` - LibreHardwareMonitor (hardware sensors)
- `lhm-helper/` - .NET helper for LHM
- `vigem/` - ViGEm virtual gamepad driver
- `hidhide/` - HidHide device cloaking
- `xinput-hook/` - XInput DLL injection for input isolation
- `nircmd/` - Windows command-line utilities

### .NET Helper (`tools/lhm-helper/`)
Separate .NET 8 project for LibreHardwareMonitor integration:
```bash
cd tools/lhm-helper
dotnet build
dotnet publish -c Release
```

## Key Patterns

### Tauri IPC
Backend commands are exposed via `#[tauri::command]` and called from frontend using `invoke()`:
```typescript
// Frontend
import { invoke } from "@tauri-apps/api/core";
const result = await invoke("set_tdp", { tdp: 25 });

// Backend (main.rs registers handlers)
#[tauri::command]
async fn set_tdp(tdp: i32) -> Result<(), String> { ... }
```

### Device Detection
The app auto-detects handheld device type for hardware-specific features:
- GPD Win series (EC RAM fan control)
- ASUS ROG Ally (ACPI fan control)
- Lenovo Legion Go (WMI fan control)
- Generic Windows handhelds

### Power State Management
Settings are stored separately for AC (plugged in) and Battery modes:
```rust
pub struct GlobalPerformanceConfig {
    ac: PerformanceSettings,
    battery: PerformanceSettings,
}
```

### Window Behavior
- Panel hides to system tray by default
- Global shortcuts toggle visibility (default: Ctrl+Alt+Shift+G)
- Supports both keyboard and gamepad bindings (LT+RT default)
- Multiple panel width options (20-40% of screen)

## Configuration Files

- `src-tauri/tauri.conf.json` - Tauri app configuration
- `src-tauri/Cargo.toml` - Rust dependencies
- `package.json` - Node/frontend dependencies
- `vite.config.ts` - Vite build configuration (dual entry points)

## Notes

- The app requires **administrator privileges** for TDP control, fan control, and driver operations
- Build outputs go to `src-tauri/target/` (Rust) and `dist/` (frontend)
- Settings are stored in `%APPDATA%/com.kopanel/` on Windows
- Comments in code are primarily in Chinese
