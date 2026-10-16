// Robustness guard for the app's security settings (plan 01 T5 item 4).
// NFR-SEC-9 (no network exposure), NFR-SEC-10 (app lockdown), NFR-REL-9 (one copy),
// Code Standards S4 (capabilities) and S5 (Content Security Policy).
//
// These tests read the real config files. They fail if a later change:
// - adds a remote source to the CSP (for example a font or script from a website);
// - gives the screens a permission outside the allow list (file system, shell, http);
// - allows a command that the backend does not register;
// - adds a Tauri plugin outside the five of Tech Stack 2, or moves single instance off first place.

import { readFileSync } from 'node:fs';
import { dirname, join, resolve } from 'node:path';
import { fileURLToPath } from 'node:url';
import { describe, expect, it } from 'vitest';

// MANDI_ROBUSTNESS_ROOT points the guard at a mutated copy, to prove each check can fail.
const ROOT =
  process.env['MANDI_ROBUSTNESS_ROOT'] ??
  resolve(dirname(fileURLToPath(import.meta.url)), '..', '..', '..');
const APP = join(ROOT, 'backend', 'app');

interface TauriConf {
  app: { withGlobalTauri?: boolean; security: { csp: string; capabilities?: string[] } };
}
interface Capability {
  permissions: (string | { identifier: string })[];
}

const readText = (...parts: string[]): string => readFileSync(join(...parts), 'utf8');
const conf = JSON.parse(readText(APP, 'tauri.conf.json')) as TauriConf;
const capability = JSON.parse(readText(APP, 'capabilities', 'default.json')) as Capability;
const libRs = readText(APP, 'src', 'lib.rs');
const buildRs = readText(APP, 'build.rs');
const appCargo = readText(APP, 'Cargo.toml');

/** Sources a CSP directive may name. Everything here stays on this computer. */
const LOCAL_SOURCES = new Set(["'self'", "'none'", 'data:', 'ipc:', 'http://ipc.localhost']);

/** Plugin permissions the shell needs (plan 01 T1.7). A new one needs a plan file. */
const PLUGIN_PERMISSIONS = new Set([
  'core:event:default',
  'window-state:default',
  // Check only. Download and install go behind an Owner-checked command (NFR-INS-3, D5).
  'updater:allow-check',
  'process:allow-restart',
  // The sign-in screen's Exit button closes the app (PRD 8.1, plan 04).
  'process:allow-exit',
  'dialog:allow-message',
  'dialog:allow-ask',
  'dialog:allow-confirm',
]);

/** The five plugins of Tech Stack 2. */
const PLUGINS = [
  'tauri_plugin_single_instance',
  'tauri_plugin_window_state',
  'tauri_plugin_updater',
  'tauri_plugin_dialog',
  'tauri_plugin_process',
];

function cspDirectives(csp: string): Map<string, string[]> {
  const map = new Map<string, string[]>();
  for (const part of csp.split(';')) {
    const [name, ...sources] = part.trim().split(/\s+/);
    if (name) map.set(name, sources);
  }
  return map;
}

describe('Content Security Policy (Code Standards S5, NFR-SEC-9)', () => {
  const directives = cspDirectives(conf.app.security.csp);

  it('names only local sources in every directive', () => {
    const remote: string[] = [];
    for (const [name, sources] of directives) {
      for (const source of sources) {
        if (!LOCAL_SOURCES.has(source)) remote.push(`${name} ${source}`);
      }
    }
    expect(remote).toEqual([]);
  });

  it('has default-src and script-src limited to self, with no inline or eval', () => {
    expect(directives.get('default-src')).toEqual(["'self'"]);
    expect(directives.get('script-src')).toEqual(["'self'"]);
  });

  it('keeps the Tauri API off the window object', () => {
    expect(conf.app.withGlobalTauri).toBe(false);
  });
});

describe('Capability file (NFR-SEC-10, Code Standards S4)', () => {
  const ids = capability.permissions.map((p) => (typeof p === 'string' ? p : p.identifier));
  // `const APP_COMMANDS: &[&str] = &["app_info", ...];` in backend/app/build.rs.
  const listed = /APP_COMMANDS[^=]*=\s*&\[([^\]]*)\]/.exec(buildRs)?.[1] ?? '';
  const appCommands = [...listed.matchAll(/"([a-z][a-z0-9_]*)"/g)].map((m) => m[1] ?? '');

  // Plan 14 added the capability `print` for the hidden print windows (labels print-*), which may
  // call one data command and nothing else. A one-line update made by frontend, not robustness.
  it('loads only the capabilities default and print', () => {
    expect(conf.app.security.capabilities).toEqual(['default', 'print']);
  });

  it('allows no plugin permission outside the allow list', () => {
    const extra = ids.filter((id) => id.includes(':') && !PLUGIN_PERMISSIONS.has(id));
    expect(extra).toEqual([]);
  });

  it('allows only commands the backend registers', () => {
    const own = ids.filter((id) => !id.includes(':'));
    const unknown = own.filter(
      (id) => !appCommands.includes(id.replace(/^allow-/, '').replaceAll('-', '_')),
    );
    expect(appCommands.length).toBeGreaterThan(0);
    expect(unknown).toEqual([]);
  });

  it('lets the screens check for an update but never download or install one', () => {
    // NFR-INS-3: the update installs only when the Owner agrees, so the screens may not call
    // the updater's download or install directly (updater:default includes both).
    expect(ids.filter((id) => id.startsWith('updater:'))).toEqual(['updater:allow-check']);
  });

  it('gives only the process permissions restart and exit', () => {
    expect(ids.filter((id) => id.startsWith('process:'))).toEqual([
      'process:allow-restart',
      'process:allow-exit',
    ]);
  });

  it('gives no file system, shell, http or opener access', () => {
    expect(ids.filter((id) => /^(fs|shell|http|opener):/.test(id))).toEqual([]);
  });
});

describe('Plugins (Tech Stack 2, NFR-REL-9)', () => {
  const used = [...libRs.matchAll(/\.plugin\(\s*(tauri_plugin_[a-z_]+)/g)].map((m) => m[1]);

  it('registers exactly the five plugins, single instance first', () => {
    expect(used).toEqual(PLUGINS);
  });

  it('depends on no other tauri plugin crate', () => {
    const crates = [...appCargo.matchAll(/^(tauri-plugin-[a-z-]+)/gm)].map((m) => m[1]);
    expect(new Set(crates)).toEqual(new Set(PLUGINS.map((p) => p.replaceAll('_', '-'))));
  });
});
