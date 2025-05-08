import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

// https://vitejs.dev/config/
export default defineConfig(async () => ({
  plugins: [react()],

  // Vite options tailored for Tauri development
  clearScreen: false,
  server: {
    port: 3000,
    strictPort: true,
  },
  build: {
    // Tauri uses Chromium on Windows and WebKit on macOS and Linux
    target: ['es2021', 'chrome100', 'safari13'],
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false,
    sourcemap: !!process.env.TAURI_DEBUG,
    
    // Optimize for fast startup
    rollupOptions: {
      output: {
        // Create chunk sizes optimized for fast startup
        manualChunks: {
          'vendor': ['react', 'react-dom'],
          'shell': ['./src/components/Shell.tsx'],
        }
      }
    },
  },
  
  // Optimize preload strategy
  optimizeDeps: {
    include: ['react', 'react-dom'],
  },
}));