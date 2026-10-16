// pnpm e2e: the Windows end-to-end suite (WebdriverIO + tauri-driver, Code Standards 9.2).
// tauri-driver drives WebView2 through the Edge driver, which exists only on Windows. On any
// other computer this prints a message and exits 0, so `pnpm e2e` never blocks a Mac.

import { spawnSync } from 'node:child_process';
import { existsSync, rmSync } from 'node:fs';
import path from 'node:path';
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

// The suite is one story that starts at the first-run wizard, so the computer must hold no app data.
// On the CI runner it never does, and a leftover is removed. On any other Windows computer the data
// might be real (the Owner's own books), so it is never deleted unless E2E_FRESH=1 says so.
const appData = path.join(process.env.APPDATA ?? '', 'pk.mandisoftware.app');
if (process.env.APPDATA !== undefined && existsSync(appData)) {
  if (process.env.CI === 'true' || process.env.E2E_FRESH === '1') {
    rmSync(appData, { recursive: true, force: true });
    console.log(`e2e: removed the old app data at ${appData}`);
  } else {
    console.error(
      `e2e: this computer already has app data at ${appData}. The suite starts at the first-run wizard, ` +
        'so it needs a computer with none. Use a clean computer, or set E2E_FRESH=1 to delete this folder first.',
    );
    process.exit(1);
  }
}

console.log(`e2e: testing ${APP_BINARY}`);
const result = spawnSync('pnpm', ['exec', 'wdio', 'run', 'tests/e2e/wdio.conf.mjs'], {
  cwd: FRONTEND_DIR,
  stdio: 'inherit',
  shell: true,
});
process.exit(result.status ?? 1);
