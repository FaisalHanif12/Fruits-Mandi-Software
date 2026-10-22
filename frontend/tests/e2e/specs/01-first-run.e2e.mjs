// D1 story, step 1 (plan 04 Section 5.6, D1-A14, plan 11 D3-A2): the first-run wizard by keyboard
// only. The window opens in Urdu, right to left (PRD FR-7.30). The firm, then the Owner, then the
// recovery code to print, then the backup key to print and type back (plan 18, PRD FR-20.10,
// NFR-BAK-7), then the partners (الف, ب, ج with 40, 30 and 30 percent, PRD 4.2). The
// Owner is signed in right after the setup, so the end of the wizard opens the app; the spec then
// signs out to see the sign-in screen. The suite starts on a computer with no app data, so this
// spec makes the Owner the later specs sign in as (support/credentials.mjs) and the partners
// the later specs split income between.
// Screenshots of each step are saved in Urdu, and of the sign-in screen in both languages.

import { $, browser, expect } from '@wdio/globals';
import { rememberKey } from '../support/backup-key.mjs';
import { FIRM, OWNER, PARTNERS } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  button,
  confirmSpellingIfAsked,
  press,
  signOut,
  type,
  switchLanguage,
  waitForHeading,
  waitForShell,
} from '../support/page.mjs';
import { shootScreen } from '../support/screens.mjs';
import { sinceSessionStart } from '../support/timing.mjs';

describe('first run (Urdu)', () => {
  it('opens in Urdu, right to left, on the first step', async () => {
    await waitForHeading('ur', 'auth-wizard-firm-title');
    await expect($('html')).toHaveAttribute('dir', 'rtl');
    await expect($('html')).toHaveAttribute('lang', 'ur');
    // NFR-SPD-1 is measured on the sign-in screen (spec 02), which is what a start shows every
    // day. This is the one start of the very first run.
    expect(sinceSessionStart()).toBeLessThan(30_000);
  });

  it('shows five steps with the first current', async () => {
    const steps = await $(
      `ol[aria-label="${t('ur', 'auth-wizard-steps')}"], ul[aria-label="${t('ur', 'auth-wizard-steps')}"]`,
    );
    await expect(steps).toBeDisplayed();
    const items = await steps.$$('li');
    expect(items).toHaveLength(5);
    await expect(items[0]).toHaveAttribute('aria-current', 'step');
  });

  it('by keyboard only: the firm names, phone and address, Enter after each', async () => {
    await shootScreen('wizard-firm', 'ur');
    // The first field has focus. The names in both scripts, the mandi in both, the phone, the address.
    await type(
      [FIRM.nameUr, FIRM.nameEn, FIRM.mandiUr, FIRM.mandiEn, FIRM.phone, FIRM.address]
        .map((text) => `${text}${Key.Enter}`)
        .join(''),
    );
    // A word made by the spelling rules alone is checked before going on (PRD FR-7.66).
    await confirmSpellingIfAsked();
    await waitForHeading('ur', 'auth-wizard-owner-title');
  });

  it('by keyboard only: the Owner, then the password twice', async () => {
    await shootScreen('wizard-owner', 'ur');
    await type(
      [OWNER.displayName, OWNER.username, OWNER.password, OWNER.password]
        .map((text) => `${text}${Key.Enter}`)
        .join(''),
    );
    await waitForHeading('ur', 'auth-wizard-code-title');
  });

  it('shows the recovery code once, in groups of four, left to right', async () => {
    const code = await $('p[aria-label]');
    await expect(code).toBeDisplayed();
    expect(await code.getText()).toMatch(/^[A-Z0-9]{4}(-[A-Z0-9]{4}){3}$/);
    await shootScreen('wizard-code', 'ur');
  });

  it('Next needs the tick, then goes on to the backup key step', async () => {
    const next = await button('ur', 'auth-wizard-next');
    await expect(next).toBeDisabled();
    await $('input[type="checkbox"]').click();
    await expect(next).toBeEnabled();
    await next.click();
    await waitForHeading('ur', 'backup-wizard-key-title');
    // The recovery code was shown once: the key is the only code on this screen.
    const steps = await $(
      `ol[aria-label="${t('ur', 'auth-wizard-steps')}"], ul[aria-label="${t('ur', 'auth-wizard-steps')}"]`,
    );
    const items = await steps.$$('li');
    await expect(items[3]).toHaveAttribute('aria-current', 'step');
  });
});

