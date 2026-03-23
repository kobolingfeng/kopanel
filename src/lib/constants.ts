import type { Tab, ColorTheme, OptimizationOption, Shortcut, MediaAction, TabItem } from './types';

// TDP 常量
export const DEFAULT_TDP_MIN = 1;
export const DEFAULT_TDP_MAX = 85;
export const TDP_ABSOLUTE_MIN = 1;
export const TDP_ABSOLUTE_MAX = 200;
export const DEFAULT_TDP_PRESETS: number[] = [8, 16, 24, 32, 45, 55, 65, 75];

// FPS 限制范围
export const FPS_LIMIT_ABSOLUTE_MIN = 1;
export const FPS_LIMIT_ABSOLUTE_MAX = 240;
export const DEFAULT_FPS_LIMIT_MIN = 30;
export const DEFAULT_FPS_LIMIT_MAX = 144;

// 风扇模式 (
export const FAN_MODES = ["fan_mode_quiet", "fan_mode_balanced", "fan_mode_performance", "fan_mode_turbo", "fan_mode_auto"];

// Tab 配置（完整列表，包含可选的修改器分页）
// label 字段现在存储 i18n key，需要通过 t() 函数获取翻译
export const TABS_ALL: Tab[] = [
  { id: "quick", label: "tab_quick" },
  { id: "guide", label: "tab_guide" },
  { id: "display", label: "tab_display" },
  { id: "performance", label: "tab_performance" },
  { id: "monitor", label: "tab_monitor" },
  { id: "library", label: "tab_library" },
  { id: "trainers", label: "tab_trainers" },
  { id: "gamepad", label: "tab_gamepad" },
  { id: "hotkey", label: "tab_hotkey" },
  { id: "settings", label: "tab_settings" },
];

// 配色主题 (name 使用 i18n key: theme_{id})
export const COLOR_THEMES: ColorTheme[] = [
  // 深色主题
  { id: 'steam', name: 'theme_steam', color: '#66c0f4', light: false },
  { id: 'purple', name: 'theme_purple', color: '#a78bfa', light: false },
  { id: 'cyber', name: 'theme_cyber', color: '#22c55e', light: false },
  { id: 'fire', name: 'theme_fire', color: '#f87171', light: false },
  { id: 'amber', name: 'theme_amber', color: '#fbbf24', light: false },
  { id: 'ice', name: 'theme_ice', color: '#38bdf8', light: false },
  { id: 'sakura', name: 'theme_sakura', color: '#f472b6', light: false },
  // 浅色主题
  { id: 'light', name: 'theme_light', color: '#3b82f6', light: true },
  { id: 'cream', name: 'theme_cream', color: '#d97706', light: true },
  { id: 'lavender', name: 'theme_lavender', color: '#9333ea', light: true },
  { id: 'mint', name: 'theme_mint', color: '#059669', light: true },
  { id: 'sky', name: 'theme_sky', color: '#0284c7', light: true },
  { id: 'rose', name: 'theme_rose', color: '#e11d48', light: true },

  // 机甲主题 (Mech Series)
  { id: 'mech_green', name: 'theme_mech_green', color: '#34d399', light: false, special: true, series: 'mech' },
  { id: 'mech_gold', name: 'theme_mech_gold', color: '#fbbf24', light: false, special: true, series: 'mech' },
  { id: 'mech_purple', name: 'theme_mech_purple', color: '#d946ef', light: false, special: true, series: 'mech' },
  { id: 'mech_blue', name: 'theme_mech_blue', color: '#38bdf8', light: false, special: true, series: 'mech' },
  { id: 'mech_white', name: 'theme_mech_white', color: '#ffffff', light: false, special: true, series: 'mech' },
  { id: 'mech_red', name: 'theme_mech_red', color: '#ff003c', light: false, special: true, series: 'mech' },

  // 故障主题 (Glitch Series)
  { id: 'glitch_red', name: 'theme_glitch_red', color: '#ff003c', light: false, special: true, series: 'glitch' },
  { id: 'glitch_cyan', name: 'theme_glitch_cyan', color: '#00ffff', light: false, special: true, series: 'glitch' },
  { id: 'glitch_purple', name: 'theme_glitch_purple', color: '#bf00ff', light: false, special: true, series: 'glitch' },
  { id: 'glitch_green', name: 'theme_glitch_green', color: '#00ff41', light: false, special: true, series: 'glitch' },

  // 二次元主题 (Anime Series)
  { id: 'anime_pink', name: 'theme_anime_pink', color: '#ff80ab', light: false, special: true, series: 'anime' },
  { id: 'anime_blue', name: 'theme_anime_blue', color: '#82b1ff', light: false, special: true, series: 'anime' },
  { id: 'anime_violet', name: 'theme_anime_violet', color: '#ea80fc', light: false, special: true, series: 'anime' },
  { id: 'anime_coral', name: 'theme_anime_coral', color: '#ff8a65', light: false, special: true, series: 'anime' },
];

