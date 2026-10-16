// The only place that calls Tauri's invoke() (Code Standards F4). Each module wraps its commands
// in its own api.ts, and components call those wrappers through hooks.
//
// Every command returns Result<T, UiError> (plan 01 Section 4.2). On failure the promise rejects
// with a CommandError that carries the UiError, so screens can show each problem in the app
// language and point at its field.

import { invoke, type InvokeArgs } from '@tauri-apps/api/core';
import type { UiError } from '../../bindings/UiError';

/** The backend refused or failed a command. `uiError` holds the codes, fields and reference. */
export class CommandError extends Error {
  readonly uiError: UiError;

  constructor(uiError: UiError) {
    super(uiError.errors.map((item) => item.code).join(', '));
    this.name = 'CommandError';
    this.uiError = uiError;
  }
}

/** Reference used when the failure did not come from the backend (for example, no app window). */
const NO_REFERENCE = '';

function isRecord(value: unknown): value is Record<string, unknown> {
  return typeof value === 'object' && value !== null;
}

/** True when the value has the UiError shape the backend sends. */
export function isUiError(value: unknown): value is UiError {
  return (
    isRecord(value) &&
    value['ok'] === false &&
    Array.isArray(value['errors']) &&
    value['errors'].every((item) => isRecord(item) && typeof item['code'] === 'string') &&
    typeof value['ref'] === 'string'
  );
}

/**
 * Turns anything a command threw into a UiError. A failure that is not the backend's own
 * error (the bridge is missing, the command does not exist) becomes ERR_INTERNAL, so the
 * screen always has a message in the app language.
 */
export function toUiError(error: unknown): UiError {
  if (isUiError(error)) {
    return error;
  }
  return {
    ok: false,
    errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
    ref: NO_REFERENCE,
  };
}

/** Calls a backend command. Resolves with its result, or rejects with a CommandError. */
export async function callCommand<T>(command: string, args?: InvokeArgs): Promise<T> {
  try {
    return await invoke<T>(command, args);
  } catch (error: unknown) {
    throw new CommandError(toUiError(error));
  }
}

/**
 * True inside the Tauri window, false in a plain browser tab (for example the Vite page opened
 * in Chrome during development). Tauri puts `__TAURI_INTERNALS__` on the window; invoke() needs it.
 */
export function hasAppBridge(): boolean {
  return typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window;
}
