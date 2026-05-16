import { useCallback, useMemo, useState, type ReactNode } from 'react'
import { setToken as persistToken, getToken } from './api'
import { AuthContext } from './auth-context'

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setTok] = useState<string | null>(() => getToken())
  const setToken = useCallback((t: string | null) => {
    persistToken(t)
    setTok(t)
  }, [])
  const v = useMemo(() => ({ token, setToken }), [token, setToken])
  return <AuthContext.Provider value={v}>{children}</AuthContext.Provider>
}
