/**
 * 后端 API 调用封装
 */
import { invoke } from "@tauri-apps/api/core";
import type { RtssResult } from './types';

// ==================== 手柄→键盘/鼠标映射 (KBM Mapper) ====================

export type KbmGamepadButton =
  | "a"
  | "b"
  | "x"
  | "y"
  | "lb"
  | "rb"
  | "lt"
  | "rt"
  | "back"
  | "start"
  | "ls"
  | "rs"
  | "dpad_up"
  | "dpad_down"
  | "dpad_left"
  | "dpad_right"
  | "guide";

// Button source supports single button or chord (combo)
export type KbmButtonSource = KbmGamepadButton | KbmGamepadButton[];

export type KbmStickId = "left" | "right";

export type KbmStickMode = "off" | "wasd" | "arrows";

export type KbmMacroStep =
  | { type: "tap"; keys: string }
  | { type: "down"; keys: string }
  | { type: "up"; keys: string }
  | { type: "delay"; ms: number };

export type KbmAction =
  | { type: "tap"; keys: string }
  | { type: "hold"; keys: string }
  | { type: "turbo"; keys: string; rate_hz: number }
  | { type: "macro"; steps: KbmMacroStep[] };

export interface KbmButtonBinding {
  source: KbmButtonSource;
  normal: KbmAction;
  shifted?: KbmAction;
  /** default: true */
  enabled?: boolean;
}

export interface KbmStickBinding {
  stick: KbmStickId;
  normal: KbmStickMode;
  shifted?: KbmStickMode;

  deadzone: number;
  sensitivity: number;
  invert_x: boolean;
  invert_y: boolean;

  press_threshold: number;
  release_threshold: number;
}

export interface KbmProfile {
  id: string;
  name: string;
  shift_button: KbmGamepadButton | null;
  button_bindings: KbmButtonBinding[];
  stick_bindings: KbmStickBinding[];
}

export async function kbmGetProfiles(): Promise<KbmProfile[]> {
  return invoke("kbm_get_profiles");
}

export async function kbmCreateProfile(name: string): Promise<KbmProfile> {
  return invoke("kbm_create_profile", { name });
}

export async function kbmUpdateProfile(profile: KbmProfile): Promise<void> {
  return invoke("kbm_update_profile", { profile });
}

export async function kbmDeleteProfile(profileId: string): Promise<void> {
  return invoke("kbm_delete_profile", { profileId });
}

export async function kbmActivateProfile(profileId: string): Promise<void> {
  return invoke("kbm_activate_profile", { profileId });
}

export async function kbmDeactivate(): Promise<void> {
  return invoke("kbm_deactivate");
}

export async function kbmGetActiveProfile(): Promise<KbmProfile | null> {
  return invoke("kbm_get_active_profile");
}

export async function kbmIsEnabled(): Promise<boolean> {
  return invoke("kbm_is_enabled");
}

export async function kbmGetAvailableGamepadButtons(): Promise<[KbmGamepadButton, string][]> {
  return invoke("kbm_get_available_gamepad_buttons");
}

export async function kbmGetCommonKeyNames(): Promise<string[]> {
  return invoke("kbm_get_common_key_names");
}

export async function kbmValidateKeysString(keys: string): Promise<number[]> {
  return invoke("kbm_validate_keys_string", { keys });
}

// ==================== RTSS 检测管理 ====================


/** 运行捆绑的 RTSS 安装程序 */
export async function rtssRunInstaller(): Promise<RtssResult> {
  return invoke("rtss_run_installer");
}

/** 检查是否存在捆绑的 RTSS 安装程序 */
export async function rtssHasBundledInstaller(): Promise<boolean> {
  return invoke("rtss_has_bundled_installer");
}

/** 下载并安装 RTSS（从官方地址下载） */
export async function rtssDownloadInstaller(): Promise<RtssResult> {
  return invoke("rtss_download_installer");
}

/** 手动导入 RTSS 路径 */
export async function rtssImportPath(exePath: string): Promise<RtssResult> {
  return invoke("rtss_import_path", { exePath });
}