// 可还原的优化选项 (label/desc 使用 i18n key: opt_{id}, opt_{id}_desc)
export const REVERSIBLE_OPTIMIZATIONS: OptimizationOption[] = [
  { id: "edge_background", label: "opt_edge_background", desc: "opt_edge_background_desc" },
  { id: "edge_update", label: "opt_edge_update", desc: "opt_edge_update_desc" },
  { id: "disable_search", label: "opt_disable_search", desc: "opt_disable_search_desc" },
  { id: "disable_connected_standby", label: "opt_disable_connected_standby", desc: "opt_disable_connected_standby_desc" },
  { id: "disable_sysmain", label: "opt_disable_sysmain", desc: "opt_disable_sysmain_desc" },
  { id: "disable_mpo", label: "opt_disable_mpo", desc: "opt_disable_mpo_desc" },
  { id: "disable_memory_integrity", label: "opt_disable_memory_integrity", desc: "opt_disable_memory_integrity_desc" },
];

// 不可还原的优化选项 (label/desc/warning 使用 i18n key)
export const IRREVERSIBLE_OPTIMIZATIONS: OptimizationOption[] = [
  { id: "import_ymelite", label: "opt_import_ymelite", desc: "opt_import_ymelite_desc", warning: "opt_import_ymelite_warning" },
  { id: "disable_hyperv", label: "opt_disable_hyperv", desc: "opt_disable_hyperv_desc", warning: "opt_disable_hyperv_warning" },
  { id: "pause_updates", label: "opt_pause_updates", desc: "opt_pause_updates_desc", warning: "opt_pause_updates_warning" },
  { id: "disable_reserved_storage", label: "opt_disable_reserved_storage", desc: "opt_disable_reserved_storage_desc", warning: "opt_disable_reserved_storage_warning" },
  { id: "amd_timer_optimization", label: "opt_amd_timer_optimization", desc: "opt_amd_timer_optimization_desc", warning: "opt_amd_timer_optimization_warning" },
  { id: "intel_fix", label: "opt_intel_fix", desc: "opt_intel_fix_desc", warning: "opt_intel_fix_warning" },
];

// 合并所有优化选项
export const ALL_OPTIMIZATIONS = [...REVERSIBLE_OPTIMIZATIONS, ...IRREVERSIBLE_OPTIMIZATIONS];

// 快捷键 (label 使用 i18n key: shortcut_{name})
export const SHORTCUTS: Shortcut[] = [
  { label: "shortcut_task_view", command: "lwin+tab", icon: "layers", color: "blue" },
  { label: "shortcut_show_desktop", command: "lwin+d", icon: "monitor", color: "cyan" },
  { label: "shortcut_switch_window", command: "alt+tab", icon: "repeat", color: "teal" },
  { label: "shortcut_osk", command: "osk", icon: "keyboard", color: "violet" },
  { label: "shortcut_steam_bigpicture", command: "steam_bigpicture", icon: "box", color: "sky" },
  { label: "shortcut_minimize_all", command: "lwin+m", icon: "minimize", color: "gray" },
  { label: "shortcut_close_window", command: "alt+f4", icon: "x", color: "red" },
  { label: "shortcut_settings", command: "lwin+i", icon: "settings", color: "slate" },
  { label: "shortcut_cast", command: "lwin+p", icon: "cast", color: "indigo" },
  { label: "shortcut_file_manager", command: "lwin+e", icon: "folder", color: "amber" },
  { label: "shortcut_game_bar", command: "lwin+g", icon: "gamepad", color: "green" },
  { label: "shortcut_task_manager", command: "ctrl+shift+esc", icon: "activity", color: "rose" },
  { label: "shortcut_hibernate", command: "system_hibernate", icon: "moon", color: "purple" },
  { label: "shortcut_sleep", command: "system_sleep", icon: "bed", color: "orange" },
  { label: "shortcut_shutdown", command: "system_shutdown", icon: "power", color: "red" },
];

