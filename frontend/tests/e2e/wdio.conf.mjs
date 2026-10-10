// WebdriverIO config for the Windows end-to-end suite (Tech Stack 3, Code Standards 9.2).
// tauri-driver (installed by the Windows workflow with `cargo install tauri-driver`) starts the
// built app and forwards WebDriver commands to WebView2 through the Edge driver.

import { spawn } from 'node:child_process';
import { mkdirSync } from 'node:fs';
import os from 'node:os';
import path from 'node:path';
import { APP_BINARY, OUTPUT_DIR, SCREENSHOT_DIR } from './paths.mjs';

const TAURI_DRIVER = path.join(
  os.homedir(),
  '.cargo',
  'bin',
  process.platform === 'win32' ? 'tauri-driver.exe' : 'tauri-driver',
);
const TAURI_DRIVER_PORT = 4444;
const TEST_TIMEOUT_MS = 120_000;

/**
 * `tauri:options` is tauri-driver's own capability, which WebdriverIO's types do not list.
 * @type {WebdriverIO.Capabilities}
 */
const TAURI_CAPABILITIES = /** @type {WebdriverIO.Capabilities} */ (
  /** @type {unknown} */ ({ 'tauri:options': { application: APP_BINARY } })
);

/** @type {import('node:child_process').ChildProcess | undefined} */
let tauriDriver;
let stopping = false;

function stopTauriDriver() {
  stopping = true;
  tauriDriver?.kill();
}

// afterSession does not run when a session fails to start, so also stop the driver on exit.
for (const signal of ['exit', 'SIGINT', 'SIGTERM']) {
  process.on(signal, stopTauriDriver);
}

/** @type {WebdriverIO.Config} */
export const config = {
  runner: 'local',
  specs: ['./specs/**/*.e2e.mjs'],
  maxInstances: 1,
  hostname: '127.0.0.1',
  port: TAURI_DRIVER_PORT,
  capabilities: [TAURI_CAPABILITIES],
  logLevel: 'warn',
  outputDir: path.join(OUTPUT_DIR, 'logs'),
  framework: 'mocha',
  mochaOpts: { ui: 'bdd', timeout: TEST_TIMEOUT_MS },
  reporters: ['spec', ['junit', { outputDir: OUTPUT_DIR, outputFileFormat: () => 'junit.xml' }]],

  onPrepare() {
    mkdirSync(SCREENSHOT_DIR, { recursive: true });
  },

  beforeSession() {
    tauriDriver = spawn(TAURI_DRIVER, [], { stdio: ['ignore', 'inherit', 'inherit'] });
    tauriDriver.on('error', (error) => {
      console.error(`e2e: could not start tauri-driver at ${TAURI_DRIVER}: ${error.message}`);
      process.exit(1);
    });
    tauriDriver.on('exit', (code) => {
      if (!stopping) {
        console.error(`e2e: tauri-driver stopped early with code ${String(code)}`);
        process.exit(1);
      }
    });
  },

  afterSession() {
    stopTauriDriver();
  },
};
