<script lang="ts">
  /**
   * 状态栏组件
   * 
   * 显示时间、电池状态和品牌信息
   */

  interface BatteryStatus {
    has_battery: boolean;
    is_charging: boolean;
    battery_percent: number;
  }

  interface Props {
    /** 是否隐藏状态栏 */
    hidden?: boolean;
    /** 当前时间字符串 */
    currentTime: string;
    /** 电池状态 */
    batteryStatus: BatteryStatus;
    /** 翻译函数 */
    t: (key: string) => string;
  }

  let {
    hidden = false,
    currentTime,
    batteryStatus,
    t,
  }: Props = $props();
</script>

<footer class="footer" class:hidden>
  <!-- 左侧：时间和电池 -->
  <span class="ios-time">{currentTime}</span>
  <span class="footer-sep">|</span>
  {#if batteryStatus.has_battery}
    <div class="ios-battery" class:charging={batteryStatus.is_charging}>
      <span class="battery-percent"
        >{Math.round(batteryStatus.battery_percent)}%</span
      >
      <div class="battery-visual">
        <svg viewBox="0 0 26 12" class="battery-svg">
          <!-- 电池外壳 -->
          <rect
            x="0.5"
            y="0.5"
            width="22"
            height="11"
            rx="2.5"
            class="battery-shell"
            stroke="currentColor"
            fill="none"
            stroke-width="1"
          />
          <!-- 电池正极 -->
          <path
            d="M24 3.5h1.5v5H24z"
            class="battery-tip"
            fill="currentColor"
          />
          <!-- 电量填充 -->
          <rect
            x="2"
            y="2"
            width={Math.max(0, (batteryStatus.battery_percent / 100) * 19)}
            height="8"
            rx="1.5"
            class="battery-level"
            style="fill: {batteryStatus.is_charging
              ? '#34C759'
              : batteryStatus.battery_percent > 70
                ? '#34C759'
                : batteryStatus.battery_percent > 30
                  ? '#FFD60A'
                  : '#FF453A'}"
          />
          <!-- 充电闪电图标 (居中) -->
          {#if batteryStatus.is_charging}
            <path
              d="M11 2 L10 6 H13 L12 10"
              stroke="white"
              stroke-width="1.2"
              fill="none"
              stroke-linecap="round"
              stroke-linejoin="round"
              style="transform-origin: center; filter: drop-shadow(0 0 2px rgba(0,0,0,0.5));"
            />
          {/if}
        </svg>
        {#if batteryStatus.is_charging}
          <div class="charging-ripple-effect"></div>
        {/if}
      </div>
    </div>
  {:else}
    <div class="ios-battery ac-mode">
      <span class="battery-percent">AC</span>
      <svg
        viewBox="0 0 24 24"
        class="ac-icon"
        width="14"
        height="14"
        fill="none"
        stroke="currentColor"
        stroke-width="2"
        stroke-linecap="round"
        stroke-linejoin="round"
      >
        <path d="M13 2L3 14h9l-1 8 10-12h-9l1-8z" />
      </svg>
    </div>
  {/if}

  <!-- 中间弹性空间 -->
  <div style="flex: 1;"></div>

  <!-- 右侧：KoPanel + 交流群 -->
  <span class="footer-brand">KoPanel</span>
  <span class="footer-sep">·</span>
  <span class="footer-qq">{t("qq_group")} 178193167</span>
</footer>

<style>
  .footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
    padding: 4px 12px;
    font-size: calc(11px * var(--font-scale, 1));
    color: var(--text-secondary);
    background: var(--bg-secondary);
    border-top: 1px solid var(--border-color);
    flex-shrink: 0;
  }

  .footer.hidden {
    display: none;
  }

  .ios-time {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .footer-sep {
    margin: 0 4px;
    opacity: 0.3;
  }

  .footer-brand {
    font-weight: 700;
    color: var(--accent-color);
  }

  .footer-qq {
    font-weight: 600;
    opacity: 0.85;
  }

  /* 电池样式 */
  .ios-battery {
    display: flex;
    align-items: center;
    gap: 4px;
  }

  .battery-percent {
    font-weight: 600;
    font-variant-numeric: tabular-nums;
  }

  .battery-visual {
    position: relative;
    display: flex;
    align-items: center;
  }

  .battery-svg {
    width: 26px;
    height: 12px;
  }

  .battery-shell {
    opacity: 0.8;
  }

  .battery-tip {
    opacity: 0.8;
  }

  .ios-battery.charging .battery-percent {
    color: #34C759;
  }

  .ios-battery.ac-mode {
    color: var(--text-secondary);
  }

  .ac-icon {
    opacity: 0.8;
  }

  /* 充电动画 */
  .charging-ripple-effect {
    position: absolute;
    top: 50%;
    left: 50%;
    width: 30px;
    height: 16px;
    transform: translate(-50%, -50%);
    border-radius: 4px;
    background: radial-gradient(ellipse, rgba(52, 199, 89, 0.3) 0%, transparent 70%);
    animation: charging-pulse 1.5s ease-in-out infinite;
    pointer-events: none;
  }

  @keyframes charging-pulse {
    0%, 100% { opacity: 0.3; transform: translate(-50%, -50%) scale(1); }
    50% { opacity: 0.6; transform: translate(-50%, -50%) scale(1.1); }
  }
</style>
