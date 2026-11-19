// Helpers of the plan 16 specs (later sale, edit, cancel, undo): type a small sale with the
// keyboard only and save it. The sellers, buyers, item and packing are the ones the earlier specs
// made (رحیم, اکرم ٹول ٹیکس, ارشد ملتانی, کیلا, پیٹی, سوات اڈا). Each spec makes its own بکری, so the
// specs do not depend on a number: the number the save gives is read from the message.

import { $, browser } from '@wdio/globals';
import { t } from './messages.mjs';
import { Key, focusField, openRoute, pageText, type, waitForHeading } from './page.mjs';

const LIST_WAIT_MS = 10_000;
const NUMBER_WAIT_MS = 15_000;

/**
 * Types the text of a picker, waits for its list to answer, and takes the first match.
 * @param {string} text
 */
export async function typeAndPick(text) {
  await type(text);
  const first = await $('[role="option"]');
  await first.waitForDisplayed({ timeout: LIST_WAIT_MS });
  await type(Key.Enter);
}

/**
 * A cell of an entry grid by the name of the grid, the row and the column.
 * @param {string} gridLabel
 * @param {number} rowIndex
 * @param {string} column
 */
export function cell(gridLabel, rowIndex, column) {
  return $(
    `//*[@role="group"][@aria-label="${gridLabel}"]//div[@role="row"][@row-index="${String(rowIndex)}"]//div[@col-id="${column}"]`,
  );
}

/**
 * Waits until the page says every one of these numbers.
 * @param {...string} numbers
 */
export async function waitForNumbers(...numbers) {
  await browser.waitUntil(
    async () => {
      const text = await pageText();
      return numbers.every((number) => text.includes(number));
    },
    { timeout: NUMBER_WAIT_MS, timeoutMsg: `The page never showed: ${numbers.join(', ')}` },
  );
}

/**
 * Types a new بکری: the lot of رحیم with `received` پیٹی کیلا, and one sale line per entry of
 * `lines` (buyer text to pick, qty, rate). The lot's freight, labour, munshiana and نقد stay empty.
 * Ends with the cursor on the first cell after the last line, nothing saved.
 * @param {number} received
 * @param {{ buyer: string, qty: number, rate: number }[]} lines
 */
export async function typeSale(received, lines) {
  await openRoute('/bikri/book');
  await waitForHeading('ur', 'glossary-bikri-book');
  await focusField('ur', 'bikri-header-truck');
  await type(`LHR-${String(received)}${Key.Enter}`);
  await typeAndPick('rahim');
  await typeAndPick('swat');
  await type(Key.Enter);
  // The seller is the زمیندار; item and packing are asked for in the lot.
  await type(Key.Enter);
  await typeAndPick('banana');
  await typeAndPick('box');
  await type(`${String(received)}${Key.Enter}${Key.Enter}${Key.Enter}${Key.Enter}${Key.Enter}`);
  for (const line of lines) {
    await typeAndPick(line.buyer);
    await type(`${String(line.qty)}${Key.Enter}${String(line.rate)}${Key.Enter}`);
  }
}

/**
 * A regular expression that finds a message with a number in it, with the number captured.
 * @param {string} id
 */
function numberedMessage(id) {
  const marker = 'NUMBER_HERE';
  const text = t('ur', id, { number: marker });
  const escaped = text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&').replace(marker, '(\\d+)');
  return new RegExp(escaped);
}

/**
 * Waits for a message with the number of a بکری in it and returns the number.
 * @param {string} id the Fluent ID of a message that takes `$number`
 * @returns {Promise<number>}
 */
export async function waitForNumberedMessage(id) {
  const pattern = numberedMessage(id);
  let found = 0;
  await browser.waitUntil(
    async () => {
      const match = pattern.exec(await pageText());
      if (match?.[1] === undefined) return false;
      found = Number(match[1]);
      return true;
    },
    { timeout: NUMBER_WAIT_MS, timeoutMsg: `The page never said: ${id}` },
  );
  return found;
}

/**
 * F10, then waits for the message of the save and returns the number the backend gave.
 * @returns {Promise<number>}
 */
export async function saveSale() {
  await type(Key.F10);
  return waitForNumberedMessage('bikri-saved-toast');
}

/** Opens the saved بکری that was saved last, from the "last saved" link of the new entry. */
export async function openLastSaved() {
  const open = await $(`//button[contains(normalize-space(.), "${t('ur', 'bikri-truck-open')}")]`);
  await open.click();
  await waitForHeading('ur', 'glossary-bikri-book').catch(() => undefined);
}

/**
 * A button of the screen by its words.
 * @param {string} id the Fluent ID
 */
export function buttonNamed(id) {
  return $(`//button[contains(normalize-space(.), "${t('ur', id)}")]`);
}
