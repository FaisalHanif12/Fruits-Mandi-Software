// Development only. The app uses hash routes (/#/dev/gallery), because a desktop app has no
// address bar. In a browser, people type /dev/gallery without the #, and Vite then serves the
// start page. This moves such a path to the hash route before the router reads the address.

const DEV_PATH_PREFIX = '/dev/';

/**
 * If the address is a path under /dev/ (no #), rewrites it to the hash route and returns true.
 * Example: /dev/gallery becomes /#/dev/gallery. It changes the address only; nothing reloads.
 */
export function moveDevPathToHash(
  location: Pick<Location, 'pathname' | 'search' | 'hash'> = window.location,
  history: Pick<History, 'replaceState' | 'state'> = window.history,
): boolean {
  if (!location.pathname.startsWith(DEV_PATH_PREFIX) || location.hash !== '') return false;
  history.replaceState(history.state, '', `/#${location.pathname}${location.search}`);
  return true;
}
