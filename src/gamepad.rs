//! 鎵嬫焺鎺у埗妯″潡

use crate::device::detect_device;
use crate::settings::load_settings;
use crate::types::{CommandResult, GamepadState, Settings};
use serde::Serialize;
use std::sync::atomic::{AtomicBool, AtomicU8, Ordering};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_global_shortcut::GlobalShortcutExt;

#[cfg(windows)]
use windows::Win32::UI::Input::XboxController::{
    XInputGetState, XInputSetState, XINPUT_GAMEPAD_A, XINPUT_GAMEPAD_B, XINPUT_GAMEPAD_BACK,
    XINPUT_GAMEPAD_DPAD_DOWN, XINPUT_GAMEPAD_DPAD_LEFT, XINPUT_GAMEPAD_DPAD_RIGHT,
    XINPUT_GAMEPAD_DPAD_UP, XINPUT_GAMEPAD_LEFT_SHOULDER, XINPUT_GAMEPAD_LEFT_THUMB,
    XINPUT_GAMEPAD_RIGHT_SHOULDER, XINPUT_GAMEPAD_RIGHT_THUMB, XINPUT_GAMEPAD_START,
    XINPUT_GAMEPAD_X, XINPUT_GAMEPAD_Y, XINPUT_STATE, XINPUT_VIBRATION,
};

#[cfg(windows)]
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_MOUSE, MOUSEEVENTF_LEFTDOWN, MOUSEEVENTF_LEFTUP,
    MOUSEEVENTF_MOVE, MOUSEEVENTF_RIGHTDOWN, MOUSEEVENTF_RIGHTUP, MOUSEINPUT,
};

#[cfg(windows)]
use windows::Win32::Foundation::HWND;

#[cfg(windows)]
use windows::Win32::Foundation::POINT;

#[cfg(windows)]
use windows::Win32::UI::WindowsAndMessaging::{
    GetAncestor, GetForegroundWindow, WindowFromPoint, GA_ROOT,
};

#[cfg(windows)]
use std::sync::OnceLock;
#[cfg(windows)]
use windows::Win32::System::LibraryLoader::{GetProcAddress, LoadLibraryW};

// XInputGetStateEx: 鏈叕寮€ API锛坸input1_4.dll ordinal #100锛夛紝鏀寔璇诲彇 Guide 鎸夐挳
#[cfg(windows)]
type XInputGetStateExFn = unsafe extern "system" fn(u32, *mut XINPUT_STATE) -> u32;
#[cfg(windows)]
static XINPUT_GET_STATE_EX: OnceLock<Option<XInputGetStateExFn>> = OnceLock::new();

#[cfg(windows)]
fn init_xinput_get_state_ex() -> Option<XInputGetStateExFn> {
    *XINPUT_GET_STATE_EX.get_or_init(|| unsafe {
        let lib = LoadLibraryW(windows::core::w!("xinput1_4.dll")).ok()?;
        // ordinal 100 = XInputGetStateEx
        let proc = GetProcAddress(lib, windows::core::PCSTR(100usize as *const u8))?;
        let func: XInputGetStateExFn = std::mem::transmute(proc);
        println!("[XInput] XInputGetStateEx loaded (Guide button support enabled)");
        Some(func)
    })
}

/// XInput 鐘舵€佽鍙栧寘瑁咃細浼樺厛鐢?XInputGetStateEx锛堝惈 Guide锛夛紝鍥為€€鍒?XInputGetState
#[cfg(windows)]
unsafe fn xinput_get_state(index: u32, state: &mut XINPUT_STATE) -> u32 {
    if let Some(get_state_ex) = init_xinput_get_state_ex() {
        get_state_ex(index, state)
    } else {
        XInputGetState(index, state)
    }
}

// Guide 鎸夐挳 bitmask锛圶InputGetStateEx 杩斿洖鐨?wButtons 涓級
#[cfg(windows)]
const XINPUT_GAMEPAD_GUIDE: u16 = 0x0400;

// 鍏ㄥ眬鐘舵€?
pub static IS_GAMEPAD_TEST_MODE: AtomicBool = AtomicBool::new(false);
pub static IS_BINDING_LISTENING_MODE: AtomicBool = AtomicBool::new(false);

// 鍏ㄥ眬鎵虫満姝诲尯锛堢敤浜庡皢鎵虫満浣滀负鈥滄寜閿€濆垽鏂椂鐨勯槇鍊硷級
// - 0..255
// - 榛樿 30 涓庡巻鍙茶涓轰竴鑷?
static TRIGGER_DEADZONE_LT: AtomicU8 = AtomicU8::new(30);
static TRIGGER_DEADZONE_RT: AtomicU8 = AtomicU8::new(30);

fn clamp_deadzone_u8(v: i32) -> u8 {
    v.clamp(0, 255) as u8
}

pub fn get_trigger_deadzone_lt() -> u8 {
    TRIGGER_DEADZONE_LT.load(Ordering::Relaxed)
}

pub fn get_trigger_deadzone_rt() -> u8 {
    TRIGGER_DEADZONE_RT.load(Ordering::Relaxed)
}

pub fn apply_trigger_deadzone_settings(settings: &Settings) {
    // 瑙勫垯锛?    // - 璁剧疆鍊奸粯璁?0锛圲I 鏄剧ず涓哄叧闂級
    // - 浣嗏€滄壋鏈哄綋鎸夐敭鎸変笅鈥濈殑鍘嗗彶闃堝€兼槸 30
    // - 鍥犳锛氬綋鍏ㄥ眬鍊?< 30 鏃朵粛浣跨敤 30锛涘綋鍏ㄥ眬鍊?>= 30 鏃惰窡闅忓叏灞€鍊?
let lt_setting = settings.trigger_deadzone_lt.unwrap_or(0);
    let rt_setting = settings.trigger_deadzone_rt.unwrap_or(0);

    let lt = clamp_deadzone_u8(lt_setting).max(30u8);
    let rt = clamp_deadzone_u8(rt_setting).max(30u8);

    TRIGGER_DEADZONE_LT.store(lt, Ordering::Relaxed);
    TRIGGER_DEADZONE_RT.store(rt, Ordering::Relaxed);
}

// 璁板綍褰撳墠娲昏穬鐨勬墜鏌勬Ы浣嶏紙0-3锛屽垵濮?1琛ㄧず鏈娴嬶級
static ACTIVE_GAMEPAD_INDEX: std::sync::atomic::AtomicI32 = std::sync::atomic::AtomicI32::new(-1);

// 璁板綍涓婁竴娆℃瘡涓墜鏌勭殑 packet number锛岀敤浜庢娴嬭緭鍏ュ彉鍖?
static LAST_PACKET_NUMBERS: Mutex<[u32; 4]> = Mutex::new([0; 4]);

#[cfg(windows)]
pub fn get_active_or_first_connected_xinput_index() -> Option<u32> {
    let current_index = ACTIVE_GAMEPAD_INDEX.load(Ordering::Relaxed);

    if current_index >= 0 {
        let mut state = XINPUT_STATE::default();
        let result = unsafe { xinput_get_state(current_index as u32, &mut state) };
        if result == 0 {
            return Some(current_index as u32);
        }
        ACTIVE_GAMEPAD_INDEX.store(-1, Ordering::Relaxed);
    }

    for i in 0..4u32 {
        let mut state = XINPUT_STATE::default();
        let result = unsafe { xinput_get_state(i, &mut state) };
        if result == 0 {
            ACTIVE_GAMEPAD_INDEX.store(i as i32, Ordering::Relaxed);
            return Some(i);
        }
    }

    None
}

// ==================== XInput 鎸夐敭缁戝畾鐩戝惉 ====================
use crate::keyboard_hook::HotkeyAction;
use crate::panel;
use lazy_static::lazy_static;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

// 缁熶竴鎵嬫焺杈撳叆绾跨▼锛堝悎骞朵簡 XInput Binding 鍜?UI Input Router锛?
static GAMEPAD_INPUT_THREAD_ACTIVE: AtomicBool = AtomicBool::new(false);

// 榧犳爣妯℃嫙鐘舵€?
static MOUSE_SIM_ACTIVE: AtomicBool = AtomicBool::new(false);
static MOUSE_SIM_LB_WAS_PRESSED: AtomicBool = AtomicBool::new(false);
static MOUSE_SIM_RB_WAS_PRESSED: AtomicBool = AtomicBool::new(false);

/// XInput 鎸夐挳缁戝畾
#[derive(Clone)]
struct XInputBinding {
    action: HotkeyAction,
    buttons: Vec<i32>,
    custom_keys: Option<u64>, // 鎺屾満鐗规畩鎸夐敭锛堜綅鎺╃爜锛寀64 鏀寔鏇村璁惧锛?
    was_pressed: bool,
    last_trigger_time: u128,
    is_hold_active: bool, // 鐢ㄤ簬鎸変綇绫诲瀷鐑敭
    long_press: bool,           // 闀挎寜妯″紡锛氶渶瑕佹寔缁寜浣?1 绉掓墠瑙﹀彂
    long_press_start: u128,     // 闀挎寜妯″紡锛氭寜涓嬪紑濮嬫椂闂达紙ms锛?
    long_press_triggered: bool, // 闀挎寜妯″紡锛氭湰娆℃寜浣忔槸鍚﹀凡瑙﹀彂
}

/// 鍒ゆ柇鏄惁涓烘寜浣忕被鍨嬬殑鐑敭
fn is_hold_type_action(action: &HotkeyAction) -> bool {
    crate::keyboard_hook::is_hold_type_action(action)
}

lazy_static! {
    /// 鎵€鏈?XInput 鎸夐挳缁戝畾
    static ref XINPUT_BINDINGS: Arc<Mutex<Vec<XInputBinding>>> = Arc::new(Mutex::new(Vec::new()));
    /// 缁熶竴鎵嬫焺杈撳叆绾跨▼鐢ㄧ殑 AppHandle
    static ref GAMEPAD_INPUT_APP_HANDLE: Mutex<Option<AppHandle>> = Mutex::new(None);
}

/// 璁剧疆 XInput 鎸夐挳缁戝畾
pub fn set_xinput_binding(action: HotkeyAction, buttons: Option<Vec<i32>>) {
    set_xinput_binding_full(action, buttons, None);
}

/// 璁剧疆 XInput 鎸夐挳缁戝畾锛堝畬鏁寸増锛屾敮鎸佹帉鏈虹壒娈婃寜閿拰闀挎寜妯″紡锛?
pub fn set_xinput_binding_full(
    action: HotkeyAction,
    buttons: Option<Vec<i32>>,
    custom_keys: Option<u64>,
) {
    set_xinput_binding_full_ex(action, buttons, custom_keys, false);
}

