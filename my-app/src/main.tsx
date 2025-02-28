import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import Update from './update.tsx'
import "./mycss.css"; // Import your CSS file here

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Update />
  </StrictMode>,
)