// 媒体控制 (label 使用 i18n key: media_{action})
export const MEDIA_ACTIONS: MediaAction[] = [
  { label: "media_mute_toggle", command: "volume_mute" },
  { label: "media_play_pause", command: "media_play_pause" },
  { label: "media_prev_track", command: "media_prev_track" },
  { label: "media_next_track", command: "media_next_track" },
];

// SVG 图标路径
export const ICONS: Record<string, string> = {
  layers: '<polygon points="12 2 2 7 12 12 22 7 12 2"/><polyline points="2 17 12 22 22 17"/><polyline points="2 12 12 17 22 12"/>',
  monitor: '<rect x="2" y="3" width="20" height="14" rx="2" ry="2"/><line x1="8" y1="21" x2="16" y2="21"/><line x1="12" y1="17" x2="12" y2="21"/>',
  settings: '<circle cx="12" cy="12" r="3"/><path d="M19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z"/>',
  cast: '<path d="M2 16.1A5 5 0 0 1 5.9 20M2 12.05A9 9 0 0 1 9.95 20M2 8V6a2 2 0 0 1 2-2h16a2 2 0 0 1 2 2v12a2 2 0 0 1-2 2h-6"/><line x1="2" y1="20" x2="2.01" y2="20"/>',
  folder: '<path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/>',
  gamepad: '<line x1="6" y1="12" x2="10" y2="12"/><line x1="8" y1="10" x2="8" y2="14"/><line x1="15" y1="13" x2="15.01" y2="13"/><line x1="18" y1="11" x2="18.01" y2="11"/><rect x="2" y="6" width="20" height="12" rx="2"/>',
  activity: '<polyline points="22 12 18 12 15 21 9 3 6 12 2 12"/>',
  moon: '<path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/>',
  bed: '<path d="M2 4v16"/><path d="M2 8h18a2 2 0 0 1 2 2v10"/><path d="M2 12h18"/><path d="M6 8v9"/>',
  power: '<path d="M18.36 6.64a9 9 0 1 1-12.73 0"/><line x1="12" y1="2" x2="12" y2="12"/>',
  keyboard: '<rect width="20" height="16" x="2" y="4" rx="2" ry="2"/><path d="M6 8h.01"/><path d="M10 8h.01"/><path d="M14 8h.01"/><path d="M18 8h.01"/><path d="M6 12h.01"/><path d="M10 12h.01"/><path d="M14 12h.01"/><path d="M18 12h.01"/><path d="M7 16h10"/>',
  cpu: '<rect x="4" y="4" width="16" height="16" rx="2" ry="2"/><rect x="9" y="9" width="6" height="6"/><line x1="9" y1="1" x2="9" y2="4"/><line x1="15" y1="1" x2="15" y2="4"/><line x1="9" y1="20" x2="9" y2="23"/><line x1="15" y1="20" x2="15" y2="23"/><line x1="20" y1="9" x2="23" y2="9"/><line x1="20" y1="15" x2="23" y2="15"/><line x1="1" y1="9" x2="4" y2="9"/><line x1="1" y1="15" x2="4" y2="15"/>',
  zap: '<polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>',
  battery: '<rect x="1" y="6" width="18" height="12" rx="2" ry="2"/><line x1="23" y1="10" x2="23" y2="14"/>',
  tablet: '<rect x="4" y="2" width="16" height="20" rx="2" ry="2"/><line x1="12" y1="18" x2="12.01" y2="18"/>',
  repeat: '<path d="m17 2 4 4-4 4"/><path d="M3 11v-1a4 4 0 0 1 4-4h14"/><path d="m7 22-4-4 4-4"/><path d="M21 13v1a4 4 0 0 1-4 4H3"/>',
  box: '<path d="M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16z"/><polyline points="3.27 6.96 12 12.01 20.73 6.96"/><line x1="12" y1="22.08" x2="12" y2="12"/>',
  minimize: '<path d="M12 17V3"/><path d="m6 11 6 6 6-6"/><path d="M19 21H5"/>',
  x: '<path d="M18 6 6 18"/><path d="m6 6 12 12"/>',
};