/// 璁剧疆 XInput 鎸夐挳缁戝畾锛堟墿灞曠増锛屾敮鎸侀暱鎸夋ā寮忥級
pub fn set_xinput_binding_full_ex(
    action: HotkeyAction,
    buttons: Option<Vec<i32>>,
    custom_keys: Option<u64>,
    long_press: bool,
) {
    if let Ok(mut bindings) = XINPUT_BINDINGS.lock() {
        // 绉婚櫎璇ュ姩浣滅殑鏃х粦瀹?        bindings.retain(|b| b.action != action);

        // 濡傛灉鏈夋柊鎸夐挳鎴栫壒娈婃寜閿紝娣诲姞缁戝畾
        let has_buttons = buttons.as_ref().is_some_and(|b| !b.is_empty());
        let has_custom_keys = custom_keys.is_some_and(|k| k != 0);

        if has_buttons || has_custom_keys {
            // 鍚?HID 鎸夐敭鐨勭粦瀹氶渶瑕?HID 鐩戞帶绾跨▼杩愯
            if has_custom_keys {
                crate::custom_keys::ensure_monitor_running();
            }
            let is_custom_key_only = !has_buttons && has_custom_keys;
            bindings.push(XInputBinding {
                action,
                buttons: buttons.clone().unwrap_or_default(),
                custom_keys,
                was_pressed: false,
                last_trigger_time: 0,
                is_hold_active: false,
                long_press,
                long_press_start: 0,
                long_press_triggered: false,
            });
            println!(
                "[XInput Binding] {:?} 缁戝畾宸茶缃? buttons={:?}, custom_keys={:?}{}",
                action,
                buttons,
                custom_keys,
                if is_custom_key_only {
                    " (浠?HID 鐗规畩鎸夐敭锛岄渶 HID 鐩戞帶姝ｅ父杩愯)"
                } else {
                    ""
                }
            );
        }
    }
}

/// 瑙﹀彂蹇嵎閿姩浣滐紙XInput 缁戝畾鐢級
fn trigger_xinput_action(action: HotkeyAction) {
    println!("[XInput Binding] >>> TRIGGER {:?} <<<", action);
    if let Ok(app_guard) = GAMEPAD_INPUT_APP_HANDLE.lock() {
        if let Some(ref app) = *app_guard {
            crate::keyboard_hook::execute_hotkey_action(app, action);
        }
    }
}

/// 妫€鏌ョ儹閿粦瀹氾紙浣跨敤宸茶В鏋愮殑鎵嬫焺鐘舵€侊紝閬垮厤閲嶅璇诲彇 XInput锛?#[cfg(windows)]
fn check_hotkey_bindings(state: &MergedGamepadState, current_custom_keys: u64) {
    // 缁戝畾/娴嬭瘯妯″紡涓嬩笉瑙﹀彂鐑敭
    if IS_BINDING_LISTENING_MODE.load(Ordering::SeqCst)
        || IS_GAMEPAD_TEST_MODE.load(Ordering::SeqCst)
    {
        return;
    }

    // 鑾峰彇鎵€鏈夌粦瀹?    let mut bindings = match XINPUT_BINDINGS.lock() {
        Ok(guard) => guard.clone(),
        Err(_) => return,
    };

    if bindings.is_empty() {
        return;
    }

    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();

    let mut updated = false;

    for binding in &mut bindings {
        // 妫€鏌?XInput 鎸夐挳锛堜娇鐢ㄥ凡瑙ｆ瀽鐨?MergedGamepadState锛?
let xinput_pressed = if binding.buttons.is_empty() {
            true // 娌℃湁 XInput 鎸夐挳瑕佹眰
        } else {
            binding.buttons.iter().all(|&btn_idx| {
                if btn_idx >= 0 && (btn_idx as usize) < state.buttons.len() {
                    state.buttons[btn_idx as usize]
                } else {
                    false
                }
            })
        };

        // 妫€鏌ユ帉鏈虹壒娈婃寜閿?
let custom_keys_pressed = if let Some(required_keys) = binding.custom_keys {
            if required_keys == 0 {
                true
            } else {
                (current_custom_keys & required_keys) == required_keys
            }
        } else {
            true
        };

        // 闇€瑕佸悓鏃舵弧瓒虫墍鏈夋潯浠?
let all_pressed = xinput_pressed && custom_keys_pressed;

        // 鐗规畩鎯呭喌锛氫粎鏈?custom_keys 娌℃湁 XInput 鎸夐挳鏃讹紝涓嶉渶瑕佹墜鏌勮繛鎺?
let needs_gamepad = !binding.buttons.is_empty();
        if needs_gamepad && !state.connected {
            continue;
        }

        if all_pressed && !binding.was_pressed {
            // 闃叉姈锛?00ms 鍐呬笉閲嶅瑙﹀彂锛堟寜浣忕被鍨嬬儹閿拰闀挎寜妯″紡涓嶉渶瑕侀槻鎶栵級
            if !is_hold_type_action(&binding.action) && !binding.long_press && now - binding.last_trigger_time < 500 {
                binding.was_pressed = true;
                updated = true;
                continue;
            }

            binding.was_pressed = true;
            binding.last_trigger_time = now;
            updated = true;

            // 鎸変綇绫诲瀷鐑敭锛氭爣璁颁负婵€娲荤姸鎬侊紙闀挎寜妯″紡涓嬪欢杩熷埌瑙﹀彂鏃讹級
            if is_hold_type_action(&binding.action) && !binding.long_press {
                binding.is_hold_active = true;
            }

            // 闀挎寜妯″紡锛氳褰曟寜涓嬫椂闂达紝涓嶇珛鍗宠Е鍙?            if binding.long_press {
                binding.long_press_start = now;
                binding.long_press_triggered = false;
            } else {
                trigger_xinput_action(binding.action);
            }
        } else if all_pressed && binding.was_pressed && binding.long_press {
            // 闀挎寜妯″紡锛氭鏌ユ槸鍚﹀凡鎸佺画鎸変綇 1 绉?            if !binding.long_press_triggered && binding.long_press_start > 0
                && now - binding.long_press_start >= 1000
            {
                binding.long_press_triggered = true;
                updated = true;
                // 鎸変綇绫诲瀷鐑敭锛氶暱鎸夎Е鍙戞椂鎵嶆縺娲?                if is_hold_type_action(&binding.action) {
                    binding.is_hold_active = true;
                }
                trigger_xinput_action(binding.action);
            }
        } else if !all_pressed && binding.was_pressed {
            binding.was_pressed = false;
            updated = true;

            // 闀挎寜妯″紡锛氶噴鏀炬椂閲嶇疆
            if binding.long_press {
                binding.long_press_start = 0;
                binding.long_press_triggered = false;
            }

            // 鎸変綇绫诲瀷鐑敭锛氳Е鍙戞澗寮€浜嬩欢
            if is_hold_type_action(&binding.action) && binding.is_hold_active {
                binding.is_hold_active = false;
                trigger_xinput_release(binding.action);
            }
        }
    }

    // 鏇存柊鐘舵€?    if updated {
        if let Ok(mut guard) = XINPUT_BINDINGS.lock() {
            *guard = bindings;
        }
    }
}

/// 瑙﹀彂鎸変綇绫诲瀷鐑敭鐨勬澗寮€浜嬩欢
fn trigger_xinput_release(action: HotkeyAction) {
    if let Ok(app_guard) = GAMEPAD_INPUT_APP_HANDLE.lock() {
        if let Some(ref app) = *app_guard {
            crate::keyboard_hook::execute_hotkey_release(app, action);
        }
    }
}

/// 鍚姩 XInput 缁戝畾杞锛堝悜鍚庡吋瀹癸紝鐜板湪涓嶅啀鍗曠嫭鍚姩绾跨▼锛?#[cfg(windows)]
pub fn start_xinput_binding_polling(_app: AppHandle) {
    // 宸插悎骞跺埌 start_ui_input_router锛屾鍑芥暟淇濈暀浠ュ吋瀹规棫璋冪敤
}

/// 鍋滄 XInput 缁戝畾杞
#[cfg(windows)]
pub fn stop_xinput_binding_polling() {
    // 娓呯┖鎵€鏈夌粦瀹?    if let Ok(mut bindings) = XINPUT_BINDINGS.lock() {
        bindings.clear();
    }
}

#[cfg(not(windows))]
pub fn start_xinput_binding_polling(_app: AppHandle) {}

#[cfg(not(windows))]
pub fn stop_xinput_binding_polling() {}

/// 妫€鏌ユ墜鏌勬槸鍚︽湁鏈夋晥杈撳叆锛堟寜閽垨鎵虫満锛屾帓闄ゆ憞鏉嗘紓绉诲櫔闊筹級
fn has_meaningful_input(state: &XINPUT_STATE) -> bool {
    let gp = &state.Gamepad;
    // 妫€鏌ユ寜閽垨鎵虫満锛堥槇鍊煎彲閰嶇疆锛?    gp.wButtons.0 != 0
        || gp.bLeftTrigger > get_trigger_deadzone_lt()
        || gp.bRightTrigger > get_trigger_deadzone_rt()
}

// ==================== UI 杈撳叆璺敱锛堝悗绔帹閫佷簨浠讹級 ====================

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum UiTarget {
    Main,
    Bigscreen,
}

#[derive(Serialize, Clone, Copy)]
#[serde(rename_all = "lowercase")]
enum UiKeyKind {
    Down,
    Up,
}

#[derive(Serialize, Clone, Copy)]
struct UiKeyEventPayload {
    kind: UiKeyKind,
    key: &'static str,
}

#[derive(Serialize, Clone, Copy)]
struct UiTabEventPayload {
    delta: i32,
}

#[derive(Serialize, Clone, Copy)]
struct UiGamepadBindingPayload {
    button: u8,
}

#[derive(Serialize, Clone, Copy)]
struct UiGamepadStatePayload {
    connected: bool,
    buttons: [bool; 17],
    axes: [f32; 4],
    left_trigger: f32,
    right_trigger: f32,
}

#[derive(Clone, Copy, Default)]
struct MergedGamepadState {
    connected: bool,
    buttons: [bool; 17],
    axes: [f32; 4],
    left_trigger: f32,
    right_trigger: f32,
}

#[cfg(windows)]
fn is_window_foreground(window: &tauri::WebviewWindow) -> bool {
    if let Ok(hwnd) = window.hwnd() {
        let hwnd = HWND(hwnd.0);
        unsafe {
            // 娉ㄦ剰锛歵auri 鐨?hwnd() 鍦ㄤ笉鍚屽钩鍙?鐗堟湰涓嬪彲鑳借繑鍥炲瓙绐楀彛鍙ユ焺銆?            // 鐢?GA_ROOT 褰掍竴鍖栧悗鍐嶄笌鍓嶅彴绐楀彛姣旇緝锛岄伩鍏嶁€滄槑鏄庡湪鍓嶅彴浣嗗垽瀹氬け璐モ€濄€?            let fg = GetForegroundWindow();
            if fg.0.is_null() {
                return false;
            }
            let fg_root = GetAncestor(fg, GA_ROOT);
            let hwnd_root = GetAncestor(hwnd, GA_ROOT);
            fg_root == hwnd_root
        }
    } else {
        false
    }
}

