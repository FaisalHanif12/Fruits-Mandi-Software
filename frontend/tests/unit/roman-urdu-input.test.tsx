// The Urdu typing field (plan 05, PRD FR-7.21, FR-7.22, NFR-URD-17): every key of its table, in the
// Urdu and in the English app, against a fake of the real translit command shapes. The field is
// written right to left in both apps.

import { act, fireEvent, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import { Field, RomanUrduInput, TextInput } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { KEY_MAP, mapPhoneticKey } from '../../src/shared/urdu/phonetic_keyboard';
import { fail } from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/urdu/api');

/** What the real command answers for a few mandi names (plan 05 worked examples). */
const TO_URDU: Record<string, string[]> = {
  tariq: ['طارق', 'تارق'],
  mehmood: ['محمود', 'مہمود', 'محمد'],
  shahid: ['شاہد', 'شاھد'],
  arshad: ['ارشد'],
  multani: ['ملتانی'],
};

function fakeCommand() {
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(suggestionsFor({ toUrdu: TO_URDU }, text, direction)),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
}

function Harness({ initial = '', readOnly = false }: { initial?: string; readOnly?: boolean }) {
  const [value, setValue] = useState(initial);
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <RomanUrduInput {...control} value={value} onChange={setValue} readOnly={readOnly} />
        )}
      </Field>
      <input data-testid="next" />
    </form>
  );
}

async function setup(language: Language, props: { initial?: string; readOnly?: boolean } = {}) {
  renderIn(language, <Harness {...props} />);
  const input = screen.getByRole<HTMLInputElement>('combobox');
  const user = userEvent.setup();
  await user.click(input);
  return { input, user };
}

const options = () => screen.queryAllByRole('option');

beforeEach(() => {
  vi.clearAllMocks();
  fakeCommand();
});