// Tab 项目配置（静态）
export const TAB_ITEMS_STATIC: Record<string, TabItem[]> = {
  guide: [
    { id: 'guide_hotkey', type: 'special' },
    { id: 'guide_game_mode', type: 'special' },
    { id: 'guide_isolation', type: 'special' },
    { id: 'guide_osd', type: 'special' },
    { id: 'guide_tdp', type: 'special' },
    { id: 'guide_anticheat', type: 'special' },
    { id: 'guide_emulator', type: 'special' },
  ],
  performance: [
    { id: 'perf_monitor', type: 'action' },
    { id: 'tdp_slider', type: 'slider' },
    { id: 'tdp_preset', type: 'stepper' },
    { id: 'fan_mode', type: 'stepper' },
    { id: 'turbo_boost', type: 'toggle' },
    { id: 'auto_tdp_toggle', type: 'toggle' },
    { id: 'core_mode', type: 'stepper' },
    { id: 'active_cores', type: 'slider' },
    // TDP 自定义折叠栏
    { id: 'tdp_custom_toggle', type: 'toggle' },
    { id: 'tdp_min', type: 'slider' },
    { id: 'tdp_max', type: 'slider' },
    { id: 'new_preset', type: 'action' },
    { id: 'preset_select', type: 'action' },
    { id: 'reset_presets', type: 'action' },
    // 风扇曲线自定义折叠栏
    { id: 'fan_custom_toggle', type: 'toggle' },
    { id: 'fan_curve_mode', type: 'stepper' },
    { id: 'fan_curve_0', type: 'slider' },
    { id: 'fan_curve_1', type: 'slider' },
    { id: 'fan_curve_2', type: 'slider' },
    { id: 'fan_curve_3', type: 'slider' },
    { id: 'fan_curve_4', type: 'slider' },
    { id: 'fan_curve_5', type: 'slider' },
    { id: 'fan_curve_save', type: 'action' },
    { id: 'fan_curve_reset', type: 'action' },
  ],
  monitor: [
    // 监控页项目在 getTabItems 中动态生成
  ],
  display: [
    { id: 'brightness', type: 'slider' },   // 索引0
    { id: 'volume', type: 'slider' },       // 索引1
    { id: 'media_mic', type: 'toggle' },    // 索引2
    { id: 'media_mute', type: 'action' },   // 索引3
    { id: 'media_play', type: 'action' },   // 索引4
    { id: 'media_prev', type: 'action' },   // 索引5
    { id: 'media_next', type: 'action' },   // 索引6
    { id: 'hdr', type: 'toggle' },          // 索引7
    { id: 'dynamic_refresh_rate', type: 'toggle' }, // 索引8 - 动态刷新率
    { id: 'adaptive_brightness', type: 'toggle' },  // 索引9 - 自适应亮度
    { id: 'refresh_rate', type: 'stepper' },// 索引10
    { id: 'resolution', type: 'stepper' },  // 索引11
    { id: 'kill_top_process', type: 'action' }, // 索引12 - 一键关闭
    { id: 'suspend_top_process', type: 'action' }, // 索引13 - 一键冻结
  ],
  audio: [
    { id: 'volume', type: 'slider' },
    { id: 'mic_mute', type: 'toggle' },
  ],
  gamepad: [
    // 手柄震动（动态手柄滑条在 vibration 之后由前端动态生成）
    { id: 'vibration', type: 'slider' },
    { id: 'disable_builtin', type: 'toggle' },
    // 全局扳机死区
    { id: 'trigger_deadzone_lt', type: 'slider' },
    { id: 'trigger_deadzone_rt', type: 'slider' },
    { id: 'test_vibration', type: 'action' },
    { id: 'gamepad_test', type: 'special' },
    // 鼠标模拟
    { id: 'mouse_sim', type: 'toggle' },
    { id: 'desktop_mode', type: 'action' },    // 桌面模式（回到桌面+模拟鼠标）
    { id: 'bigscreen_mode', type: 'action' },  // 大屏幕模式
    // 输入隔离（XInput Hook）
    { id: 'xinput_isolation_toggle', type: 'toggle' },
    // 兆容性输入隔离（HidHide）
    { id: 'input_isolation_toggle', type: 'toggle' },
    // 非 Xbox 手柄检测 (gilrs)
    { id: 'gilrs_detection_toggle', type: 'toggle' },
    // 按键映射（展开/折叠）
    { id: 'button_mapping_toggle', type: 'toggle' },
    // 陀螺仪（只保留开关，其他选项在 getTabItems 中根据状态动态添加）
    { id: 'gyro_enable', type: 'toggle' },
    // 屏幕键盘类型选择
    { id: 'osk_type_toggle', type: 'toggle' },
    // 系统快捷键开关
    { id: 'shortcut_toggle', type: 'toggle' },
  ],
  settings: [
    { id: 'bigscreen_mode', type: 'toggle' },
    { id: 'language_select', type: 'stepper' },
    { id: 'theme_toggle', type: 'toggle' },
    { id: 'theme_select', type: 'stepper' },
    { id: 'panel_width', type: 'slider' },
    { id: 'font_size', type: 'slider' },
    { id: 'gesture_toggle', type: 'toggle' },
    { id: 'sound_toggle', type: 'toggle' },
    // 窗口行为折叠组
    { id: 'window_behavior_toggle', type: 'toggle' },
    { id: 'no_focus_mode', type: 'toggle' },
    { id: 'blur_hide_toggle', type: 'toggle' },
    { id: 'focus_return_toggle', type: 'toggle' },
    { id: 'fullscreen_mode', type: 'toggle' },
    { id: 'safe_mode', type: 'toggle' },
    { id: 'suspend_on_sleep', type: 'toggle' },
    { id: 'autostart', type: 'toggle' },
    { id: 'start_minimized', type: 'toggle' },
    { id: 'screen_rotation', type: 'toggle' },
    { id: 'tablet_mode', type: 'toggle' },
    { id: 'optical_mouse', type: 'toggle' },
    { id: 'touchscreen_toggle', type: 'toggle' },
    // 连接设置折叠栏
    { id: 'connection_toggle', type: 'toggle' },
    { id: 'wifi_toggle', type: 'toggle' },
    { id: 'bluetooth_toggle', type: 'toggle' },
    { id: 'hotspot_toggle', type: 'toggle' },
    { id: 'hotspot_config', type: 'action' },
    // 布局设置折叠栏
    { id: 'layout_settings_toggle', type: 'toggle' },
    // 布局控制（布局设置展开后显示）
    { id: 'hide_tab_bar_toggle', type: 'toggle' },
    { id: 'hide_status_bar_toggle', type: 'toggle' },
    // 分页可见性折叠栏
    { id: 'tab_visibility_toggle', type: 'toggle' },
    // 各分页可见性开关（需展开后显示）
    { id: 'guide_tab_toggle', type: 'toggle' },
    { id: 'display_tab_toggle', type: 'toggle' },
    { id: 'performance_tab_toggle', type: 'toggle' },
    { id: 'monitor_tab_toggle', type: 'toggle' },
    { id: 'library_tab_toggle', type: 'toggle' },
    { id: 'trainer_tab_toggle', type: 'toggle' },
    { id: 'gamepad_tab_toggle', type: 'toggle' },
    { id: 'hotkey_tab_toggle', type: 'toggle' },
    { id: 'quick_tab_toggle', type: 'toggle' },
    // 重置配置
    { id: 'reset_settings', type: 'action' },
    // RTSS 状态展示
    { id: 'rtss_status', type: 'display' },
    // 系统优化相关（必须放最后，优化选项会紧跟其后）
    { id: 'optimize_toggle', type: 'toggle' },
  ],
  trainers: [
    { id: 'update_database', type: 'action' },
    { id: 'search_input', type: 'special' },
  ],
  hotkey: [
    { id: 'summon_binding', type: 'action' },
    { id: 'summon_binding_2', type: 'action' },
    { id: 'osd_toggle_hotkey', type: 'action' },
    { id: 'tdp_up_hotkey', type: 'action' },
    { id: 'tdp_down_hotkey', type: 'action' },
    { id: 'show_desktop_hotkey', type: 'action' },
    { id: 'alt_tab_hotkey', type: 'action' },
    { id: 'desktop_mode_hotkey', type: 'action' },
    { id: 'bigscreen_mode_hotkey', type: 'action' },
    { id: 'steam_bigpicture_hotkey', type: 'action' },
    { id: 'hotkey_hibernate', type: 'action' },
    { id: 'hotkey_sleep', type: 'action' },
    { id: 'hotkey_shutdown', type: 'action' },
    { id: 'hotkey_task_manager', type: 'action' },
    { id: 'mouse_sim_toggle_hotkey', type: 'action' },
    { id: 'gyro_toggle_hotkey', type: 'action' },
    { id: 'gyro_hold_hotkey', type: 'action' },
    { id: 'gyro_hold_2_hotkey', type: 'action' },
    { id: 'gyro_hold_3_hotkey', type: 'action' },
    { id: 'screenshot_hotkey', type: 'action' },
    { id: 'record_toggle_hotkey', type: 'action' },
  ],
};

