export type BoardRole = 'owner' | 'editor' | 'viewer'

export interface User {
  id: string
  email: string
  display_name: string
}

export interface Board {
  id: string
  owner_id: string
  title: string
  created_at: string
}

export interface BoardElement {
  id: string
  board_id: string
  element_type: string
  payload: Record<string, unknown>
  z_index: number
  created_at: string
}

export interface Member {
  user_id: string
  email: string
  display_name: string
  role: BoardRole
}

export type ServerWsMessage =
  | { op: 'snapshot'; elements: BoardElement[] }
  | { op: 'element_added'; element: BoardElement }
  | { op: 'element_removed'; id: string }
  | { op: 'cleared' }
  | { op: 'error'; message: string }
