<script lang="ts">
  /**
   * 滑块组件
   * 
   * 通用的滑块组件，支持：
   * - 拖拽调整值
   * - 点击定位
   * - 键盘控制（左右箭头）
   * - 自定义范围
   */

  interface Props {
    /** 当前值（0-100 百分比或实际值） */
    value: number;
    /** 最小值（默认 0） */
    min?: number;
    /** 最大值（默认 100） */
    max?: number;
    /** 步进值（默认 1） */
    step?: number;
    /** 是否禁用 */
    disabled?: boolean;
    /** 是否使用百分比模式（默认 true） */
    percentMode?: boolean;
    /** 是否显示左右标签 */
    showLabels?: boolean;
    /** 左侧标签 */
    labelLeft?: string;
    /** 右侧标签 */
    labelRight?: string;
    /** 值变化时的回调（拖拽中持续触发） */
    oninput?: (value: number) => void;
    /** 值变化完成时的回调（拖拽结束触发） */
    onchange?: (value: number) => void;
  }

  let {
    value,
    min = 0,
    max = 100,
    step = 1,
    disabled = false,
    percentMode = true,
    showLabels = false,
    labelLeft = '',
    labelRight = '',
    oninput,
    onchange,
  }: Props = $props();

  let isDragging = $state(false);
  let lastDragValue = $state(0); // 跟踪拖拽过程中的最新值（在 pointerdown 时从 props.value 初始化）
  let trackElement: HTMLElement | null = $state(null); // 缓存 track 元素引用

  // 计算百分比（$derived 确保 value/min/max 变化时自动更新 UI）
  let percent = $derived.by(() => {
    if (percentMode) {
      return Math.max(0, Math.min(100, value));
    }
    const range = max - min;
    if (range === 0) return 0; // 防止除零
    return Math.max(0, Math.min(100, ((value - min) / range) * 100));
  });

  // 从百分比转换为实际值（clamp 防止 step 舍入超出范围）
  function percentToValue(pct: number): number {
    if (percentMode) {
      return Math.max(0, Math.min(100, Math.round(pct / step) * step));
    }
    const range = max - min;
    const rawValue = min + (pct / 100) * range;
    return Math.max(min, Math.min(max, Math.round(rawValue / step) * step));
  }

  // 处理指针按下
  function handlePointerDown(e: PointerEvent) {
    if (disabled) return;
    
    const target = e.currentTarget as HTMLElement;
    if (!target) return;
    
    isDragging = true;
    lastDragValue = value; // 初始化拖拽值
    target.setPointerCapture(e.pointerId);
    
    updateValueFromEvent(e);
  }

  // 处理指针移动
  function handlePointerMove(e: PointerEvent) {
    if (!isDragging || disabled) return;
    updateValueFromEvent(e);
  }

  // 处理指针释放
  function handlePointerUp(e: PointerEvent) {
    if (!isDragging) return;
    
    isDragging = false;
    const target = e.currentTarget as HTMLElement;
    if (target) {
      target.releasePointerCapture(e.pointerId);
    }
    
    // 触发 onchange，使用拖拽过程中的最新值
    onchange?.(lastDragValue);
  }

  // 从事件更新值
  function updateValueFromEvent(e: PointerEvent) {
    // 优先使用缓存的 track 元素，避免每次都查询 DOM
    const track = trackElement || (e.currentTarget as HTMLElement);
    if (!track) return;
    
    const rect = track.getBoundingClientRect();
    const pct = Math.max(0, Math.min(100, ((e.clientX - rect.left) / rect.width) * 100));
    const newValue = percentToValue(pct);
    
    if (newValue !== lastDragValue) {
      lastDragValue = newValue;
      oninput?.(newValue);
    }
  }

  // 键盘控制
  function handleKeyDown(e: KeyboardEvent) {
    if (disabled) return;
    
    let newValue = value;
    
    if (e.key === 'ArrowLeft' || e.key === 'ArrowDown') {
      e.preventDefault();
      newValue = Math.max(percentMode ? 0 : min, value - step);
    } else if (e.key === 'ArrowRight' || e.key === 'ArrowUp') {
      e.preventDefault();
      newValue = Math.min(percentMode ? 100 : max, value + step);
    }
    
    if (newValue !== value) {
      oninput?.(newValue);
      onchange?.(newValue);
    }
  }
</script>

<div class="slider-container" class:disabled>
  <div
    class="slider-track"
    role="slider"
    tabindex={disabled ? -1 : 0}
    aria-valuemin={min}
    aria-valuemax={max}
    aria-valuenow={value}
    aria-disabled={disabled}
    onpointerdown={handlePointerDown}
    onpointermove={handlePointerMove}
    onpointerup={handlePointerUp}
    onpointercancel={handlePointerUp}
    onkeydown={handleKeyDown}
    bind:this={trackElement}
  >
    <div class="slider-fill" style="width: {percent}%"></div>
    <div
      class="slider-thumb"
      class:dragging={isDragging}
      style="left: {percent}%"
    ></div>
  </div>
  
  {#if showLabels && (labelLeft || labelRight)}
    <div class="slider-labels">
      <span>{labelLeft}</span>
      <span>{labelRight}</span>
    </div>
  {/if}
</div>

<style>
  .slider-container {
    width: 100%;
  }

  .slider-container.disabled {
    opacity: 0.5;
    pointer-events: none;
  }

  .slider-track {
    position: relative;
    height: 6px;
    background: var(--slider-track-bg, rgba(120, 120, 128, 0.32));
    border-radius: 3px;
    cursor: pointer;
    touch-action: none;
  }

  .slider-track:focus-visible {
    outline: 2px solid var(--accent-color, #66c0f4);
    outline-offset: 2px;
  }

  .slider-fill {
    position: absolute;
    top: 0;
    left: 0;
    height: 100%;
    background: var(--accent-color, #66c0f4);
    border-radius: 3px;
    transition: width 0.05s ease-out;
    pointer-events: none;
  }

  .slider-thumb {
    position: absolute;
    top: 50%;
    width: 16px;
    height: 16px;
    background: white;
    border-radius: 50%;
    box-shadow: 0 1px 4px rgba(0, 0, 0, 0.3);
    transform: translate(-50%, -50%);
    transition: transform 0.1s ease, box-shadow 0.1s ease;
    pointer-events: none;
  }

  .slider-thumb.dragging {
    transform: translate(-50%, -50%) scale(1.15);
    box-shadow: 0 2px 8px rgba(0, 0, 0, 0.4);
  }

  .slider-track:hover .slider-thumb {
    transform: translate(-50%, -50%) scale(1.05);
  }

  .slider-labels {
    display: flex;
    justify-content: space-between;
    margin-top: 4px;
    font-size: calc(11px * var(--font-scale, 1));
    color: var(--text-secondary);
  }
</style>
