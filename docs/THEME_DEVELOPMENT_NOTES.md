# 主题开发经验总结 - Neo-Brutalism (粗野主义) 主题

## 概述

本文档记录了在为 GPD Control Panel 添加粗野主义（Neo-Brutalism）主题系列时遇到的问题和解决方案。

## 主题特征

粗野主义风格特点：
- **粗黑边框**：3px solid #1c1917
- **硬偏移阴影**：4px 4px 0 #1c1917（无模糊）
- **无圆角**：border-radius: 0
- **高对比度**：浅色背景 + 深色文字/边框
- **交互动效**：hover 上浮（阴影变大），active 下压（阴影缩小）

## 遇到的问题及解决方案

### 1. Slider 滑条定位问题

**问题**：滑块（thumb）和填充（fill）位置错乱，不在轨道上。

**原因**：默认 slider-track 使用 `padding: 24px 0; margin: -24px 0;` 扩大点击区域，真正的轨道是用 `::before` 伪元素绘制的。直接修改 slider-track 的样式会破坏这个结构。

**解决方案**：
```css
/* 方案A：保持结构，只修改伪元素样式 */
.slider-track {
  background: transparent;
}
.slider-track::before {
  /* 绘制实际轨道 */
}

/* 方案B：brutal主题完全重写（简化定位） */
.slider-track {
  height: 24px !important;
  padding: 0 !important;
  margin: 12px 0 !important;
  overflow: visible !important;
}
```

### 2. 文字颜色闪烁白色

**问题**：切换焦点时，文字短暂闪烁成白色然后恢复正常。

**原因**：App.svelte 中的 scoped 样式 `.list-item.focused { color: white; }` 优先级高于 styles.css 中的全局样式。

**解决方案**：
1. 在 App.svelte 的 `<style>` 中使用 `:global()` 选择器覆盖
2. 添加 `!important` 强制覆盖
3. 添加 `transition: none !important` 禁用过渡动画

```css
:global([data-theme="brutal_yellow"]) .list-item.focused {
  color: var(--text-primary) !important;
}

:global([data-theme="brutal_yellow"]) .stepper-label {
  color: #1c1917 !important;
  transition: none !important;
}
```

### 3. TDP 调节区域背景色问题

**问题**：TDP 调节区域选中时背景是深色，而不是主题色。

**原因**：TDP 调节区域使用的是 `.tdp-combo-item` 类，而不是 `.list-item`。

**解决方案**：单独为 `.tdp-combo-item` 和 `.tdp-combo-box` 添加样式覆盖：

```css
:global([data-theme="brutal_yellow"]) .tdp-combo-item.focused {
  background: var(--accent-color) !important;
  color: #1c1917 !important;
}
```

### 4. CSS 注释未闭合导致编译错误

**问题**：`Expected token */` 错误。

**原因**：在编辑过程中，注释 `/* 超小屏幕` 没有正确闭合。

**解决方案**：确保所有 CSS 注释正确闭合 `/* ... */`

### 5. 浅色主题下元素不可见

**问题**：浅色背景上的白色文字看不清（如 beta-notice、about-name）。

**解决方案**：为所有浅色主题添加文字颜色覆盖：

```css
:global([data-theme="light"]) .beta-title,
:global([data-theme="cream"]) .beta-title,
/* ... 其他浅色主题 ... */ {
  color: var(--text-primary);
}
```

### 6. Slider Track 伪元素可见

**问题**：浅色主题下 slider-track 的 padding 区域可见。

**原因**：`.slider-track` 设置了 `background: rgba(0, 0, 0, 0.1)`，但 padding 区域应该透明。

**解决方案**：
```css
.slider-track {
  background: transparent; /* track 本身透明 */
}
.slider-track::before {
  background: rgba(0, 0, 0, 0.1); /* 伪元素绘制实际轨道 */
}
```

### 7. 选中状态滑块变小

**问题**：brutal 主题下，slider 选中时滑块反而变小了。

**原因**：App.svelte 中 focused 状态的样式覆盖了 brutal 主题的大滑块设置。

**解决方案**：在 focused 状态样式中也使用 `!important` 设置正确的尺寸：

```css
:global([data-theme="brutal_yellow"]) .list-item.focused .slider-thumb {
  width: 24px !important;
  height: 48px !important;
}
```

## 最佳实践

### 深色/浅色主题选中样式规则

**重要**：为保持视觉一致性，不同深浅的主题在选中状态时应采用不同的视觉反馈方式：

- **深色主题**：选中时修改 **背景颜色** (`background: var(--accent-color)`)
- **浅色主题**：选中时修改 **边框颜色** (`border-color: var(--accent-color)`)

示例：
```css
/* 深色主题 - 背景色变化 */
html[data-theme="glitch_red"] .stepper-item.active {
  background: var(--accent-color);
  color: #ffffff;
}

/* 浅色主题 (brutal) - 边框色变化 */
html[data-theme="brutal_yellow"] .stepper-item.active {
  background: #ffffff;
  border-color: var(--accent-color);
  box-shadow: 1px 1px 0 var(--accent-color);
}
```

### 样式优先级管理

1. **Svelte scoped 样式** > **全局 CSS**
2. 需要覆盖 scoped 样式时，在 `<style>` 中使用 `:global()` 选择器
3. 必要时使用 `!important`，但要谨慎

### 主题特殊组件开发流程

1. 先研究默认组件的 HTML 结构和 CSS
2. 识别哪些样式在 styles.css（全局），哪些在 App.svelte（scoped）
3. 为新主题创建覆盖样式，注意选择器优先级
4. 测试所有状态：normal、hover、focused、active、disabled

### 文件组织

- `src/styles.css`：全局主题变量和组件样式
- `src/App.svelte <style>`：组件 scoped 样式和主题覆盖
- `src/lib/constants.ts`：主题配置（id, name, color, series 等）
- `src/lib/i18n.ts`：主题名称翻译

## 新增主题清单

添加新主题系列需要修改：

1. **constants.ts**：在 `COLOR_THEMES` 数组中添加主题配置
2. **i18n.ts**：添加主题名称翻译（4种语言）
3. **styles.css**：添加 CSS 变量和特殊组件样式
4. **App.svelte**：添加 scoped 样式覆盖（如需要）
5. **App.svelte**：在主题选择器中添加系列分组显示

## 粗野主义主题组件列表

已定制的组件：
- Toggle Switch（电闸式开关）
- Slider（粗轨道+大方块滑块+斜条纹填充）
- Arrow Stepper（分离式大方块按钮）
- Stepper Row（独立方块按钮组）
- Mode Item（精细调节模式按钮）
- Collapse Header（折叠栏）
- Toast（通知框）
- Beta Notice（实验室预览版提示）
- About Card（关于卡片）
- Trainer Search（修改器搜索页面）

---

*文档创建时间：2026-01-05*
