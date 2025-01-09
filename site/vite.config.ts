import {defineConfig, loadEnv} from 'vite'
import {resolve} from 'path'
import react from '@vitejs/plugin-react'

export default defineConfig(({mode}) => {
  const env = loadEnv(mode, process.cwd())
  if (mode == "development") {
    return {
      plugins: [react()],
      server: {
        host: '0.0.0.0',
        port: Number(env.VITE_PORT || "3033"),
      },
    }
  }

  return {
    plugins: [react()],
    build: {
      rollupOptions: {
        input: {
          main: resolve(__dirname, 'index.html'),
          about: resolve(__dirname, 'about', 'index.html'),
        }
      }
    },
  }
})
