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

  const { data: boards, error } = useQuery({
    queryKey: ['boards'],
    queryFn: () => api<Board[]>('/api/boards'),
  })

  const create = useMutation({
    mutationFn: (t: string) =>
      api<Board>('/api/boards', { method: 'POST', body: JSON.stringify({ title: t }) }),
    onSuccess: () => qc.invalidateQueries({ queryKey: ['boards'] }),
  })

  function onCreate(e: FormEvent) {
    e.preventDefault()
    if (!title.trim()) return
    create.mutate(title.trim())
    setTitle('')
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
        />
        <button type="submit" disabled={create.isPending}>
          Создать
        </button>
      </form>
      <ul className="board-list">
        {boards?.map((b) => (
          <li key={b.id}>
            <Link to={`/boards/${b.id}`}>{b.title}</Link>
          </li>
        ))}
      </ul>
    </div>
  )
}