#[cfg(windows)]
fn is_window_on_top(window: &tauri::WebviewWindow) -> bool {
    let Ok(hwnd) = window.hwnd() else {
        // 鏃犳硶鑾峰彇 hwnd 鏃讹紝涓嶉樆濉炶緭鍏ヨ矾鐢憋紙淇濇寔鍘嗗彶瀹芥澗绛栫暐锛?        return true;
    };
    let hwnd = HWND(hwnd.0);

    // 濡傛灉鎷夸笉鍒颁綅缃?澶у皬锛屼篃涓嶉樆濉?    let Ok(pos) = window.outer_position() else {
        return true;
    };
    let Ok(size) = window.outer_size() else {
        return true;
    };

    // 鍙栫獥鍙ｅ唴閮ㄤ竴鐐逛綔涓哄懡涓祴璇曠偣
    let x = pos.x + (size.width as i32 / 2).max(1);
    let y = pos.y + (size.height as i32 / 4).max(1);

    unsafe {
        let hit = WindowFromPoint(POINT { x, y });
        if hit.0.is_null() {
            return false;
        }
        let hit_root = GetAncestor(hit, GA_ROOT);
        let hwnd_root = GetAncestor(hwnd, GA_ROOT);
        hit_root == hwnd_root
    }
}

#[cfg(windows)]
fn compute_ui_target(app: &AppHandle) -> Option<UiTarget> {
    // 涓婚潰鏉匡紙KO 渚ц竟鏍忥級锛氭墦寮€鏃跺搷搴旓紙鍗充娇绐楀彛娌℃湁鐒︾偣锛夈€?    // 浣嗗湪澶у睆骞曟ā寮忎笅锛屽鏋滀富闈㈡澘瀹為檯琚洊浣忥紙涓嶅湪鏈€涓婂眰锛夛紝灏变笉瑕佹姠璧拌緭鍏ワ紝
    // 杩欐牱鐢ㄦ埛鐐瑰嚮/鑱氱劍澶у睆骞曞悗锛屽ぇ灞忓箷涔熻兘姝ｅ父鐢ㄦ墜鏌勬搷浣溿€?
if panel::PANEL_IS_OPEN.load(Ordering::SeqCst) {
        if let Some(w) = app.get_webview_window("main") {
            if !panel::is_bigscreen_mode() || is_window_on_top(&w) {
                return Some(UiTarget::Main);
            }
        }
    }

    // 澶у睆骞曠獥鍙ｏ細閬垮厤鍚庡彴璇Е鍙戙€?    // 娉ㄦ剰锛氭煇浜涙儏鍐典笅锛圖PI/鍏ㄥ睆/瀹夸富绐楀彛鍙ユ焺宸紓绛夛級浠呯敤 foreground 鍒ゅ畾浼氬嚭鐜板亣闃存€э紝
    // 瀵艰嚧澶у睆鐣岄潰鈥滄槑鏄庡湪鏈€鍓嶄絾娌℃湁鎵嬫焺杈撳叆鈥濄€傚洜姝よ繖閲屽鍔?on_top 鍛戒腑娴嬭瘯浣滀负鍏滃簳銆?
if panel::is_bigscreen_mode() {
        if let Some(w) = app.get_webview_window("bigscreen") {
            if is_window_foreground(&w) || is_window_on_top(&w) {
                return Some(UiTarget::Bigscreen);
            }
        }
        return None;
    }

    None
}

#[cfg(windows)]
fn target_label(target: UiTarget) -> &'static str {
    match target {
        UiTarget::Main => "main",
        UiTarget::Bigscreen => "bigscreen",
    }
}

#[cfg(windows)]
fn emit_ui_key(app: &AppHandle, target: UiTarget, kind: UiKeyKind, key: &'static str) {
    use tauri::Emitter;
    if let Some(w) = app.get_webview_window(target_label(target)) {
        let _ = w.emit("ui-key", UiKeyEventPayload { kind, key });
    }
}

#[cfg(windows)]
fn emit_ui_tab(app: &AppHandle, delta: i32) {
    use tauri::Emitter;
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.emit("ui-tab", UiTabEventPayload { delta });
    }
}

#[cfg(windows)]
fn emit_ui_gamepad_binding(app: &AppHandle, button: u8) {
    use tauri::Emitter;
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.emit("ui-gamepad-binding", UiGamepadBindingPayload { button });
    }
}

#[cfg(windows)]
fn emit_ui_gamepad_state(app: &AppHandle, state: MergedGamepadState) {
    use tauri::Emitter;
    if let Some(w) = app.get_webview_window("main") {
        let _ = w.emit(
            "ui-gamepad-state",
            UiGamepadStatePayload {
                connected: state.connected,
                buttons: state.buttons,
                axes: state.axes,
                left_trigger: state.left_trigger,
                right_trigger: state.right_trigger,
            },
        );
    }
}

#[cfg(windows)]
fn parse_xinput_state_fast(state: &XINPUT_STATE) -> ([bool; 17], [f32; 4], f32, f32) {
    let gp = state.Gamepad;
    let buttons_raw = gp.wButtons;

    let buttons = [
        (buttons_raw.0 & XINPUT_GAMEPAD_A.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_B.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_X.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_Y.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_LEFT_SHOULDER.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_RIGHT_SHOULDER.0) != 0,
        gp.bLeftTrigger > get_trigger_deadzone_lt(),
        gp.bRightTrigger > get_trigger_deadzone_rt(),
        (buttons_raw.0 & XINPUT_GAMEPAD_BACK.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_START.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_LEFT_THUMB.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_RIGHT_THUMB.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_DPAD_UP.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_DPAD_DOWN.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_DPAD_LEFT.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_DPAD_RIGHT.0) != 0,
        (buttons_raw.0 & XINPUT_GAMEPAD_GUIDE) != 0,
    ];

    let normalize_axis = |val: i16| -> f32 {
        let v = val as f32;
        if v > 0.0 {
            v / 32767.0
        } else {
            v / 32768.0
        }
    };

    let axes = [
        normalize_axis(gp.sThumbLX),
        -normalize_axis(gp.sThumbLY),
        normalize_axis(gp.sThumbRX),
        -normalize_axis(gp.sThumbRY),
    ];

    let lt = gp.bLeftTrigger as f32 / 255.0;
    let rt = gp.bRightTrigger as f32 / 255.0;

    (buttons, axes, lt, rt)
}

#[cfg(windows)]
fn get_xinput_state_raw(full_scan: bool) -> Option<XINPUT_STATE> {
    let current_index = ACTIVE_GAMEPAD_INDEX.load(Ordering::Relaxed);

    if full_scan {
        let mut last_packets = match LAST_PACKET_NUMBERS.lock() {
            Ok(g) => g,
            Err(e) => e.into_inner(),
        };

        // 鍏堝揩閫熸壂鎻忔墍鏈夋Ы浣嶏紝鎵惧埌鏈夋寜閽緭鍏ョ殑鎵嬫焺锛堢敤浜庡垏鎹級
        for i in 0..4u32 {
            let mut state = XINPUT_STATE::default();
            let result = unsafe { xinput_get_state(i, &mut state) };
            if result == 0 {
                let old_packet = last_packets[i as usize];
                if state.dwPacketNumber != old_packet && has_meaningful_input(&state) {
                    last_packets[i as usize] = state.dwPacketNumber;
                    if current_index != i as i32 {
                        ACTIVE_GAMEPAD_INDEX.store(i as i32, Ordering::Relaxed);
                    }
                    return Some(state);
                }
            }
        }
    }

    // 娌℃湁鎸夐挳杈撳叆锛岃繑鍥炲綋鍓嶆椿璺冩Ы浣嶇殑鐘舵€侊紙鍖呭惈鎽囨潌锛?    if current_index >= 0 {
        let mut state = XINPUT_STATE::default();
        let result = unsafe { xinput_get_state(current_index as u32, &mut state) };
        if result == 0 {
            if let Ok(mut last_packets) = LAST_PACKET_NUMBERS.lock() {
                last_packets[current_index as usize] = state.dwPacketNumber;
            }
            return Some(state);
        } else {
            ACTIVE_GAMEPAD_INDEX.store(-1, Ordering::Relaxed);
        }
    }

    // 娌℃湁娲昏穬妲戒綅锛岃繑鍥炵涓€涓繛鎺ョ殑鎵嬫焺
    for i in 0..4u32 {
        let mut state = XINPUT_STATE::default();
        let result = unsafe { xinput_get_state(i, &mut state) };
        if result == 0 {
            ACTIVE_GAMEPAD_INDEX.store(i as i32, Ordering::Relaxed);
            if let Ok(mut last_packets) = LAST_PACKET_NUMBERS.lock() {
                last_packets[i as usize] = state.dwPacketNumber;
            }
            return Some(state);
        }
    }

    None
}

#[cfg(windows)]
fn get_merged_gamepad_state(full_scan: bool) -> MergedGamepadState {
    // XInput
    let xinput_raw = get_xinput_state_raw(full_scan);
    let (x_buttons, x_axes, x_lt, x_rt) = if let Some(ref s) = xinput_raw {
        parse_xinput_state_fast(s)
    } else {
        ([false; 17], [0.0; 4], 0.0, 0.0)
    };

    // gilrs锛堜粎鍦ㄥ彲鐢ㄦ椂璇诲彇蹇収锛岄伩鍏嶆棤鎰忎箟閿侊級
    let gilrs = if crate::gilrs_gamepad::is_gilrs_gamepad_available() {
        crate::gilrs_gamepad::get_gilrs_gamepad_snapshot()
    } else {
        crate::gilrs_gamepad::GilrsGamepadSnapshot::default()
    };

    // 鍚堝苟
    if xinput_raw.is_none() && !gilrs.connected {
        return MergedGamepadState::default();
    }

    let mut merged_buttons = [false; 17];
    for i in 0..17 {
        merged_buttons[i] = x_buttons[i] || gilrs.buttons[i];
    }

    let mut merged_axes = [0.0f32; 4];
    for i in 0..4 {
        let xv = x_axes[i];
        let gv = gilrs.axes[i];
        merged_axes[i] = if xv.abs() > gv.abs() { xv } else { gv };
    }

    let merged_lt = x_lt.max(gilrs.left_trigger);
    let merged_rt = x_rt.max(gilrs.right_trigger);

    MergedGamepadState {
        connected: true,
        buttons: merged_buttons,
        axes: merged_axes,
        left_trigger: merged_lt,
        right_trigger: merged_rt,
    }
}

#[cfg(windows)]
fn emit_all_keyups(
    app: &AppHandle,
    target: UiTarget,
    prev_buttons: &[bool; 17],
    prev_dirs: (bool, bool, bool, bool),
) {
    let (up, down, left, right) = prev_dirs;
    if up {
        emit_ui_key(app, target, UiKeyKind::Up, "ArrowUp");
    }
    if down {
        emit_ui_key(app, target, UiKeyKind::Up, "ArrowDown");
    }
    if left {
        emit_ui_key(app, target, UiKeyKind::Up, "ArrowLeft");
    }
    if right {
        emit_ui_key(app, target, UiKeyKind::Up, "ArrowRight");
    }

    if prev_buttons[0] {
        emit_ui_key(app, target, UiKeyKind::Up, "a");
    }
    if prev_buttons[1] {
        emit_ui_key(app, target, UiKeyKind::Up, "b");
    }
    if prev_buttons[2] {
        emit_ui_key(app, target, UiKeyKind::Up, "x");
    }
    if prev_buttons[3] {
        emit_ui_key(app, target, UiKeyKind::Up, "y");
    }
}

