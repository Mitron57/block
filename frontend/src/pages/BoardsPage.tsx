import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'
import { useState, type FormEvent } from 'react'
import { Link } from 'react-router-dom'
import { api } from '../api'
import { useAuth } from '../useAuth'
import type { Board } from '../types'

export function BoardsPage() {
  const qc = useQueryClient()
  const { setToken } = useAuth()
  const [title, setTitle] = useState('')
  const [createErr, setCreateErr] = useState<string | null>(null)
  const [deleteErr, setDeleteErr] = useState<string | null>(null)

  const { data: me } = useQuery({
    queryKey: ['me'],
    queryFn: () => api<{ id: string }>('/api/me'),
  })

  const { data: boards, error, isLoading } = useQuery({
    queryKey: ['boards'],
    queryFn: () => api<Board[]>('/api/boards'),
  })

  const create = useMutation({
    mutationFn: (t: string) =>
      api<Board>('/api/boards', { method: 'POST', body: JSON.stringify({ title: t }) }),
    onSuccess: () => {
      setCreateErr(null)
      qc.invalidateQueries({ queryKey: ['boards'] })
    },
  })

  const remove = useMutation({
    mutationFn: (id: string) => api<void>(`/api/boards/${id}`, { method: 'DELETE' }),
    onSuccess: () => {
      setDeleteErr(null)
      qc.invalidateQueries({ queryKey: ['boards'] })
    },
  })

  async function onCreate(e: FormEvent) {
    e.preventDefault()
    const t = title.trim()
    if (!t) {
      setCreateErr('Введите название доски')
      return
    }
    setCreateErr(null)
    try {
      await create.mutateAsync(t)
      setTitle('')
    } catch (err) {
      setCreateErr(err instanceof Error ? err.message : 'Не удалось создать доску')
    }
  }

  async function onDelete(board: Board) {
    if (!window.confirm(`Удалить доску «${board.title}»?`)) return
    setDeleteErr(null)
    try {
      await remove.mutateAsync(board.id)
    } catch (err) {
      setDeleteErr(err instanceof Error ? err.message : 'Не удалось удалить доску')
    }
  }

  return (
    <div className="page">
      <header className="boards-header">
        <h1>Доски</h1>
        <button type="button" className="linkish" onClick={() => setToken(null)}>
          Выйти
        </button>
      </header>
      {error && <p className="error">{(error as Error).message}</p>}
      <form onSubmit={onCreate} className="row">
        <input
          placeholder="Название новой доски"
          value={title}
          onChange={(e) => setTitle(e.target.value)}
          required
          minLength={1}
          disabled={create.isPending}
        />
        <button type="submit" disabled={create.isPending}>
          {create.isPending ? 'Создание…' : 'Создать'}
        </button>
      </form>
      {createErr && <p className="error">{createErr}</p>}
      {deleteErr && <p className="error">{deleteErr}</p>}
      {isLoading && <p>Загрузка…</p>}
      <ul className="board-list">
        {boards?.map((b) => {
          const isOwner = me?.id === b.owner_id
          return (
            <li key={b.id} className="board-list-item">
              <Link to={`/boards/${b.id}`} className="board-list-title">
                {b.title}
              </Link>
              {isOwner && (
                <button
                  type="button"
                  className="btn-danger btn-sm"
                  disabled={remove.isPending}
                  onClick={() => onDelete(b)}
                >
                  Удалить
                </button>
              )}
            </li>
          )
        })}
      </ul>
      {!isLoading && boards?.length === 0 && (
        <p className="muted">Пока нет досок — создайте первую выше.</p>
      )}
    </div>
  )
}
