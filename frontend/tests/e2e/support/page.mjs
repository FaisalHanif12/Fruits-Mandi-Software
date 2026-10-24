// Small helpers for the specs: find a field by its label, a button by its words, a heading by its
// text, type, and wait for the app. They read the page the way a person does, by the words on
// the screen, which come from the Fluent files (messages.mjs). Fluent's isolation marks are left
// out of the page text before it is compared.

import { $, browser } from '@wdio/globals';
import { Key } from 'webdriverio';
import { DIRECTION, t } from './messages.mjs';

/** @typedef {import('./messages.mjs').Language} Language */

const ISOLATION_MARKS = '⁨⁩';
const WAIT_MS = 15_000;
const SHORT_WAIT_MS = 1500;
const MAX_TABS = 40;

/** XPath for the text of the current node with isolation marks and extra spaces taken out. */
const PLAIN = `normalize-space(translate(., "${ISOLATION_MARKS}", ""))`;

/**
 * A text as an XPath string. The words of the app never hold a double quote.
 * @param {string} text
 */
function quoted(text) {
  if (text.includes('"')) throw new Error(`A text with a double quote cannot be searched: ${text}`);
  return `"${text}"`;
}

export { Key };

/**
 * Types text into whatever has focus.
 * @param {string} text
 */
export async function type(text) {
  await browser.keys(text);
}

/**
 * Presses keys one after the other. A list is pressed together, for example [Key.Alt, 'l'].
 * @param {...(string | string[])} keys
 */
export async function press(...keys) {
  for (const key of keys) await browser.keys(key);
}

/**
 * The field with this label, in this language.
 * @param {Language} language
 * @param {string} messageId the Fluent ID of the label
 */
export async function field(language, messageId) {
  const wanted = t(language, messageId);
  const id = await browser.execute(
    (text, marks) => {
      const clean = (/** @type {string | null} */ value) =>
        (value ?? '')
          .replace(new RegExp(`[${marks}]`, 'g'), '')
          .replace(/\s+/g, ' ')
          .trim();
      const label = Array.from(document.querySelectorAll('label')).find(
        (candidate) => clean(candidate.textContent) === text,
      );
      return label?.htmlFor ?? null;
    },
    wanted,
    ISOLATION_MARKS,
  );
  if (id === null || id === '') throw new Error(`No field labelled "${wanted}" (${language})`);
  return $(`[id="${id}"]`);
}

/**
 * Puts the cursor in the field with this label.
 * @param {Language} language
 * @param {string} messageId
 */
export async function focusField(language, messageId) {
  const control = await field(language, messageId);
  await control.waitForDisplayed({ timeout: WAIT_MS });
  await control.click();
  return control;
}

/**
 * A button that has these words in it (its key chip, such as F10, may follow them).
 * @param {Language} language
 * @param {string} messageId
 */
export async function button(language, messageId) {
  return $(`//button[contains(${PLAIN}, ${quoted(t(language, messageId))})]`);
}

/**
 * A heading with exactly these words.
 * @param {Language} language
 * @param {string} messageId
 */
export async function heading(language, messageId) {
  return $(`//*[self::h1 or self::h2][${PLAIN} = ${quoted(t(language, messageId))}]`);
}

/**
 * Any element that has these words in it.
 * @param {string} text
 */
export async function withText(text) {
  return $(`//*[contains(${PLAIN}, ${quoted(text)})]`);
}

/**
 * Waits until a heading with these words is on screen.
 * @param {Language} language
 * @param {string} messageId
 */
export async function waitForHeading(language, messageId) {
  const found = await heading(language, messageId);
  await found.waitForDisplayed({ timeout: WAIT_MS });
  return found;
}

/**
 * Waits for the signed-in shell: the main menu.
 * @param {Language} language
 */
export async function waitForShell(language) {
  const menu = await $(`nav[aria-label=${quoted(t(language, 'layout-nav-label'))}]`);
  await menu.waitForDisplayed({ timeout: WAIT_MS });
}

