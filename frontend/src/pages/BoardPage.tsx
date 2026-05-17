import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useCallback, useEffect, useMemo, useRef, useState, type FormEvent } from 'react'
import { Link, useNavigate, useParams } from 'react-router-dom'
import { api, getToken } from '../api'
import { hitTestElement } from '../boardHitTest'
import type { BoardElement, BoardRole, Member, ServerWsMessage } from '../types'

type Tool = 'pen' | 'line' | 'rect' | 'circle' | 'eraser'

const COLORS = [
  '#1a1a1a', '#ef4444', '#3b82f6', '#22c55e',
  '#f97316', '#a855f7', '#eab308', '#06b6d4',
]
const STROKE_WIDTHS = [2, 4, 8]

const ASSIGNABLE_ROLES: BoardRole[] = ['editor', 'viewer']

const ROLE_LABELS: Record<BoardRole, string> = {
  owner: 'владелец',
  editor: 'редактор',
  viewer: 'наблюдатель',
}

function drawElement(ctx: CanvasRenderingContext2D, el: BoardElement) {
  const color = (el.payload.color as string) || '#1a1a1a'
  const lw = (el.payload.lineWidth as number) || 2
  ctx.strokeStyle = color
  ctx.lineWidth = lw
  ctx.lineCap = 'round'
  ctx.lineJoin = 'round'

  if (el.element_type === 'stroke') {
    const pts = el.payload.points as number[][] | undefined
    if (!pts || pts.length < 2) return
    ctx.beginPath()
    ctx.moveTo(pts[0][0], pts[0][1])
    for (let i = 1; i < pts.length; i++) ctx.lineTo(pts[i][0], pts[i][1])
    ctx.stroke()
  } else if (el.element_type === 'line') {
    const p = el.payload as { x1: number; y1: number; x2: number; y2: number }
    ctx.beginPath()
    ctx.moveTo(p.x1, p.y1)
    ctx.lineTo(p.x2, p.y2)
    ctx.stroke()
  } else if (el.element_type === 'rect') {
    const p = el.payload as { x: number; y: number; w: number; h: number }
    ctx.strokeRect(p.x, p.y, p.w, p.h)
  } else if (el.element_type === 'circle') {
    const p = el.payload as { cx: number; cy: number; r: number }
    ctx.beginPath()
    ctx.arc(p.cx, p.cy, Math.max(1, Math.abs(p.r)), 0, Math.PI * 2)
    ctx.stroke()
  }
}

function redraw(
  ctx: CanvasRenderingContext2D,
  elements: BoardElement[],
  w: number,
  h: number,
  preview?: BoardElement,
  eraserAt?: [number, number],
  eraserRadius?: number,
) {
  ctx.fillStyle = '#ffffff'
  ctx.fillRect(0, 0, w, h)
  const sorted = [...elements].sort((a, b) => a.z_index - b.z_index)
  for (const el of sorted) drawElement(ctx, el)
  if (preview) drawElement(ctx, preview)
  if (eraserAt && eraserRadius) {
    const [ex, ey] = eraserAt
    ctx.beginPath()
    ctx.arc(ex, ey, eraserRadius, 0, Math.PI * 2)
    ctx.fillStyle = 'rgba(239, 68, 68, 0.15)'
    ctx.fill()
    ctx.strokeStyle = 'rgba(239, 68, 68, 0.5)'
    ctx.lineWidth = 1
    ctx.stroke()
  }
}

function relPos(e: React.PointerEvent<HTMLCanvasElement>, c: HTMLCanvasElement): [number, number] {
  const r = c.getBoundingClientRect()
  const scaleX = c.width / r.width
  const scaleY = c.height / r.height
  return [(e.clientX - r.left) * scaleX, (e.clientY - r.top) * scaleY]
}

