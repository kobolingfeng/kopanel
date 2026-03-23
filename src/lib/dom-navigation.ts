/**
 * DOM-based 导航系统
 * 
 * 核心理念：不再硬编码布局，而是从实际渲染的 DOM 元素位置推断导航方向
 * 每个可聚焦项都应有 data-focus-index 属性
 * 
 * 性能优化：缓存位置信息，直到显式失效（invalidatePositionCache）
 */

/** 导航结果 */
export interface DomNavResult {
    /** 新的焦点索引 */
    newIndex: number;
    /** 是否发生了移动 */
    moved: boolean;
}

/** 元素位置信息 */
interface ItemPosition {
    index: number;
    rect: DOMRect;
    centerX: number;
    centerY: number;
}

// ==================== 位置信息缓存 ====================
interface PositionCache {
    positions: ItemPosition[];
    byIndex: Map<number, ItemPosition>;
    maxIndex: number;
    containerRef: WeakRef<HTMLElement> | null;
}

let positionCache: PositionCache | null = null;

/**
 * 使缓存失效（在布局变化时调用）
 */
export function invalidatePositionCache(): void {
    positionCache = null;
}

/**
 * 获取可聚焦元素的位置信息缓存（如无缓存则重建）
 * 支持多种属性名：data-focus-index, data-game-index 等
 */
function getPositionCache(container?: HTMLElement | null): PositionCache {
    // 检查缓存是否有效（仅当容器相同且未显式失效时复用）
    if (positionCache) {
        const sameContainer = container
            ? positionCache.containerRef?.deref() === container
            : positionCache.containerRef === null;

        if (sameContainer) {
            return positionCache;
        }
    }

    // 缓存失效或不存在，重新查询
    const selectors = '[data-focus-index], [data-game-index]';
    const elements = container
        ? container.querySelectorAll(selectors)
        : document.querySelectorAll(selectors);

    const positions: ItemPosition[] = [];
    const byIndex = new Map<number, ItemPosition>();
    let maxIndex = -1;

    for (let i = 0; i < elements.length; i++) {
        const el = elements[i] as Element;
        const indexAttr = el.getAttribute('data-focus-index')
            ?? el.getAttribute('data-game-index');
        if (indexAttr === null) continue;

        const index = parseInt(indexAttr, 10);
        if (isNaN(index) || index < 0) continue;
        // 防御：避免重复 index 导致导航结果不确定（保留首次出现的元素）
        if (byIndex.has(index)) continue;

        const rect = el.getBoundingClientRect();
        // 跳过不可见的元素（宽高为0）
        if (rect.width === 0 || rect.height === 0) continue;

        const pos: ItemPosition = {
            index,
            rect,
            centerX: rect.left + rect.width / 2,
            centerY: rect.top + rect.height / 2,
        };

        positions.push(pos);
        byIndex.set(index, pos);
        if (index > maxIndex) maxIndex = index;
    }

    // 更新缓存
    positionCache = {
        positions,
        byIndex,
        maxIndex,
        containerRef: container ? new WeakRef(container) : null,
    };

    return positionCache;
}

/**
 * 判断两个元素是否在同一行（垂直位置重叠）
 * 使用较大的容差以适应不同高度的元素
 */
function isSameRow(a: ItemPosition, b: ItemPosition): boolean {
    // 检查两个元素的垂直范围是否重叠
    const aTop = a.rect.top;
    const aBottom = a.rect.bottom;
    const bTop = b.rect.top;
    const bBottom = b.rect.bottom;

    // 有任何垂直重叠就算同一行
    return !(aBottom < bTop || bBottom < aTop);
}

/**
 * 判断两个元素是否在同一列（水平位置重叠）
 */
function isSameColumn(a: ItemPosition, b: ItemPosition, tolerance: number = 30): boolean {
    // 检查水平中心是否接近，或水平范围重叠
    const horizontalOverlap = !(a.rect.right < b.rect.left || b.rect.right < a.rect.left);
    if (horizontalOverlap) return true;

    // 中心点接近也算同列
    return Math.abs(a.centerX - b.centerX) < tolerance;
}

/**
 * 获取页面最底部的元素（Y坐标最大的）
 */
function getBottomMostItem(positions: ItemPosition[], currentIndex: number): ItemPosition | null {
    let bottom: ItemPosition | null = null;
    let maxY = -Infinity;

    for (let i = 0; i < positions.length; i++) {
        const p = positions[i];
        if (p.index === currentIndex) continue;
        if (p.centerY > maxY) {
            maxY = p.centerY;
            bottom = p;
        }
    }

    return bottom;
}

/**
 * 获取页面最顶部的元素（Y坐标最小的）
 */
function getTopMostItem(positions: ItemPosition[], currentIndex: number): ItemPosition | null {
    let top: ItemPosition | null = null;
    let minY = Infinity;

    for (let i = 0; i < positions.length; i++) {
        const p = positions[i];
        if (p.index === currentIndex) continue;
        if (p.centerY < minY) {
            minY = p.centerY;
            top = p;
        }
    }

    return top;
}

/**
 * 向上导航
 * 优先找同列上方的元素，没有则找最近的上方元素
 * 如果已在顶部，循环到底部
 */