/** The language the page is in now. @returns {Promise<Language>} */
export async function currentLanguage() {
  const lang = await browser.execute(() => document.documentElement.lang);
  return lang === 'en' ? 'en' : 'ur';
}

/**
 * Makes the page the wanted language with Alt+L (PRD FR-7.30), if it is not already.
 * @param {Language} language
 */
export async function switchLanguage(language) {
  if ((await currentLanguage()) === language) return;
  await browser.keys([Key.Alt, 'l']);
  await browser.waitUntil(
    async () => (await browser.execute(() => document.documentElement.dir)) === DIRECTION[language],
    { timeout: WAIT_MS, timeoutMsg: `The page did not turn to ${language}.` },
  );
}

/**
 * Signs in with the keyboard: the user name, Enter, the password, Enter.
 * @param {Language} language
 * @param {{ username: string, password: string }} owner
 */
export async function signIn(language, owner) {
  await waitForHeading(language, 'auth-login-title');
  await focusField(language, 'auth-login-username');
  await type(`${owner.username}${Key.Enter}${owner.password}${Key.Enter}`);
}

/**
 * Opens a screen by its address. The app keeps its address in the hash (a desktop app has no
 * address bar), so this changes the hash and the router follows, with nothing reloaded. Used where
 * the menu path is not what the spec is about.
 * @param {string} route for example "/index/accounts/new"
 */
export async function openRoute(route) {
  await browser.execute((hash) => {
    window.location.hash = hash;
  }, route);
}

/**
 * Presses Tab until the focus is on an element that matches `selector`, so a spec can reach a
 * place the way a keyboard user does.
 * @param {string} selector
 */
export async function tabTo(selector) {
  for (let count = 0; count < MAX_TABS; count += 1) {
    const there = await browser.execute(
      (css) => document.activeElement?.closest(css) !== null && document.activeElement !== null,
      selector,
    );
    if (there) return;
    await browser.keys(Key.Tab);
  }
  throw new Error(`Tab did not reach ${selector} in ${String(MAX_TABS)} presses`);
}

/**
 * If the spelling check asks about a word (PRD FR-7.66), confirms it with Enter. The dialog
 * opens with the confirm button focused. Does nothing when no dialog comes.
 */
export async function confirmSpellingIfAsked() {
  const dialog = await $('[role="alertdialog"]');
  const asked = await dialog
    .waitForDisplayed({ timeout: SHORT_WAIT_MS })
    .then(() => true)
    .catch(() => false);
  if (!asked) return false;
  await browser.keys(Key.Enter);
  await dialog.waitForDisplayed({ timeout: WAIT_MS, reverse: true });
  return true;
}

/**
 * The text of the whole page without isolation marks, for a spec that checks a number is shown.
 * @returns {Promise<string>}
 */
export async function pageText() {
  return browser.execute(
    (marks) => document.body.innerText.replace(new RegExp(`[${marks}]`, 'g'), ''),
    ISOLATION_MARKS,
  );
}

/**
 * Closes the first-run guide (PRD FR-7.12) the first time it opens, with Esc. The app opens it by
 * itself at the first sign-in and keeps a mark when it is closed, so it never opens by itself again:
 * only the spec of the first sign-in needs this. It waits for the guide, so it does not close
 * something that is not there yet; it does nothing when the guide does not appear.
 * @param {number} [waitMs] how long to wait for the guide to appear
 */
export async function closeGuide(waitMs = SHORT_WAIT_MS * 2) {
  const card = await $('[data-guide-step]');
  try {
    await card.waitForDisplayed({ timeout: waitMs });
  } catch {
    return;
  }
  await browser.keys(Key.Escape);
  await card.waitForDisplayed({ timeout: WAIT_MS, reverse: true });
}

/**
 * Signs out from the user menu.
 * @param {Language} language
 */
export async function signOut(language) {
  const menu = await $(`button[aria-label=${quoted(t(language, 'layout-user-menu'))}]`);
  await menu.click();
  const item = await $(
    `//*[@role="menuitem"][contains(${PLAIN}, ${quoted(t(language, 'layout-user-signout'))})]`,
  );
  await item.waitForDisplayed({ timeout: WAIT_MS });
  await item.click();
  await waitForHeading(language, 'auth-login-title');
}

