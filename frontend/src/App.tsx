import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import { BrowserRouter, Navigate, Route, Routes } from 'react-router-dom'
import { AuthProvider } from './auth'
import { useAuth } from './useAuth'
import { BoardPage } from './pages/BoardPage'
import { BoardsPage } from './pages/BoardsPage'
import { LoginPage } from './pages/LoginPage'
import { RegisterPage } from './pages/RegisterPage'
import './App.css'

const queryClient = new QueryClient()

function Private({ children }: { children: React.ReactNode }) {
  const { token } = useAuth()
  if (!token) return <Navigate to="/login" replace />
  return <>{children}</>
}

export default function App() {
  return (
    <QueryClientProvider client={queryClient}>
      <AuthProvider>
        <BrowserRouter>
          <Routes>
            <Route path="/login" element={<LoginPage />} />
            <Route path="/register" element={<RegisterPage />} />
            <Route
              path="/boards"
              element={
                <Private>
                  <BoardsPage />
                </Private>
              }
            />
            <Route
              path="/boards/:id"
              element={
                <Private>
                  <BoardPage />
                </Private>
              }
            />
            <Route path="*" element={<Navigate to="/boards" replace />} />
          </Routes>
        </BrowserRouter>
      </AuthProvider>
    </QueryClientProvider>
  )
}