describe.each(LANGUAGES)('RomanUrduInput in roman mode (%s)', (language) => {
  it('shows up to three Urdu spellings under the field, the first one highlighted', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('mehmood');
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    expect(options().map((option) => option.textContent)).toEqual(['محمود', 'مہمود', 'محمد']);
    expect(options()[0]).toHaveAttribute('aria-selected', 'true');
    expect(options()[1]).toHaveAttribute('aria-selected', 'false');
    expect(input).toHaveAttribute('aria-expanded', 'true');
    expect(screen.getByRole('listbox')).toHaveAccessibleName(
      message(language, 'translit-suggestions-label'),
    );
    // The field is written right to left in both apps, and the English letters stay as typed.
    expect(input.closest('[dir]')).toHaveAttribute('dir', 'rtl');
    expect(input.value).toBe('mehmood');
  });

  it('asks for the word on each key press, and a later word replaces the earlier answer', async () => {
    const { user } = await setup(language);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    expect(urduApi.translitSuggest).toHaveBeenLastCalledWith({
      text: 'tariq',
      direction: 'to_urdu',
    });
    expect(urduApi.translitSuggest).toHaveBeenCalledTimes(5);
  });

  it('↓ and ↑ move the highlight, and wrap round', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('mehmood');
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    await user.keyboard('{ArrowDown}');
    expect(options()[1]).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{ArrowDown}{ArrowDown}');
    expect(options()[0]).toHaveAttribute('aria-selected', 'true');
    await user.keyboard('{ArrowUp}');
    expect(options()[2]).toHaveAttribute('aria-selected', 'true');
    expect(input).toHaveAttribute('aria-activedescendant', options()[2]?.id);
    // The arrows moved the highlight, not the focus.
    expect(input).toHaveFocus();
  });

  it('Space takes the highlighted spelling and stays in the field for the next word', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('tariq ');
    await waitFor(() => {
      expect(input.value).toBe('طارق ');
    });
    expect(input).toHaveFocus();
    expect(options()).toHaveLength(0);
    // The first spelling is the app's own guess: nothing is learned from it.
    expect(urduApi.translitLearn).not.toHaveBeenCalled();
  });

  it('Enter takes the spelling, then moves on to the next field', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    await user.keyboard('{Enter}');
    expect(input.value).toBe('طارق');
    expect(screen.getByTestId('next')).toHaveFocus();
  });

  it('with no word being typed, Enter just moves on', async () => {
    const { user } = await setup(language, { initial: 'طارق' });
    await user.keyboard('{Enter}');
    expect(screen.getByTestId('next')).toHaveFocus();
  });

  it('leaving the field takes the highlighted spelling', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('arshad');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.tab();
    expect(input.value).toBe('ارشد');
  });

  it('turns a phrase word by word: "tariq mehmood" becomes "طارق محمود"', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('tariq ');
    await waitFor(() => {
      expect(input.value).toBe('طارق ');
    });
    await user.keyboard('mehmood');
    await waitFor(() => {
      expect(options()).toHaveLength(3);
    });
    await user.keyboard('{Enter}');
    expect(input.value).toBe('طارق محمود');
  });

  it('picking another spelling is learned, as one word, with its kind', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('shahid');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    await user.keyboard('{ArrowDown} ');
    expect(input.value).toBe('شاھد ');
    expect(urduApi.translitLearn).toHaveBeenCalledWith({
      roman: 'shahid',
      urdu: 'شاھد',
      kind: 'name',
    });
  });

  it('a word he turned into Urdu and then corrected by hand is learned when he leaves the field', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('shahid ');
    await waitFor(() => {
      expect(input.value).toBe('شاہد ');
    });
    expect(urduApi.translitLearn).not.toHaveBeenCalled();
    await user.clear(input);
    await user.keyboard('شاھد');
    await user.tab();
    expect(urduApi.translitLearn).toHaveBeenCalledTimes(1);
    expect(urduApi.translitLearn).toHaveBeenCalledWith({
      roman: 'shahid',
      urdu: 'شاھد',
      kind: 'name',
    });
  });

  it('an untouched converted word is not learned, and neither is a text of another length', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('arshad ');
    await waitFor(() => {
      expect(input.value).toBe('ارشد ');
    });
    await user.keyboard('ملتانی');
    await user.tab();
    expect(urduApi.translitLearn).not.toHaveBeenCalled();
  });

  it('a click on a spelling takes it, and the field keeps its focus', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    const second = options()[1];
    expect(second).toBeDefined();
    if (second !== undefined) fireEvent.mouseDown(second);
    expect(input.value).toBe('تارق');
    expect(input).toHaveFocus();
    expect(urduApi.translitLearn).toHaveBeenCalledWith(expect.objectContaining({ urdu: 'تارق' }));
  });

  it('Esc closes the list and keeps the English letters, and leaving does not convert them', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    await user.keyboard('{Escape}');
    expect(options()).toHaveLength(0);
    expect(input.value).toBe('tariq');
    await user.tab();
    expect(input.value).toBe('tariq');
    expect(urduApi.translitLearn).not.toHaveBeenCalled();
  });

  it('typing on after Esc asks for the longer word again', async () => {
    const { user } = await setup(language);
    await user.keyboard('tari');
    await user.keyboard('{Escape}');
    await user.keyboard('q');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
  });

  it('Backspace straight after Space brings back the English letters of that word', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('tariq ');
    await waitFor(() => {
      expect(input.value).toBe('طارق ');
    });
    await user.keyboard('{Backspace}');
    expect(input.value).toBe('tariq');
    // The spellings are offered again, so he can choose another.
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
  });

  it('Backspace straight after Enter-taking does the same; Backspace after other typing is normal', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('arshad');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.keyboard('{Enter}');
    await user.click(input);
    await user.keyboard('{End}{Backspace}');
    expect(input.value).toBe('arshad');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
    await user.keyboard('{Backspace}');
    expect(input.value).toBe('arsha');
  });

  it('letters from a Windows Urdu keyboard pass through unchanged, with no request', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('ارشد ملتانی');
    expect(input.value).toBe('ارشد ملتانی');
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
    expect(options()).toHaveLength(0);
  });

  it('digits and punctuation are not words to convert', async () => {
    const { user } = await setup(language);
    await user.keyboard('12 (3)');
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
  });

  it('a command that is not ready gives no list and no error, and the letters stay', async () => {
    vi.mocked(urduApi.translitSuggest).mockRejectedValue(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    const { input, user } = await setup(language);
    await user.keyboard('tariq');
    await act(async () => {
      await Promise.resolve();
    });
    expect(options()).toHaveLength(0);
    expect(input.value).toBe('tariq');
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
    await user.keyboard('{Enter}');
    expect(screen.getByTestId('next')).toHaveFocus();
  });

  it('a slow answer for an older word never shows under the newer word', async () => {
    let answerFirst: (items: string[]) => void = () => undefined;
    vi.mocked(urduApi.translitSuggest).mockImplementation(({ text }) =>
      text === 'tariq'
        ? Promise.resolve(suggestionsFor({ toUrdu: { tariq: ['طارق'] } }, text, 'to_urdu'))
        : new Promise((resolve) => {
            answerFirst = (items) => {
              resolve(suggestionsFor({ toUrdu: { [text]: items } }, text, 'to_urdu'));
            };
          }),
    );
    const { user } = await setup(language);
    await user.keyboard('tari');
    await user.keyboard('q');
    await waitFor(() => {
      expect(options().map((option) => option.textContent)).toEqual(['طارق']);
    });
    await act(async () => {
      answerFirst(['غلط']);
      await Promise.resolve();
    });
    expect(options().map((option) => option.textContent)).toEqual(['طارق']);
  });

  it('a read-only field converts nothing and shows no list', async () => {
    const { input, user } = await setup(language, { initial: 'طارق', readOnly: true });
    await user.keyboard('tariq');
    expect(input.value).toBe('طارق');
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
  });
});

