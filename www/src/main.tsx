import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import { BrowserRouter } from 'react-router-dom'
import { Provider as JotaiProvider } from 'jotai'
import { QueryClient, QueryClientProvider } from '@tanstack/react-query'
import './index.css'
import App from './App'

const queryClient = new QueryClient()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <BrowserRouter>
      <JotaiProvider>
        <QueryClientProvider client={queryClient}>
          <App />
        </QueryClientProvider>
      </JotaiProvider>
    </BrowserRouter>
  </StrictMode>,
)
