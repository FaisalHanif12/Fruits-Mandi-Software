// D1 story, step 1 (plan 04 Section 5.6, D1-A14): the first-run wizard by keyboard only. The
// window opens in Urdu, right to left (PRD FR-7.30). The firm, then the Owner, then the recovery
// code to print; Finish leads to the sign-in screen. The suite starts on a computer with no app
// data, so this spec makes the Owner the later specs sign in as (support/credentials.mjs).
// Screenshots of each step are saved in Urdu, and of the sign-in screen in both languages.

import { $, expect } from '@wdio/globals';
import { FIRM, OWNER } from '../support/credentials.mjs';
import { t } from '../support/messages.mjs';
import {
  Key,
  confirmSpellingIfAsked,
  type,
  switchLanguage,
  waitForHeading,
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

  it('shows three steps with the first current', async () => {
    const steps = await $(
      `ol[aria-label="${t('ur', 'auth-wizard-steps')}"], ul[aria-label="${t('ur', 'auth-wizard-steps')}"]`,
    );
    await expect(steps).toBeDisplayed();
    const items = await steps.$$('li');
    expect(items).toHaveLength(3);
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

  it('Finish needs the tick, then leads to the sign-in screen', async () => {
    const finish = await $(
      `//button[contains(normalize-space(.), "${t('ur', 'auth-wizard-finish')}")]`,
    );
    await expect(finish).toBeDisabled();
    await $('input[type="checkbox"]').click();
    await expect(finish).toBeEnabled();
    await finish.click();
    await waitForHeading('ur', 'auth-login-title');
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