describe.each(LANGUAGES)('RomanUrduInput typing modes (%s)', (language) => {
  const badge = () =>
    screen.getByRole('button', {
      name: (name) => visible(name).startsWith(message(language, 'translit-mode-label')),
    });

  it('starts in رومن and shows the mode as a badge inside the field', async () => {
    await setup(language);
    expect(badge()).toHaveTextContent(message(language, 'translit-mode-roman'));
    expect(badge()).toHaveAttribute('title', message(language, 'translit-mode-full-roman'));
  });

  it('Alt+U goes رومن, اردو, EN, and round again; the badge follows', async () => {
    const { user } = await setup(language);
    await user.keyboard('{Alt>}u{/Alt}');
    expect(badge()).toHaveTextContent(message(language, 'translit-mode-urdu'));
    expect(badge()).toHaveAttribute('title', message(language, 'translit-mode-full-urdu'));
    await user.keyboard('{Alt>}u{/Alt}');
    expect(badge()).toHaveTextContent(message(language, 'translit-mode-english'));
    await user.keyboard('{Alt>}u{/Alt}');
    expect(badge()).toHaveTextContent(message(language, 'translit-mode-roman'));
  });

  it('a click on the badge changes the mode too, and keeps the focus in the field', async () => {
    const { input, user } = await setup(language);
    await user.click(badge());
    expect(badge()).toHaveTextContent(message(language, 'translit-mode-urdu'));
    expect(input).toHaveFocus();
  });

  it('in اردو the phonetic keyboard types Urdu letters: a ا, b ب, p پ, t ت, T ٹ, h ہ, H ھ', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('{Alt>}u{/Alt}');
    await user.keyboard('abptThH');
    expect(input.value).toBe('ابپتٹہھ');
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
  });

  it('in اردو digits, spaces and Urdu letters pass through, and a comma is the Urdu comma', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('{Alt>}u{/Alt}');
    await user.keyboard('a 12 ارشد,');
    expect(input.value).toBe('ا 12 ارشد،');
  });

  it('in اردو a key typed in the middle of the text goes at the cursor', async () => {
    const { input, user } = await setup(language, { initial: 'اب' });
    await user.keyboard('{Alt>}u{/Alt}');
    input.setSelectionRange(1, 1);
    await user.keyboard('p');
    expect(input.value).toBe('اپب');
    expect(input.selectionStart).toBe(2);
  });

  it('the key map opens with one click, shows every key with its letter, and a click types it', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('{Alt>}u{/Alt}');
    await user.click(
      screen.getByRole('button', { name: message(language, 'translit-keymap-open') }),
    );
    const map = screen.getByRole('group', { name: message(language, 'translit-keymap-title') });
    const keys = map.querySelectorAll('li');
    expect(keys).toHaveLength(KEY_MAP.length);
    expect(map).toHaveTextContent('T');
    expect(map).toHaveTextContent('ٹ');
    // Every key on the map types the letter it shows.
    for (const { key, letter } of KEY_MAP) expect(mapPhoneticKey(key)).toBe(letter);
    const tKey = Array.from(keys).find((item) => item.textContent === 'Tٹ');
    expect(tKey).toBeDefined();
    const tButton = tKey?.querySelector('button');
    expect(tButton).not.toBeNull();
    if (tButton != null) await user.click(tButton);
    expect(input.value).toBe('ٹ');
    expect(input).toHaveFocus();
    await user.click(
      screen.getByRole('button', { name: message(language, 'translit-keymap-close') }),
    );
    expect(screen.queryByRole('group')).not.toBeInTheDocument();
  });

  it('the key map button exists only in اردو mode', async () => {
    const { user } = await setup(language);
    expect(
      screen.queryByRole('button', { name: message(language, 'translit-keymap-open') }),
    ).not.toBeInTheDocument();
    await user.keyboard('{Alt>}u{/Alt}');
    expect(
      screen.getByRole('button', { name: message(language, 'translit-keymap-open') }),
    ).toBeInTheDocument();
    await user.keyboard('{Alt>}u{/Alt}');
    expect(
      screen.queryByRole('button', { name: message(language, 'translit-keymap-open') }),
    ).not.toBeInTheDocument();
  });

  it('in EN nothing is converted: English letters stay English, with no request and no list', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('{Alt>}u{/Alt}{Alt>}u{/Alt}');
    await user.keyboard('Tariq ');
    expect(input.value).toBe('Tariq ');
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
    expect(options()).toHaveLength(0);
  });

  it('going back to رومن converts again', async () => {
    const { user } = await setup(language);
    await user.keyboard('{Alt>}u{/Alt}{Alt>}u{/Alt}{Alt>}u{/Alt}');
    await user.keyboard('arshad');
    await waitFor(() => {
      expect(options()).toHaveLength(1);
    });
  });
});

