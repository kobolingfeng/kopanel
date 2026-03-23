<script lang="ts">
  /**
   * 图标组件
   * 
   * 统一管理常用的 SVG 图标，减少重复代码
   * 支持多元素 SVG（path, polygon, polyline, circle, rect, line）
   */

  export type IconName = 
    | 'check'
    | 'x'
    | 'warning'
    | 'info'
    | 'star'
    | 'star-path'
    | 'settings'
    | 'gamepad'
    | 'gamepad-lines'
    | 'monitor'
    | 'monitor-lines'
    | 'cpu'
    | 'fan'
    | 'battery'
    | 'volume'
    | 'volume-2'
    | 'volume-x'
    | 'brightness'
    | 'refresh'
    | 'play'
    | 'pause'
    | 'skip-forward'
    | 'skip-back'
    | 'chevron-left'
    | 'chevron-right'
    | 'chevron-up'
    | 'chevron-down'
    | 'plus'
    | 'minus'
    | 'trash'
    | 'edit'
    | 'save'
    | 'folder'
    | 'file'
    | 'link'
    | 'grid'
    | 'grid-rects'
    | 'list'
    | 'search'
    | 'clock'
    | 'target'
    | 'power'
    | 'zap'
    | 'thermometer'
    | 'keyboard'
    | 'mouse'
    | 'circle'
    | 'square'
    | 'activity'
    | 'package'
    | 'sliders'
    | 'wifi'
    | 'bluetooth'
    | 'download'
    | 'upload'
    | 'eye'
    | 'eye-off'
    | 'lock'
    | 'unlock'
    | 'home'
    | 'layers'
    | 'maximize'
    | 'minimize'
    | 'move'
    | 'external-link'
    | 'alert-circle'
    | 'alert-triangle'
    | 'rotate-cw'
    | 'rotate-ccw'
    | 'droplet'
    | 'sun'
    | 'moon'
    | 'tool'
    | 'mic'
    | 'mic-off'
    | 'speaker'
    | 'image'
    | 'box'
    | 'copy'
    | 'clipboard'
    | 'smartphone'
    | 'device'
    | 'windows'
    | 'sensitivity';

  interface Props {
    /** 图标名称 */
    name: IconName;
    /** 图标大小（像素） */
    size?: number;
    /** 自定义类名 */
    class?: string;
    /** 自定义 stroke 颜色 */
    stroke?: string;
    /** 自定义样式 */
    style?: string;
  }

  let {
    name,
    size = 18,
    class: className = '',
    stroke = 'currentColor',
    style = '',
  }: Props = $props();

  // SVG 元素类型
  type SvgElement = 
    | { type: 'path'; d: string }
    | { type: 'polygon'; points: string }
    | { type: 'polyline'; points: string }
    | { type: 'circle'; cx: number; cy: number; r: number }
    | { type: 'rect'; x: number; y: number; width: number; height: number; rx?: number }
    | { type: 'line'; x1: number; y1: number; x2: number; y2: number };

  // 图标定义 - 支持多元素
  const iconDefs: Record<IconName, SvgElement[]> = {
    'check': [{ type: 'path', d: 'M20 6L9 17l-5-5' }],
    'x': [{ type: 'path', d: 'M18 6L6 18M6 6l12 12' }],
    'warning': [{ type: 'path', d: 'M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0zM12 9v4M12 17h.01' }],
    'info': [{ type: 'path', d: 'M12 2a10 10 0 1 0 0 20 10 10 0 0 0 0-20zM12 16v-4M12 8h.01' }],
    'star': [{ type: 'polygon', points: '12 2 15.09 8.26 22 9.27 17 14.14 18.18 21.02 12 17.77 5.82 21.02 7 14.14 2 9.27 8.91 8.26 12 2' }],
    'star-path': [{ type: 'path', d: 'M12 2l3.09 6.26L22 9.27l-5 4.87 1.18 6.88L12 17.77l-6.18 3.25L7 14.14 2 9.27l6.91-1.01L12 2z' }],
    'settings': [{ type: 'path', d: 'M12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6zM19.4 15a1.65 1.65 0 0 0 .33 1.82l.06.06a2 2 0 0 1 0 2.83 2 2 0 0 1-2.83 0l-.06-.06a1.65 1.65 0 0 0-1.82-.33 1.65 1.65 0 0 0-1 1.51V21a2 2 0 0 1-2 2 2 2 0 0 1-2-2v-.09A1.65 1.65 0 0 0 9 19.4a1.65 1.65 0 0 0-1.82.33l-.06.06a2 2 0 0 1-2.83 0 2 2 0 0 1 0-2.83l.06-.06a1.65 1.65 0 0 0 .33-1.82 1.65 1.65 0 0 0-1.51-1H3a2 2 0 0 1-2-2 2 2 0 0 1 2-2h.09A1.65 1.65 0 0 0 4.6 9a1.65 1.65 0 0 0-.33-1.82l-.06-.06a2 2 0 0 1 0-2.83 2 2 0 0 1 2.83 0l.06.06a1.65 1.65 0 0 0 1.82.33H9a1.65 1.65 0 0 0 1-1.51V3a2 2 0 0 1 2-2 2 2 0 0 1 2 2v.09a1.65 1.65 0 0 0 1 1.51 1.65 1.65 0 0 0 1.82-.33l.06-.06a2 2 0 0 1 2.83 0 2 2 0 0 1 0 2.83l-.06.06a1.65 1.65 0 0 0-.33 1.82V9a1.65 1.65 0 0 0 1.51 1H21a2 2 0 0 1 2 2 2 2 0 0 1-2 2h-.09a1.65 1.65 0 0 0-1.51 1z' }],
    'gamepad': [{ type: 'path', d: 'M6 12h4M8 10v4M15 13h.01M18 11h.01M17.32 5H6.68a4 4 0 0 0-3.978 3.59c-.006.052-.01.101-.017.152C2.604 9.416 2 14.456 2 16a3 3 0 0 0 6 0c0-1.317.926-3.036 2.067-3.798A2.982 2.982 0 0 1 12 11c.738 0 1.412.279 1.933.728 1.141.685 2.067 2.426 2.067 3.772a3 3 0 0 0 6 0c0-1.546-.604-6.584-.685-7.258-.007-.05-.011-.1-.017-.151A4 4 0 0 0 17.32 5z' }],
    'gamepad-lines': [
      { type: 'line', x1: 6, y1: 12, x2: 10, y2: 12 },
      { type: 'line', x1: 8, y1: 10, x2: 8, y2: 14 },
      { type: 'line', x1: 15, y1: 13, x2: 15.01, y2: 13 },
      { type: 'line', x1: 18, y1: 11, x2: 18.01, y2: 11 },
      { type: 'rect', x: 2, y: 6, width: 20, height: 12, rx: 2 },
    ],
    'monitor': [{ type: 'path', d: 'M2 3h20v14H2zM8 21h8M12 17v4' }],
    'monitor-lines': [
      { type: 'rect', x: 2, y: 3, width: 20, height: 14, rx: 2 },
      { type: 'line', x1: 8, y1: 21, x2: 16, y2: 21 },
      { type: 'line', x1: 12, y1: 17, x2: 12, y2: 21 },
    ],
    'cpu': [{ type: 'path', d: 'M6 4h12v16H6zM9 9h6M9 13h6M2 9h2M2 13h2M20 9h2M20 13h2M9 2v2M15 2v2M9 20v2M15 20v2' }],
    'fan': [{ type: 'path', d: 'M12 12c-2.333-2.333-5.833-1.167-7 0l7 7c1.167-1.167 2.333-4.667 0-7zm0 0c2.333 2.333 5.833 1.167 7 0l-7-7c-1.167 1.167-2.333 4.667 0 7zm0 0c-2.333 2.333-1.167 5.833 0 7l7-7c-1.167-1.167-4.667-2.333-7 0zm0 0c2.333-2.333 1.167-5.833 0-7l-7 7c1.167 1.167 4.667 2.333 7 0z' }],
    'battery': [{ type: 'path', d: 'M17 6H3v12h14V6zM21 10h-1v4h1v-4z' }],
    'volume': [{ type: 'path', d: 'M11 5L6 9H2v6h4l5 4V5z' }],
    'volume-2': [{ type: 'path', d: 'M11 5L6 9H2v6h4l5 4V5zM19.07 4.93a10 10 0 0 1 0 14.14M15.54 8.46a5 5 0 0 1 0 7.07' }],
    'volume-x': [{ type: 'path', d: 'M11 5L6 9H2v6h4l5 4V5zM23 9l-6 6M17 9l6 6' }],
    'brightness': [{ type: 'path', d: 'M12 17a5 5 0 1 0 0-10 5 5 0 0 0 0 10zM12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42' }],
    'refresh': [{ type: 'path', d: 'M1 4v6h6M23 20v-6h-6M20.49 9A9 9 0 0 0 5.64 5.64L1 10M23 14l-4.64 4.36A9 9 0 0 1 3.51 15' }],
    'play': [{ type: 'polygon', points: '5 3 19 12 5 21 5 3' }],
    'pause': [{ type: 'path', d: 'M6 4h4v16H6zM14 4h4v16h-4z' }],
    'skip-forward': [{ type: 'path', d: 'M5 4l10 8-10 8V4zM19 5v14' }],
    'skip-back': [{ type: 'path', d: 'M19 20L9 12l10-8v16zM5 19V5' }],
    'chevron-left': [{ type: 'path', d: 'M15 18l-6-6 6-6' }],
    'chevron-right': [{ type: 'path', d: 'M9 18l6-6-6-6' }],
    'chevron-up': [{ type: 'path', d: 'M18 15l-6-6-6 6' }],
    'chevron-down': [{ type: 'path', d: 'M6 9l6 6 6-6' }],
    'plus': [{ type: 'path', d: 'M12 5v14M5 12h14' }],
    'minus': [{ type: 'path', d: 'M5 12h14' }],
    'trash': [{ type: 'path', d: 'M3 6h18M19 6v14a2 2 0 0 1-2 2H7a2 2 0 0 1-2-2V6M8 6V4a2 2 0 0 1 2-2h4a2 2 0 0 1 2 2v2' }],
    'edit': [{ type: 'path', d: 'M11 4H4a2 2 0 0 0-2 2v14a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7M18.5 2.5a2.121 2.121 0 0 1 3 3L12 15l-4 1 1-4 9.5-9.5z' }],
    'save': [{ type: 'path', d: 'M19 21H5a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h11l5 5v11a2 2 0 0 1-2 2zM17 21v-8H7v8M7 3v5h8' }],
    'folder': [{ type: 'path', d: 'M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z' }],
    'file': [{ type: 'path', d: 'M13 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V9zM13 2v7h7' }],
    'link': [{ type: 'path', d: 'M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71' }],
    'grid': [{ type: 'path', d: 'M3 3h7v7H3zM14 3h7v7h-7zM14 14h7v7h-7zM3 14h7v7H3z' }],
    'grid-rects': [
      { type: 'rect', x: 3, y: 3, width: 7, height: 7, rx: 1 },
      { type: 'rect', x: 14, y: 3, width: 7, height: 7, rx: 1 },
      { type: 'rect', x: 3, y: 14, width: 7, height: 7, rx: 1 },
      { type: 'rect', x: 14, y: 14, width: 7, height: 7, rx: 1 },
    ],
    'list': [{ type: 'path', d: 'M8 6h13M8 12h13M8 18h13M3 6h.01M3 12h.01M3 18h.01' }],
    'search': [{ type: 'path', d: 'M11 17.25a6.25 6.25 0 1 0 0-12.5 6.25 6.25 0 0 0 0 12.5zM16 16l4.5 4.5' }],
    'clock': [{ type: 'path', d: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 6v6l4 2' }],
    'target': [{ type: 'path', d: 'M12 22c5.523 0 10-4.477 10-10S17.523 2 12 2 2 6.477 2 12s4.477 10 10 10zM12 18a6 6 0 1 0 0-12 6 6 0 0 0 0 12zM12 14a2 2 0 1 0 0-4 2 2 0 0 0 0 4z' }],
    'power': [{ type: 'path', d: 'M18.36 6.64a9 9 0 1 1-12.73 0M12 2v10' }],
    'zap': [{ type: 'polygon', points: '13 2 3 14 12 14 11 22 21 10 12 10 13 2' }],
    'thermometer': [{ type: 'path', d: 'M14 14.76V3.5a2.5 2.5 0 0 0-5 0v11.26a4.5 4.5 0 1 0 5 0z' }],
    'keyboard': [{ type: 'path', d: 'M2 6h20v12H2zM6 10h.01M10 10h.01M14 10h.01M18 10h.01M8 14h8' }],
    'mouse': [{ type: 'path', d: 'M12 2v6M2 10a10 10 0 0 0 20 0c0-5.523-4.477-10-10-10S2 4.477 2 10z' }],
    'circle': [{ type: 'circle', cx: 12, cy: 12, r: 10 }],
    'square': [{ type: 'rect', x: 3, y: 3, width: 18, height: 18 }],
    'activity': [{ type: 'polyline', points: '22 12 18 12 15 21 9 3 6 12 2 12' }],
    'package': [{ type: 'path', d: 'M16.5 9.4l-9-5.19M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16zM3.27 6.96L12 12.01l8.73-5.05M12 22.08V12' }],
    'sliders': [{ type: 'path', d: 'M4 21v-7M4 10V3M12 21v-9M12 8V3M20 21v-5M20 12V3M1 14h6M9 8h6M17 16h6' }],
    'wifi': [{ type: 'path', d: 'M5 12.55a11 11 0 0 1 14.08 0M1.42 9a16 16 0 0 1 21.16 0M8.53 16.11a6 6 0 0 1 6.95 0M12 20h.01' }],
    'bluetooth': [{ type: 'path', d: 'M6.5 6.5l11 11L12 23V1l5.5 5.5-11 11' }],
    'download': [{ type: 'path', d: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M7 10l5 5 5-5M12 15V3' }],
    'upload': [{ type: 'path', d: 'M21 15v4a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2v-4M17 8l-5-5-5 5M12 3v12' }],
    'eye': [{ type: 'path', d: 'M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8zM12 15a3 3 0 1 0 0-6 3 3 0 0 0 0 6z' }],
    'eye-off': [{ type: 'path', d: 'M17.94 17.94A10.07 10.07 0 0 1 12 20c-7 0-11-8-11-8a18.45 18.45 0 0 1 5.06-5.94M9.9 4.24A9.12 9.12 0 0 1 12 4c7 0 11 8 11 8a18.5 18.5 0 0 1-2.16 3.19m-6.72-1.07a3 3 0 1 1-4.24-4.24M1 1l22 22' }],
    'lock': [{ type: 'path', d: 'M19 11H5a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2zM7 11V7a5 5 0 0 1 10 0v4' }],
    'unlock': [{ type: 'path', d: 'M19 11H5a2 2 0 0 0-2 2v7a2 2 0 0 0 2 2h14a2 2 0 0 0 2-2v-7a2 2 0 0 0-2-2zM7 11V7a5 5 0 0 1 9.9-1' }],
    'home': [{ type: 'path', d: 'M3 9l9-7 9 7v11a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2zM9 22V12h6v10' }],
    'layers': [{ type: 'path', d: 'M12 2L2 7l10 5 10-5-10-5zM2 17l10 5 10-5M2 12l10 5 10-5' }],
    'maximize': [{ type: 'path', d: 'M8 3H5a2 2 0 0 0-2 2v3m18 0V5a2 2 0 0 0-2-2h-3m0 18h3a2 2 0 0 0 2-2v-3M3 16v3a2 2 0 0 0 2 2h3' }],
    'minimize': [{ type: 'path', d: 'M8 3v3a2 2 0 0 1-2 2H3m18 0h-3a2 2 0 0 1-2-2V3m0 18v-3a2 2 0 0 1 2-2h3M3 16h3a2 2 0 0 1 2 2v3' }],
    'move': [{ type: 'path', d: 'M5 9l-3 3 3 3M9 5l3-3 3 3M15 19l-3 3-3-3M19 9l3 3-3 3M2 12h20M12 2v20' }],
    'external-link': [{ type: 'path', d: 'M18 13v6a2 2 0 0 1-2 2H5a2 2 0 0 1-2-2V8a2 2 0 0 1 2-2h6M15 3h6v6M10 14L21 3' }],
    'alert-circle': [{ type: 'circle', cx: 12, cy: 12, r: 10 }, { type: 'path', d: 'M12 8v4M12 16h.01' }],
    'alert-triangle': [{ type: 'path', d: 'M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0zM12 9v4M12 17h.01' }],
    'rotate-cw': [{ type: 'path', d: 'M23 4v6h-6M20.49 15a9 9 0 1 1-2.12-9.36L23 10' }],
    'rotate-ccw': [{ type: 'path', d: 'M1 4v6h6M3.51 15a9 9 0 1 0 2.13-9.36L1 10' }],
    'droplet': [{ type: 'path', d: 'M12 2.69l5.66 5.66a8 8 0 1 1-11.31 0z' }],
    'sun': [
      { type: 'circle', cx: 12, cy: 12, r: 5 },
      { type: 'path', d: 'M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42' },
    ],
    'moon': [{ type: 'path', d: 'M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z' }],
    'tool': [{ type: 'path', d: 'M14.7 6.3a1 1 0 0 0 0 1.4l1.6 1.6a1 1 0 0 0 1.4 0l3.77-3.77a6 6 0 0 1-7.94 7.94l-6.91 6.91a2.12 2.12 0 0 1-3-3l6.91-6.91a6 6 0 0 1 7.94-7.94l-3.76 3.76z' }],
    'mic': [{ type: 'path', d: 'M12 1a3 3 0 0 0-3 3v8a3 3 0 0 0 6 0V4a3 3 0 0 0-3-3zM19 10v2a7 7 0 0 1-14 0v-2M12 19v4M8 23h8' }],
    'mic-off': [{ type: 'path', d: 'M1 1l22 22M9 9v3a3 3 0 0 0 5.12 2.12M15 9.34V4a3 3 0 0 0-5.94-.6M17 16.95A7 7 0 0 1 5 12v-2m14 0v2a7 7 0 0 1-.11 1.23M12 19v4M8 23h8' }],
    'speaker': [{ type: 'path', d: 'M11 5L6 9H2v6h4l5 4V5zM15.54 8.46a5 5 0 0 1 0 7.07M19.07 4.93a10 10 0 0 1 0 14.14' }],
    'image': [{ type: 'rect', x: 3, y: 3, width: 18, height: 18, rx: 2 }, { type: 'circle', cx: 8.5, cy: 8.5, r: 1.5 }, { type: 'path', d: 'M21 15l-5-5L5 21' }],
    'box': [{ type: 'path', d: 'M21 16V8a2 2 0 0 0-1-1.73l-7-4a2 2 0 0 0-2 0l-7 4A2 2 0 0 0 3 8v8a2 2 0 0 0 1 1.73l7 4a2 2 0 0 0 2 0l7-4A2 2 0 0 0 21 16zM3.27 6.96L12 12.01l8.73-5.05M12 22.08V12' }],
    'copy': [{ type: 'rect', x: 9, y: 9, width: 13, height: 13, rx: 2 }, { type: 'path', d: 'M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1' }],
    'clipboard': [{ type: 'path', d: 'M16 4h2a2 2 0 0 1 2 2v14a2 2 0 0 1-2 2H6a2 2 0 0 1-2-2V6a2 2 0 0 1 2-2h2M9 2h6a1 1 0 0 1 1 1v2a1 1 0 0 1-1 1H9a1 1 0 0 1-1-1V3a1 1 0 0 1 1-1z' }],
    'smartphone': [
      { type: 'rect', x: 5, y: 2, width: 14, height: 20, rx: 2 },
      { type: 'line', x1: 12, y1: 18, x2: 12.01, y2: 18 },
    ],
    'device': [
      { type: 'rect', x: 2, y: 7, width: 20, height: 14, rx: 2 },
      { type: 'path', d: 'M16 3h-8v4h8z' },
    ],
    'windows': [{ type: 'path', d: 'M3 5l8-1v8H3zM3 13h8v8l-8-1zM13 4l8-1v10h-8zM13 14h8v9l-8-1z' }],
    'sensitivity': [
      { type: 'circle', cx: 12, cy: 12, r: 9 },
      { type: 'path', d: 'M12 8v8' },
    ],
  };

  $effect(() => {
    if (!iconDefs[name]) {
      console.warn(`Icon "${name}" not found`);
    }
  });
</script>

<svg
  width={size}
  height={size}
  viewBox="0 0 24 24"
  fill="none"
  stroke={stroke}
  stroke-width="2"
  stroke-linecap="round"
  stroke-linejoin="round"
  class={className}
  {style}
>
  {#each iconDefs[name] || [] as el}
    {#if el.type === 'path'}
      <path d={el.d} />
    {:else if el.type === 'polygon'}
      <polygon points={el.points} />
    {:else if el.type === 'polyline'}
      <polyline points={el.points} />
    {:else if el.type === 'circle'}
      <circle cx={el.cx} cy={el.cy} r={el.r} />
    {:else if el.type === 'rect'}
      <rect x={el.x} y={el.y} width={el.width} height={el.height} rx={el.rx || 0} />
    {:else if el.type === 'line'}
      <line x1={el.x1} y1={el.y1} x2={el.x2} y2={el.y2} />
    {/if}
  {/each}
</svg>

<style>
  svg {
    flex-shrink: 0;
  }
</style>
