// Vite builds the screens that Tauri shows (Tech Stack 3 and 5.5).
// Tauri's dev URL is http://localhost:5173, so the port is fixed and must not move.
import tailwindcss from '@tailwindcss/vite';
import react from '@vitejs/plugin-react';
import { configDefaults, defineConfig } from 'vitest/config';

const DEV_PORT = 5173;
// A whole test, with every wait inside it, gets this long (D1+D2 review, finding 26).
const TEST_TIMEOUT_MS = 30_000;
const HOOK_TIMEOUT_MS = 30_000;

// These files type into pickers, one picker after another, with the backend's real answers, or draw a
// long list. On a machine that is also running the other test files they take several times longer,
// and a click on a list that is drawn again at that moment is lost. They run alone, one file at a time,
// after the other files, with more room per test (`pnpm test` runs the project `main` and then
// `sensitive`; a plain `vitest run` also puts `sensitive` after `main`, see `sequence.groupOrder`).
// The fruit helper also tries a lost pick again (tests/unit/fruit-real-answers.test.tsx).
// The trip file is here because its first test, a purchase of three lines typed with the keyboard
// only, passes alone and as a whole file (57 of 57) and fails only when the whole suite runs at once.
// The six files after it (D8 end gate) failed the same way and only that way: in a full run on a
// machine with a load average above 100 their tests ended with "Test timed out in 30000ms" or a
// default one-second wait, and every one of them passed when its file ran alone at that same load.
// Any other failure in these files, a wrong value or a missing element after the wait, is still a
// failure: the room is only for a wait that is slow.
const LOAD_SENSITIVE_TESTS = [
  'tests/unit/fruit-real-answers.test.tsx',
  'tests/unit/bikri-edit-rules.test.tsx',
  'tests/unit/bikri-later-sale.test.tsx',
  'tests/unit/trips-trip.test.tsx',
  'tests/unit/bikri-save.test.tsx',
  'tests/unit/bikri-edit-cancel.test.tsx',
  'tests/unit/bikri-states.test.tsx',
  'tests/unit/bikri-strip.test.tsx',
  'tests/unit/bikri-whole-view.test.tsx',
  'tests/unit/reports-screens-round8.test.tsx',
];
const SENSITIVE_TEST_TIMEOUT_MS = 90_000;

const TEST_FILES = [
  'tests/unit/**/*.test.{ts,tsx}',
  'tests/robustness/**/*.test.{ts,tsx}',
  'src/**/*.test.{ts,tsx}',
];
const COMMON_TEST = {
  environment: 'jsdom' as const,
  setupFiles: ['./tests/setup.ts'],
  css: false,
  testTimeout: TEST_TIMEOUT_MS,
  hookTimeout: HOOK_TIMEOUT_MS,
};

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
    // The settings every test file has; the two projects below add what makes them different.
    projects: [
      {
        extends: true,
        test: {
          ...COMMON_TEST,
          name: 'main',
          include: TEST_FILES,
          exclude: [...configDefaults.exclude, ...LOAD_SENSITIVE_TESTS],
        },
      },
      {
        extends: true,
        test: {
          ...COMMON_TEST,
          name: 'sensitive',
          setupFiles: [...COMMON_TEST.setupFiles, './tests/setup-sensitive.ts'],
          include: LOAD_SENSITIVE_TESTS,
          fileParallelism: false,
          testTimeout: SENSITIVE_TEST_TIMEOUT_MS,
          // After the project `main`, so a plain `vitest run` does not run them beside the other files.
          sequence: { groupOrder: 1 },
        },
      },
    ],
  },
});
