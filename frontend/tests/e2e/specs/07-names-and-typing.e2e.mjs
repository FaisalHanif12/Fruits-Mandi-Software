// D2-A4 and E2E-23: names in two scripts, and the three ways to type Urdu with no Windows setting
// (PRD FR-7.21, FR-7.22, FR-7.24, FR-7.40, FR-7.66). In the new-account screen:
//  - the Urdu name field starts in "Roman to Urdu"; Alt+U goes to the Urdu keyboard, then to
//    English, then round again, and the badge in the field says which;
//  - Urdu letters typed with a Windows Urdu keyboard pass through unchanged in every mode;
//  - `tariq mehmood` becomes طارق محمود and the English name fills itself as "Tariq Mehmood";
//  - saving gives the next buyer code, and the box in the top bar finds him in both languages,
//    also with a small spelling mistake ("tarq").
// The PRD test says code 0022 because two buyers exist by then; this run starts with none,
// so it checks that the code is the next free buyer code (002 and a running number).

import { $, browser, expect } from '@wdio/globals';
import { OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  confirmSpellingIfAsked,
  field,
  focusField,
  openRoute,
  pageText,
  startSignedIn,
  type,
  switchLanguage,
  waitForHeading,
  waitForText,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';

const URDU_A_B = 'اب';
const URDU_NAME_TYPED = 'طارق';
const SUGGEST_WAIT_MS = 10_000;

/**
 * The words in the typing-mode badge of the Urdu name field.
 * @returns {Promise<string>}
 */
async function badgeText() {
  return browser.execute(() => {
    const input = document.activeElement;
    const badge = input?.parentElement?.querySelector('button[aria-label*=":"]');
    return (badge?.textContent ?? '').replace(/[⁨⁩]/g, '').trim();
  });
}

describe('typing names (Urdu app)', () => {
  before(async () => {
    await startSignedIn('ur', OWNER);
    await openRoute('/index/accounts/new');
    await waitForHeading('ur', 'accounts-new-title');
  });

  it('the Urdu name starts in "Roman to Urdu", and Alt+U goes round the three modes', async () => {
    await focusField('ur', 'accounts-field-name-ur');
    expect(await badgeText()).toBe(t('ur', 'translit-mode-roman'));
    await browser.keys([Key.Alt, 'u']);
    expect(await badgeText()).toBe(t('ur', 'translit-mode-urdu'));
    await browser.keys([Key.Alt, 'u']);
    expect(await badgeText()).toBe(t('ur', 'translit-mode-english'));
    await browser.keys([Key.Alt, 'u']);
    expect(await badgeText()).toBe(t('ur', 'translit-mode-roman'));
    await shootScreen('account-new-typing-modes', 'ur');
  });

  it('the Urdu keyboard mode types Urdu letters from English keys (a is ا, b is ب)', async () => {
    const urdu = await focusField('ur', 'accounts-field-name-ur');
    await browser.keys([Key.Alt, 'u']);
    await urdu.setValue('');
    await type('ab');
    await browser.waitUntil(async () => (await urdu.getValue()) === URDU_A_B, {
      timeoutMsg: 'The keys a and b did not give ا and ب.',
    });
    // The mode that does no conversion keeps what is typed.
    await browser.keys([Key.Alt, 'u']);
    await type('x');
    expect(await urdu.getValue()).toBe(`${URDU_A_B}x`);
    await browser.keys([Key.Alt, 'u']);
  });

  it('Urdu letters from a Windows Urdu keyboard pass through unchanged', async () => {
    const urdu = await focusField('ur', 'accounts-field-name-ur');
    await urdu.setValue('');
    await type(URDU_NAME_TYPED);
    expect(await urdu.getValue()).toBe(URDU_NAME_TYPED);
  });

  it('tariq mehmood becomes طارق محمود and the English name fills itself (E2E-23)', async () => {
    const urdu = await focusField('ur', 'accounts-field-name-ur');
    await urdu.setValue('');
    await type('tariq ');
    await type('mehmood');
    const list = await $('[role="listbox"]');
    await list.waitForDisplayed({ timeout: SUGGEST_WAIT_MS });
    await type(Key.Enter);
    await browser.waitUntil(async () => (await urdu.getValue()) === 'طارق محمود', {
      timeout: SUGGEST_WAIT_MS,
      timeoutMsg: `The Urdu name is "${await urdu.getValue()}", not طارق محمود.`,
    });
    const english = await field('ur', 'accounts-field-name-en');
    await browser.waitUntil(async () => (await english.getValue()) === 'Tariq Mehmood', {
      timeout: SUGGEST_WAIT_MS,
      timeoutMsg: 'The English name did not fill itself as Tariq Mehmood.',
    });
  });

  it('saves with F10 (a spelling made by rules alone is confirmed first) and gives the next buyer code', async () => {
    await type(Key.F10);
    await confirmSpellingIfAsked();
    await waitForText(t('ur', 'accounts-saved'));
  });

  it('the box in the top bar finds him with a small spelling mistake, in both languages', async () => {
    for (const language of /** @type {const} */ (['ur', 'en'])) {
      await switchLanguage(language);
      await openRoute('/');
      await browser.keys([Key.Ctrl, 'k']);
      await type('tarq');
      const row = await $('[role="option"]');
      await row.waitForDisplayed({ timeout: SUGGEST_WAIT_MS });
      const shown = (await pageText()).includes(language === 'ur' ? 'طارق محمود' : 'Tariq Mehmood');
      expect(shown).toBe(true);
      expect(await pageText()).toMatch(/002\d+/);
      await type(Key.Escape);
    }
    await switchLanguage('ur');
  });
});