/// 缁熶竴鎵嬫焺杈撳叆绾跨▼锛堝悎骞朵簡 XInput Binding 鍜?UI Input Router锛?#[cfg(windows)]
fn gamepad_input_thread(app: AppHandle) {
    println!("[Gamepad Input] Unified thread started (hotkeys + UI routing)");

    let mut prev_target: Option<UiTarget> = None;
    let mut prev_buttons: [bool; 17] = [false; 17];
    let mut prev_dir_up = false;
    let mut prev_dir_down = false;
    let mut prev_dir_left = false;
    let mut prev_dir_right = false;

    let mut last_full_scan = Instant::now();
    let full_scan_interval = Duration::from_millis(50);
    let dir_threshold: f32 = 0.5;

    // 璇婃柇鏃ュ織锛氳褰曚笂娆?custom key 鐘舵€侊紝閬垮厤鍒峰睆
    let mut prev_custom_keys: u64 = 0;

    while GAMEPAD_INPUT_THREAD_ACTIVE.load(Ordering::SeqCst) {
        let is_binding = IS_BINDING_LISTENING_MODE.load(Ordering::SeqCst);
        let is_test = IS_GAMEPAD_TEST_MODE.load(Ordering::SeqCst);

        // 鏍规嵁绐楀彛鐘舵€侀€夋嫨鐩爣
        let new_target = compute_ui_target(&app);
        if prev_target != new_target {
            // 鐩爣鍒囨崲鏃讹紝缁欐棫鐩爣鍙戦€佷竴娆?keyup锛岄伩鍏嶅墠绔€滄寜閿崱浣?杩炲彂鈥?            if let Some(old) = prev_target {
                emit_all_keyups(
                    &app,
                    old,
                    &prev_buttons,
                    (prev_dir_up, prev_dir_down, prev_dir_left, prev_dir_right),
                );
            }
            prev_target = new_target;
        }

        // 鎺у埗鍏ㄦЫ浣嶆壂鎻忛鐜囷紙鐢ㄤ簬蹇€熷垏鎹㈡椿璺冩墜鏌勶級
        let now = Instant::now();
        let do_full_scan = now.duration_since(last_full_scan) >= full_scan_interval
            || ACTIVE_GAMEPAD_INDEX.load(Ordering::Relaxed) < 0;
        if do_full_scan {
            last_full_scan = now;
        }

        let state = get_merged_gamepad_state(do_full_scan);

        // === 鐑敭妫€鏌ワ紙姣忔寰幆閮芥鏌ワ紝涓嶅彈 UI 鐘舵€佸奖鍝嶏級 ===
        let current_custom_keys = crate::custom_keys::get_custom_key_state();

        // 璇婃柇鏃ュ織锛歝ustom key 鐘舵€佸彉鍖栨椂鎵撳嵃锛堜粎鍙樺寲鏃舵墦鍗帮紝涓嶅埛灞忥級
        if current_custom_keys != prev_custom_keys {
            if current_custom_keys != 0 {
                println!(
                    "[Gamepad Hotkey] HID custom key 妫€娴嬪埌: 0x{:X}, 缁戝畾鏁? {}",
                    current_custom_keys,
                    XINPUT_BINDINGS.lock().map(|b| b.len()).unwrap_or(0)
                );
            }
            prev_custom_keys = current_custom_keys;
        }

        check_hotkey_bindings(&state, current_custom_keys);

        // === UI 璺敱澶勭悊 ===
        if is_test {
            // 娴嬭瘯妯″紡锛氬彧鎺ㄩ€佸師濮嬬姸鎬侊紙鐢卞墠绔礋璐ｅ彲瑙嗗寲/闀挎寜閫€鍑猴級
            if matches!(prev_target, Some(UiTarget::Main)) {
                emit_ui_gamepad_state(&app, state);
            }
            // 鏇存柊鏈湴鐘舵€侊紝閬垮厤閫€鍑烘祴璇曟ā寮忓悗鈥滆瑙﹀彂鈥?            prev_buttons = state.buttons;
            prev_dir_up = state.buttons[12] || state.axes[1] < -dir_threshold;
            prev_dir_down = state.buttons[13] || state.axes[1] > dir_threshold;
            prev_dir_left = state.buttons[14] || state.axes[0] < -dir_threshold;
            prev_dir_right = state.buttons[15] || state.axes[0] > dir_threshold;

            thread::sleep(Duration::from_millis(16));
            continue;
        }

        if is_binding {
            // 缁戝畾褰曞埗妯″紡锛氬彧鎺ㄩ€佲€滄寜涓嬧€濈殑鎸夐挳杈圭紭
            if matches!(prev_target, Some(UiTarget::Main)) {
                #[allow(clippy::needless_range_loop)]
                for i in 0..17 {
                    if state.buttons[i] && !prev_buttons[i] {
                        emit_ui_gamepad_binding(&app, i as u8);
                    }
                }
            }

            prev_buttons = state.buttons;
            prev_dir_up = state.buttons[12] || state.axes[1] < -dir_threshold;
            prev_dir_down = state.buttons[13] || state.axes[1] > dir_threshold;
            prev_dir_left = state.buttons[14] || state.axes[0] < -dir_threshold;
            prev_dir_right = state.buttons[15] || state.axes[0] > dir_threshold;

            thread::sleep(Duration::from_millis(16));
            continue;
        }

        // 姝ｅ父妯″紡锛氭柟鍚戦敭锛圖-Pad + 宸︽憞鏉嗭級 + ABXY -> ui-key
        let dir_up = state.buttons[12] || state.axes[1] < -dir_threshold;
        let dir_down = state.buttons[13] || state.axes[1] > dir_threshold;
        let dir_left = state.buttons[14] || state.axes[0] < -dir_threshold;
        let dir_right = state.buttons[15] || state.axes[0] > dir_threshold;

        if let Some(target) = prev_target {
            // 鏂瑰悜閿?            if dir_up != prev_dir_up {
                emit_ui_key(
                    &app,
                    target,
                    if dir_up {
                        UiKeyKind::Down
                    } else {
                        UiKeyKind::Up
                    },
                    "ArrowUp",
                );
            }
            if dir_down != prev_dir_down {
                emit_ui_key(
                    &app,
                    target,
                    if dir_down {
                        UiKeyKind::Down
                    } else {
                        UiKeyKind::Up
                    },
                    "ArrowDown",
                );
            }
            if dir_left != prev_dir_left {
                emit_ui_key(
                    &app,
                    target,
                    if dir_left {
                        UiKeyKind::Down
                    } else {
                        UiKeyKind::Up
                    },
                    "ArrowLeft",
                );
            }
            if dir_right != prev_dir_right {
                emit_ui_key(
                    &app,
                    target,
                    if dir_right {
                        UiKeyKind::Down
                    } else {
                        UiKeyKind::Up
                    },
                    "ArrowRight",
                );
            }

            // ABXY
            let a = state.buttons[0];
            let b = state.buttons[1];
            let x = state.buttons[2];
            let y = state.buttons[3];

            if a != prev_buttons[0] {
                emit_ui_key(
                    &app,
                    target,
                    if a { UiKeyKind::Down } else { UiKeyKind::Up },
                    "a",
                );
            }
            if b != prev_buttons[1] {
                emit_ui_key(
                    &app,
                    target,
                    if b { UiKeyKind::Down } else { UiKeyKind::Up },
                    "b",
                );
            }
            if x != prev_buttons[2] {
                emit_ui_key(
                    &app,
                    target,
                    if x { UiKeyKind::Down } else { UiKeyKind::Up },
                    "x",
                );
            }
            if y != prev_buttons[3] {
                emit_ui_key(
                    &app,
                    target,
                    if y { UiKeyKind::Down } else { UiKeyKind::Up },
                    "y",
                );
            }

            // LB/RB锛氫粎涓婚潰鏉垮垏鎹?Tab
            if matches!(target, UiTarget::Main) {
                let lb = state.buttons[4];
                let rb = state.buttons[5];
                if lb && !prev_buttons[4] {
                    emit_ui_tab(&app, -1);
                }
                if rb && !prev_buttons[5] {
                    emit_ui_tab(&app, 1);
                }
            }
        }

        // 鏇存柊缂撳瓨
        prev_buttons = state.buttons;
        prev_dir_up = dir_up;
        prev_dir_down = dir_down;
        prev_dir_left = dir_left;
        prev_dir_right = dir_right;

        // 鍔ㄦ€佽皟鏁磋疆璇㈤鐜囷細
        // - UI 娲昏穬鏃?16ms锛垀60Hz锛岀‘淇?UI 鍝嶅簲娴佺晠锛?        // - 绌洪棽鏃?32ms锛垀30Hz锛屼粛鑳藉強鏃舵娴嬬儹閿級
        let sleep_ms = if prev_target.is_some() { 16 } else { 32 };
        thread::sleep(Duration::from_millis(sleep_ms));
    }

    println!("[Gamepad Input] Unified thread stopped");
}

/// 鍚姩缁熶竴鎵嬫焺杈撳叆绾跨▼锛堝寘鍚儹閿娴嬪拰 UI 璺敱锛?#[cfg(windows)]
pub fn start_ui_input_router(app: AppHandle) {
    // 淇濆瓨 AppHandle
    if let Ok(mut handle) = GAMEPAD_INPUT_APP_HANDLE.lock() {
        *handle = Some(app.clone());
    }

    if GAMEPAD_INPUT_THREAD_ACTIVE.load(Ordering::SeqCst) {
        return;
    }

    GAMEPAD_INPUT_THREAD_ACTIVE.store(true, Ordering::SeqCst);
    thread::spawn(move || {
        // 纭繚绾跨▼閫€鍑烘椂閲嶇疆鏍囧織
        struct ResetOnDrop;
        impl Drop for ResetOnDrop {
            fn drop(&mut self) {
                GAMEPAD_INPUT_THREAD_ACTIVE.store(false, Ordering::SeqCst);
            }
        }
        let _reset = ResetOnDrop;

        gamepad_input_thread(app);
    });

    println!("[Gamepad Input] Unified thread started");
}

#[cfg(not(windows))]
pub fn start_ui_input_router(_app: AppHandle) {}

