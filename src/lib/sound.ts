/**
 * 音效系统模块
 * 
 * 提供简单的音效播放功能，用于 UI 反馈
 */

import type { SoundType } from './types';

// 音效上下文（单例）
let audioContext: AudioContext | null = null;

/**
 * 初始化音频上下文
 * 处理浏览器自动暂停策略：确保 AudioContext 处于 running 状态
 */
function initAudioContext(): AudioContext {
  if (!audioContext) {
    audioContext = new AudioContext();
  }
  
  // 处理 AudioContext suspended 状态（浏览器自动暂停策略）
  if (audioContext.state === 'suspended') {
    audioContext.resume().catch(() => {
      // 静默失败，可能需要用户交互才能恢复
    });
  }
  
  return audioContext;
}

/**
 * 播放音效
 * @param type 音效类型
 * @param enabled 是否启用音效（传入当前的音效开关状态）
 */
export function playSound(type: SoundType, enabled: boolean = true): void {
  if (!enabled) return;

  try {
    const ctx = initAudioContext();
    const oscillator = ctx.createOscillator();
    const gainNode = ctx.createGain();

    oscillator.connect(gainNode);
    gainNode.connect(ctx.destination);

    // 根据类型设置不同音效
    switch (type) {
      case "click":
        oscillator.frequency.setValueAtTime(600, ctx.currentTime);
        gainNode.gain.setValueAtTime(0.1, ctx.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(
          0.001,
          ctx.currentTime + 0.05,
        );
        oscillator.start(ctx.currentTime);
        oscillator.stop(ctx.currentTime + 0.05);
        break;
      case "navigate":
        oscillator.frequency.setValueAtTime(400, ctx.currentTime);
        gainNode.gain.setValueAtTime(0.08, ctx.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(
          0.001,
          ctx.currentTime + 0.03,
        );
        oscillator.start(ctx.currentTime);
        oscillator.stop(ctx.currentTime + 0.03);
        break;
      case "success":
        oscillator.type = "sine";
        oscillator.frequency.setValueAtTime(523, ctx.currentTime);
        oscillator.frequency.setValueAtTime(659, ctx.currentTime + 0.1);
        oscillator.frequency.setValueAtTime(784, ctx.currentTime + 0.2);
        gainNode.gain.setValueAtTime(0.15, ctx.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(
          0.001,
          ctx.currentTime + 0.35,
        );
        oscillator.start(ctx.currentTime);
        oscillator.stop(ctx.currentTime + 0.35);
        break;
      case "error":
        oscillator.type = "sawtooth";
        oscillator.frequency.setValueAtTime(200, ctx.currentTime);
        oscillator.frequency.setValueAtTime(150, ctx.currentTime + 0.1);
        gainNode.gain.setValueAtTime(0.12, ctx.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(
          0.001,
          ctx.currentTime + 0.2,
        );
        oscillator.start(ctx.currentTime);
        oscillator.stop(ctx.currentTime + 0.2);
        break;
      case "warning":
        oscillator.type = "triangle";
        oscillator.frequency.setValueAtTime(440, ctx.currentTime);
        oscillator.frequency.setValueAtTime(350, ctx.currentTime + 0.15);
        gainNode.gain.setValueAtTime(0.12, ctx.currentTime);
        gainNode.gain.exponentialRampToValueAtTime(
          0.001,
          ctx.currentTime + 0.25,
        );
        oscillator.start(ctx.currentTime);
        oscillator.stop(ctx.currentTime + 0.25);
        break;
    }
  } catch (e) {
    // 静默失败
  }
}

