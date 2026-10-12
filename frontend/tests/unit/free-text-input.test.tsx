// The free text field (PRD FR-7.22, FR-7.25, Code Standards F41): notes, narrations, reasons and
// addresses. In the Urdu app it starts in رومن mode and converts English letters; in the English
// app it starts in EN mode, converts nothing and is written left to right. Alt+U switches the mode.
// What is typed is kept exactly as typed.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import { Field, FreeTextInput } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { message, renderIn } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/urdu/api');

const TO_URDU: Record<string, string[]> = { ghar: ['گھر'], kharch: ['خرچ'] };

function Harness() {
  const [value, setValue] = useState('');
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => <FreeTextInput {...control} value={value} onChange={setValue} />}
      </Field>
    </form>
  );
}

async function setup(language: Language) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(language, <Harness />);
  const input = screen.getByRole<HTMLInputElement>('combobox');
  const user = userEvent.setup();
  await user.click(input);
  const wrapper = input.closest<HTMLElement>('[dir]');
  if (wrapper === null) throw new Error('no wrapper');
  return { input, user, wrapper };
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(suggestionsFor({ toUrdu: TO_URDU }, text, direction)),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

describe.each(LANGUAGES)('the free text field (%s)', (language) => {
  it(
    language === 'ur'
      ? 'starts in رومن mode: English letters become Urdu, and it is written right to left'
      : 'starts in EN mode: nothing is converted, and it is written left to right',
    async () => {
      const { input, user, wrapper } = await setup(language);
      const full = message(
        language,
        language === 'ur' ? 'translit-mode-full-roman' : 'translit-mode-full-english',
      );
      expect(
        screen.getByRole('button', { name: (name) => name.endsWith(full) }),
      ).toBeInTheDocument();
      await user.keyboard('ghar kharch ');
      if (language === 'ur') {
        await waitFor(() => {
          expect(input.value).toBe('گھر خرچ ');
        });
        expect(wrapper).toHaveAttribute('dir', 'rtl');
        expect(input).toHaveAttribute('lang', 'ur');
      } else {
        expect(input.value).toBe('ghar kharch ');
        expect(urduApi.translitSuggest).not.toHaveBeenCalled();
        expect(wrapper).toHaveAttribute('dir', 'ltr');
        expect(input).toHaveAttribute('lang', 'en');
      }
    },
  );

  it('Alt+U changes the mode, and the English app writes right to left again once it converts', async () => {
    const { input, user, wrapper } = await setup(language);
    // Urdu app: رومن, then اردو کی بورڈ, then EN. English app: EN, then رومن, then اردو کی بورڈ.
    await user.keyboard('{Alt>}u{/Alt}');
    if (language === 'en') {
      expect(wrapper).toHaveAttribute('dir', 'rtl');
      expect(input).toHaveAttribute('lang', 'ur');
      await user.keyboard('ghar ');
      await waitFor(() => {
        expect(input.value).toBe('گھر ');
      });
    } else {
      expect(wrapper).toHaveAttribute('dir', 'rtl');
    }
  });

  it('what is typed in EN mode is kept exactly as typed', async () => {
    const { input, user } = await setup(language);
    await user.keyboard('{Alt>}u{/Alt}');
    await user.keyboard('{Alt>}u{/Alt}');
    if (language === 'ur') {
      // The Urdu app is now in EN mode after two presses.
      await user.keyboard('Wrong rate');
      expect(input.value).toBe('Wrong rate');
    } else {
      // The English app went EN, رومن, اردو کی بورڈ: the phonetic keys type Urdu letters.
      await user.keyboard('b');
      expect(input.value).toBe('ب');
    }
  });
});
