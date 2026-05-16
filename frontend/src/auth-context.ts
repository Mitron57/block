import { createContext } from 'react'

export type AuthCtx = {
  token: string | null
  setToken: (t: string | null) => void
}

export const AuthContext = createContext<AuthCtx | null>(null)
