export interface Game {
  id: string;
  name: string;
  path?: string;
  steamAppId?: string;
  source?: string;
  cover?: string;
  lastPlayedAt?: number;
  installLocation?: string;
  profileId?: string;  // 关联的性能场景 ID（旧版兼容）
  acProfileId?: string;  // 插电时使用的场景 ID
  batteryProfileId?: string;  // 电池时使用的场景 ID
  lsEnabled?: boolean;  // 启动游戏时自动启动 Lossless Scaling
  pinned?: boolean;      // 置顶到大屏幕快捷启动
}

// 性能场景配置
export interface PerformanceProfile {
  id: string;
  name: string;
  tdp?: number;
  tdp_boost?: number;
  tdp_fppt?: number;
  fan_mode?: number;
  turbo_boost?: boolean;
  epp?: number;
  auto_tdp_enabled?: boolean;
  auto_tdp_target_fps?: number;
  auto_fan_enabled?: boolean;
  auto_fan_temp_min?: number;
  auto_fan_temp_max?: number;
  fps_limit_enabled?: boolean;
  fps_limit_value?: number;
  active_cores?: number;      // 活跃核心数 (0=不限制)
  cpu_max_freq?: number;      // CPU 最大频率限制 (MHz, 0=不限制)
}

// 呼出键绑定
export interface SummonBinding {
  type: 'keyboard' | 'gamepadButton' | 'none';
  accelerator?: string;
  buttons?: number[];
  custom_keys?: number;  // 掌机特殊按键（位掩码）
  long_press?: boolean;  // 长按模式：需要按住 2 秒才触发
}

// 媒体信息
export interface MediaInfo {
  hasMedia: boolean;
  title?: string;
  artist?: string;
  album?: string;
  status?: string;
  error?: string;
}

// 进程
export interface WindowProcess {
  pid: number;
  name: string;
  exe_name: string;
  is_suspended: boolean;
}

// 电源计划
export interface PowerPlan {
  guid: string;
  name: string;
  is_active: boolean;
}

export interface PowerPlansResult {
  plans: PowerPlan[];
  active_guid: string | null;
}

// 显示
export interface DisplayInfo {
  width: number;
  height: number;
  refresh_rate: number;
  scaling: number;
}

export interface DisplayMode {
  width: number;
  height: number;
}

export interface HdrStatus {
  supported: boolean;
  enabled: boolean;
  error?: string | null;
}

// 电池
export interface BatteryStatus {
  has_battery: boolean;
  is_charging: boolean;
  is_ac_connected: boolean;
  battery_percent: number;
  battery_life_time: number | null;
  power_status: string;
}

export interface BatteryDetails {
  design_capacity: number | null;
  full_charge_capacity: number | null;
  current_capacity: number | null;
  voltage: number | null;
  charge_rate: number | null;
  health_percent: number | null;
  cycle_count: number | null;
  temperature: number | null;
  manufacturer: string | null;
  chemistry: string | null;
  serial_number: string | null;
}

// ADLX
export interface AdlxStatus {
  available: boolean;
  message: string;
}

export interface RsrSettings {
  supported: boolean;
  enabled: boolean;
  sharpness: number;
  support_status: number; // 诊断: 0=支持, 1=不支持, -1=初始化失败, -2=获取3D服务失败, -3=获取RSR接口失败
}

export interface DisplaySettings {
  integer_scaling_supported: boolean;
  integer_scaling_enabled: boolean;
  gpu_scaling_supported: boolean;
  gpu_scaling_enabled: boolean;
  scaling_mode: number;
  freesync_supported: boolean;
  freesync_enabled: boolean;
}

export interface AdlxResult {
  success: boolean;
  message?: string;
  error?: string;
}

// 手柄状态
export interface BackendGamepadState {
  connected: boolean;
  buttons: boolean[];
  axes: number[];
  left_trigger: number;
  right_trigger: number;
}

// Tab 配置
export type ItemType = 'slider' | 'stepper' | 'toggle' | 'action' | 'special' | 'display';

export interface TabItem {
  id: string;
  type: ItemType;
}

// 快捷分页项目（引用其他分页的控件）
export interface QuickItem {
  tab_id: string;      // 原始分页 ID
  item_id: string;     // 原始项目 ID
  item_type: string;   // 项目类型
  label: string;       // 显示标签
}

export interface Tab {
  id: string;
  label: string;
}

// 颜色主题
export interface ColorTheme {
  id: string;
  name: string;
  color: string;
  light: boolean;
  special?: boolean;
  series?: string;
}

// 优化选项
export interface OptimizationOption {
  id: string;
  label: string;
  desc: string;
  warning?: string;
}

