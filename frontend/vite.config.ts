// Vite builds the screens that Tauri shows (Tech Stack 3 and 5.5).
// Tauri's dev URL is http://localhost:5173, so the port is fixed and must not move.
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { defineConfig } from 'vitest/config';

const DEV_PORT = 5173;
// A whole test, with every wait inside it, gets this long (D1+D2 review, finding 26).
const TEST_TIMEOUT_MS = 30_000;
const HOOK_TIMEOUT_MS = 30_000;

export default defineConfig({
  plugins: [react(), tailwindcss()],
  clearScreen: false,
  server: {
    port: DEV_PORT,
    strictPort: true,
    host: 'localhost',
  },
  build: {
    // WebView2 on Windows 10 and 11 is evergreen Chromium.
    target: 'chrome120',
    sourcemap: false,
  },
  test: {
    environment: 'jsdom',
    setupFiles: ['./tests/setup.ts'],
    include: [
      'tests/unit/**/*.test.{ts,tsx}',
      'tests/robustness/**/*.test.{ts,tsx}',
      'src/**/*.test.{ts,tsx}',
    ],
    css: false,
    testTimeout: TEST_TIMEOUT_MS,
    hookTimeout: HOOK_TIMEOUT_MS,
  },
});