export function domNavigateUp(currentIndex: number, container?: HTMLElement | null): DomNavResult {
    const cache = getPositionCache(container);
    const current = cache.byIndex.get(currentIndex);

    if (!current) {
        return { newIndex: currentIndex, moved: false };
    }

    const positions = cache.positions;
    const curTop = current.rect.top;
    const curCenterX = current.centerX;
    const curCenterY = current.centerY;

    // 单次扫描：同时记录同列最佳候选 & 非同列最佳候选
    let bestSameColumn: ItemPosition | null = null;
    let bestSameColumnY = -Infinity;

    let bestOther: ItemPosition | null = null;
    let bestOtherScore = Infinity;

    for (let i = 0; i < positions.length; i++) {
        const p = positions[i];
        if (p.index === currentIndex) continue;
        // 仅考虑在当前元素上方的候选
        if (p.centerY >= curTop) continue;

        if (isSameColumn(current, p)) {
            // 同列：选最近的（Y最大的）
            if (p.centerY > bestSameColumnY) {
                bestSameColumnY = p.centerY;
                bestSameColumn = p;
            }
        } else {
            // 非同列：用加权距离选最近的
            const score =
                (curCenterY - p.centerY) + Math.abs(curCenterX - p.centerX) * 0.3;
            if (score < bestOtherScore) {
                bestOtherScore = score;
                bestOther = p;
            }
        }
    }

    const best = bestSameColumn ?? bestOther;
    if (!best) {
        // 已在顶部，循环到底部
        const bottom = getBottomMostItem(positions, currentIndex);
        if (bottom) {
            return { newIndex: bottom.index, moved: true };
        }
        return { newIndex: currentIndex, moved: false };
    }

    return { newIndex: best.index, moved: true };
}

/**
 * 向下导航
 * 优先找同列下方的元素，没有则找最近的下方元素
 * 如果已在底部，循环到顶部
 */
export function domNavigateDown(currentIndex: number, container?: HTMLElement | null): DomNavResult {
    const cache = getPositionCache(container);
    const current = cache.byIndex.get(currentIndex);

    if (!current) {
        return { newIndex: currentIndex, moved: false };
    }

    const positions = cache.positions;
    const curBottom = current.rect.bottom;
    const curCenterX = current.centerX;
    const curCenterY = current.centerY;

    let bestSameColumn: ItemPosition | null = null;
    let bestSameColumnY = Infinity;

    let bestOther: ItemPosition | null = null;
    let bestOtherScore = Infinity;

    for (let i = 0; i < positions.length; i++) {
        const p = positions[i];
        if (p.index === currentIndex) continue;
        // 仅考虑在当前元素下方的候选
        if (p.centerY <= curBottom) continue;

        if (isSameColumn(current, p)) {
            // 同列：选最近的（Y最小的）
            if (p.centerY < bestSameColumnY) {
                bestSameColumnY = p.centerY;
                bestSameColumn = p;
            }
        } else {
            const score =
                (p.centerY - curCenterY) + Math.abs(curCenterX - p.centerX) * 0.3;
            if (score < bestOtherScore) {
                bestOtherScore = score;
                bestOther = p;
            }
        }
    }

    const best = bestSameColumn ?? bestOther;
    if (!best) {
        // 已在底部，循环到顶部
        const top = getTopMostItem(positions, currentIndex);
        if (top) {
            return { newIndex: top.index, moved: true };
        }
        return { newIndex: currentIndex, moved: false };
    }

    return { newIndex: best.index, moved: true };
}

/**
 * 向左导航
 * 只在同行内左移
 */
export function domNavigateLeft(currentIndex: number, container?: HTMLElement | null): DomNavResult {
    const cache = getPositionCache(container);
    const current = cache.byIndex.get(currentIndex);

    if (!current) {
        return { newIndex: currentIndex, moved: false };
    }

    const positions = cache.positions;
    const curCenterX = current.centerX;

    let best: ItemPosition | null = null;
    let bestX = -Infinity;

    for (let i = 0; i < positions.length; i++) {
        const p = positions[i];
        if (p.index === currentIndex) continue;
        if (p.centerX >= curCenterX) continue;
        if (!isSameRow(current, p)) continue;

        // 同行内：选最近的（X最大的）
        if (p.centerX > bestX) {
            bestX = p.centerX;
            best = p;
        }
    }

    if (!best) {
        // 没有同行左侧的，返回 moved: false 让调用者处理（可能是调整值）
        return { newIndex: currentIndex, moved: false };
    }

    return { newIndex: best.index, moved: true };
}

/**
 * 向右导航
 * 只在同行内右移
 */
export function domNavigateRight(currentIndex: number, container?: HTMLElement | null): DomNavResult {
    const cache = getPositionCache(container);
    const current = cache.byIndex.get(currentIndex);

    if (!current) {
        return { newIndex: currentIndex, moved: false };
    }

    const positions = cache.positions;
    const curCenterX = current.centerX;

    let best: ItemPosition | null = null;
    let bestX = Infinity;

    for (let i = 0; i < positions.length; i++) {
        const p = positions[i];
        if (p.index === currentIndex) continue;
        if (p.centerX <= curCenterX) continue;
        if (!isSameRow(current, p)) continue;

        // 同行内：选最近的（X最小的）
        if (p.centerX < bestX) {
            bestX = p.centerX;
            best = p;
        }
    }

    if (!best) {
        // 没有同行右侧的，返回 moved: false
        return { newIndex: currentIndex, moved: false };
    }

    return { newIndex: best.index, moved: true };
}

