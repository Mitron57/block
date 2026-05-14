import type { BoardElement } from './types'

function distToSegment(px: number, py: number, x1: number, y1: number, x2: number, y2: number): number {
  const dx = x2 - x1
  const dy = y2 - y1
  const len2 = dx * dx + dy * dy
  if (len2 === 0) return Math.hypot(px - x1, py - y1)
  let t = ((px - x1) * dx + (py - y1) * dy) / len2
  t = Math.max(0, Math.min(1, t))
  return Math.hypot(px - (x1 + t * dx), py - (y1 + t * dy))
}

/** Попадание курсора/ластика в элемент (radius — радиус стирания в px). */
export function hitTestElement(el: BoardElement, x: number, y: number, radius: number): boolean {
  const lw = ((el.payload.lineWidth as number) || 2) / 2

  if (el.element_type === 'stroke') {
    const pts = el.payload.points as number[][] | undefined
    if (!pts || pts.length === 0) return false
    if (pts.length === 1) return Math.hypot(x - pts[0][0], y - pts[0][1]) <= radius + lw
    for (let i = 1; i < pts.length; i++) {
      if (distToSegment(x, y, pts[i - 1][0], pts[i - 1][1], pts[i][0], pts[i][1]) <= radius + lw) {
        return true
      }
    }
    return false
  }

  if (el.element_type === 'line') {
    const p = el.payload as { x1: number; y1: number; x2: number; y2: number }
    return distToSegment(x, y, p.x1, p.y1, p.x2, p.y2) <= radius + lw
  }

  if (el.element_type === 'rect') {
    const p = el.payload as { x: number; y: number; w: number; h: number }
    const pad = radius + lw
    return x >= p.x - pad && x <= p.x + p.w + pad && y >= p.y - pad && y <= p.y + p.h + pad
  }

  if (el.element_type === 'circle') {
    const p = el.payload as { cx: number; cy: number; r: number }
    const r = Math.max(1, Math.abs(p.r))
    return Math.hypot(x - p.cx, y - p.cy) <= r + radius + lw
  }

  return false
}
