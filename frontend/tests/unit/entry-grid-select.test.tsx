// The quality list of a sale line in the shared entry grid (PRD FR-12.42, entry-grid skill): a
// `select` cell holds one of a few fixed words or nothing. It is reached with the arrow keys or a
// click, never an Enter stop (a line stays buyer, qty, rate, NFR-USE-6), opens with Space, Alt+↓ or
// a click, takes ↑ ↓ and Enter in the list, keeps the focus on the cell, and nothing can be typed in
// it. Urdu and English. A language switch in the sale book is tested with the whole screen
// (bikri-grade.test.tsx), because this rig has a fixed language.

import { screen, waitFor, within } from '@testing-library/react';
import { describe, expect, it } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { renderGrid } from '../helpers/entry-grid';
import { message, visible } from '../helpers/render';

const RIGHT = '{ArrowRight}';
const LEFT = '{ArrowLeft}';
/** The key that moves to the next column in reading order: → in English, ← in Urdu. */
const forward = (language: Language) => (language === 'ur' ? LEFT : RIGHT);

describe.each(LANGUAGES)('the quality list cell of the entry grid (%s)', (language) => {
  const say = (id: string) => message(language, id);
  const first = say('glossary-darja-awwal');
  const second = say('glossary-darja-doyam');
  const list = () => screen.queryByRole('listbox');
  const words = () =>
    within(screen.getByRole('listbox'))
      .getAllByRole('option')
      .map((option) => visible(option.textContent));

  it('takes no Enter stop: a line is still buyer, qty, rate, then the next line', async () => {
    const rig = await renderGrid(language, { withSelect: true });
    await rig.focusCell(0, 'buyer');
    await rig.user.keyboard('1{Enter}');
    expect(rig.focused()).toBe('0:qty');
    await rig.user.keyboard('20{Enter}');
    expect(rig.focused()).toBe('0:rate');
    await rig.user.keyboard('600{Enter}');
    await waitFor(() => {
      expect(rig.focused()).toBe('1:buyer');
    });
    expect(rig.typed(0, 'grade') ?? '').toBe('');
  });

  it('is reached with the arrow key from the rate, and Enter on it goes on like on any cell', async () => {
    const rig = await renderGrid(language, { withSelect: true, rows: [{}, {}] });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard(forward(language));
    expect(rig.focused()).toBe('0:grade');
    expect(list()).toBeNull();
    await rig.user.keyboard('{Enter}');
    await waitFor(() => {
      expect(rig.focused()).toBe('1:buyer');
    });
  });

  it('shows nothing in the cell while it is empty, and the word once chosen', async () => {
    const rig = await renderGrid(language, { withSelect: true, rows: [{}, { grade: 'first' }] });
    expect(visible(rig.cell(0, 'grade').textContent)).toBe('');
    expect(visible(rig.cell(1, 'grade').textContent)).toBe(first);
  });

  it('opens with Space, lists the empty choice and the three words, and Enter chooses', async () => {
    const rig = await renderGrid(language, { withSelect: true, rows: [{}] });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard(forward(language));
    await rig.user.keyboard(' ');
    await waitFor(() => {
      expect(list()).not.toBeNull();
    });
    expect(words()).toEqual([say('bikri-grade-none'), first, second, say('glossary-darja-soyam')]);
    await rig.user.keyboard('{ArrowDown}{ArrowDown}{Enter}');
    expect(rig.typed(0, 'grade')).toBe('second');
    expect(list()).toBeNull();
    // The focus stays on the cell and shows the word.
    expect(rig.focused()).toBe('0:grade');
    expect(visible(rig.cell(0, 'grade').textContent)).toContain(second);
  });

  it('opens with Alt+↓ and a click, and a click on a word chooses it', async () => {
    const rig = await renderGrid(language, { withSelect: true, rows: [{}, {}] });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard(forward(language));
    await rig.user.keyboard('{Alt>}{ArrowDown}{/Alt}');
    await waitFor(() => {
      expect(list()).not.toBeNull();
    });
    await rig.user.keyboard('{Escape}');
    await waitFor(() => {
      expect(list()).toBeNull();
    });
    await rig.focusCell(1, 'grade');
    await waitFor(() => {
      expect(list()).not.toBeNull();
    });
    await rig.user.click(screen.getByRole('option', { name: (name) => visible(name) === first }));
    expect(rig.typed(1, 'grade')).toBe('first');
    expect(list()).toBeNull();
  });

  it('opens with the word of the cell highlighted, and Esc keeps the old word', async () => {
    const rig = await renderGrid(language, { withSelect: true, rows: [{ grade: 'second' }] });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard(forward(language));
    await rig.user.keyboard(' ');
    await waitFor(() => {
      expect(list()).not.toBeNull();
    });
    expect(
      screen.getByRole('option', { selected: true, name: (name) => visible(name) === second }),
    ).toBeInTheDocument();
    await rig.user.keyboard('{ArrowDown}{Escape}');
    await waitFor(() => {
      expect(list()).toBeNull();
    });
    expect(rig.typed(0, 'grade')).toBe('second');
  });

  it('nothing can be typed in it: a letter or digit does nothing, Delete takes the word away', async () => {
    const rig = await renderGrid(language, { withSelect: true, rows: [{ grade: 'third' }] });
    await rig.focusCell(0, 'rate');
    await rig.user.keyboard(forward(language));
    await rig.user.keyboard('a9');
    expect(list()).toBeNull();
    expect(rig.isEditing()).toBe(false);
    expect(rig.typed(0, 'grade')).toBe('third');
    await rig.user.keyboard('{Delete}');
    expect(rig.typed(0, 'grade')).toBe('');
    expect(rig.focused()).toBe('0:grade');
  });
});
