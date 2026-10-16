// Paths shared by the end-to-end runner and the WebdriverIO config.
// Folder names can contain spaces, so every path is built with path.join, never by hand.

import { existsSync } from 'node:fs';
import path from 'node:path';
import { fileURLToPath } from 'node:url';

export const E2E_DIR = path.dirname(fileURLToPath(import.meta.url));
export const FRONTEND_DIR = path.resolve(E2E_DIR, '..', '..');
export const ROOT_DIR = path.resolve(FRONTEND_DIR, '..');

const RELEASE_DIR = path.join(ROOT_DIR, 'backend', 'target', 'release');
const EXE = process.platform === 'win32' ? '.exe' : '';

/**
 * Where `pnpm tauri build` may leave the app: the cargo binary name (mandi-app), or the
 * productName from tauri.conf.json when the Tauri CLI renames it ("Mandi Software").
 */
export const APP_BINARY_CANDIDATES = [
  path.join(RELEASE_DIR, `mandi-app${EXE}`),
  path.join(RELEASE_DIR, `Mandi Software${EXE}`),
];

/** The app the suite drives: the first candidate that exists, else the cargo name. */
export const APP_BINARY =
  APP_BINARY_CANDIDATES.find((candidate) => existsSync(candidate)) ??
  APP_BINARY_CANDIDATES[0] ??
  '';

/** Report, screenshots and logs. The Windows workflow uploads this folder. */
export const OUTPUT_DIR = path.join(E2E_DIR, 'output');
export const SCREENSHOT_DIR = path.join(OUTPUT_DIR, 'screenshots');
