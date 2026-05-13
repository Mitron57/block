const API_BASE = import.meta.env.VITE_API_URL ?? ''

export function getToken(): string | null {
  return localStorage.getItem('token')
}

export function setToken(token: string | null) {
  if (token) localStorage.setItem('token', token)
  else localStorage.removeItem('token')
}

export async function api<T = unknown>(
  path: string,
  init: RequestInit = {},
): Promise<T> {
  const headers = new Headers(init.headers)
  const token = getToken()
  if (token) headers.set('Authorization', `Bearer ${token}`)
  if (init.body && !(init.body instanceof FormData) && !headers.has('Content-Type')) {
    headers.set('Content-Type', 'application/json')
  }
  const url = `${API_BASE}${path}`
  const res = await fetch(url, { ...init, headers })
  if (!res.ok) {
    let msg = res.statusText
    try {
      const j = (await res.json()) as { error?: string }
      if (j.error) msg = j.error
    } catch {
      /* ignore */
    }
    throw new Error(msg)
  }
  if (res.status === 204) return undefined as T
  const ct = res.headers.get('content-type')
  if (ct?.includes('application/json')) return (await res.json()) as T
  return (await res.text()) as T
}
