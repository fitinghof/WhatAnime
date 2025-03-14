import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react-swc'
/// <reference types="vite/client" />

// https://vite.dev/config/
export default defineConfig({
  server: {
    host: true,
    port: 5173,
    proxy: {
      "/api": {
        target: "http://whatanime.ddns.net:8000",
        changeOrigin: true,
        secure: false,
      }
    },
    allowedHosts: ['whatanime.ddns.net'],
  },
  plugins: [react()],
})
