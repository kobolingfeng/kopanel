<script lang="ts">
  /**
   * 监听面板组件
   * 
   * 用于显示按键录制状态或长按清除进度
   */

  interface Props {
    /** 模式: recording=录制中, clearing=长按清除 */
    mode: 'recording' | 'clearing';
    /** 倒计时秒数 (recording模式) */
    countdown?: number;
    /** 总时长秒数 (recording模式，用于计算进度) */
    totalDuration?: number;
    /** 清除进度: 非X模式时为 0-1，X模式时为 0-100 (clearing模式) */
    clearProgress?: number;
    /** 已检测的按键 (recording模式) */
    detectedKeys?: string[];
    /** 检测到按键的文案 */
    detectedKeysLabel?: string;
    /** 长按清除文案 */
    clearLabel?: string;
    /** X键长按清除文案 */
    xClearLabel?: string;
    /** 是否使用X键进度（X模式时 clearProgress 为 0-100） */
    useXProgress?: boolean;
  }

  let {
    mode,
    countdown = 5,
    totalDuration = 5,
    clearProgress = 0,
    detectedKeys = [],
    detectedKeysLabel = 'Detected keys',
    clearLabel = 'Hold to clear',
    xClearLabel,
    useXProgress = false,
  }: Props = $props();

  // 计算录制进度百分比（防止除零）
  const recordingProgress = $derived(totalDuration > 0 ? (countdown / totalDuration) * 100 : 0);
  
  // 计算清除进度百分比
  const clearingProgress = $derived(clearProgress * 100);
  
  // 计算清除倒计时
  const clearCountdown = $derived(Math.ceil((1 - clearProgress) * 1.5));
  
  // X键清除倒计时（X模式下 clearProgress 范围是 0-100）
  const xClearCountdown = $derived(Math.ceil((1 - clearProgress / 100) * 2));
</script>

<div class="listening-panel">
  <div class="listening-header">
    {#if mode === 'recording'}
      <span>Recording...</span>
      <span class="countdown">{countdown}s</span>
    {:else}
      <span>{xClearLabel || clearLabel}</span>
      <span class="countdown">{useXProgress ? xClearCountdown : clearCountdown}s</span>
    {/if}
  </div>
  
  <div 
    class="listening-bar"
    style={mode === 'clearing' ? 'background: rgba(239, 68, 68, 0.2);' : ''}
  >
    <div 
      class="listening-fill"
      style="width: {mode === 'recording' ? recordingProgress : (useXProgress ? clearProgress : clearingProgress)}%; {mode === 'clearing' ? 'background: #ef4444;' : ''}"
    ></div>
  </div>
  
  {#if mode === 'recording' && detectedKeys.length > 0}
    <div class="detected-keys">
      {detectedKeysLabel}:
      <span class="keys">{detectedKeys.join(' + ')}</span>
    </div>
  {/if}
</div>

<style>
  .listening-panel {
    background: var(--panel-bg, rgba(59, 130, 246, 0.1));
    padding: 0.75rem;
    border-radius: 8px;
    margin-top: 0.5rem;
  }

  .listening-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.5rem;
    font-size: 0.85rem;
    color: var(--text-secondary, #94a3b8);
  }

  .countdown {
    font-weight: 600;
    color: var(--accent-color, #3b82f6);
  }

  .listening-bar {
    height: 4px;
    background: var(--bar-bg, rgba(59, 130, 246, 0.2));
    border-radius: 2px;
    overflow: hidden;
  }

  .listening-fill {
    height: 100%;
    background: var(--accent-color, #3b82f6);
    transition: width 1s linear;
  }

  .detected-keys {
    margin-top: 0.5rem;
    font-size: 0.8rem;
    color: var(--text-muted, #64748b);
  }

  .detected-keys .keys {
    font-weight: 600;
    color: var(--text-primary, #f8fafc);
    margin-left: 0.25rem;
  }
</style>