/// 鑾峰彇鎵嬫焺鐘舵€侊紙浼樺寲鐗堬細蹇€熸娴嬫墍鏈夋Ы浣嶇殑鎸夐挳杈撳叆锛屾櫤鑳藉垏鎹級
#[cfg(windows)]
#[tauri::command]
pub async fn get_gamepad_state() -> GamepadState {
    // 鍚屾椂鑾峰彇 XInput 鍜?gilrs 鎵嬫焺鐘舵€侊紝鍚堝苟杈撳叆
    let xinput_state = get_xinput_state();
    let gilrs_state = crate::gilrs_gamepad::get_gilrs_gamepad_state();

    // 濡傛灉涓や釜閮芥病杩炴帴
    if !xinput_state.connected && !gilrs_state.connected {
        return GamepadState {
            connected: false,
            buttons: vec![false; 17],
            axes: vec![0.0; 4],
            left_trigger: 0.0,
            right_trigger: 0.0,
        };
    }

    // 濡傛灉鍙湁涓€涓繛鎺ワ紝杩斿洖閭ｄ釜
    if !xinput_state.connected {
        return gilrs_state;
    }
    if !gilrs_state.connected {
        return xinput_state;
    }

    // 涓や釜閮借繛鎺ヤ簡锛屽悎骞惰緭鍏ワ紙浠讳竴鎵嬫焺鐨勬寜閽寜涓嬮兘绠楁寜涓嬶級
    let mut merged_buttons = vec![false; 17];
    #[allow(clippy::needless_range_loop)]
    for i in 0..17 {
        merged_buttons[i] = xinput_state.buttons.get(i).copied().unwrap_or(false)
            || gilrs_state.buttons.get(i).copied().unwrap_or(false);
    }

    // 鎽囨潌锛氬彇缁濆鍊艰緝澶х殑閭ｄ釜
    let mut merged_axes = vec![0.0; 4];
    #[allow(clippy::needless_range_loop)]
    for i in 0..4 {
        let xinput_val = xinput_state.axes.get(i).copied().unwrap_or(0.0);
        let gilrs_val = gilrs_state.axes.get(i).copied().unwrap_or(0.0);
        merged_axes[i] = if xinput_val.abs() > gilrs_val.abs() {
            xinput_val
        } else {
            gilrs_val
        };
    }

    // 鎵虫満锛氬彇杈冨ぇ鍊?    let merged_lt = xinput_state.left_trigger.max(gilrs_state.left_trigger);
    let merged_rt = xinput_state.right_trigger.max(gilrs_state.right_trigger);

    GamepadState {
        connected: true,
        buttons: merged_buttons,
        axes: merged_axes,
        left_trigger: merged_lt,
        right_trigger: merged_rt,
    }
}

/// 鑾峰彇 XInput 鎵嬫焺鐘舵€侊紙鍐呴儴鍑芥暟锛?#[cfg(windows)]
fn get_xinput_state() -> GamepadState {
    let current_index = ACTIVE_GAMEPAD_INDEX.load(Ordering::Relaxed);

    // 鍏堝揩閫熸壂鎻忔墍鏈夋Ы浣嶏紝鎵惧埌鏈夋寜閽緭鍏ョ殑鎵嬫焺锛堢敤浜庡垏鎹級
    for i in 0..4 {
        let mut state = XINPUT_STATE::default();
        let result = unsafe { xinput_get_state(i, &mut state) };

        if result == 0 {
            let mut last_packets = match LAST_PACKET_NUMBERS.lock() {
                Ok(g) => g,
                Err(e) => e.into_inner(),
            };
            let old_packet = last_packets[i as usize];

            // 鍙湁 packet number 鍙樺寲 + 鏈夋寜閽?鎵虫満杈撳叆鎵嶅垏鎹?            if state.dwPacketNumber != old_packet && has_meaningful_input(&state) {
                last_packets[i as usize] = state.dwPacketNumber;

                // 鍒囨崲鍒拌繖涓墜鏌?                if current_index != i as i32 {
                    ACTIVE_GAMEPAD_INDEX.store(i as i32, Ordering::Relaxed);
                }

                drop(last_packets);
                return parse_gamepad_state(state);
            }
        }
    }

    // 娌℃湁鎸夐挳杈撳叆锛岃繑鍥炲綋鍓嶆椿璺冩Ы浣嶇殑鐘舵€侊紙鍖呭惈鎽囨潌锛?    if current_index >= 0 {
        let mut state = XINPUT_STATE::default();
        let result = unsafe { xinput_get_state(current_index as u32, &mut state) };

        if result == 0 {
            // 鏇存柊 packet number
            if let Ok(mut last_packets) = LAST_PACKET_NUMBERS.lock() {
                last_packets[current_index as usize] = state.dwPacketNumber;
            }

            return parse_gamepad_state(state);
        } else {
            // 褰撳墠妲戒綅宸叉柇寮€锛岄噸缃?
ACTIVE_GAMEPAD_INDEX.store(-1, Ordering::Relaxed);
        }
    }

    // 娌℃湁娲昏穬妲戒綅锛岃繑鍥炵涓€涓繛鎺ョ殑鎵嬫焺
    for i in 0..4 {
        let mut state = XINPUT_STATE::default();
        let result = unsafe { xinput_get_state(i, &mut state) };

        if result == 0 {
            ACTIVE_GAMEPAD_INDEX.store(i as i32, Ordering::Relaxed);
            return parse_gamepad_state(state);
        }
    }

    // 娌℃湁浠讳綍 XInput 鎵嬫焺杩炴帴
    GamepadState {
        connected: false,
        buttons: vec![false; 17],
        axes: vec![0.0; 4],
        left_trigger: 0.0,
        right_trigger: 0.0,
    }
}

/// 鑾峰彇鎵嬫焺鐘舵€侊紙闈?Windows 骞冲彴 - 浣跨敤 gilrs锛?#[cfg(not(windows))]
#[tauri::command]
pub async fn get_gamepad_state() -> GamepadState {
    // 闈?Windows 骞冲彴浣跨敤 gilrs
    crate::gilrs_gamepad::get_gilrs_gamepad_state()
}

/// 瑙ｆ瀽 XInput 鐘舵€佷负 GamepadState锛堝鐢?parse_xinput_state_fast锛宖32鈫抐64 鏃犳崯杞崲锛?#[cfg(windows)]
fn parse_gamepad_state(state: XINPUT_STATE) -> GamepadState {
    let (buttons, axes, lt, rt) = parse_xinput_state_fast(&state);
    GamepadState {
        connected: true,
        buttons: buttons.to_vec(),
        axes: axes.iter().map(|&v| v as f64).collect(),
        left_trigger: lt as f64,
        right_trigger: rt as f64,
    }
}

/// 鎵嬫焺娴嬭瘯妯″紡
#[tauri::command]
pub async fn set_gamepad_test_mode(is_test_mode: bool) -> Result<(), String> {
    IS_GAMEPAD_TEST_MODE.store(is_test_mode, Ordering::SeqCst);
    println!(
        "Gamepad test mode: {}",
        if is_test_mode { "ON" } else { "OFF" }
    );
    Ok(())
}

/// 缁戝畾鐩戝惉妯″紡
#[tauri::command]
pub async fn set_binding_listening_mode(app: AppHandle, is_listening: bool) -> Result<(), String> {
    IS_BINDING_LISTENING_MODE.store(is_listening, Ordering::SeqCst);

    println!(
        "Binding listening mode: {}",
        if is_listening { "ON" } else { "OFF" }
    );

    // 璇诲彇璁剧疆澶辫触鏃惰褰曟棩蹇椾絾缁х画锛堜娇鐢ㄩ粯璁ゅ€硷級
    let settings = match load_settings(app.clone()).await {
        Ok(s) => s,
        Err(e) => {
            println!("[Binding] 璇诲彇璁剧疆澶辫触锛屼娇鐢ㄩ粯璁ゅ€? {}", e);
            crate::types::Settings::with_defaults()
        }
    };
    println!(
        "[Binding] Current summon_binding: {:?}",
        settings.summon_binding
    );

    if is_listening {
        // 鍋滄 XInput 杞
        stop_xinput_binding_polling();

        // 纭繚 HID 鐩戞帶绾跨▼鍦ㄨ繍琛岋紙鎸夐渶鍚姩锛岀敤浜庢崟鑾?HID 鑷畾涔夋寜閿級
        crate::custom_keys::ensure_monitor_running();

        // 杩涘叆缁戝畾妯″紡锛氬仠姝㈡墍鏈夊揩鎹烽敭鐩戝惉
        // 1. 鍋滄杞锛堣嚜瀹氫箟閿洏缁勫悎锛?
crate::keyboard_hook::uninstall_keyboard_hook();
        println!("[Binding] Stopped keyboard polling");

        // 2. 涓存椂娉ㄩ攢榛樿蹇嵎閿?Ctrl+Alt+Shift+G
        use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
        let default_shortcut = Shortcut::new(
            Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
            Code::KeyG,
        );
        let _ = app.global_shortcut().unregister(default_shortcut);
        println!("[Binding] Temporarily unregistered Ctrl+Alt+Shift+G");
    } else {
        // 閫€鍑虹粦瀹氭ā寮忥細鎭㈠鎵€鏈夊揩鎹烽敭
        // 1. 鎭㈠榛樿蹇嵎閿?Ctrl+Alt+Shift+G
        use tauri_plugin_global_shortcut::{Code, Modifiers, Shortcut};
        let default_shortcut = Shortcut::new(
            Some(Modifiers::CONTROL | Modifiers::ALT | Modifiers::SHIFT),
            Code::KeyG,
        );
        let app_handle = app.clone();
        let _ =
            app.global_shortcut()
                .on_shortcut(default_shortcut, move |_app, _shortcut, event| {
                    if event.state != tauri_plugin_global_shortcut::ShortcutState::Pressed {
                        return;
                    }
                    if let Some(_window) = app_handle.get_webview_window("main") {
                        use crate::panel::PANEL_IS_OPEN;
                        use std::sync::atomic::Ordering;

                        let is_open = PANEL_IS_OPEN.load(Ordering::SeqCst);
                        if is_open {
                            // 浣跨敤 hide_panel 鍛戒护锛屽甫鍔ㄧ敾
                            let handle_clone = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = crate::panel::hide_panel(handle_clone).await;
                            });
                        } else {
                            // 浣跨敤 show_panel 鍛戒护锛屽甫鍔ㄧ敾
                            let handle_clone = app_handle.clone();
                            tauri::async_runtime::spawn(async move {
                                let _ = crate::panel::show_panel(handle_clone).await;
                            });
                        }
                    }
                });
        println!("[Binding] Restored Ctrl+Alt+Shift+G");

        // 2. 浠?settings.json 鎭㈠鎵€鏈夌粦瀹氾紙閬垮厤 summon_binding_2/鍔熻兘鐑敭鍦ㄧ粦瀹氭ā寮忓悗涓㈠け锛?
use crate::keyboard_hook::HotkeyAction;

        // 鍚屾鈥滃綍鍒跺紑鍏斥€濆畨鍏ㄥ紑鍏筹紙閬垮厤鏈缃椂璇Е Win+Alt+R 褰曞睆锛?
