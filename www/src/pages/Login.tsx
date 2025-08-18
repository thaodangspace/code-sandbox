import { useAtom } from 'jotai'
import { authAtom } from '../state/auth'

export default function Login() {
  const [, setAuth] = useAtom(authAtom)

  const handleSubmit = (e: React.FormEvent) => {
    e.preventDefault()
    setAuth({ isAuthenticated: true, token: 'demo' })
  }

  return (
    <div className="flex items-center justify-center min-h-screen">
      <form onSubmit={handleSubmit} className="p-4 space-y-4 bg-white rounded shadow">
        <h1 className="text-xl font-bold">Login</h1>
        <input className="w-full p-2 border" type="email" placeholder="email" />
        <input className="w-full p-2 border" type="password" placeholder="password" />
        <button className="w-full p-2 text-white bg-blue-500" type="submit">
          Sign In
        </button>
      </form>
    </div>
  )
}