// 快捷键
export interface Shortcut {
  label: string;
  command: string;
  icon: string;
  color: string;
}

// 媒体操作
export interface MediaAction {
  label: string;
  command: string;
}

// RTSS 检测结果
export type RtssDetectionMethod = 'PersistedPath' | 'Registry' | 'RunningProcess' | 'NotFound';

export interface RtssDetectionResult {
  path: string | null;
  running: boolean;
  detection_method: RtssDetectionMethod;
  need_user_action: boolean;
  error: string | null;
}

// RTSS 相关类型
export interface RtssStatus {
  installed: boolean;
  running: boolean;
  install_path: string | null;
  version: string | null;
}

export interface RtssResult {
  success: boolean;
  message: string;
}

/** Lossless Scaling FPS 数据 */
export interface LsFpsData {
  /** 游戏原始 FPS */
  game_fps: number;
  /** 游戏帧时间 (ms) */
  game_frametime: number;
  /** Lossless Scaling 输出 FPS（插帧后） */
  ls_fps: number;
  /** Lossless Scaling 是否正在运行 */
  ls_running: boolean;
}

// 系统监控
export interface SystemMetrics {
  cpu_usage: number;
  cpu_temp: number;
  cpu_power: number;
  /** SoC/APU 总功耗（若可用） */
  soc_power?: number;
  ram_used_gb: number;
  ram_total_gb: number;
  ram_usage: number;
  fan_speed?: number;

  // GPU (LHM)
  gpu_usage?: number;
  gpu_temp?: number;
  gpu_power?: number;
  vram_used_mb?: number;
  vram_total_mb?: number;

  // 电池 (LHM)
  battery_power?: number;  // 电池功耗 W (放电为正,充电为负)
}

// ==================== Toast 通知系统 ====================

export type ToastType = 'success' | 'error' | 'warning' | 'info';

export type SoundType = 'click' | 'success' | 'error' | 'warning' | 'navigate';

export interface Toast {
  id: number;
  message: string;
  type: ToastType;
  duration: number;
}

// ==================== 陀螺仪 ====================

export type GyroType = 'None' | 'BuiltIn' | 'DualSense' | 'DualShock4' | 'UsbImu' | 'XInputGamepad';

export type GyroMappingMode = 'Disabled' | 'Mouse' | 'RightStick';

export interface GyroData {
  accel: [number, number, number];  // 加速度 [x, y, z] (m/s²)
  gyro: [number, number, number];   // 角速度 [x, y, z] (deg/s)
  timestamp: number;                // 时间戳 (毫秒)
}

export interface GyroConfig {
  enabled: boolean;                 // 是否启用
  device_type: GyroType;            // 设备类型
  sensitivity: number;              // 灵敏度 (0.1 - 10.0)
  deadzone: number;                 // 死区 (0.0 - 1.0)
  mapping_mode: GyroMappingMode;    // 映射模式
  invert_x: boolean;                // X轴反转
  invert_y: boolean;                // Y轴反转
  use_filtering: boolean;           // 启用滤波
  x_sensitivity?: number;           // X轴独立灵敏度
  y_sensitivity?: number;           // Y轴独立灵敏度
  auto_calibration?: boolean;       // 自动漂移补偿
  stick_only_mode?: boolean;        // 仅在按住扳机时生效
  trigger_threshold?: number;       // 扳机激活阈值 (0.0-1.0)
}

export interface GyroDeviceInfo {
  device_type: GyroType;
  name: string;
  available: boolean;
  description?: string;
}

// ==================== 自动更新 ====================

export interface UpdateInfo {
  version: string;
  notes: string;
  pub_date: string;
  download_url: string;
  portable_url?: string;
  file_size: number;
  sha256?: string;
}

export interface UpdateCheckResult {
  has_update: boolean;
  current_version: string;
  latest_version?: string;
  update_info?: UpdateInfo;
  error?: string;
}

export interface DownloadProgress {
  downloaded: number;
  total: number;
  percent: number;
}

// ==================== 自定义 OSD ====================

// 自定义 OSD 可选项目
export type OsdItemKey = 
  | 'fps'           // 帧率
  | 'frametime'     // 帧时间
  | 'cpu_temp'      // CPU温度
  | 'gpu_temp'      // GPU温度
  | 'cpu_power'     // CPU功耗/TDP
  | 'gpu_power'     // GPU功耗
  | 'battery'       // 电池电量
  | 'battery_power' // 电池功耗
  | 'remaining'     // 剩余时间
  | 'fan'           // 风扇转速
  | 'ram'           // 内存使用
  | 'vram'          // 显存使用
  | 'time';         // 时间