let record_toggle_allowed = settings
            .record_toggle_binding
            .as_ref()
            .map(|b| b.binding_type != "none")
            .unwrap_or(false);
        crate::keyboard_hook::set_record_toggle_allowed(record_toggle_allowed);

        let mut need_xinput_polling = false;

        // 鎭㈠鍛煎嚭閿?1
        if let Some(ref binding) = settings.summon_binding {
            if binding.binding_type == "keyboard" {
                if let Some(ref accel) = binding.accelerator {
                    // 榛樿閿敱 global_shortcut 璐熻矗
                    if accel.to_uppercase() != "CTRL+ALT+SHIFT+G" {
                        let lp = binding.long_press.unwrap_or(false);
                        println!("[Binding] Restoring keyboard combo: {} (long_press={})", accel, lp);
                        if let Err(e) =
                            crate::keyboard_hook::set_hotkey_binding_ex(app.clone(), HotkeyAction::PanelToggle, Some(accel.as_str()), lp)
                        {
                            println!("[Binding] Failed to restore: {}", e);
                        } else {
                            println!("[Binding] Restored custom shortcut: {}", accel);
                        }
                    }
                }
            } else if binding.binding_type == "gamepadButton" {
                let lp = binding.long_press.unwrap_or(false);
                set_xinput_binding_full_ex(
                    HotkeyAction::PanelToggle,
                    binding.buttons.clone(),
                    binding.custom_keys,
                    lp,
                );
                need_xinput_polling = true;
                println!(
                    "[Binding] Restored XInput binding: buttons={:?}, custom_keys={:?}, long_press={}",
                    binding.buttons, binding.custom_keys, lp
                );
            } else if binding.binding_type == "gilrsButton" {
                if let Some(ref buttons) = binding.buttons {
                    let lp = binding.long_press.unwrap_or(false);
                    crate::gilrs_gamepad::set_gilrs_binding_ex(
                        HotkeyAction::PanelToggle,
                        Some(buttons.clone()),
                        lp,
                    );
                }
            }
        }

        // 鎭㈠鍛煎嚭閿?2
        if let Some(ref binding) = settings.summon_binding_2 {
            if binding.binding_type == "keyboard" {
                if let Some(ref accel) = binding.accelerator {
                    let lp = binding.long_press.unwrap_or(false);
                    println!("[Binding] Restoring keyboard combo for binding2: {} (long_press={})", accel, lp);
                    if let Err(e) =
                        crate::keyboard_hook::set_hotkey_binding_ex(app.clone(), HotkeyAction::PanelToggle2, Some(accel.as_str()), lp)
                    {
                        println!("[Binding] Failed to restore binding2: {}", e);
                    }
                }
            } else if binding.binding_type == "gamepadButton" {
                let lp = binding.long_press.unwrap_or(false);
                set_xinput_binding_full_ex(
                    HotkeyAction::PanelToggle2,
                    binding.buttons.clone(),
                    binding.custom_keys,
                    lp,
                );
                need_xinput_polling = true;
                println!(
                    "[Binding] Restored XInput binding2: buttons={:?}, custom_keys={:?}, long_press={}",
                    binding.buttons, binding.custom_keys, lp
                );
            } else if binding.binding_type == "gilrsButton" {
                if let Some(ref buttons) = binding.buttons {
                    let lp = binding.long_press.unwrap_or(false);
                    crate::gilrs_gamepad::set_gilrs_binding_ex(
                        HotkeyAction::PanelToggle2,
                        Some(buttons.clone()),
                        lp,
                    );
                }
            }
        }

        // 鎭㈠鍔熻兘鐑敭锛圤SD/TDP/妗岄潰/闄€铻轰华绛夛級
        let mut restore_hotkey = |binding: &Option<crate::types::SummonBinding>,
                                  action: HotkeyAction| {
            if let Some(ref b) = binding {
                if b.binding_type == "none" {
                    return;
                }
                let lp = b.long_press.unwrap_or(false);
                if b.binding_type == "keyboard" {
                    if let Some(ref accel) = b.accelerator {
                        println!("[Binding] Restoring {:?} keyboard: {} (long_press={})", action, accel, lp);
                        let _ = crate::keyboard_hook::set_hotkey_binding_ex(
                            app.clone(),
                            action,
                            Some(accel.as_str()),
                            lp,
                        );
                    }
                } else if b.binding_type == "gamepadButton" {
                    set_xinput_binding_full_ex(action, b.buttons.clone(), b.custom_keys, lp);
                    need_xinput_polling = true;
                } else if b.binding_type == "gilrsButton" {
                    if let Some(ref buttons) = b.buttons {
                        crate::gilrs_gamepad::set_gilrs_binding_ex(action, Some(buttons.clone()), lp);
                    }
                }
            }
        };

        restore_hotkey(&settings.osd_toggle_binding, HotkeyAction::OsdToggle);
        restore_hotkey(&settings.tdp_up_binding, HotkeyAction::TdpUp);
        restore_hotkey(&settings.tdp_down_binding, HotkeyAction::TdpDown);
        restore_hotkey(&settings.show_desktop_binding, HotkeyAction::ShowDesktop);
        restore_hotkey(&settings.desktop_mode_binding, HotkeyAction::DesktopMode);
        restore_hotkey(
            &settings.bigscreen_mode_binding,
            HotkeyAction::BigscreenMode,
        );
        restore_hotkey(
            &settings.steam_bigpicture_binding,
            HotkeyAction::SteamBigPicture,
        );
        restore_hotkey(&settings.hibernate_binding, HotkeyAction::Hibernate);
        restore_hotkey(&settings.sleep_binding, HotkeyAction::Sleep);
        restore_hotkey(&settings.shutdown_binding, HotkeyAction::Shutdown);
        restore_hotkey(&settings.task_manager_binding, HotkeyAction::TaskManager);
        restore_hotkey(
            &settings.mouse_sim_toggle_binding,
            HotkeyAction::MouseSimToggle,
        );
        restore_hotkey(&settings.screenshot_binding, HotkeyAction::Screenshot);
        restore_hotkey(&settings.record_toggle_binding, HotkeyAction::RecordToggle);
        restore_hotkey(&settings.gyro_toggle_binding, HotkeyAction::GyroToggle);
        restore_hotkey(&settings.gyro_hold_binding, HotkeyAction::GyroHold);
        restore_hotkey(&settings.gyro_hold_binding_2, HotkeyAction::GyroHold2);
        restore_hotkey(&settings.gyro_hold_binding_3, HotkeyAction::GyroHold3);
        restore_hotkey(&settings.alt_tab_binding, HotkeyAction::AltTab);

        if need_xinput_polling {
            start_xinput_binding_polling(app.clone());
        }

        // 涓嶅啀鍚姩 PowerShell 鑴氭湰锛岄伩鍏嶉粯璁ょ粦瀹氬啿绐?        println!("[Binding] Exiting binding mode");
    }
    Ok(())
}

