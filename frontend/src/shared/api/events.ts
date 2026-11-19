// Tauri events, the other way a window talks to the backend (plan 14: the hidden print window
// says `print-ready-<job>` when its page is laid out). Like invoke(), the Tauri call is made only
// here, so a screen never imports the Tauri package itself (Code Standards F4).
//
// In a plain browser tab there is no app window (hasAppBridge), so an event goes nowhere and the
// call does nothing. That is what the development pages and the tests rely on.

import { emit, listen } from '@tauri-apps/api/event';
import { hasAppBridge } from './invoke';

/** Tells the backend (and every window) that something happened. Never throws. */
export async function emitEvent(name: string, payload?: unknown): Promise<void> {
  if (!hasAppBridge()) return;
  try {
    await emit(name, payload);
  } catch {
    // The window may be closing; an event nobody hears is not an error the user can act on.
  }
}

/**
 * Listens to an event the backend sends (plan 18: `backup-exit` while the copy at exit runs).
 * Returns the function that stops listening. In a plain browser tab there is no app window, so
 * nothing is heard and the returned function does nothing. Never throws.
 */
export async function listenEvent(
  name: string,
  handler: (payload: unknown) => void,
): Promise<() => void> {
  if (!hasAppBridge()) return () => undefined;
  try {
    return await listen<unknown>(name, (event) => {
      handler(event.payload);
    });
  } catch {
    // No window to listen in: the screen simply never hears the event.
    return () => undefined;
  }
}
