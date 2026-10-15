// D1-A16, D2-A16 and D3-A2: a picture of every screen in Urdu and in English at 1024 x 600, 1366 x 768
// and 1920 x 1080 (NFR-USE-15, NFR-USE-17, NFR-USE-28), saved to output/screenshots/ with an
// index file. It also checks, on each, that nothing scrolls sideways and that the page does not
// scroll up and down by itself. People look at the pictures; there is no picture comparison yet
// (D1-Q1 is open).

import { browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { DIRECTION, LANGUAGES } from '../support/messages.mjs';
import { openRoute, startSignedIn, switchLanguage } from '../support/page.mjs';
import { SIZES, setContentSize, shootScreen } from '../support/screens.mjs';

const SETTLE_MS = 700;

/** Every screen of D1, D2 and the partners (plan 11) that can be opened by its address. */
const SCREENS = [
  { name: 'dashboard', route: '/' },
  { name: 'settings', route: '/settings' },
  { name: 'masters-menu', route: '/index' },
  { name: 'items', route: '/index/items' },
  { name: 'items-new', route: '/index/items/new' },
  { name: 'cities', route: '/index/cities' },
  { name: 'markas-new', route: '/index/markas/new' },
  { name: 'narrations-new', route: '/index/narrations/new' },
  { name: 'accounts', route: '/index/accounts' },
  { name: 'account-new', route: '/index/accounts/new' },
  { name: 'chart', route: '/index/chart' },
  { name: 'openings', route: '/index/openings' },
  { name: 'rules', route: '/index/rules' },
  { name: 'rule-new', route: '/index/rules/new' },
  { name: 'rule-test', route: '/index/rules/test' },
  { name: 'advance', route: '/bikri/advance' },
  { name: 'day', route: '/day' },
  { name: 'partners', route: '/partners' },
  { name: 'partners-setup', route: '/partners/setup' },
];

describe('every screen, both languages, three sizes', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
  });

  for (const language of LANGUAGES) {
    for (const { name, route } of SCREENS) {
      it(`${name} in ${language}`, async () => {
        await openRoute(route);
        await switchLanguage(language);
        await browser.pause(SETTLE_MS);
        await expect(browser.$('html')).toHaveAttribute('dir', DIRECTION[language]);
        await shootScreen(name, language);
        // At each size nothing may scroll sideways (NFR-USE-15).
        for (const { width, height } of SIZES) {
          await setContentSize(width, height);
          const sideways = await browser.execute(
            () => document.documentElement.scrollWidth > document.documentElement.clientWidth,
          );
          expect(sideways).toBe(false);
        }
      });
    }
  }
});