export function BoardPage() {
  const { id } = useParams<{ id: string }>()
  const boardId = id!
  const nav = useNavigate()
  const qc = useQueryClient()
  const canvasRef = useRef<HTMLCanvasElement>(null)
  const wsRef = useRef<WebSocket | null>(null)
  const [elements, setElements] = useState<BoardElement[]>([])
  const [wsState, setWsState] = useState<'connecting' | 'open' | 'closed'>('closed')

  const [tool, setTool] = useState<Tool>('pen')
  const [color, setColor] = useState(COLORS[0])
  const [strokeWidth, setStrokeWidth] = useState(STROKE_WIDTHS[0])
  const [inviteEmail, setInviteEmail] = useState('')
  const [inviteRole, setInviteRole] = useState<BoardRole>('editor')
  const [memberErr, setMemberErr] = useState<string | null>(null)

  // drawing state
  const drawingRef = useRef(false)
  const startRef = useRef<[number, number]>([0, 0])
  const pointsRef = useRef<number[][]>([])
  const zIndexRef = useRef(0)
  const erasedIdsRef = useRef<Set<string>>(new Set())
  const eraserPosRef = useRef<[number, number] | null>(null)

  const { data: board } = useQuery({
    queryKey: ['board', boardId],
    queryFn: () => api<{ id: string; title: string; owner_id: string }>(`/api/boards/${boardId}`),
  })

  const { data: me } = useQuery({
    queryKey: ['me'],
    queryFn: () => api<{ id: string }>('/api/me'),
  })

  const { data: members } = useQuery({
    queryKey: ['members', boardId],
    queryFn: () => api<Member[]>(`/api/boards/${boardId}/members`),
  })

  const role: BoardRole | null = useMemo(() => {
    if (!me || !members) return null
    return members.find((m) => m.user_id === me.id)?.role ?? null
  }, [me, members])

  const canEdit = role === 'owner' || role === 'editor'
  const isOwner = role === 'owner'
  const canManageMembers = isOwner

  const invalidateMembers = () => qc.invalidateQueries({ queryKey: ['members', boardId] })

  const addMember = useMutation({
    mutationFn: ({ email, role }: { email: string; role: BoardRole }) =>
      api<void>(`/api/boards/${boardId}/members`, {
        method: 'POST',
        body: JSON.stringify({ email, role }),
      }),
    onSuccess: () => {
      setMemberErr(null)
      invalidateMembers()
    },
  })

  const setMemberRole = useMutation({
    mutationFn: ({ userId, role }: { userId: string; role: BoardRole }) =>
      api<void>(`/api/boards/${boardId}/members/${userId}`, {
        method: 'PATCH',
        body: JSON.stringify({ role }),
      }),
    onSuccess: () => {
      setMemberErr(null)
      invalidateMembers()
    },
  })

  const removeMember = useMutation({
    mutationFn: (userId: string) =>
      api<void>(`/api/boards/${boardId}/members/${userId}`, { method: 'DELETE' }),
    onSuccess: () => {
      setMemberErr(null)
      invalidateMembers()
    },
  })

  const membersBusy =
    addMember.isPending || setMemberRole.isPending || removeMember.isPending

  async function onInviteMember(e: FormEvent) {
    e.preventDefault()
    if (!canManageMembers) return
    const email = inviteEmail.trim()
    if (!email) {
      setMemberErr('Введите email пользователя')
      return
    }
    setMemberErr(null)
    try {
      await addMember.mutateAsync({ email, role: inviteRole })
      setInviteEmail('')
    } catch (err) {
      setMemberErr(err instanceof Error ? err.message : 'Не удалось добавить участника')
    }
  }

  async function onChangeMemberRole(userId: string, newRole: BoardRole) {
    if (!canManageMembers || !ASSIGNABLE_ROLES.includes(newRole)) return
    setMemberErr(null)
    try {
      await setMemberRole.mutateAsync({ userId, role: newRole })
    } catch (err) {
      setMemberErr(err instanceof Error ? err.message : 'Не удалось изменить роль')
    }
  }

  async function onRemoveMember(m: Member) {
    if (!canManageMembers || m.role === 'owner') return
    if (!window.confirm(`Исключить ${m.display_name} с доски?`)) return
    setMemberErr(null)
    try {
      await removeMember.mutateAsync(m.user_id)
    } catch (err) {
      setMemberErr(err instanceof Error ? err.message : 'Не удалось удалить участника')
    }
  }

  const deleteBoard = useMutation({
    mutationFn: () => api<void>(`/api/boards/${boardId}`, { method: 'DELETE' }),
    onSuccess: () => {
      qc.invalidateQueries({ queryKey: ['boards'] })
      nav('/boards')
    },
  })

  async function onDeleteBoard() {
    if (!board || !isOwner) return
    if (!window.confirm(`Удалить доску «${board.title}»?`)) return
    try {
      await deleteBoard.mutateAsync()
    } catch (e) {
      alert(e instanceof Error ? e.message : 'Не удалось удалить доску')
    }
  }

  useEffect(() => {
    const token = getToken()
    if (!token) return
    let active = true
    const proto = window.location.protocol === 'https:' ? 'wss:' : 'ws:'
    const u = new URL(`/api/boards/${boardId}/ws`, window.location.origin)
    u.protocol = proto
    u.searchParams.set('token', token)
    queueMicrotask(() => {
      if (active) setWsState('connecting')
    })
    const ws = new WebSocket(u.toString())
    wsRef.current = ws
    ws.onopen = () => {
      if (active) setWsState('open')
    }
    ws.onclose = () => {
      if (active) setWsState('closed')
    }
    ws.onmessage = (ev) => {
      try {
        const msg = JSON.parse(String(ev.data)) as ServerWsMessage
        if (msg.op === 'snapshot') {
          setElements(msg.elements)
          zIndexRef.current = msg.elements.reduce((m, e) => Math.max(m, e.z_index), 0)
        } else if (msg.op === 'element_added') {
          setElements((prev) => {
            const next = [...prev.filter((e) => e.id !== msg.element.id), msg.element]
            zIndexRef.current = Math.max(zIndexRef.current, msg.element.z_index)
            return next
          })
        } else if (msg.op === 'element_removed') {
          setElements((prev) => prev.filter((e) => e.id !== msg.id))
        } else if (msg.op === 'cleared') {
          setElements([])
          zIndexRef.current = 0
        } else if (msg.op === 'error') {
          console.warn(msg.message)
        }
      } catch { /* ignore */ }
    }
    return () => {
      active = false
      ws.close()
      wsRef.current = null
      setWsState('closed')
    }
  }, [boardId])

  // full redraw when elements change (no preview during idle)
  useEffect(() => {
    const c = canvasRef.current
    if (!c) return
    const ctx = c.getContext('2d')
    if (!ctx) return
    if (!drawingRef.current) redraw(ctx, elements, c.width, c.height)
  }, [elements])

  const getPreview = useCallback(
    (cur: [number, number]): BoardElement => {
      const [sx, sy] = startRef.current
      const [cx, cy] = cur
      const base = {
        id: '__preview__',
        board_id: boardId,
        z_index: zIndexRef.current + 1,
        created_at: '',
        element_type: tool as string,
        payload: {} as Record<string, unknown>,
      }
      if (tool === 'pen') {
        base.element_type = 'stroke'
        base.payload = { points: pointsRef.current, color, lineWidth: strokeWidth }
      } else if (tool === 'line') {
        base.payload = { x1: sx, y1: sy, x2: cx, y2: cy, color, lineWidth: strokeWidth }
      } else if (tool === 'rect') {
        base.payload = { x: Math.min(sx, cx), y: Math.min(sy, cy), w: Math.abs(cx - sx), h: Math.abs(cy - sy), color, lineWidth: strokeWidth }
      } else if (tool === 'circle') {
        const r = Math.sqrt((cx - sx) ** 2 + (cy - sy) ** 2)
        base.payload = { cx: sx, cy: sy, r, color, lineWidth: strokeWidth }
      }
      return base as BoardElement
    },
    [tool, color, strokeWidth, boardId],
  )

  const eraserRadius = strokeWidth * 5

  const eraseAt = useCallback(
    (pos: [number, number]) => {
      const sorted = [...elements].sort((a, b) => b.z_index - a.z_index)
      for (const el of sorted) {
        if (erasedIdsRef.current.has(el.id)) continue
        if (!hitTestElement(el, pos[0], pos[1], eraserRadius)) continue
        erasedIdsRef.current.add(el.id)
        if (wsRef.current?.readyState === WebSocket.OPEN) {
          wsRef.current.send(JSON.stringify({ op: 'remove_element', id: el.id }))
        }
      }
    },
    [elements, eraserRadius],
  )

  function onPointerDown(e: React.PointerEvent<HTMLCanvasElement>) {
    if (!canEdit) return
    const c = canvasRef.current
    if (!c) return
    c.setPointerCapture(e.pointerId)
    const pos = relPos(e, c)
    drawingRef.current = true
    startRef.current = pos
    pointsRef.current = [pos]

    if (tool === 'eraser') {
      erasedIdsRef.current = new Set()
      eraserPosRef.current = pos
      eraseAt(pos)
      const ctx = c.getContext('2d')
      if (ctx) redraw(ctx, elements, c.width, c.height, undefined, pos, eraserRadius)
    }
  }

  function onPointerMove(e: React.PointerEvent<HTMLCanvasElement>) {
    if (!drawingRef.current) return
    const c = canvasRef.current
    if (!c) return
    const ctx = c.getContext('2d')
    if (!ctx) return
    const pos = relPos(e, c)

    if (tool === 'eraser') {
      eraserPosRef.current = pos
      eraseAt(pos)
      redraw(ctx, elements, c.width, c.height, undefined, pos, eraserRadius)
      return
    }

    if (tool === 'pen') pointsRef.current = [...pointsRef.current, pos]
    const preview = getPreview(pos)
    redraw(ctx, elements, c.width, c.height, preview)
  }

  function onPointerUp(e: React.PointerEvent<HTMLCanvasElement>) {
    if (!drawingRef.current) return
    try { canvasRef.current?.releasePointerCapture(e.pointerId) } catch { /* ignore */ }
    drawingRef.current = false
    eraserPosRef.current = null

    const c = canvasRef.current
    if (!c) return
    const pos = relPos(e, c)

    if (tool === 'eraser') {
      eraseAt(pos)
      const ctx = c.getContext('2d')
      if (ctx) redraw(ctx, elements, c.width, c.height)
      return
    }

    let payload: Record<string, unknown>
    let element_type: string
    const [sx, sy] = startRef.current
    const [cx, cy] = pos

    if (tool === 'pen') {
      const pts = [...pointsRef.current, pos]
      if (pts.length < 2) return
      element_type = 'stroke'
      payload = { points: pts, color, lineWidth: strokeWidth }
    } else if (tool === 'line') {
      element_type = 'line'
      payload = { x1: sx, y1: sy, x2: cx, y2: cy, color, lineWidth: strokeWidth }
    } else if (tool === 'rect') {
      if (Math.abs(cx - sx) < 3 && Math.abs(cy - sy) < 3) return
      element_type = 'rect'
      payload = { x: Math.min(sx, cx), y: Math.min(sy, cy), w: Math.abs(cx - sx), h: Math.abs(cy - sy), color, lineWidth: strokeWidth }
    } else {
      const r = Math.sqrt((cx - sx) ** 2 + (cy - sy) ** 2)
      if (r < 3) return
      element_type = 'circle'
      payload = { cx: sx, cy: sy, r, color, lineWidth: strokeWidth }
    }

    if (wsRef.current?.readyState === WebSocket.OPEN) {
      wsRef.current.send(JSON.stringify({
        op: 'add_element',
        element_type,
        payload,
        z_index: zIndexRef.current + 1,
      }))
    }

    // redraw without preview so the canvas is clean while waiting for WS echo
    const ctx = c.getContext('2d')
    if (ctx) redraw(ctx, elements, c.width, c.height)
  }

  async function clearBoard() {
    if (!canEdit) return
    await api(`/api/boards/${boardId}/elements`, { method: 'DELETE' })
    await qc.invalidateQueries({ queryKey: ['elements', boardId] })
    setElements([])
  }

  const TOOL_ICONS: Record<Tool, string> = {
    pen: '✏️',
    line: '/',
    rect: '▭',
    circle: '○',
    eraser: '🧹',
  }

  const TOOL_LABELS: Record<Tool, string> = {
    pen: 'Карандаш',
    line: 'Линия',
    rect: 'Прямоугольник',
    circle: 'Круг',
    eraser: 'Ластик',
  }

  const wsColor = wsState === 'open' ? '#22c55e' : wsState === 'connecting' ? '#f97316' : '#ef4444'

  return (
    <div className="board-layout">
      <header className="board-header">
        <Link to="/boards" className="back-link">← К списку</Link>
        <h1 className="board-title">{board?.title ?? '…'}</h1>
        <span className="ws-badge" style={{ background: wsColor }}>
          {wsState === 'open' ? 'онлайн' : wsState === 'connecting' ? 'подключение…' : 'офлайн'}
        </span>
        {isOwner && (
          <button
            type="button"
            className="btn-danger btn-sm"
            disabled={deleteBoard.isPending}
            onClick={onDeleteBoard}
          >
            Удалить доску
          </button>
        )}
      </header>

      {canEdit && (
        <div className="toolbar">
          <div className="tool-group">
            {(['pen', 'line', 'rect', 'circle', 'eraser'] as Tool[]).map((t) => (
              <button
                key={t}
                type="button"
                className={`tool-btn${tool === t ? ' active' : ''}`}
                onClick={() => setTool(t)}
                title={TOOL_LABELS[t]}
              >
                {TOOL_ICONS[t]}
              </button>
            ))}
          </div>
          {tool !== 'eraser' && (
            <>
              <div className="tool-divider" />
              <div className="tool-group">
                {COLORS.map((c) => (
                  <button
                    key={c}
                    type="button"
                    className={`color-btn${color === c ? ' active' : ''}`}
                    style={{ background: c }}
                    onClick={() => setColor(c)}
                    title={c}
                  />
                ))}
              </div>
              <div className="tool-divider" />
            </>
          )}
          <div className="tool-group">
            {STROKE_WIDTHS.map((w) => (
              <button
                key={w}
                type="button"
                className={`width-btn${strokeWidth === w ? ' active' : ''}`}
                onClick={() => setStrokeWidth(w)}
                title={tool === 'eraser' ? `Размер ластика: ${w * 5}px` : `${w}px`}
              >
                <span
                  style={{
                    display: 'block',
                    width: tool === 'eraser' ? w * 5 : 20,
                    height: tool === 'eraser' ? w * 5 : w,
                    background: tool === 'eraser' ? 'rgba(239,68,68,0.25)' : '#1a1a1a',
                    borderRadius: 999,
                    border: tool === 'eraser' ? '1px solid rgba(239,68,68,0.5)' : 'none',
                  }}
                />
              </button>
            ))}
          </div>
          <div className="tool-divider" />
          <button type="button" className="clear-btn" onClick={() => void clearBoard()}>
            Очистить
          </button>
        </div>
      )}

      {!canEdit && role && (
        <div className="viewer-notice">Роль «наблюдатель» — только просмотр</div>
      )}

      <div className="canvas-area">
        <canvas
          ref={canvasRef}
          width={1200}
          height={700}
          className={`board-canvas${canEdit ? '' : ' readonly'}${tool === 'eraser' && canEdit ? ' eraser-cursor' : ''}`}
          onPointerDown={onPointerDown}
          onPointerMove={onPointerMove}
          onPointerUp={onPointerUp}
          onPointerCancel={onPointerUp}
        />
      </div>

      <aside className="members-panel">
        <h3>Участники</h3>
        {canManageMembers && (
          <form className="member-invite" onSubmit={onInviteMember}>
            <input
              type="email"
              placeholder="email@example.com"
              value={inviteEmail}
              onChange={(e) => setInviteEmail(e.target.value)}
              disabled={membersBusy}
              required
            />
            <div className="member-invite-row">
              <select
                value={inviteRole}
                onChange={(e) => setInviteRole(e.target.value as BoardRole)}
                disabled={membersBusy}
              >
                {ASSIGNABLE_ROLES.map((r) => (
                  <option key={r} value={r}>
                    {ROLE_LABELS[r]}
                  </option>
                ))}
              </select>
              <button type="submit" className="btn-primary btn-sm" disabled={membersBusy}>
                Добавить
              </button>
            </div>
          </form>
        )}
        {memberErr && <p className="error member-error">{memberErr}</p>}
        <ul className="member-list">
          {members?.map((m) => (
            <li key={m.user_id} className="member-row">
              <div className="member-info">
                <span className="member-name">{m.display_name}</span>
                <span className="member-email">{m.email}</span>
              </div>
              <div className="member-actions">
                {canManageMembers && m.role !== 'owner' ? (
                  <>
                    <select
                      className="role-select"
                      value={m.role}
                      disabled={membersBusy}
                      onChange={(e) =>
                        void onChangeMemberRole(m.user_id, e.target.value as BoardRole)
                      }
                    >
                      {ASSIGNABLE_ROLES.map((r) => (
                        <option key={r} value={r}>
                          {ROLE_LABELS[r]}
                        </option>
                      ))}
                    </select>
                    <button
                      type="button"
                      className="btn-danger btn-sm"
                      disabled={membersBusy}
                      title="Исключить"
                      onClick={() => void onRemoveMember(m)}
                    >
                      ×
                    </button>
                  </>
                ) : (
                  <span className={`role-badge role-${m.role}`}>{ROLE_LABELS[m.role]}</span>
                )}
              </div>
            </li>
          ))}
        </ul>
      </aside>
    </div>
  )
}
