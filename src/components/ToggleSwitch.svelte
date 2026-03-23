<script lang="ts">
  /**
   * 切换开关组件
   * 
   * 可复用的开关组件，支持：
   * - 受控模式（通过 checked 和 onchange 控制）
   * - 禁用状态
   * - 自定义样式
   */

  interface Props {
    /** 是否选中 */
    checked?: boolean;
    /** 是否禁用 */
    disabled?: boolean;
    /** 无障碍标签 */
    ariaLabel?: string;
    /** 自定义样式 */
    style?: string;
    /** 变化回调 */
    onchange?: (checked: boolean) => void;
  }

  let {
    checked = false,
    disabled = false,
    ariaLabel = '',
    style = '',
    onchange,
  }: Props = $props();

  function handleClick(e: MouseEvent) {
    if (disabled) return;
    // 如果有 onchange 回调，调用它并阻止事件冒泡
    // 否则让事件冒泡到父元素处理
    if (onchange) {
      e.stopPropagation();
      const newValue = !checked;
      onchange(newValue);
    }
    // 没有 onchange 时，事件会自然冒泡到父元素
  }

  function handleKeydown(e: KeyboardEvent) {
    if (disabled) return;
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      if (onchange) {
        onchange(!checked);
      } else {
        // 无 onchange 时分发真实 DOM click 事件，确保冒泡到父元素
        (e.currentTarget as HTMLElement)?.click();
      }
    }
  }
</script>

<div
  class="toggle-switch"
  class:active={checked}
  class:disabled
  role="switch"
  tabindex={disabled ? -1 : 0}
  aria-checked={checked}
  aria-label={ariaLabel}
  aria-disabled={disabled}
  {style}
  onclick={handleClick}
  onkeydown={handleKeydown}
>
  <div class="toggle-thumb"></div>
</div>

<style>
  .toggle-switch {
    position: relative;
    width: 44px;
    height: 24px;
    background: var(--toggle-bg, rgba(120, 120, 128, 0.32));
    border-radius: 12px;
    cursor: pointer;
    transition: background 0.2s ease;
    flex-shrink: 0;
  }

  .toggle-switch.active {
    background: var(--accent-color, #66c0f4);
  }

  .toggle-switch.disabled {
    opacity: 0.5;
    cursor: not-allowed;
  }

  .toggle-thumb {
    position: absolute;
    top: 2px;
    left: 2px;
    width: 20px;
    height: 20px;
    background: white;
    border-radius: 50%;
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.3);
    transition: transform 0.2s ease;
  }

  .toggle-switch.active .toggle-thumb {
    transform: translateX(20px);
  }

  /* 焦点状态 */
  .toggle-switch:focus-visible {
    outline: 2px solid var(--accent-color, #66c0f4);
    outline-offset: 2px;
  }

  /* 悬停状态 */
  .toggle-switch:not(.disabled):hover {
    opacity: 0.9;
  }
</style>
