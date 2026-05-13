import { useState, type FormEvent } from 'react'
import { Link, useNavigate } from 'react-router-dom'
import { api } from '../api'
import { useAuth } from '../auth'

export function RegisterPage() {
  const nav = useNavigate()
  const { setToken } = useAuth()
  const [email, setEmail] = useState('')
  const [password, setPassword] = useState('')
  const [displayName, setDisplayName] = useState('')
  const [err, setErr] = useState<string | null>(null)

  async function onSubmit(e: FormEvent) {
    e.preventDefault()
    setErr(null)
    try {
      const r = await api<{ token: string }>('/api/auth/register', {
        method: 'POST',
        body: JSON.stringify({
          email,
          password,
          display_name: displayName,
        }),
      })
      setToken(r.token)
      nav('/boards')
    } catch (e) {
      setErr(e instanceof Error ? e.message : 'register failed')
    }
  }

  return (
    <div className="auth-page">
      <div className="auth">
        <h1>Регистрация</h1>
        <form onSubmit={onSubmit}>
          <label>
            Имя
            <input value={displayName} onChange={(e) => setDisplayName(e.target.value)} required />
          </label>
          <label>
            Email
            <input value={email} onChange={(e) => setEmail(e.target.value)} type="email" required />
          </label>
          <label>
            Пароль (≥ 8 символов)
            <input
              value={password}
              onChange={(e) => setPassword(e.target.value)}
              type="password"
              minLength={8}
              required
            />
          </label>
          {err && <p className="error">{err}</p>}
          <button type="submit">Создать</button>
        </form>
        <p>
          Уже есть аккаунт? <Link to="/login">Вход</Link>
        </p>
      </div>
    </div>
  )
}
