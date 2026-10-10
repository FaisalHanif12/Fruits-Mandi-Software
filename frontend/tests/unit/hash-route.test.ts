// Development only: /dev/gallery typed without the # opens the gallery, not the start page.

import { createHashHistory } from '@tanstack/react-router';
import { afterEach, describe, expect, it, vi } from 'vitest';
import { moveDevPathToHash } from '../../src/app/dev/hash-route';

function openAddress(address: string) {
  window.history.replaceState(null, '', address);
}

afterEach(() => {
  openAddress('/');
  vi.resetModules();
});

describe('moveDevPathToHash', () => {
  it('moves /dev/gallery to the hash route without reloading', () => {
    openAddress('/dev/gallery');
    expect(moveDevPathToHash()).toBe(true);
    expect(window.location.pathname).toBe('/');
    expect(window.location.hash).toBe('#/dev/gallery');
    expect(createHashHistory().location.pathname).toBe('/dev/gallery');
  });

  it('keeps a query string', () => {
    openAddress('/dev/gallery?x=1');
    expect(moveDevPathToHash()).toBe(true);
    expect(window.location.hash).toBe('#/dev/gallery?x=1');
  });

  it.each(['/', '/other', '/#/dev/gallery', '/dev/gallery#/'])('leaves %s alone', (address) => {
    openAddress(address);
    const before = window.location.href;
    expect(moveDevPathToHash()).toBe(false);
    expect(window.location.href).toBe(before);
  });

  it('is used by the router, so the first screen is the gallery', async () => {
    openAddress('/dev/gallery');
    const { router } = await import('../../src/app/router');
    expect(router.history.location.pathname).toBe('/dev/gallery');
  });
});