/**
 * Marks the nth element that matches `selector` as the one a timer waits for, so the stop
 * condition can name it (timing.mjs: kind "focus" with selector `[data-e2e-target]`).
 * @param {string} selector
 * @param {number} index
 */
export async function markTarget(selector, index) {
  await browser.execute(
    (css, position) => {
      for (const old of document.querySelectorAll('[data-e2e-target]')) {
        old.removeAttribute('data-e2e-target');
      }
      document.querySelectorAll(css)[position]?.setAttribute('data-e2e-target', 'true');
    },
    selector,
    index,
  );
}

/**
 * Index of the element that has focus among the elements that match `selector`, or -1.
 * @param {string} selector
 * @returns {Promise<number>}
 */
export async function focusedIndex(selector) {
  return browser.execute(
    (css) =>
      Array.from(document.querySelectorAll(css)).indexOf(document.activeElement ?? document.body),
    selector,
  );
}

/**
 * The first thing every later spec does: wait for the sign-in screen (the app has just started)
 * and sign in as the Owner the first-run spec made.
 * @param {Language} language
 * @param {{ username: string, password: string }} owner
 */
export async function startSignedIn(language, owner) {
  await waitForHeading(language, 'auth-login-title');
  await signIn(language, owner);
  await waitForShell(language);
}

/**
 * Opens a screen of the main menu with the keyboard: the focus goes to the menu, Home or End
 * jumps to the first or the last item, and Enter opens it. `where` is "first" for the dashboard
 * and "last" for Settings, the two ends of the menu (PRD FR-5.2).
 * @param {'first' | 'last'} where
 */
export async function openEndOfMenu(where) {
  await focusMenu();
  await browser.keys(where === 'first' ? Key.Home : Key.End);
  await browser.keys(Key.Enter);
}

/** Puts the focus on the item of the main menu that is open (or the first), where a keyboard user is. */
export async function focusMenu() {
  await browser.execute(() => {
    const items = Array.from(document.querySelectorAll('nav [data-nav-item]'));
    const current = items.find((item) => item.getAttribute('aria-current') === 'page');
    const target = current ?? items[0];
    if (target instanceof HTMLElement) target.focus();
  });
}

/**
 * Chooses an option of a drop-down (a Select) by its words: opens it and picks the option.
 * @param {Language} language
 * @param {string} fieldId the label of the drop-down
 * @param {string} optionId the Fluent ID of the option
 */
export async function choose(language, fieldId, optionId) {
  const box = await focusField(language, fieldId);
  await box.click();
  const option = await $(
    `//*[@role="option"][contains(${PLAIN}, ${quoted(t(language, optionId))})]`,
  );
  await option.waitForDisplayed({ timeout: WAIT_MS });
  await option.click();
}

/**
 * Types into a picker, waits for its list to answer, and takes the first match with Enter. It
 * waits until the field shows the name, so the next keys go to the next field.
 * @param {Language} language
 * @param {string} fieldId the label of the picker
 * @param {string} text what to type
 * @param {string} shown a part of the name the field shows when the pick is done
 */
export async function pick(language, fieldId, text, shown) {
  const box = await focusField(language, fieldId);
  await box.setValue(text);
  const first = await $('[role="option"]');
  await first.waitForDisplayed({ timeout: WAIT_MS });
  await browser.keys(Key.Enter);
  await browser.waitUntil(async () => (await box.getValue()).includes(shown), {
    timeout: WAIT_MS,
    timeoutMsg: `The field "${fieldId}" never showed "${shown}".`,
  });
}

/**
 * Waits until the page says something, for example a saved message in the toast.
 * @param {string} text
 */
export async function waitForText(text) {
  await browser.waitUntil(async () => (await pageText()).includes(text), {
    timeout: WAIT_MS,
    timeoutMsg: `The page never said: ${text}`,
  });
}
