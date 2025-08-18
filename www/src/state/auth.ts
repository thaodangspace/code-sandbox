import { atom } from 'jotai'

export interface AuthState {
  isAuthenticated: boolean
  token?: string
}

export const authAtom = atom<AuthState>({ isAuthenticated: false })
