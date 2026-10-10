// pnpm e2e: the Windows end-to-end suite (WebdriverIO + tauri-driver, Code Standards 9.2).
// tauri-driver drives WebView2 through the Edge driver, which exists only on Windows. On any
// other computer this prints a message and exits 0, so `pnpm e2e` never blocks a Mac.

import { spawnSync } from 'node:child_process';
import { existsSync } from 'node:fs';
import { APP_BINARY, APP_BINARY_CANDIDATES, FRONTEND_DIR } from './paths.mjs';

if (process.platform !== 'win32') {
  console.log('e2e runs on Windows only');
  process.exit(0);
}

if (!existsSync(APP_BINARY)) {
  console.error(
    `e2e: the app is not built yet. Looked for: ${APP_BINARY_CANDIDATES.join(' | ')}. Run "pnpm tauri build" first.`,
  );
  process.exit(1);
}

console.log(`e2e: testing ${APP_BINARY}`);
const result = spawnSync('pnpm', ['exec', 'wdio', 'run', 'tests/e2e/wdio.conf.mjs'], {
  cwd: FRONTEND_DIR,
  stdio: 'inherit',
  shell: true,
});
process.exit(result.status ?? 1);
