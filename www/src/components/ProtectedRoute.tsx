import { ReactNode } from 'react'
import { Navigate } from 'react-router-dom'
import { useAtom } from 'jotai'
import { authAtom } from '../state/auth'

interface Props {
  children: ReactNode
}

export default function ProtectedRoute({ children }: Props) {
  const [auth] = useAtom(authAtom)
  if (!auth.isAuthenticated) {
    return <Navigate to="/login" replace />
  }
  return <>{children}</>
}