// 掌机特殊按键（位掩码 -> 标签）
// 注意：使用 u64，JavaScript 可以安全处理到 2^53-1
export const CUSTOM_KEY_LABELS: Record<number, string> = {
  // Legion Go S
  1: "LgnL",      // Legion Go S 左背键
  2: "LgnR",      // Legion Go S 右背键
  4: "Y1",        // Legion Go S Y1
  8: "Y2",        // Legion Go S Y2
  16: "Y3",       // Legion Go S Y3
  // ROG Ally
  32: "M1",       // ROG Ally M1
  64: "M2",       // ROG Ally M2
  128: "M3",      // ROG Ally M3
  // MSI Claw
  256: "ClawL",   // MSI Claw 左背键
  512: "ClawR",   // MSI Claw 右背键
  // ROG Ally (续)
  1024: "AllyL",  // ROG Ally 左背键
  2048: "AllyR",  // ROG Ally 右背键
  4096: "AllyRL", // ROG Ally 双背键
  // GPD WIN
  8192: "L4",     // GPD WIN L4
  16384: "R4",    // GPD WIN R4
  32768: "L5",    // GPD WIN5 L5
  65536: "R5",    // GPD WIN5 R5
  // ROG Ally 扩展
  131072: "M4",           // M4 / ROG key
  262144: "CommandCenter", // Command Center
  // OneXPlayer / AOKZOE
  524288: "Turbo",    // OXP Turbo
  1048576: "Home",    // OXP Home
  2097152: "K1",      // OXP K1
  4194304: "K2",      // OXP K2
  8388608: "M1背",    // OXP M1 背键
  16777216: "M2背",   // OXP M2 背键
  33554432: "M3",     // OXP M3
  67108864: "M4",     // OXP M4
  134217728: "M5",    // OXP M5
  268435456: "M6",    // OXP M6
  536870912: "LT",    // OXP LT 分离
  1073741824: "RT",   // OXP RT 分离
  // AYANEO (u64 高位 bit 32+)
  4294967296: "AYA_LC",   // AYANEO 左背键 (0x1_0000_0000)
  8589934592: "AYA_RC",   // AYANEO 右背键 (0x2_0000_0000)
  17179869184: "AYA_LC肩", // AYANEO 左自定义肩键 (0x4_0000_0000)
  34359738368: "AYA_RC肩", // AYANEO 右自定义肩键 (0x8_0000_0000)
  68719476736: "AYA_Key", // AYANEO 专用键 (0x10_0000_0000)
  // GPD Win5 V1.09+ HID
  137438953472: "GPD",    // GPD 模式键/菜单键 (0x20_0000_0000)
};

// 将位掩码转换为可读标签
// 注意：现在使用 u64，所有值都在 JavaScript 安全整数范围内 (2^53-1)
// 但位运算只支持 32 位，所以对于高位需要使用数学运算
export function customKeysToLabel(keys: number): string {
  if (!keys) return "";
  const labels: string[] = [];
  
  for (const [bitStr, label] of Object.entries(CUSTOM_KEY_LABELS)) {
    const bit = Number(bitStr);
    
    // 对于超过 32 位的值，使用数学运算而非位运算
    if (bit > 0x7FFFFFFF) {
      // 检查高位：使用除法和取模来检测
      if (keys >= bit && Math.floor(keys / bit) % 2 === 1) {
        labels.push(label);
      }
    } else {
      // 32 位内也使用数学方式，保持一致性
      if (keys >= bit && Math.floor(keys / bit) % 2 === 1) {
        labels.push(label);
      }
    }
  }
  return labels.join(" + ");
}
