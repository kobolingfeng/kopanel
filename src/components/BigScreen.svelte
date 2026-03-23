<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import { invoke, convertFileSrc } from "@tauri-apps/api/core";
  import type { Game } from '../lib/types';

  // 转换封面路径为可访问 URL
  function getCoverUrl(coverPath: string | undefined): string | null {
    if (!coverPath) return null;
    if (coverPath.match(/^[A-Z]:/i)) {
      return convertFileSrc(coverPath);
    }
    return coverPath;
  }

  // Props - 从 App.svelte 传入的状态和回调
  let { 
    games = [],
    onExit = () => {},
    onLaunchGame = (_game: Game) => {},
    onAddGame = () => {},
    onTogglePin = (_game: Game) => {},
    // 性能设置
    tdpWatts = 15,
    tdpMin = 5,
    tdpMax = 30,
    onTdpChange = (_v: number) => {},
    fanModeIndex = 1,
    fanModes = ['Quiet', 'Balanced', 'Performance'],
    onFanModeChange = (_i: number) => {},
    // 显示设置
    brightness = 80,
    onBrightnessChange = (_v: number) => {},
    volume = 60,
    onVolumeChange = (_v: number) => {},
    refreshRateIndex = 0,
    refreshRates = [] as string[],
    onRefreshRateChange = (_i: number) => {},
    // i18n
    t = (key: string) => key,
  }: {
    games: Game[];
    onExit: () => void;
    onLaunchGame: (game: Game) => void;
    onAddGame: () => void;
    onTogglePin: (game: Game) => void;
    tdpWatts: number;
    tdpMin: number;
    tdpMax: number;
    onTdpChange: (v: number) => void;
    fanModeIndex: number;
    fanModes: string[];
    onFanModeChange: (i: number) => void;
    brightness: number;
    onBrightnessChange: (v: number) => void;
    volume: number;
    onVolumeChange: (v: number) => void;
    refreshRateIndex: number;
    refreshRates: string[];
    onRefreshRateChange: (i: number) => void;
    t: (key: string) => string;
  } = $props();

  // 状态
  let currentView = $state<'home' | 'library' | 'settings' | 'search'>('home');
  // focusedIndex 直接在游戏卡片区域导航
  let focusedIndex = $state(0); // 当前区域内的索引
  let libraryFocusIndex = $state(0); // 资源库内的焦点索引
  let libraryGridRef: HTMLDivElement | undefined = $state(undefined); // 资源库网格引用
  let currentTime = $state('');
  let isLoaded = $state(false);
  let controlCenterOpen = $state(false);
  let launchingGame: Game | null = $state(null);
  let searchQuery = $state('');

  // 快捷启动游戏列表：置顶游戏在前，然后按最近游玩时间排序，最多显示15个
  const displayGames = $derived(
    (() => {
      const pinned = games.filter(g => g.pinned);
      const unpinned = games.filter(g => !g.pinned);
      pinned.sort((a, b) => a.name.localeCompare(b.name));
      unpinned.sort((a, b) => {
        const timeA = a.lastPlayedAt ? new Date(a.lastPlayedAt).getTime() : 0;
        const timeB = b.lastPlayedAt ? new Date(b.lastPlayedAt).getTime() : 0;
        return timeB - timeA;
      });
      return [...pinned, ...unpinned].slice(0, 15);
    })()
  );

  // 搜索结果
  const searchResults = $derived(
    searchQuery.trim() 
      ? games.filter(g => g.name.toLowerCase().includes(searchQuery.toLowerCase()))
      : []
  );

  // 系统状态
  let batteryPercent = $state(100);
  let isCharging = $state(false);
  let isAC = $state(false); // 纯 AC 电源（无电池或满电）

  // 视频/大图缓存
  let videoCache: Record<string, { videoUrl?: string; heroUrl?: string }> = $state({});

  // 横向滚动容器引用
  let ribbonRef: HTMLDivElement | undefined = $state(undefined);

  // Y键长按置顶状态
  let yHoldStart: number | null = null;
  let yHoldInterval: number | null = null;
  let yHoldProgress = $state(0);
  const Y_HOLD_DURATION = 1000; // 长按1秒置顶

  // Toast 提示
  let toastMessage = $state('');
  let toastTimer: ReturnType<typeof setTimeout> | null = null;

  function showBsToast(msg: string) {
    toastMessage = msg;
    if (toastTimer) clearTimeout(toastTimer);
    toastTimer = setTimeout(() => { toastMessage = ''; toastTimer = null; }, 2000);
  }

  // 时间更新
  let timeInterval: number;
  // 追踪所有 setTimeout 句柄，确保组件销毁时清理
  let pendingTimeouts = new Set<ReturnType<typeof setTimeout>>();
  // 电池轮询 interval
  let batteryInterval: number | undefined;
  
  function startYHold() {
    if (currentView !== 'library') return;
    if (libraryFocusIndex >= games.length) return; // "添加游戏"按钮不支持置顶
    yHoldStart = Date.now();
    yHoldProgress = 0;
    if (yHoldInterval) clearInterval(yHoldInterval);
    yHoldInterval = window.setInterval(() => {
      if (yHoldStart) {
        const elapsed = Date.now() - yHoldStart;
        yHoldProgress = Math.min(100, (elapsed / Y_HOLD_DURATION) * 100);
        if (elapsed >= Y_HOLD_DURATION) {
          const game = games[libraryFocusIndex];
          if (game) {
            onTogglePin(game);
            const isPinned = !game.pinned;
            showBsToast(isPinned ? t('bs_pinned_to_home') : t('bs_unpinned_from_home'));
          }
          clearYHold();
        }
      }
    }, 50);
  }

  function clearYHold() {
    if (yHoldInterval) {
      clearInterval(yHoldInterval);
      yHoldInterval = null;
    }
    yHoldStart = null;
    yHoldProgress = 0;
  }

  function handleKeyUp(e: KeyboardEvent) {
    if (e.key === 'y' || e.key === 'Y') {
      clearYHold();
    }
  }

  onMount(() => {
    const loadTimeout = setTimeout(() => { pendingTimeouts.delete(loadTimeout); isLoaded = true; }, 100);
    pendingTimeouts.add(loadTimeout);
    
    const updateTime = () => {
      const now = new Date();
      currentTime = now.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' });
    };
    updateTime();
    timeInterval = setInterval(updateTime, 1000);

    // 获取电池状态（初始 + 定期轮询）
    const fetchBattery = () => {
      invoke<{ battery_percent: number; is_charging: boolean; is_ac_connected: boolean; has_battery: boolean }>('get_battery_status').then(status => {
        batteryPercent = status.battery_percent;
        isCharging = status.is_charging;
        isAC = !status.has_battery || (status.is_ac_connected && !status.is_charging);
      }).catch(() => {});
    };
    fetchBattery();
    batteryInterval = setInterval(fetchBattery, 30000); // 每 30 秒刷新

    // 键盘/手柄导航
    window.addEventListener('keydown', handleKeyDown);
    window.addEventListener('keyup', handleKeyUp);
  });

  onDestroy(() => {
    if (timeInterval) clearInterval(timeInterval);
    if (batteryInterval) clearInterval(batteryInterval);
    // 清理所有待执行的 setTimeout（防止组件销毁后回调触发）
    for (const t of pendingTimeouts) clearTimeout(t);
    pendingTimeouts.clear();
    if (yHoldInterval) clearInterval(yHoldInterval);
    if (toastTimer) clearTimeout(toastTimer);
    window.removeEventListener('keydown', handleKeyDown);
    window.removeEventListener('keyup', handleKeyUp);
  });

  function handleKeyDown(e: KeyboardEvent) {
    // 窗口没有焦点时不响应键盘事件，防止游戏中误触发
    if (!document.hasFocus()) return;
    
    const totalGameCards = displayGames.length + 1; // +1 是资源库卡片
    
    // B键/Escape: 只在子视图返回主界面，主界面不响应
    // 如果焦点在输入框/文本框上，不拦截 B 键（允许正常输入）
    const activeTag = (e.target as HTMLElement)?.tagName;
    if (e.key === 'Escape' || e.key === 'b' || e.key === 'B') {
      if ((e.key === 'b' || e.key === 'B') && (activeTag === 'INPUT' || activeTag === 'TEXTAREA')) {
        return; // 允许输入框正常输入 b/B
      }
      e.preventDefault();
      if (controlCenterOpen) {
        controlCenterOpen = false;
      } else if (currentView !== 'home') {
        // 从子视图返回主界面
        currentView = 'home';
      }
      // 主界面不响应 B 键
      return;
    }
    
    // 资源库视图导航
    if (currentView === 'library') {
      const total = games.length + 1; // +1 是"添加游戏"按钮
      // 动态计算列数（根据网格实际布局）
      let cols = 5;
      if (libraryGridRef) {
        const items = libraryGridRef.querySelectorAll('.library-item');
        if (items.length >= 2) {
          const first = items[0] as HTMLElement;
          const second = items[1] as HTMLElement;
          // 如果第二个元素的 top 与第一个相同，则它们在同一行
          if (first.offsetTop === second.offsetTop) {
            // 计算实际列数
            let count = 1;
            const firstTop = first.offsetTop;
            for (let i = 1; i < items.length; i++) {
              if ((items[i] as HTMLElement).offsetTop === firstTop) count++;
              else break;
            }
            cols = count;
          }
        }
      }
      
      switch(e.key) {
        case 'ArrowLeft':
          e.preventDefault();
          if (libraryFocusIndex > 0) libraryFocusIndex--;
          break;
        case 'ArrowRight':
          e.preventDefault();
          if (libraryFocusIndex < total - 1) libraryFocusIndex++;
          break;
        case 'ArrowUp':
          e.preventDefault();
          if (libraryFocusIndex >= cols) libraryFocusIndex -= cols;
          break;
        case 'ArrowDown':
          e.preventDefault();
          // 智能下移：如果下一行没有对应位置，移动到最后一个
          if (libraryFocusIndex + cols < total) {
            libraryFocusIndex += cols;
          } else if (libraryFocusIndex < total - 1) {
            libraryFocusIndex = total - 1;
          }
          break;
        case 'Enter':
        case ' ':
        case 'a':
        case 'A':
          e.preventDefault();
          if (libraryFocusIndex === games.length) {
            // "添加游戏"按钮
            onAddGame();
          } else if (games[libraryFocusIndex]) {
            handleLaunch(games[libraryFocusIndex]);
          }
          break;
      case 'y':
      case 'Y':
          // Y键长按置顶/取消置顶游戏
          if (!e.repeat && !yHoldStart) {
            e.preventDefault();
            startYHold();
          }
          break;
      }
      return;
    }
    
    // 主页导航
    if (currentView !== 'home') return;
    
    switch(e.key) {
      case 'ArrowLeft':
        e.preventDefault();
        if (focusedIndex > 0) focusedIndex--;
        break;
      case 'ArrowRight':
        e.preventDefault();
        if (focusedIndex < totalGameCards - 1) focusedIndex++;
        break;
      case 'Enter':
      case ' ':
      case 'a':
      case 'A':
        e.preventDefault();
        if (focusedIndex < displayGames.length) {
          // 游戏卡片：启动游戏
          handleLaunch(displayGames[focusedIndex]);
        } else {
          // 资源库卡片：进入资源库
          libraryFocusIndex = 0; // 重置资源库焦点
          currentView = 'library';
        }
        break;
      case 'Tab':
        e.preventDefault();
        controlCenterOpen = !controlCenterOpen;
        break;
    }
  }

  function handleLaunch(game: Game) {
    if (launchingGame) return; // 防止重复启动
    launchingGame = game;
    const t = setTimeout(() => {
      pendingTimeouts.delete(t);
      launchingGame = null;
      onLaunchGame(game);
    }, 1500);
    pendingTimeouts.add(t);
  }

  // 滑条处理
  let draggingSlider: string | null = null;

  function handleSlider(e: PointerEvent, type: string) {
    draggingSlider = type;
    (e.target as HTMLElement).setPointerCapture(e.pointerId);
    updateSliderValue(e, type);
  }

  function handleSliderMove(e: PointerEvent, type: string) {
    if (draggingSlider !== type) return;
    updateSliderValue(e, type);
  }

  function handleSliderUp() {
    draggingSlider = null;
  }

  function updateSliderValue(e: PointerEvent, type: string) {
    const track = e.currentTarget as HTMLElement;
    const rect = track.getBoundingClientRect();
    const percent = Math.max(0, Math.min(1, (e.clientX - rect.left) / rect.width));
    
    switch(type) {
      case 'tdp':
        onTdpChange(Math.round(tdpMin + percent * (tdpMax - tdpMin)));
        break;
      case 'brightness':
        onBrightnessChange(Math.round(percent * 100));
        break;
      case 'volume':
        onVolumeChange(Math.round(percent * 100));
        break;
    }
  }

  // 限制焦点索引在有效范围内（包括资源库卡片）
  $effect(() => {
    const maxIndex = displayGames.length; // 资源库是最后一个
    if (focusedIndex > maxIndex) {
      focusedIndex = maxIndex;
    }
  });

  // 主页焦点变化时滚动到可见区域
  $effect(() => {
    if (ribbonRef && currentView === 'home') {
      const cards = ribbonRef.querySelectorAll('.bigscreen-game-card');
      const targetCard = cards[focusedIndex] as HTMLElement | undefined;
      if (targetCard) {
        targetCard.scrollIntoView({ behavior: 'smooth', inline: 'nearest', block: 'nearest' });
      }
    }
  });

  // 资源库焦点变化时滚动到可见区域
  $effect(() => {
    if (libraryGridRef && currentView === 'library') {
      const items = libraryGridRef.querySelectorAll('.library-item');
      const targetItem = items[libraryFocusIndex] as HTMLElement | undefined;
      if (targetItem) {
        targetItem.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    }
  });

  // 计算当前游戏
  const focusedGame = $derived(
    focusedIndex < displayGames.length 
      ? displayGames[focusedIndex] 
      : null
  );

  // 加载视频/大图
  $effect(() => {
    const game = focusedGame;
    if (!game?.steamAppId) {
      return;
    }
    if (videoCache[game.steamAppId]) {
      return;
    }
    
    invoke<{ success: boolean; videoUrl?: string; heroUrl?: string }>('get_steam_video_url', { steamAppId: game.steamAppId })
      .then(result => {
        if (result.success) {
          videoCache[game.steamAppId!] = { videoUrl: result.videoUrl, heroUrl: result.heroUrl };
          videoCache = { ...videoCache };
        }
      })
      .catch((e) => console.error('[BigScreen] 视频请求失败:', e));
  });

</script>

<div class="bigscreen-container" class:loaded={isLoaded}>
  <!-- 背景图层 -->
  {#if currentView === 'home'}
    {#each displayGames as game, index}
      <div class="bg-layer" class:active={index === focusedIndex}>
        {#if game.steamAppId && videoCache[game.steamAppId]?.heroUrl}
          <img src={videoCache[game.steamAppId].heroUrl} alt="" class="bg-image hero-image" />
        {:else if getCoverUrl(game.cover)}
          <img src={getCoverUrl(game.cover)} alt="" class="bg-image" />
        {:else}
          <div class="bg-placeholder"></div>
        {/if}
        <div class="bg-overlay-top"></div>
        <div class="bg-overlay-bottom"></div>
        <div class="bg-overlay-left"></div>
      </div>
    {/each}
  {/if}

  <!-- 顶部导航 -->
  <div class="top-nav">
    <div class="nav-tabs">
      <button class="nav-tab" class:active={currentView === 'home'} onclick={() => currentView = 'home'}>
        {t('bs_games')}
      </button>
    </div>

    <div class="nav-right">
      <button class="nav-pill-btn" aria-label="搜索" onclick={() => { currentView = 'search'; searchQuery = ''; }}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <circle cx="11" cy="11" r="8"/>
          <path d="m21 21-4.35-4.35"/>
        </svg>
      </button>
      <button class="nav-pill-btn" aria-label="设置" onclick={() => currentView = 'settings'}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M12.22 2h-.44a2 2 0 0 0-2 2v.18a2 2 0 0 1-1 1.73l-.43.25a2 2 0 0 1-2 0l-.15-.08a2 2 0 0 0-2.73.73l-.22.38a2 2 0 0 0 .73 2.73l.15.1a2 2 0 0 1 1 1.72v.51a2 2 0 0 1-1 1.74l-.15.09a2 2 0 0 0-.73 2.73l.22.38a2 2 0 0 0 2.73.73l.15-.08a2 2 0 0 1 2 0l.43.25a2 2 0 0 1 1 1.73V20a2 2 0 0 0 2 2h.44a2 2 0 0 0 2-2v-.18a2 2 0 0 1 1-1.73l.43-.25a2 2 0 0 1 2 0l.15.08a2 2 0 0 0 2.73-.73l.22-.39a2 2 0 0 0-.73-2.73l-.15-.08a2 2 0 0 1-1-1.74v-.5a2 2 0 0 1 1-1.74l.15-.09a2 2 0 0 0 .73-2.73l-.22-.38a2 2 0 0 0-2.73-.73l-.15.08a2 2 0 0 1-2 0l-.43-.25a2 2 0 0 1-1-1.73V4a2 2 0 0 0-2-2z"/>
          <circle cx="12" cy="12" r="3"/>
        </svg>
      </button>
      <div class="nav-status">
        <!-- 电量组件 -->
        <div class="battery-widget" class:ac={isAC} class:charging={isCharging && !isAC} class:low={batteryPercent <= 20 && !isAC && !isCharging}>
          <span class="battery-percent">{isAC ? 'AC' : `${batteryPercent}%`}</span>
          {#if isAC}
            <div class="ac-plug">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M12 2v6M8 2v6M16 2v6M8 8h8a4 4 0 0 1 4 4v2a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2v-2a4 4 0 0 1 4-4ZM12 16v6"/>
              </svg>
              <div class="ac-dot"></div>
            </div>
          {:else}
            <div class="battery-shell">
              <div class="battery-tip"></div>
              <div class="battery-fill" style="width: {Math.max(5, batteryPercent)}%"></div>
              {#if isCharging}
                <div class="charging-bolt">
                  <svg viewBox="0 0 24 24" fill="currentColor"><polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/></svg>
                </div>
              {/if}
            </div>
          {/if}
        </div>
        <!-- 分隔线 + 头像 + 时间 -->
        <div class="user-time-group">
          <div class="user-avatar"></div>
          <span class="time-display">{currentTime}</span>
        </div>
      </div>
    </div>
  </div>

  <!-- 主内容区 -->
  {#if currentView === 'home'}
    <div class="main-content" class:loaded={isLoaded}>
      <!-- 游戏封面横向滚动 -->
      <div class="game-ribbon" bind:this={ribbonRef}>
        {#each displayGames as game, index}
          <button
            class="bigscreen-game-card"
            class:focused={index === focusedIndex}
            onclick={() => { focusedIndex = index; }}
            ondblclick={() => handleLaunch(game)}
          >
            <div class="card-image-wrapper">
              {#if getCoverUrl(game.cover)}
                <img src={getCoverUrl(game.cover)} alt={game.name} class="card-image" />
              {:else}
                <div class="card-placeholder">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="2" y="6" width="20" height="12" rx="2"/>
                    <line x1="6" y1="12" x2="10" y2="12"/>
                    <line x1="8" y1="10" x2="8" y2="14"/>
                    <circle cx="17" cy="12" r="1"/>
                  </svg>
                </div>
              {/if}
              {#if game.pinned}
                <span class="pin-badge" title="{t('bs_pinned_to_home')}">📌</span>
              {/if}
            </div>
            {#if index !== focusedIndex}
              <span class="card-title">{game.name}</span>
            {/if}
          </button>
        {/each}

        <!-- 游戏库入口 -->
        <button class="bigscreen-game-card library-card" class:focused={focusedIndex === displayGames.length} onclick={() => { focusedIndex = displayGames.length; }}>
          <div class="card-image-wrapper">
            <div class="library-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M4 19.5A2.5 2.5 0 0 1 6.5 17H20"/>
                <path d="M6.5 2H20v20H6.5A2.5 2.5 0 0 1 4 19.5v-15A2.5 2.5 0 0 1 6.5 2z"/>
              </svg>
              <span>{t('bs_library')}</span>
            </div>
          </div>
        </button>
      </div>

      <!-- 焦点游戏信息 -->
      {#if focusedGame}
        <div class="hero-content">
          <h1 class="game-logo">{focusedGame.name}</h1>
          <div class="game-meta">
            {#if focusedGame.source === 'steam'}
              <span class="meta-tag">Steam</span>
              <span class="meta-dot">•</span>
            {/if}
            {#if focusedGame.lastPlayedAt}
              <span class="meta-tag">{t('bs_recently_played')}</span>
              <span class="meta-dot">•</span>
            {/if}
            <span class="meta-badge">KOPANEL</span>
          </div>

          <div class="action-row">
            <button class="play-btn" onclick={() => handleLaunch(focusedGame)}>
              <svg viewBox="0 0 24 24" fill="currentColor">
                <polygon points="5 3 19 12 5 21 5 3"/>
              </svg>
              {focusedGame.lastPlayedAt ? t('bs_continue_game') : t('bs_start_game')}
            </button>
            <button class="more-btn" aria-label="更多选项">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <circle cx="12" cy="12" r="1"/>
                <circle cx="19" cy="12" r="1"/>
                <circle cx="5" cy="12" r="1"/>
              </svg>
            </button>

          </div>
        </div>
      {/if}
    </div>
  {:else if currentView === 'library'}
    <!-- 游戏库视图 -->
    <div class="library-view">
      <div class="library-header">
        <button class="back-btn" aria-label="{t('bs_cancel')}" onclick={() => currentView = 'home'}>
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5M12 19l-7-7 7-7"/>
          </svg>
        </button>
        <h2>{t('bs_game_library')}</h2>
      </div>

      <div class="library-grid" bind:this={libraryGridRef}>
        {#each games as game, index}
          <div 
            class="library-item"
            class:focused={libraryFocusIndex === index}
            role="button"
            tabindex="0"
            onclick={() => { libraryFocusIndex = index; handleLaunch(game); }}
            onkeydown={() => {}}
          >
            <div class="library-cover">
              {#if getCoverUrl(game.cover)}
                <img src={getCoverUrl(game.cover)} alt={game.name} />
              {:else}
                <div class="cover-placeholder">
                  <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                    <rect x="2" y="6" width="20" height="12" rx="2"/>
                  </svg>
                </div>
              {/if}
              {#if game.pinned}
                <span class="pin-badge">📌</span>
              {/if}
              {#if yHoldProgress > 0 && libraryFocusIndex === index}
                <div class="y-hold-overlay">
                  <div class="y-hold-bar" style="width: {yHoldProgress}%"></div>
                </div>
              {/if}
            </div>
            <span class="library-title">{game.name}</span>
          </div>
        {/each}
        <!-- 添加游戏按钮 -->
        <div 
          class="library-item add-game-item"
          class:focused={libraryFocusIndex === games.length}
          role="button"
          tabindex="0"
          onclick={() => { libraryFocusIndex = games.length; onAddGame(); }}
          onkeydown={() => {}}
        >
          <div class="library-cover add-game-cover">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <line x1="12" y1="5" x2="12" y2="19"/>
              <line x1="5" y1="12" x2="19" y2="12"/>
            </svg>
          </div>
          <span class="library-title">{t('bs_add_game')}</span>
        </div>
      </div>
    </div>
  {:else if currentView === 'settings'}
    <!-- 设置视图 -->
    <div class="settings-view">
      <div class="settings-header">
        <button class="back-btn" onclick={() => currentView = 'home'} aria-label="{t('bs_cancel')}">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
            <path d="M19 12H5M12 19l-7-7 7-7"/>
          </svg>
        </button>
        <h2>{t('bs_settings')}</h2>
      </div>

      <div class="settings-list">
        <!-- 性能设置分组 -->
        <div class="settings-group">
          <h3 class="group-title">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="13 2 3 14 12 14 11 22 21 10 12 10 13 2"/>
            </svg>
            {t('bs_performance')}
          </h3>

          <!-- TDP 滑条 -->
          <div class="settings-slider-item">
            <div class="slider-header">
              <span class="slider-label">{t('bs_tdp_power')}</span>
              <span class="slider-value">{tdpWatts}W</span>
            </div>
            <div class="bs-slider-track" 
                 onpointerdown={(e) => handleSlider(e, 'tdp')}
                 onpointermove={(e) => handleSliderMove(e, 'tdp')}
                 onpointerup={handleSliderUp}>
              <div class="bs-slider-fill" style="width: {((tdpWatts - tdpMin) / (tdpMax - tdpMin)) * 100}%"></div>
              <div class="bs-slider-thumb" style="left: {((tdpWatts - tdpMin) / (tdpMax - tdpMin)) * 100}%"></div>
            </div>
            <div class="slider-range">
              <span>{tdpMin}W</span>
              <span>{tdpMax}W</span>
            </div>
          </div>

          <!-- 风扇模式 -->
          <div class="settings-stepper-item">
            <span class="stepper-label">{t('bs_fan_mode')}</span>
            <div class="bs-stepper">
              <button class="bs-stepper-btn" onclick={() => onFanModeChange(Math.max(0, fanModeIndex - 1))} aria-label="prev">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="15 18 9 12 15 6"/>
                </svg>
              </button>
              <span class="bs-stepper-value">{fanModes[fanModeIndex] || 'Balanced'}</span>
              <button class="bs-stepper-btn" onclick={() => onFanModeChange(Math.min(fanModes.length - 1, fanModeIndex + 1))} aria-label="next">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="9 18 15 12 9 6"/>
                </svg>
              </button>
            </div>
          </div>
        </div>

        <!-- 显示设置分组 -->
        <div class="settings-group">
          <h3 class="group-title">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <rect x="2" y="3" width="20" height="14" rx="2" ry="2"/>
              <line x1="8" y1="21" x2="16" y2="21"/>
              <line x1="12" y1="17" x2="12" y2="21"/>
            </svg>
            {t('bs_display')}
          </h3>

          <!-- 亮度滑条 -->
          <div class="settings-slider-item">
            <div class="slider-header">
              <span class="slider-label">{t('bs_brightness')}</span>
              <span class="slider-value">{brightness}%</span>
            </div>
            <div class="bs-slider-track"
                 onpointerdown={(e) => handleSlider(e, 'brightness')}
                 onpointermove={(e) => handleSliderMove(e, 'brightness')}
                 onpointerup={handleSliderUp}>
              <div class="bs-slider-fill" style="width: {brightness}%"></div>
              <div class="bs-slider-thumb" style="left: {brightness}%"></div>
            </div>
          </div>

          <!-- 刷新率 -->
          {#if refreshRates.length > 0}
          <div class="settings-stepper-item">
            <span class="stepper-label">{t('bs_refresh_rate')}</span>
            <div class="bs-stepper">
              <button class="bs-stepper-btn" onclick={() => onRefreshRateChange(Math.max(0, refreshRateIndex - 1))} aria-label="prev">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="15 18 9 12 15 6"/>
                </svg>
              </button>
              <span class="bs-stepper-value">{refreshRates[refreshRateIndex] || '--'}</span>
              <button class="bs-stepper-btn" onclick={() => onRefreshRateChange(Math.min(refreshRates.length - 1, refreshRateIndex + 1))} aria-label="next">
                <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                  <polyline points="9 18 15 12 9 6"/>
                </svg>
              </button>
            </div>
          </div>
          {/if}
        </div>

        <!-- 音频设置分组 -->
        <div class="settings-group">
          <h3 class="group-title">
            <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <polygon points="11 5 6 9 2 9 2 15 6 15 11 19 11 5"/>
              <path d="M19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07"/>
            </svg>
            {t('bs_audio')}
          </h3>

          <!-- 音量滑条 -->
          <div class="settings-slider-item">
            <div class="slider-header">
              <span class="slider-label">{t('bs_volume')}</span>
              <span class="slider-value">{volume}%</span>
            </div>
            <div class="bs-slider-track"
                 onpointerdown={(e) => handleSlider(e, 'volume')}
                 onpointermove={(e) => handleSliderMove(e, 'volume')}
                 onpointerup={handleSliderUp}>
              <div class="bs-slider-fill" style="width: {volume}%"></div>
              <div class="bs-slider-thumb" style="left: {volume}%"></div>
            </div>
          </div>
        </div>

        <!-- 退出按钮 -->
        <div class="settings-group">
          <button class="settings-item exit-btn" onclick={onExit}>
            <div class="settings-icon">
              <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                <path d="M9 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h4"/>
                <polyline points="16 17 21 12 16 7"/>
                <line x1="21" y1="12" x2="9" y2="12"/>
              </svg>
            </div>
            <div class="settings-text">
              <h3>{t('bs_exit_bigscreen')}</h3>
              <p>{t('bs_back_to_panel')}</p>
            </div>
            <svg class="settings-arrow" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
              <path d="M9 18l6-6-6-6"/>
            </svg>
          </button>
        </div>
      </div>
    </div>
  {:else if currentView === 'search'}
    <!-- 搜索视图 -->
    <div class="search-view">
      <div class="search-header">
        <div class="search-input-wrapper">
          <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" class="search-icon">
            <circle cx="11" cy="11" r="8"/>
            <path d="m21 21-4.35-4.35"/>
          </svg>
          <input 
            type="text" 
            class="search-input" 
            placeholder={t('bs_search_games')}
            bind:value={searchQuery}
          />
          <button class="search-cancel" onclick={() => currentView = 'home'}>{t('bs_cancel')}</button>
        </div>
      </div>

      <div class="search-results">
        {#if searchQuery.trim()}
          <div class="search-count">{t('bs_found_results').replace('{count}', String(searchResults.length))}</div>
          <div class="search-grid">
            {#each searchResults as game}
              <div 
                class="search-item"
                role="button"
                tabindex="0"
                onclick={() => handleLaunch(game)}
                onkeydown={() => {}}
              >
                <div class="search-cover">
                  {#if getCoverUrl(game.cover)}
                    <img src={getCoverUrl(game.cover)} alt={game.name} />
                  {:else}
                    <div class="cover-placeholder">
                      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
                        <rect x="2" y="6" width="20" height="12" rx="2"/>
                      </svg>
                    </div>
                  {/if}
                </div>
                <div class="search-info">
                  <span class="search-title">{game.name}</span>
                  {#if game.source === 'steam'}
                    <span class="search-source">Steam</span>
                  {/if}
                </div>
              </div>
            {/each}
          </div>
        {:else}
          <div class="search-hint">{t('bs_search_hint')}</div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- 控制中心 -->
  <div class="control-center" class:open={controlCenterOpen}>
    <div class="cc-header">
      <h3>{t('bs_control_center')}</h3>
      <button class="cc-close" aria-label="{t('bs_cancel')}" onclick={() => controlCenterOpen = false}>
        <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2">
          <path d="M18 6L6 18M6 6l12 12"/>
        </svg>
      </button>
    </div>

    <div class="cc-grid">
      <div class="cc-card">
        <span class="cc-label">{t('bs_battery')}</span>
        <div class="cc-value">{batteryPercent}%</div>
        <span class="cc-status">{isCharging ? t('bs_charging') : t('bs_on_battery')}</span>
      </div>

      <button class="cc-card danger" onclick={onExit}>
        <span class="cc-label">{t('bs_exit')}</span>
        <div class="cc-value">{t('bs_back_panel')}</div>
      </button>
    </div>
  </div>

  <!-- 浮动返回按钮 -->
  <div class="float-btn-container">
    <button class="float-btn" onclick={() => {
      if (currentView !== 'home') {
        currentView = 'home';
      } else {
        controlCenterOpen = !controlCenterOpen;
      }
    }}>
      <svg viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="3">
        {#if currentView !== 'home'}
          <path d="M19 12H5m7-7l-7 7 7 7"/>
        {:else}
          <path d="M6 4v16"/>
          <path d="M19 4l-11 8 11 8"/>
        {/if}
      </svg>
    </button>
  </div>

  <!-- Toast 提示 -->
  {#if toastMessage}
    <div class="bs-toast">{toastMessage}</div>
  {/if}

  <!-- 启动动画 -->
  {#if launchingGame}
    <div class="launch-overlay">
      <div class="launch-content">
        {#if getCoverUrl(launchingGame.cover)}
          <img src={getCoverUrl(launchingGame.cover)} alt="" class="launch-cover" />
        {/if}
        <h2>{t('bs_launching')}</h2>
        <p>{launchingGame.name}</p>
      </div>
    </div>
  {/if}
</div>

<style>
  .bigscreen-container {
    position: fixed;
    inset: 0;
    z-index: 9999;
    background: #000;
    color: #fff;
    font-family: system-ui, -apple-system, sans-serif;
    overflow: hidden;
    opacity: 0;
    transition: opacity 0.5s ease;
    /* 确保不穿透鼠标事件 */
    pointer-events: auto;
  }

  .bigscreen-container.loaded {
    opacity: 1;
  }

  /* 背景图层 */
  .bg-layer {
    position: absolute;
    inset: 0;
    opacity: 0;
    transition: opacity 0.7s ease;
  }

  .bg-layer.active {
    opacity: 1;
  }

  .bg-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transform: scale(1.02);
  }

  .bg-placeholder {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #1a1a2e 0%, #16213e 100%);
  }

  .bg-overlay-top {
    position: absolute;
    inset: 0;
    background: linear-gradient(to bottom, rgba(0,0,0,0.6) 0%, transparent 40%);
  }

  .bg-overlay-bottom {
    position: absolute;
    inset: 0;
    background: linear-gradient(to top, #101010 0%, rgba(0,0,0,0.4) 40%, transparent 60%);
  }

  .bg-overlay-left {
    position: absolute;
    inset: 0;
    background: linear-gradient(to right, rgba(0,0,0,0.6) 0%, transparent 50%);
  }

  /* 顶部导航 */
  .top-nav {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    z-index: 50;
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    padding: 40px 48px;
    animation: slideDown 0.7s ease;
  }

  @keyframes slideDown {
    from { transform: translateY(-40px); opacity: 0; }
    to { transform: translateY(0); opacity: 1; }
  }

  .nav-tabs {
    display: flex;
    gap: 32px;
  }

  .nav-tab {
    background: none;
    border: none;
    color: rgba(255,255,255,0.5);
    font-size: 20px;
    font-weight: 500;
    padding-bottom: 4px;
    cursor: pointer;
    transition: all 0.3s;
  }

  .nav-tab.active {
    color: #fff;
    border-bottom: 2px solid #fff;
    font-weight: 700;
  }

  .nav-tab:hover {
    color: #fff;
  }

  .nav-right {
    display: flex;
    align-items: center;
    gap: 24px;
  }

  .nav-pill-btn {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 8px 12px;
    background: rgba(0,0,0,0.2);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 9999px;
    cursor: pointer;
    transition: all 0.3s;
  }

  .nav-pill-btn:hover {
    background: rgba(255,255,255,0.1);
  }

  .nav-pill-btn svg {
    width: 18px;
    height: 18px;
    color: rgba(255,255,255,0.8);
  }

  .nav-status {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: 8px;
  }

  .user-avatar {
    width: 32px;
    height: 32px;
    border-radius: 50%;
    background: linear-gradient(135deg, #6366f1 0%, #8b5cf6 100%);
    border: 2px solid rgba(255,255,255,0.2);
    box-shadow: 0 2px 8px rgba(0,0,0,0.3);
  }

  .battery-widget {
    display: flex;
    align-items: center;
    gap: 8px;
    padding: 6px 12px;
    background: rgba(0,0,0,0.2);
    backdrop-filter: blur(12px);
    border-radius: 9999px;
    border: 1px solid rgba(255,255,255,0.1);
    color: rgba(255,255,255,0.9);
    transition: all 0.3s;
  }

  .battery-widget:hover {
    background: rgba(255,255,255,0.1);
  }

  .battery-widget.ac {
    color: #60a5fa;
  }

  .battery-widget.charging {
    color: #4ade80;
  }

  .battery-widget.low {
    color: #ef4444;
  }

  .ac-plug {
    position: relative;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .ac-plug svg {
    width: 18px;
    height: 18px;
    filter: drop-shadow(0 0 8px rgba(96,165,250,0.6));
  }

  .ac-dot {
    position: absolute;
    top: -4px;
    right: -4px;
    width: 6px;
    height: 6px;
    background: #93c5fd;
    border-radius: 50%;
    animation: pulse 2s infinite;
    box-shadow: 0 0 5px rgba(96,165,250,1);
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.5; }
  }

  .battery-percent {
    font-size: 14px;
    font-weight: 600;
    font-family: ui-monospace, monospace;
    letter-spacing: -0.5px;
  }

  .battery-shell {
    position: relative;
    width: 26px;
    height: 14px;
    border-radius: 3px;
    border: 1.5px solid currentColor;
    opacity: 0.6;
    display: flex;
    align-items: center;
    padding: 1.5px;
  }

  .battery-tip {
    position: absolute;
    right: -4px;
    top: 50%;
    transform: translateY(-50%);
    width: 2px;
    height: 5px;
    border-radius: 0 1px 1px 0;
    background: currentColor;
  }

  .battery-fill {
    height: 100%;
    border-radius: 1px;
    background: currentColor;
    transition: width 0.5s ease-out;
  }

  .battery-widget.charging .battery-fill {
    box-shadow: 0 0 10px rgba(74,222,128,0.4);
  }

  .charging-bolt {
    position: absolute;
    top: -8px;
    right: -6px;
    width: 14px;
    height: 14px;
    background: #4ade80;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    border: 1px solid #000;
    box-shadow: 0 2px 8px rgba(0,0,0,0.3);
    animation: bounce 2s infinite;
  }

  .charging-bolt svg {
    width: 10px;
    height: 10px;
    color: #000;
  }

  @keyframes bounce {
    0%, 100% { transform: translateY(0); }
    50% { transform: translateY(-2px); }
  }

  .user-time-group {
    display: flex;
    align-items: center;
    gap: 12px;
    margin-left: 8px;
    padding-left: 16px;
    border-left: 1px solid rgba(255,255,255,0.1);
  }

  .time-display {
    font-size: 20px;
    font-weight: 300;
    font-family: ui-monospace, monospace;
    color: #fff;
    text-shadow: 0 2px 10px rgba(0,0,0,0.5);
  }

  /* 主内容区 */
  .main-content {
    position: relative;
    z-index: 10;
    height: 100%;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    padding: 112px 48px 48px;
    opacity: 0;
    transform: translateY(40px);
    transition: all 1s ease;
  }

  .main-content.loaded {
    opacity: 1;
    transform: translateY(0);
  }

  /* 游戏卡片横条 */
  .game-ribbon {
    display: flex;
    gap: 24px;
    padding: 16px 0 32px 8px;
    overflow-x: auto;
    overflow-y: visible;
    scrollbar-width: none;
    background: transparent;
    border: none;
  }

  .game-ribbon::-webkit-scrollbar {
    display: none;
  }

  .bigscreen-game-card {
    /* 固定外层尺寸，防止内部变化影响布局 */
    width: 192px;
    height: 300px;
    flex-shrink: 0;
    background: transparent;
    border: none;
    cursor: pointer;
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 12px;
    outline: none;
    overflow: visible;
  }

  .card-image-wrapper {
    width: 192px;
    height: 256px;
    border-radius: 16px;
    overflow: hidden;
    box-shadow: 0 8px 32px rgba(0,0,0,0.4);
    /* 只对 transform 和 opacity 进行过渡，避免奇怪的动画 */
    transition: transform 0.25s ease-out, opacity 0.25s ease-out, box-shadow 0.25s ease-out;
    transform: scale(0.85);
    opacity: 0.6;
  }

  .bigscreen-game-card:hover .card-image-wrapper {
    opacity: 1;
    transform: scale(0.9);
  }

  .bigscreen-game-card.focused .card-image-wrapper {
    transform: scale(1);
    opacity: 1;
    box-shadow: 0 20px 50px rgba(0,0,0,0.5), 0 0 0 4px white;
    outline: 4px solid white;
  }

  .card-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .card-placeholder {
    width: 100%;
    height: 100%;
    background: linear-gradient(135deg, #2a2a4a 0%, #1a1a3e 100%);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .card-placeholder svg {
    width: 48px;
    height: 48px;
    color: rgba(255,255,255,0.3);
  }

  .card-title {
    font-size: 14px;
    font-weight: 500;
    color: #fff;
    text-shadow: 0 2px 8px rgba(0,0,0,0.8);
    opacity: 0;
    transition: opacity 0.3s;
    white-space: nowrap;
    max-width: 100%;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .bigscreen-game-card:not(.focused):hover .card-title {
    opacity: 1;
  }

  .library-card .card-image-wrapper {
    background: rgba(255,255,255,0.1);
    border: 2px dashed rgba(255,255,255,0.2);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .library-card:hover .card-image-wrapper {
    background: rgba(255,255,255,0.15);
    border-color: rgba(255,255,255,0.4);
  }

  .library-icon {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    color: rgba(255,255,255,0.6);
  }

  .library-icon svg {
    width: 32px;
    height: 32px;
  }

  .library-icon span {
    font-size: 14px;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  /* Hero 内容区 */
  .hero-content {
    max-width: 900px;
    margin-top: auto;
    animation: fadeSlideUp 0.5s ease;
  }

  @keyframes fadeSlideUp {
    from { opacity: 0; transform: translateY(20px); }
    to { opacity: 1; transform: translateY(0); }
  }

  .game-logo {
    font-size: clamp(32px, 5vw, 72px);
    font-weight: 900;
    text-transform: uppercase;
    font-style: italic;
    letter-spacing: -0.02em;
    line-height: 1;
    text-shadow: 0 4px 30px rgba(0,0,0,0.5);
    margin-bottom: 16px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  .game-meta {
    display: flex;
    align-items: center;
    gap: 16px;
    font-size: 14px;
    font-weight: 500;
    color: rgba(255,255,255,0.9);
    margin-bottom: 24px;
  }

  .meta-tag {
    text-transform: uppercase;
    letter-spacing: 0.1em;
  }

  .meta-dot {
    color: rgba(255,255,255,0.4);
  }

  .meta-badge {
    background: rgba(255,255,255,0.2);
    padding: 4px 8px;
    border-radius: 4px;
    font-size: 12px;
    backdrop-filter: blur(10px);
  }

  .action-row {
    display: flex;
    gap: 16px;
  }

  .play-btn {
    display: flex;
    align-items: center;
    gap: 12px;
    background: #fff;
    color: #000;
    border: none;
    padding: 16px 48px;
    border-radius: 9999px;
    font-size: 18px;
    font-weight: 700;
    cursor: pointer;
    transition: all 0.3s;
    box-shadow: 0 0 30px rgba(255,255,255,0.2);
  }

  .play-btn:hover {
    transform: scale(1.05);
  }

  .play-btn svg {
    width: 20px;
    height: 20px;
  }

  .more-btn {
    width: 56px;
    height: 56px;
    background: rgba(255,255,255,0.1);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255,255,255,0.2);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.3s;
  }

  .more-btn:hover {
    background: rgba(255,255,255,0.2);
  }

  .more-btn svg {
    width: 24px;
    height: 24px;
    color: #fff;
  }


  /* 游戏库视图 */
  .library-view,
  .settings-view {
    position: fixed;
    inset: 0;
    z-index: 60;
    background: #121212;
    display: flex;
    flex-direction: column;
    animation: slideFromRight 0.3s ease;
  }

  @keyframes slideFromRight {
    from { transform: translateX(100%); }
    to { transform: translateX(0); }
  }

  .library-header,
  .settings-header {
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 80px 64px 32px;
    border-bottom: 1px solid rgba(255,255,255,0.05);
  }

  .library-header h2,
  .settings-header h2 {
    font-size: 32px;
    font-weight: 300;
  }

  .back-btn {
    width: 48px;
    height: 48px;
    background: rgba(255,255,255,0.05);
    border: none;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.3s;
  }

  .back-btn:hover {
    background: rgba(255,255,255,0.1);
  }

  .back-btn svg {
    width: 24px;
    height: 24px;
    color: #fff;
  }

  .library-grid {
    flex: 1;
    overflow-y: auto;
    padding: 32px 64px 64px;
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(160px, 1fr));
    gap: 24px 16px;
  }

  .library-item {
    display: flex;
    flex-direction: column;
    gap: 12px;
    cursor: pointer;
    transition: all 0.2s;
    background: none;
    border: none;
    text-align: left;
    color: inherit;
  }

  .library-item:hover {
    transform: scale(1.02);
  }

  .library-item.focused {
    transform: scale(1.05);
  }

  .library-item.focused .library-cover {
    box-shadow: 0 15px 40px rgba(0,0,0,0.6);
    outline: 4px solid #6366f1;
  }

  .library-cover {
    aspect-ratio: 2/3;
    border-radius: 12px;
    overflow: hidden;
    background: #202020;
    box-shadow: 0 8px 24px rgba(0,0,0,0.4);
    transition: all 0.2s;
  }

  .library-item:hover .library-cover {
    box-shadow: 0 15px 40px rgba(0,0,0,0.6);
    outline: 4px solid white;
  }

  .library-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .cover-placeholder {
    width: 100%;
    height: 100%;
    display: flex;
    align-items: center;
    justify-content: center;
    background: linear-gradient(135deg, #2a2a4a 0%, #1a1a3e 100%);
  }

  .cover-placeholder svg {
    width: 32px;
    height: 32px;
    color: rgba(255,255,255,0.3);
  }

  .library-title {
    font-size: 14px;
    font-weight: 500;
    color: rgba(255,255,255,0.9);
    padding: 0 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
    max-width: 100%;
  }

  /* 添加游戏按钮 */
  .add-game-cover {
    display: flex;
    align-items: center;
    justify-content: center;
    background: rgba(255,255,255,0.05);
    border: 2px dashed rgba(255,255,255,0.2);
  }

  .add-game-cover svg {
    width: 48px;
    height: 48px;
    color: rgba(255,255,255,0.4);
    transition: all 0.2s;
  }

  .add-game-item:hover .add-game-cover,
  .add-game-item.focused .add-game-cover {
    background: rgba(255,255,255,0.1);
    border-color: rgba(255,255,255,0.4);
  }

  .add-game-item:hover .add-game-cover svg,
  .add-game-item.focused .add-game-cover svg {
    color: rgba(255,255,255,0.8);
    transform: scale(1.1);
  }

  /* 设置列表 */
  .settings-list {
    flex: 1;
    overflow-y: auto;
    padding: 32px 64px;
    max-width: 800px;
  }

  .settings-item {
    width: 100%;
    display: flex;
    align-items: center;
    gap: 24px;
    padding: 24px;
    background: none;
    border: none;
    border-radius: 16px;
    cursor: pointer;
    transition: all 0.3s;
    text-align: left;
    color: inherit;
  }

  .settings-item:hover {
    background: rgba(255,255,255,0.05);
  }

  .settings-icon {
    width: 48px;
    height: 48px;
    background: rgba(255,255,255,0.1);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    transition: all 0.3s;
  }

  .settings-item:hover .settings-icon {
    transform: scale(1.1);
  }

  .settings-icon svg {
    width: 24px;
    height: 24px;
    color: rgba(255,255,255,0.8);
  }

  .settings-text {
    flex: 1;
  }

  .settings-text h3 {
    font-size: 20px;
    font-weight: 500;
    margin-bottom: 4px;
  }

  .settings-text p {
    font-size: 14px;
    color: rgba(255,255,255,0.4);
  }

  .settings-arrow {
    width: 24px;
    height: 24px;
    color: rgba(255,255,255,0.2);
    transition: all 0.3s;
  }

  .settings-item:hover .settings-arrow {
    color: #fff;
    transform: translateX(8px);
  }

  /* 控制中心 */
  .control-center {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    z-index: 100;
    background: #1a1a1a;
    border-top-left-radius: 48px;
    border-top-right-radius: 48px;
    padding: 40px;
    transform: translateY(100%);
    opacity: 0;
    transition: all 0.5s cubic-bezier(0.16, 1, 0.3, 1);
    box-shadow: 0 -20px 60px rgba(0,0,0,0.8);
    border-top: 1px solid rgba(255,255,255,0.1);
  }

  .control-center.open {
    transform: translateY(0);
    opacity: 1;
  }

  .cc-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 32px;
    max-width: 1400px;
    margin-left: auto;
    margin-right: auto;
  }

  .cc-header h3 {
    font-size: 24px;
    font-weight: 300;
    letter-spacing: 0.05em;
  }

  .cc-close {
    width: 40px;
    height: 40px;
    background: rgba(255,255,255,0.1);
    border: none;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.3s;
  }

  .cc-close:hover {
    background: rgba(255,255,255,0.2);
  }

  .cc-close svg {
    width: 24px;
    height: 24px;
    color: #fff;
  }

  .cc-grid {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 24px;
    max-width: 1400px;
    margin: 0 auto;
    padding-bottom: 32px;
  }

  .cc-card {
    background: rgba(255,255,255,0.05);
    border: 1px solid rgba(255,255,255,0.05);
    border-radius: 24px;
    padding: 24px;
    display: flex;
    flex-direction: column;
    gap: 8px;
    cursor: pointer;
    transition: all 0.3s;
  }

  .cc-card:hover {
    background: rgba(255,255,255,0.1);
  }

  .cc-card.danger {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.2);
  }

  .cc-card.danger:hover {
    background: rgba(239, 68, 68, 0.2);
  }

  .cc-label {
    font-size: 12px;
    color: rgba(255,255,255,0.6);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    font-weight: 700;
  }

  .cc-card.danger .cc-label {
    color: rgba(239, 68, 68, 0.8);
  }

  .cc-value {
    font-size: 24px;
    font-weight: 300;
  }

  .cc-status {
    font-size: 14px;
    color: #22c55e;
  }

  /* 浮动按钮 */
  .float-btn-container {
    position: absolute;
    bottom: 32px;
    right: 32px;
    z-index: 50;
  }

  .float-btn {
    width: 64px;
    height: 64px;
    background: rgba(255,255,255,0.1);
    backdrop-filter: blur(20px);
    border: 1px solid rgba(255,255,255,0.2);
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.3s;
    box-shadow: 0 0 30px rgba(0,0,0,0.3);
  }

  .float-btn:hover {
    background: rgba(255,255,255,0.2);
    transform: scale(1.1);
    border-color: rgba(255,255,255,0.5);
    box-shadow: 0 0 20px rgba(255,255,255,0.2);
  }

  .float-btn svg {
    width: 28px;
    height: 28px;
    color: #fff;
  }

  /* 启动动画 */
  .launch-overlay {
    position: fixed;
    inset: 0;
    z-index: 200;
    background: #000;
    display: flex;
    align-items: center;
    justify-content: center;
    animation: fadeIn 0.3s ease;
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .launch-content {
    text-align: center;
  }

  .launch-cover {
    width: 128px;
    height: 192px;
    border-radius: 12px;
    box-shadow: 0 8px 32px rgba(0,0,0,0.5);
    margin-bottom: 32px;
    animation: pulse 1.5s infinite;
  }

  @keyframes pulse {
    0%, 100% { opacity: 1; }
    50% { opacity: 0.7; }
  }

  .launch-content h2 {
    font-size: 24px;
    font-weight: 300;
    letter-spacing: 0.2em;
    text-transform: uppercase;
    margin-bottom: 8px;
  }

  .launch-content p {
    font-size: 14px;
    color: rgba(255,255,255,0.4);
  }

  /* 设置页面 - 分组样式 */
  .settings-group {
    margin-bottom: 32px;
  }

  .group-title {
    display: flex;
    align-items: center;
    gap: 12px;
    font-size: 14px;
    font-weight: 600;
    color: rgba(255,255,255,0.5);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 16px;
    padding-left: 8px;
  }

  .group-title svg {
    width: 18px;
    height: 18px;
  }

  /* 滑条样式 */
  .settings-slider-item {
    background: rgba(255,255,255,0.05);
    border-radius: 16px;
    padding: 20px 24px;
    margin-bottom: 12px;
  }

  .slider-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 16px;
  }

  .slider-label {
    font-size: 16px;
    font-weight: 500;
    color: #fff;
  }

  .slider-value {
    font-size: 18px;
    font-weight: 600;
    color: var(--accent-color, #66c0f4);
    font-family: ui-monospace, monospace;
  }

  .bs-slider-track {
    position: relative;
    height: 8px;
    background: rgba(255,255,255,0.1);
    border-radius: 4px;
    cursor: pointer;
    touch-action: none;
  }

  .bs-slider-fill {
    position: absolute;
    left: 0;
    top: 0;
    height: 100%;
    background: linear-gradient(90deg, var(--accent-color, #66c0f4), #4fa3d1);
    border-radius: 4px;
    pointer-events: none;
  }

  .bs-slider-thumb {
    position: absolute;
    top: 50%;
    width: 24px;
    height: 24px;
    background: #fff;
    border-radius: 50%;
    transform: translate(-50%, -50%);
    box-shadow: 0 2px 8px rgba(0,0,0,0.3);
    pointer-events: none;
    transition: transform 0.1s;
  }

  .bs-slider-track:hover .bs-slider-thumb {
    transform: translate(-50%, -50%) scale(1.1);
  }

  .slider-range {
    display: flex;
    justify-content: space-between;
    margin-top: 8px;
    font-size: 12px;
    color: rgba(255,255,255,0.4);
  }

  /* Stepper 样式 */
  .settings-stepper-item {
    display: flex;
    justify-content: space-between;
    align-items: center;
    background: rgba(255,255,255,0.05);
    border-radius: 16px;
    padding: 16px 24px;
    margin-bottom: 12px;
  }

  .stepper-label {
    font-size: 16px;
    font-weight: 500;
    color: #fff;
  }

  .bs-stepper {
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .bs-stepper-btn {
    width: 40px;
    height: 40px;
    background: rgba(255,255,255,0.1);
    border: none;
    border-radius: 50%;
    display: flex;
    align-items: center;
    justify-content: center;
    cursor: pointer;
    transition: all 0.2s;
  }

  .bs-stepper-btn:hover {
    background: rgba(255,255,255,0.2);
  }

  .bs-stepper-btn:active {
    transform: scale(0.95);
  }

  .bs-stepper-btn svg {
    width: 20px;
    height: 20px;
    color: #fff;
  }

  .bs-stepper-value {
    min-width: 80px;
    text-align: center;
    font-size: 16px;
    font-weight: 600;
    color: var(--accent-color, #66c0f4);
  }

  /* 退出按钮特殊样式 */
  .exit-btn {
    margin-top: 16px;
    border: 1px solid rgba(239, 68, 68, 0.3);
  }

  .exit-btn:hover {
    background: rgba(239, 68, 68, 0.1);
    border-color: rgba(239, 68, 68, 0.5);
  }

  .exit-btn .settings-icon {
    background: rgba(239, 68, 68, 0.2);
  }

  .exit-btn .settings-icon svg {
    color: #ef4444;
  }

  /* 美化滚动条 */
  .library-grid::-webkit-scrollbar,
  .settings-list::-webkit-scrollbar {
    width: 8px;
  }

  .library-grid::-webkit-scrollbar-track,
  .settings-list::-webkit-scrollbar-track {
    background: rgba(255,255,255,0.03);
    border-radius: 4px;
  }

  .library-grid::-webkit-scrollbar-thumb,
  .settings-list::-webkit-scrollbar-thumb {
    background: rgba(255,255,255,0.15);
    border-radius: 4px;
    transition: background 0.2s;
  }

  .library-grid::-webkit-scrollbar-thumb:hover,
  .settings-list::-webkit-scrollbar-thumb:hover {
    background: rgba(255,255,255,0.25);
  }

  .library-grid::-webkit-scrollbar-thumb:active,
  .settings-list::-webkit-scrollbar-thumb:active {
    background: rgba(255,255,255,0.35);
  }

  /* 搜索视图 */
  .search-view {
    position: fixed;
    inset: 0;
    z-index: 70;
    background: rgba(10,10,10,0.95);
    backdrop-filter: blur(20px);
    display: flex;
    flex-direction: column;
    animation: fadeIn 0.3s ease;
  }

  .search-header {
    padding: 80px 64px 32px;
    border-bottom: 1px solid rgba(255,255,255,0.1);
  }

  .search-input-wrapper {
    max-width: 800px;
    margin: 0 auto;
    position: relative;
    display: flex;
    align-items: center;
    gap: 16px;
  }

  .search-icon {
    position: absolute;
    left: 24px;
    width: 28px;
    height: 28px;
    color: rgba(255,255,255,0.4);
  }

  .search-input {
    flex: 1;
    background: rgba(255,255,255,0.05);
    border: 1px solid rgba(255,255,255,0.1);
    border-radius: 9999px;
    padding: 20px 24px 20px 64px;
    font-size: 20px;
    color: #fff;
    outline: none;
    transition: all 0.3s;
  }

  .search-input::placeholder {
    color: rgba(255,255,255,0.3);
  }

  .search-input:focus {
    background: rgba(255,255,255,0.1);
    border-color: rgba(255,255,255,0.3);
  }

  .search-cancel {
    background: none;
    border: none;
    color: rgba(255,255,255,0.6);
    font-size: 16px;
    cursor: pointer;
    padding: 12px 16px;
    border-radius: 8px;
    transition: all 0.2s;
  }

  .search-cancel:hover {
    color: #fff;
    background: rgba(255,255,255,0.1);
  }

  .search-results {
    flex: 1;
    overflow-y: auto;
    padding: 32px 64px;
  }

  .search-count {
    font-size: 14px;
    font-weight: 700;
    color: rgba(255,255,255,0.4);
    text-transform: uppercase;
    letter-spacing: 0.1em;
    margin-bottom: 24px;
  }

  .search-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(300px, 1fr));
    gap: 16px;
  }

  .search-item {
    display: flex;
    align-items: center;
    gap: 16px;
    padding: 16px;
    background: rgba(255,255,255,0.05);
    border: 1px solid rgba(255,255,255,0.05);
    border-radius: 16px;
    cursor: pointer;
    transition: all 0.2s;
  }

  .search-item:hover {
    background: rgba(255,255,255,0.1);
    transform: scale(1.02);
  }

  .search-cover {
    width: 64px;
    height: 80px;
    border-radius: 8px;
    overflow: hidden;
    flex-shrink: 0;
  }

  .search-cover img {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .search-info {
    flex: 1;
    min-width: 0;
  }

  .search-title {
    display: block;
    font-size: 16px;
    font-weight: 600;
    color: #fff;
    margin-bottom: 4px;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .search-source {
    font-size: 12px;
    color: rgba(255,255,255,0.4);
  }

  .search-hint {
    text-align: center;
    color: rgba(255,255,255,0.3);
    font-size: 18px;
    padding: 64px 0;
  }

  /* 置顶徽章 */
  .pin-badge {
    position: absolute;
    top: 6px;
    right: 6px;
    font-size: 16px;
    line-height: 1;
    filter: drop-shadow(0 1px 3px rgba(0,0,0,0.6));
    pointer-events: none;
    z-index: 2;
  }

  .card-image-wrapper,
  .library-cover {
    position: relative;
  }

  /* Y 长按进度条 */
  .y-hold-overlay {
    position: absolute;
    bottom: 0;
    left: 0;
    right: 0;
    height: 4px;
    background: rgba(0,0,0,0.4);
    z-index: 3;
  }

  .y-hold-bar {
    height: 100%;
    background: #ffcc00;
    transition: width 0.05s linear;
    border-radius: 0 2px 2px 0;
  }

  /* Toast 提示 */
  .bs-toast {
    position: fixed;
    bottom: 60px;
    left: 50%;
    transform: translateX(-50%);
    background: rgba(0,0,0,0.85);
    color: #fff;
    padding: 12px 28px;
    border-radius: 8px;
    font-size: 16px;
    z-index: 10001;
    pointer-events: none;
    animation: bsToastIn 0.25s ease;
    backdrop-filter: blur(8px);
    border: 1px solid rgba(255,255,255,0.1);
  }

  @keyframes bsToastIn {
    from { opacity: 0; transform: translateX(-50%) translateY(12px); }
    to { opacity: 1; transform: translateX(-50%) translateY(0); }
  }
</style>
