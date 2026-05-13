import {
  createContext,
  useCallback,
  useContext,
  useMemo,
  useState,
  type ReactNode,
} from 'react'
import { setToken as persistToken, getToken } from './api'

type AuthCtx = {
  token: string | null
  setToken: (t: string | null) => void
}

const Ctx = createContext<AuthCtx | null>(null)

export function AuthProvider({ children }: { children: ReactNode }) {
  const [token, setTok] = useState<string | null>(() => getToken())
  const setToken = useCallback((t: string | null) => {
    persistToken(t)
    setTok(t)
  }, [])
  const v = useMemo(() => ({ token, setToken }), [token, setToken])
  return <Ctx.Provider value={v}>{children}</Ctx.Provider>
}

export function useAuth() {
  const c = useContext(Ctx)
  if (!c) throw new Error('useAuth outside provider')
  return c
}