describe('first run: the backup key step (Urdu)', () => {
  /** @type {string} */
  let key = '';

  it('shows the backup key once, in six groups of four, left to right', async () => {
    const shown = await $('p[aria-label]');
    await shown.waitForDisplayed();
    key = (await shown.getAttribute('aria-label')) ?? '';
    expect(key).toMatch(/^[A-Z0-9]{4}(-[A-Z0-9]{4}){5}$/);
    rememberKey(key);
    await shootScreen('wizard-key', 'ur');
  });

  it('confirming the key needs the tick that it was printed', async () => {
    const confirm = await button('ur', 'backup-key-confirm');
    await expect(confirm).toBeDisabled();
    await $('input[type="checkbox"]').click();
    await expect(confirm).toBeDisabled();
  });

  it('typing a wrong key is refused at its field; the right key goes on to the partners', async () => {
    const typed = await $(`input[lang="en"]`);
    await typed.click();
    await type(`ABCD-EFGH-JKLM-NPQR-STUV-WXYZ${Key.Enter}`);
    await browser.waitUntil(async () => (await typed.getAttribute('aria-invalid')) === 'true', {
      timeout: 10_000,
    });
    await shootScreen('wizard-key-wrong', 'ur');
    await typed.click();
    await press([Key.Ctrl, 'a']);
    await type(`${key}${Key.Enter}`);
    await waitForHeading('ur', 'auth-wizard-partners-title');
    // The key was shown once: it is not on this screen any more.
    await expect($('p[aria-label]')).not.toBeExisting();
    const steps = await $(
      `ol[aria-label="${t('ur', 'auth-wizard-steps')}"], ul[aria-label="${t('ur', 'auth-wizard-steps')}"]`,
    );
    const items = await steps.$$('li');
    await expect(items[4]).toHaveAttribute('aria-current', 'step');
    await shootScreen('wizard-partners-empty', 'ur');
  });
});

describe('first run: the partners step (Urdu)', () => {
  it('adds the three partners by keyboard: F2, the names, Ctrl+Enter for the next one', async () => {
    for (const [index, partner] of PARTNERS.entries()) {
      await press(Key.F2);
      await waitForHeading('ur', 'partners-add-title');
      // The Urdu name has focus first in the Urdu app; Enter goes to the English name.
      await type(`${partner.nameUr}${Key.Enter}${partner.nameEn}`);
      await confirmSpellingIfAsked();
      if (index === PARTNERS.length - 1) {
        await press(Key.F10);
      } else {
        await press([Key.Ctrl, Key.Enter]);
        // The dialog stays open and is empty again for the next partner.
        await browser.pause(400);
      }
    }
    await waitForHeading('ur', 'auth-wizard-partners-title');
    await expect($('[role="dialog"]')).not.toBeDisplayed();
  });

  it('types the shares with Enter between the boxes; the total turns green at 100', async () => {
    // Focus went back to the first share box when the dialog closed.
    await type(PARTNERS.map((partner) => String(partner.share)).join(Key.Enter));
    const total = await $('[role="status"]');
    await total.waitForDisplayed();
    await browser.waitUntil(async () => (await total.getText()).includes('100%'), {
      timeout: 10_000,
    });
    await expect(total).toHaveText(expect.stringContaining(t('ur', 'partners-total-ok')));
    await shootScreen('wizard-partners-filled', 'ur');
  });

  it('Save and finish ends the wizard in the app, already signed in', async () => {
    const save = await button('ur', 'auth-wizard-partners-save');
    await expect(save).toBeEnabled();
    await save.click();
    await waitForShell('ur');
  });
});

describe('first run: the sign-in screen after the wizard', () => {
  it('signing out shows the sign-in screen in Urdu', async () => {
    await signOut('ur');
  });
});

describe('the sign-in screen after the first run', () => {
  it('is in Urdu, and Alt+L turns it to English, left to right', async () => {
    await shootScreen('login', 'ur');
    await switchLanguage('en');
    await waitForHeading('en', 'auth-login-title');
    await expect($('html')).toHaveAttribute('dir', 'ltr');
    await shootScreen('login', 'en');
    await switchLanguage('ur');
    await waitForHeading('ur', 'auth-login-title');
  });
});