describe('what never converts (Code Standards F41)', () => {
  it('a plain text field, as a picker or search box uses, takes English letters as typed', async () => {
    const Plain = () => {
      const [value, setValue] = useState('');
      return <TextInput value={value} onChange={setValue} />;
    };
    renderIn('ur', <Plain />);
    const user = userEvent.setup();
    await user.type(screen.getByRole('textbox'), 'arshd ');
    expect(screen.getByRole<HTMLInputElement>('textbox').value).toBe('arshd ');
    expect(urduApi.translitSuggest).not.toHaveBeenCalled();
  });
});

describe('the phonetic keyboard table', () => {
  it.each([
    ['a', 'ا'],
    ['b', 'ب'],
    ['p', 'پ'],
    ['t', 'ت'],
    ['T', 'ٹ'],
    ['h', 'ہ'],
    ['H', 'ھ'],
    ['A', 'آ'],
    ['S', 'ص'],
    ['s', 'س'],
    ['N', 'ں'],
    ['?', '؟'],
  ])('%s types %s', (key, letter) => {
    expect(mapPhoneticKey(key)).toBe(letter);
  });

  it('leaves digits, spaces, Urdu letters, dots and keys like Enter alone', () => {
    for (const key of ['1', ' ', 'ا', '.', '-', 'Enter', 'Backspace', '(']) {
      expect(mapPhoneticKey(key)).toBeNull();
    }
  });

  it('gives every Latin letter a key', () => {
    for (const letter of 'abcdefghijklmnopqrstuvwxyz') {
      expect(mapPhoneticKey(letter), letter).not.toBeNull();
      expect(mapPhoneticKey(letter.toUpperCase()), letter.toUpperCase()).not.toBeNull();
    }
  });
});