/// 鑾峰彇鍒濆璁剧疆
#[tauri::command]
pub async fn get_initial_settings(app: AppHandle) -> Result<serde_json::Value, String> {
    // 璇诲彇璁剧疆澶辫触鏃朵娇鐢ㄩ粯璁ゅ€硷紙鍓嶇闇€瑕佽繖浜涙暟鎹潵鍚姩锛?    let settings = load_settings(app.clone()).await.unwrap_or_default();

    // 鍚屾鎵虫満姝诲尯鍒拌繍琛屾椂锛堢敤浜庣儹閿?娴嬭瘯/瀵艰埅锛?    apply_trigger_deadzone_settings(&settings);
    // 鍚屾鎽囨潌鏄犲皠鍒拌繍琛屾椂锛堢敤浜庢憞鏉嗏啋WASD/鏂瑰悜閿級
    crate::kbm_mapper::apply_stick_mode_settings(&settings);

    // 璋冭瘯鏃ュ織
    println!(
        "[Settings] get_initial_settings: theme={:?}, panel_width={:?}",
        settings.theme, settings.panel_width
    );

    // 浜害鐩存帴浣跨敤鎸佷箙鍖栫殑鍊硷紝閬垮厤 WMI 鏌ヨ寤惰繜锛堥粯璁?80锛?    let brightness = settings.brightness.unwrap_or(80);

    // 璁惧妫€娴嬪湪鍚庡彴绾跨▼鎵ц锛屽苟璁剧疆 5 绉掕秴鏃朵繚鎶?    // 閬垮厤 WMI 鏈嶅姟鏈氨缁椂鏌ヨ鎸傝捣瀵艰嚧鍓嶇鍔犺浇鍗℃
    let device_info = tokio::time::timeout(
        std::time::Duration::from_secs(5),
        tokio::task::spawn_blocking(detect_device),
    )
    .await
    .ok()
    .and_then(|r| r.ok())
    .flatten();

    let (
        fan_control_available,
        detected_device_name,
        fan_control_type,
        device_tdp_min,
        device_tdp_max,
    ) = match device_info {
        Some(device) => (
            device.fan_control_available,
            Some(device.name),
            device.fan_control_type,
            device.tdp_min,
            device.tdp_max,
        ),
        None => (false, None, None, None, None),
    };

    if let Some(ref name) = detected_device_name {
        println!(
            "Device detected: {}, fan control: {}, type: {:?}, tdp: {:?}-{:?}W",
            name, fan_control_available, fan_control_type, device_tdp_min, device_tdp_max
        );
    } else {
        // 璁惧妫€娴嬪け璐ワ紝鍚姩閫掑寤惰繜閲嶈瘯浠诲姟
        // 杩欓€氬父鍙戠敓鍦ㄥ紑鏈鸿嚜鍚椂 WMI 鏈嶅姟鏈氨缁?        let app_clone = app.clone();
        tokio::spawn(async move {
            // 閫掑寤惰繜閲嶈瘯锛?s, 5s, 10s, 15s, 30s锛堟€诲叡绾?63 绉掔殑閲嶈瘯绐楀彛锛?            let retry_delays = [3, 5, 10, 15, 30];
            
            for (i, delay) in retry_delays.iter().enumerate() {
                tokio::time::sleep(std::time::Duration::from_secs(*delay)).await;
                
                // 閲嶈瘯璁惧妫€娴?
match tokio::task::spawn_blocking(detect_device).await {
                    Ok(Some(device)) => {
                        println!(
                            "[Device] 寤惰繜閲嶈瘯鎴愬姛 (绗?{} 娆? 寤惰繜 {}s): {}, fan_control: {}",
                            i + 1, delay, device.name, device.fan_control_available
                        );
                        
                        // 璁惧鏀寔椋庢墖鎺у埗鏃讹紝鍚姩鐙珛鐨勯鎵?RPM 杞绾跨▼
                        // 锛堝惎鍔ㄩ樁娈?detect_device 澶辫触瀵艰嚧 start_fan_rpm_monitor 璺宠繃鐨勮ˉ鏁戯級
                        if device.fan_control_available {
                            crate::ec::start_fan_rpm_monitor(app_clone.clone());
                        }

                        // 閫氱煡鍓嶇鏇存柊璁惧淇℃伅
                        let _ = app_clone.emit(
                            "device_detected",
                            serde_json::json!({
                                "fanControlAvailable": device.fan_control_available,
                                "detectedDevice": device.name,
                                "fanControlType": device.fan_control_type,
                                "deviceTdpMin": device.tdp_min,
                                "deviceTdpMax": device.tdp_max,
                            }),
                        );
                        return; // 鎴愬姛锛岄€€鍑洪噸璇曞惊鐜?                    }
                    _ => {
                        println!(
                            "[Device] 寤惰繜閲嶈瘯澶辫触 (绗?{} 娆? 寤惰繜 {}s)锛寋}",
                            i + 1, delay,
                            if i < retry_delays.len() - 1 { "缁х画閲嶈瘯..." } else { "宸茶揪鏈€澶ч噸璇曟鏁? }
                        );
                    }
                }
            }
        });
    }

    // TDP 鑼冨洿浼樺厛绾э細鐢ㄦ埛璁剧疆 > 璁惧榛樿 > 鍏ㄥ眬榛樿
    let tdp_min = settings.tdp_min.or(device_tdp_min).unwrap_or(1);
    let tdp_max = settings.tdp_max.or(device_tdp_max).unwrap_or(85);

    let result = serde_json::json!({
        "tdp": settings.tdp,
        "brightness": brightness,
        "volume": settings.volume.unwrap_or(60),
        "fanMode": settings.fan_mode.unwrap_or(1),
        "fanControlAvailable": fan_control_available,
        "detectedDevice": detected_device_name,
        "fanControlType": fan_control_type,
        "summonBinding": settings.summon_binding,
        "summonBinding2": settings.summon_binding_2,
        // 鍔熻兘鐑敭缁戝畾锛堢敤浜庡墠绔樉绀猴級
        "osd_toggle_binding": settings.osd_toggle_binding,
        "tdp_up_binding": settings.tdp_up_binding,
        "tdp_down_binding": settings.tdp_down_binding,
        "show_desktop_binding": settings.show_desktop_binding,
        "desktop_mode_binding": settings.desktop_mode_binding,
        "bigscreen_mode_binding": settings.bigscreen_mode_binding,
        "steam_bigpicture_binding": settings.steam_bigpicture_binding,
        "hibernate_binding": settings.hibernate_binding,
        "sleep_binding": settings.sleep_binding,
        "shutdown_binding": settings.shutdown_binding,
        "task_manager_binding": settings.task_manager_binding,
        "mouse_sim_toggle_binding": settings.mouse_sim_toggle_binding,
        "screenshot_binding": settings.screenshot_binding,
        "record_toggle_binding": settings.record_toggle_binding,
        "gyro_toggle_binding": settings.gyro_toggle_binding,
        "gyro_hold_binding": settings.gyro_hold_binding,
        "gyro_hold_binding_2": settings.gyro_hold_binding_2,
        "gyro_hold_binding_3": settings.gyro_hold_binding_3,
        "alt_tab_binding": settings.alt_tab_binding,
        "tdp_min": tdp_min,
        "tdp_max": tdp_max,
        // 璁惧鎺ㄨ崘鐨?TDP 鑼冨洿锛堢敤浜庡墠绔樉绀哄拰閲嶇疆锛?        "deviceTdpMin": device_tdp_min,
        "deviceTdpMax": device_tdp_max,
        // UI 璁剧疆瀛楁锛堝墠绔渶瑕佺敤浜庡垵濮嬪寲锛?        "theme": settings.theme,
        "language": settings.language,
        "sound_enabled": settings.sound_enabled,
        "tdp_presets": settings.tdp_presets,
        "panel_width": settings.panel_width,
        "font_size": settings.font_size,
        "gesture_enabled": settings.gesture_enabled,
        "blur_hide_enabled": settings.blur_hide_enabled,
        "no_focus_mode": settings.no_focus_mode,
        "focus_return_enabled": settings.focus_return_enabled,
        "fullscreen_mode": settings.fullscreen_mode,
        "safe_mode": settings.safe_mode,
        "suspend_on_sleep": settings.suspend_on_sleep,
        "input_isolation_enabled": settings.input_isolation_enabled,
        "xinput_isolation_enabled": settings.xinput_isolation_enabled,
        "gilrs_detection_enabled": settings.gilrs_detection_enabled,
        "vibration_intensity": settings.vibration_intensity,
        // 鍏ㄥ眬鎵虫満姝诲尯璁剧疆鍊硷紙0-255, 0=鍏抽棴/榛樿锛?        "trigger_deadzone_lt": settings.trigger_deadzone_lt.unwrap_or(0),
        "trigger_deadzone_rt": settings.trigger_deadzone_rt.unwrap_or(0),
        // 鎽囨潌鏄犲皠锛堝叏灞€璁剧疆锛氬叧闂?WASD/鏂瑰悜閿級
        "left_stick_mode": settings.left_stick_mode,
        "right_stick_mode": settings.right_stick_mode,
        "gamepad_vibration_intensity": settings.gamepad_vibration_intensity,
        "touchscreen_disabled": settings.touchscreen_disabled,
        "optical_mouse_disabled": settings.optical_mouse_disabled,
        "autostart": settings.autostart,
        "start_minimized": settings.start_minimized,
        "hide_tab_bar": settings.hide_tab_bar,
        "hide_status_bar": settings.hide_status_bar,
        "use_osk_keyboard": settings.use_osk_keyboard,
        "quick_items": settings.quick_items,
        "active_cores": settings.active_cores,
        "core_mode": settings.core_mode,
        "core_scheduling": settings.core_scheduling,
        "float_gpu_enabled": settings.float_gpu_enabled,
    });

    Ok(result)
}

/// 鑾峰彇宸茶繛鎺ョ殑鎵嬫焺鍒楄〃
#[tauri::command]
pub async fn get_connected_gamepads() -> serde_json::Value {
    let mut gamepads = Vec::new();

    #[cfg(windows)]
    {
        // XInput 鎵嬫焺
        for i in 0..4u32 {
            let mut state = XINPUT_STATE::default();
            let result = unsafe { xinput_get_state(i, &mut state) };
            if result == 0 {
                // 鎵嬫焺宸茶繛鎺?
let is_active = ACTIVE_GAMEPAD_INDEX.load(Ordering::Relaxed) == i as i32;
                let name = if i == 0 {
                    "builtin_gamepad".to_string()
                } else {
                    format!("gamepad_{}", i + 1)
                };
                gamepads.push(serde_json::json!({
                    "index": i,
                    "connected": true,
                    "isActive": is_active,
                    "name": name,
                    "type": "XInput",
                    "source": "xinput"
                }));
            }
        }

        // gilrs 鎵嬫焺 (DS4/DS5/NS 绛?
        let gilrs_gamepads = crate::gilrs_gamepad::get_connected_gilrs_gamepads();
        for gp in gilrs_gamepads {
            gamepads.push(gp);
        }
    }

    #[cfg(not(windows))]
    {
        // 闈?Windows 骞冲彴鍙娇鐢?gilrs
        let gilrs_gamepads = crate::gilrs_gamepad::get_connected_gilrs_gamepads();
        for gp in gilrs_gamepads {
            gamepads.push(gp);
        }
    }

    serde_json::json!({
        "gamepads": gamepads,
        "count": gamepads.len()
    })
}

/// 绂佺敤/鍚敤鎺屾満鎵嬫焺锛堣澶栨帴鎵嬫焺鎴愪负绱㈠紩0锛?/// 閫氳繃 XInput Hook 灞忚斀绱㈠紩 0 鎵嬫焺鐨勮緭鍏?
#[tauri::command]
pub async fn toggle_builtin_gamepad(enable: bool) -> CommandResult {
    // enable = true 琛ㄧず鍚敤鎵嬫焺锛宔nable = false 琛ㄧず绂佺敤锛堝睆钄斤級鎵嬫焺
    // 鎵€浠?blocked = !enable
    let blocked = !enable;

    println!(
        "[Gamepad] 閫氳繃 XInput Hook {}绱㈠紩 0 鎵嬫焺...",
        if blocked { "灞忚斀" } else { "鎭㈠" }
    );

    // 浣跨敤 XInput Hook 鐨勫叡浜唴瀛樺姛鑳藉睆钄界储寮?0 鐨勬墜鏌?
crate::vibration::set_gamepad_input_blocked(0, blocked);

    // 濡傛灉鏄睆钄芥搷浣滐紝鍚屾椂瑙﹀彂娉ㄥ叆浠ョ‘淇?Hook 鐢熸晥
    if blocked {
        crate::vibration::inject_to_all_xinput_processes();
    }

    CommandResult {
        success: true,
        message: Some(format!(
            "鎺屾満鎵嬫焺宸瞷}",
            if enable { "鍚敤" } else { "绂佺敤" }
        )),
        error: None,
    }
}

/// 瑙﹀彂闇囧姩锛堝褰撳墠娲昏穬鐨勬墜鏌勶級
/// intensity: 0-100 闇囧姩寮哄害鐧惧垎姣?
#[tauri::command]
pub async fn trigger_vibration(duration: u64, intensity: Option<u32>) -> CommandResult {
    #[cfg(windows)]
    {
        // 棣栧厛妫€鏌ユ槸鍚︽湁鎵嬫焺杩炴帴
        let mut found_gamepad = false;
        let mut gamepad_index = 0u32;

        // 浼樺厛浣跨敤娲昏穬鐨勬墜鏌勭储寮?
let active_index = ACTIVE_GAMEPAD_INDEX.load(Ordering::Relaxed);
        if active_index >= 0 {
            let mut state = XINPUT_STATE::default();
            if unsafe { xinput_get_state(active_index as u32, &mut state) } == 0 {
                gamepad_index = active_index as u32;
                found_gamepad = true;
            }
        }

        // 濡傛灉娌℃湁娲昏穬鎵嬫焺锛屾壂鎻忔墍鏈夋Ы浣嶆壘鍒扮涓€涓繛鎺ョ殑鎵嬫焺
        if !found_gamepad {
            for i in 0..4u32 {
                let mut state = XINPUT_STATE::default();
                if unsafe { xinput_get_state(i, &mut state) } == 0 {
                    gamepad_index = i;
                    found_gamepad = true;
                    // 鏇存柊娲昏穬绱㈠紩
                    ACTIVE_GAMEPAD_INDEX.store(i as i32, Ordering::Relaxed);
                    println!("[Vibration] 鑷姩閫夋嫨鎵嬫焺妲戒綅 {}", i);
                    break;
                }
            }
        }

        if !found_gamepad {
            return CommandResult {
                success: false,
                message: None,
                error: Some("娌℃湁妫€娴嬪埌杩炴帴鐨勬墜鏌?.to_string()),
            };
        }

        // 璁＄畻闇囧姩寮哄害 (0-65535)
        let intensity_percent = intensity.unwrap_or(100).min(100) as f32 / 100.0;
        let motor_speed = (65535.0 * intensity_percent) as u16;

        println!(
            "[Vibration] 瑙﹀彂闇囧姩: 鎵嬫焺={}, 寮哄害={}%, 鎸佺画={}ms",
            gamepad_index,
            (intensity_percent * 100.0) as u32,
            duration
        );

        std::thread::spawn(move || {
            let mut vibration = XINPUT_VIBRATION {
                wLeftMotorSpeed: motor_speed,
                wRightMotorSpeed: motor_speed,
            };
            let result = unsafe { XInputSetState(gamepad_index, &vibration) };
            if result != 0 {
                println!("[Vibration] XInputSetState 澶辫触: 閿欒鐮?{}", result);
            }
            std::thread::sleep(std::time::Duration::from_millis(duration));
            vibration.wLeftMotorSpeed = 0;
            vibration.wRightMotorSpeed = 0;
            let _ = unsafe { XInputSetState(gamepad_index, &vibration) };
        });

        CommandResult {
            success: true,
            message: Some(format!(
                "闇囧姩鎸囦护宸插彂閫?(鎵嬫焺: {}, 寮哄害: {}%)",
                gamepad_index,
                intensity.unwrap_or(100)
            )),
            error: None,
        }
    }

    #[cfg(not(windows))]
    {
        CommandResult {
            success: false,
            message: None,
            error: Some("浠呮敮鎸?Windows 骞冲彴".to_string()),
        }
    }
}

// ==================== 榧犳爣妯℃嫙鍔熻兘 ====================

/// 搴旂敤姝诲尯鍜岄潪绾挎€ф洸绾匡紙鏇寸簿纭殑灏忓箙搴︽帶鍒讹紝杈圭紭鏇村揩锛?
fn apply_stick_curve(value: f64, deadzone: f64, sensitivity: f64) -> i32 {
    let abs_val = value.abs();
    if abs_val < deadzone {
        return 0;
    }
    // 閲嶆柊鏄犲皠鍒?0-1 鑼冨洿锛堝幓闄ゆ鍖猴級
    let normalized = (abs_val - deadzone) / (1.0 - deadzone);
    // 搴旂敤骞虫柟鏇茬嚎锛堝皬骞呭害鏇寸簿纭紝澶у箙搴︽洿蹇級
    let curved = normalized * normalized;
    // 搴旂敤鐏垫晱搴﹀拰绗﹀彿
    (curved * sensitivity * value.signum()) as i32
}

/// 榧犳爣妯℃嫙杞绾跨▼
#[cfg(windows)]
fn mouse_simulation_thread() {
    println!("[Mouse Sim] 榧犳爣妯℃嫙绾跨▼鍚姩");

    // 榧犳爣绉诲姩鐏垫晱搴︼紙鍍忕礌/杞村€硷級
    const SENSITIVITY: f64 = 20.0;
    // 姝诲尯锛堝拷鐣ュ皬骞呭害鎽囨潌婕傜Щ锛?
const DEADZONE: f64 = 0.15;

    while MOUSE_SIM_ACTIVE.load(Ordering::SeqCst) {
        // 璺宠繃娴嬭瘯妯″紡鍜岀粦瀹氭ā寮?
if IS_GAMEPAD_TEST_MODE.load(Ordering::SeqCst)
            || IS_BINDING_LISTENING_MODE.load(Ordering::SeqCst)
        {
            thread::sleep(Duration::from_millis(50));
            continue;
        }

        // 鑾峰彇褰撳墠娲昏穬鎵嬫焺
        let active_index = ACTIVE_GAMEPAD_INDEX.load(Ordering::Relaxed);
        let gamepad_index = if active_index >= 0 {
            active_index as u32
        } else {
            0
        };

        let mut state = XINPUT_STATE::default();
        let result = unsafe { xinput_get_state(gamepad_index, &mut state) };

        if result == 0 {
            let gp = &state.Gamepad;

            // 鍙虫憞鏉嗘ā鎷熼紶鏍囩Щ鍔?
let rx = gp.sThumbRX as f64 / 32767.0;
            let ry = gp.sThumbRY as f64 / 32767.0;

            // 搴旂敤姝诲尯鍜岄潪绾挎€ф洸绾?
let dx = apply_stick_curve(rx, DEADZONE, SENSITIVITY);
            let dy = apply_stick_curve(-ry, DEADZONE, SENSITIVITY); // Y杞村弽杞?
            // 鍙戦€侀紶鏍囩Щ鍔ㄤ簨浠?
if dx != 0 || dy != 0 {
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx,
                            dy,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_MOVE,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
            }

            // LB 妯℃嫙榧犳爣宸﹂敭
            let lb_pressed = (gp.wButtons.0 & XINPUT_GAMEPAD_LEFT_SHOULDER.0) != 0;
            let lb_was_pressed = MOUSE_SIM_LB_WAS_PRESSED.load(Ordering::SeqCst);

            if lb_pressed && !lb_was_pressed {
                // 鎸変笅宸﹂敭
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTDOWN,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                MOUSE_SIM_LB_WAS_PRESSED.store(true, Ordering::SeqCst);
            } else if !lb_pressed && lb_was_pressed {
                // 閲婃斁宸﹂敭
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_LEFTUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                MOUSE_SIM_LB_WAS_PRESSED.store(false, Ordering::SeqCst);
            }

            // RB 妯℃嫙榧犳爣鍙抽敭
            let rb_pressed = (gp.wButtons.0 & XINPUT_GAMEPAD_RIGHT_SHOULDER.0) != 0;
            let rb_was_pressed = MOUSE_SIM_RB_WAS_PRESSED.load(Ordering::SeqCst);

            if rb_pressed && !rb_was_pressed {
                // 鎸変笅鍙抽敭
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_RIGHTDOWN,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                MOUSE_SIM_RB_WAS_PRESSED.store(true, Ordering::SeqCst);
            } else if !rb_pressed && rb_was_pressed {
                // 閲婃斁鍙抽敭
                let input = INPUT {
                    r#type: INPUT_MOUSE,
                    Anonymous: INPUT_0 {
                        mi: MOUSEINPUT {
                            dx: 0,
                            dy: 0,
                            mouseData: 0,
                            dwFlags: MOUSEEVENTF_RIGHTUP,
                            time: 0,
                            dwExtraInfo: 0,
                        },
                    },
                };
                unsafe { SendInput(&[input], std::mem::size_of::<INPUT>() as i32) };
                MOUSE_SIM_RB_WAS_PRESSED.store(false, Ordering::SeqCst);
            }
        }

        // 杞闂撮殧 16ms锛堢害60Hz锛?        thread::sleep(Duration::from_millis(16));
    }

    println!("[Mouse Sim] 榧犳爣妯℃嫙绾跨▼鍋滄");
}

/// 鍚姩榧犳爣妯℃嫙
#[cfg(windows)]
fn start_mouse_simulation() {
    if MOUSE_SIM_ACTIVE.load(Ordering::SeqCst) {
        return; // 宸茬粡鍦ㄨ繍琛?    }

    MOUSE_SIM_ACTIVE.store(true, Ordering::SeqCst);
    MOUSE_SIM_LB_WAS_PRESSED.store(false, Ordering::SeqCst);
    MOUSE_SIM_RB_WAS_PRESSED.store(false, Ordering::SeqCst);

    thread::spawn(mouse_simulation_thread);

    println!("[Mouse Sim] 榧犳爣妯℃嫙宸插惎鍔?);
}

/// 鍋滄榧犳爣妯℃嫙
#[cfg(windows)]
fn stop_mouse_simulation() {
    if MOUSE_SIM_ACTIVE.load(Ordering::SeqCst) {
        println!("[Mouse Sim] 鍋滄榧犳爣妯℃嫙...");
        MOUSE_SIM_ACTIVE.store(false, Ordering::SeqCst);
        thread::sleep(Duration::from_millis(20));
    }
}

/// 璁剧疆榧犳爣妯℃嫙寮€鍏筹紙Tauri 鍛戒护锛?
#[tauri::command]
pub async fn set_mouse_simulation(enabled: bool) -> CommandResult {
    #[cfg(windows)]
    {
        if enabled {
            start_mouse_simulation();
            CommandResult {
                success: true,
                message: Some("榧犳爣妯℃嫙宸插紑鍚?.to_string()),
                error: None,
            }
        } else {
            stop_mouse_simulation();
            CommandResult {
                success: true,
                message: Some("榧犳爣妯℃嫙宸插叧闂?.to_string()),
                error: None,
            }
        }
    }

    #[cfg(not(windows))]
    {
        CommandResult {
            success: false,
            message: None,
            error: Some("浠呮敮鎸?Windows".to_string()),
        }
    }
}

// ==================== gilrs 鎵嬫焺鏀寔 ====================

/// 鍒濆鍖?gilrs 鎵嬫焺鏀寔锛圖S4/DS5/NS 绛夛級
#[tauri::command]
pub async fn init_gilrs_gamepad() -> Result<bool, String> {
    crate::gilrs_gamepad::init_gilrs()?;
    crate::gilrs_gamepad::start_gilrs_polling();
    Ok(crate::gilrs_gamepad::is_gilrs_gamepad_available())
}

/// 鑾峰彇 gilrs 鎵嬫焺鐘舵€?
#[tauri::command]
pub async fn get_gilrs_gamepad_state() -> GamepadState {
    crate::gilrs_gamepad::get_gilrs_gamepad_state()
}

/// 鑾峰彇 gilrs 鎵嬫焺淇℃伅
#[tauri::command]
pub async fn get_gilrs_gamepad_info() -> Option<serde_json::Value> {
    crate::gilrs_gamepad::get_gilrs_gamepad_info().map(|(name, gamepad_type)| {
        serde_json::json!({
            "name": name,
            "type": gamepad_type
        })
    })
}

/// 妫€鏌ユ槸鍚︽湁 gilrs 鎵嬫焺鍙敤
#[tauri::command]
pub async fn is_gilrs_gamepad_available() -> bool {
    crate::gilrs_gamepad::is_gilrs_gamepad_available()
}

/// 璁剧疆 gilrs 鎵嬫焺鎸夐敭缁戝畾
#[tauri::command]
pub async fn set_gilrs_binding(action: String, buttons: Option<Vec<i32>>) -> Result<(), String> {
    use crate::keyboard_hook::HotkeyAction;

    let action = match action.as_str() {
        "PanelToggle" => HotkeyAction::PanelToggle,
        "OsdToggle" => HotkeyAction::OsdToggle,
        "TdpUp" => HotkeyAction::TdpUp,
        "TdpDown" => HotkeyAction::TdpDown,
        "ShowDesktop" => HotkeyAction::ShowDesktop,
        "DesktopMode" => HotkeyAction::DesktopMode,
        "BigscreenMode" => HotkeyAction::BigscreenMode,
        "SteamBigPicture" => HotkeyAction::SteamBigPicture,
        "Hibernate" => HotkeyAction::Hibernate,
        "Sleep" => HotkeyAction::Sleep,
        "Shutdown" => HotkeyAction::Shutdown,
        "TaskManager" => HotkeyAction::TaskManager,
        "MouseSimToggle" => HotkeyAction::MouseSimToggle,
        "Screenshot" => HotkeyAction::Screenshot,
        "RecordToggle" => HotkeyAction::RecordToggle,
        "GyroToggle" => HotkeyAction::GyroToggle,
        "GyroHold" => HotkeyAction::GyroHold,
        _ => return Err(format!("鏈煡鐨勫姩浣? {}", action)),
    };

    crate::gilrs_gamepad::set_gilrs_binding(action, buttons);
    Ok(())
}

