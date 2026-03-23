/**
 * Toast 通知系统 Store
 * 
 * 提供全局 Toast 通知管理功能
 * 使用 Svelte 5 runes 进行响应式状态管理
 */

import type { Toast, ToastType } from './types';
import { playSound } from './sound';

// Toast 状态
let toasts = $state<Toast[]>([]);
let toastIdCounter = 0;
let soundEnabled = $state(true);

// 最大同时显示的 Toast 数量（防止内存积累）
const MAX_TOASTS = 5;

// 安全的 ID 计数器上限（避免极端情况下的溢出）
const MAX_TOAST_ID = Number.MAX_SAFE_INTEGER - 1000;

// 跟踪每个 Toast 的自动消失定时器，便于提前 dismiss 或驱逐时清除
const toastTimers = new Map<number, ReturnType<typeof setTimeout>>();

/**
 * 获取当前所有 Toast（响应式）
 */
export function getToasts(): Toast[] {
  return toasts;
}

/**
 * 设置音效开关状态
 */
export function setSoundEnabled(enabled: boolean): void {
  soundEnabled = enabled;
}

/**
 * 显示 Toast 通知
 * @param message 消息内容
 * @param type Toast 类型
 * @param duration 显示时长（毫秒）
 */
export function showToast(
  message: string,
  type: ToastType = 'info',
  duration: number = 3000,
): void {
  // 安全递增 ID，在极端情况下重置（防止溢出）
  toastIdCounter = toastIdCounter >= MAX_TOAST_ID ? 1 : toastIdCounter + 1;
  const id = toastIdCounter;
  
  // 如果超过最大数量，移除最旧的 Toast（并清除其定时器）
  if (toasts.length >= MAX_TOASTS) {
    const evicted = toasts[0];
    if (evicted) {
      const timer = toastTimers.get(evicted.id);
      if (timer !== undefined) { clearTimeout(timer); toastTimers.delete(evicted.id); }
    }
    toasts = [...toasts.slice(1), { id, message, type, duration }];
  } else {
    toasts = [...toasts, { id, message, type, duration }];
  }
  
  // 自动消失（保存定时器 ID 便于提前清理）
  toastTimers.set(id, setTimeout(() => {
    toastTimers.delete(id);
    dismissToast(id);
  }, duration));
  
  // 播放对应音效
  if (type === 'success') playSound('success', soundEnabled);
  else if (type === 'error') playSound('error', soundEnabled);
  else if (type === 'warning') playSound('warning', soundEnabled);
}

/**
 * 关闭指定 Toast
 * @param id Toast ID
 */
export function dismissToast(id: number): void {
  const timer = toastTimers.get(id);
  if (timer !== undefined) { clearTimeout(timer); toastTimers.delete(id); }
  toasts = toasts.filter((t) => t.id !== id);
}

