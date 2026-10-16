// A name in two scripts (plan 05, PRD FR-7.24, FR-7.22, Code Standards F41): the one typed fills
// the other, and the user only checks. In the Urdu and in the English app, against a fake of the
// real translit command shapes. The worked examples of plan 05 Section 3 are here.

import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import { NamePair, type NamePairValue } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { fail } from '../helpers/app';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/urdu/api');

const TO_URDU: Record<string, string[]> = {
  tariq: ['طارق', 'تارق'],
  mehmood: ['محمود', 'مہمود'],
  arshad: ['ارشد'],
  multani: ['ملتانی'],
  'arshad multani': ['ارشد ملتانی'],
  'tariq mehmood': ['طارق محمود'],
};
const TO_ENGLISH: Record<string, string> = { ارشد: 'Arshad', ملتانی: 'Multani' };

function fakeCommand() {
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(suggestionsFor({ toUrdu: TO_URDU, toEnglish: TO_ENGLISH }, text, direction)),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
}

interface HarnessProps {
  initial?: NamePairValue;
  urduError?: boolean;
  englishError?: boolean;
  onValue?: (value: NamePairValue) => void;
}

function Harness({ initial, urduError, englishError, onValue }: HarnessProps) {
  const [value, setValue] = useState<NamePairValue>(initial ?? { urdu: '', english: '' });
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <NamePair
        value={value}
        onChange={(next) => {
          setValue(next);
          onValue?.(next);
        }}
        urduLabel={message('en', 'common-loading')}
        englishLabel={message('en', 'common-retry')}
        urduError={urduError === true ? message('en', 'err-required') : undefined}
        englishError={englishError === true ? message('en', 'err-overflow') : undefined}
      />
      <input data-testid="next" />
    </form>
  );
}

function render(language: Language, props: HarnessProps = {}) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(language, <Harness {...props} />);
  // The Urdu field is the one with the typing mode; the English one is a plain text field.
  const urdu = screen.getByRole<HTMLInputElement>('combobox');
  const english = screen.getByLabelText<HTMLInputElement>(message('en', 'common-retry'));
  return { urdu, english, user: userEvent.setup() };
}

const options = () => screen.queryAllByRole('option');
const noteOf = (input: HTMLElement) =>
  visible(document.getElementById(input.getAttribute('aria-describedby') ?? '')?.textContent);

beforeEach(() => {
  vi.clearAllMocks();
  fakeCommand();
});

describe.each(LANGUAGES)('NamePair (%s)', (language) => {
  it('puts the name to type first: Urdu in the Urdu app, English in the English app', () => {
    const { urdu, english } = render(language);
    const [first, second] = language === 'ur' ? [urdu, english] : [english, urdu];
    expect(first.compareDocumentPosition(second) & Node.DOCUMENT_POSITION_FOLLOWING).not.toBe(0);
  });

  it('"tariq mehmood" gives طارق محمود, and the English name is "Tariq Mehmood"', async () => {
    const { urdu, english, user } = render(language);
    await user.click(urdu);
    await user.keyboard('tariq ');
    await user.keyboard('mehmood');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    await user.keyboard('{Enter}');
    expect(urdu.value).toBe('طارق محمود');
    // Exactly what was typed, with capital first letters, with no help from the backend.
    await waitFor(() => {
      expect(english.value).toBe('Tariq Mehmood');
    });
    expect(urduApi.translitSuggest).not.toHaveBeenCalledWith(
      expect.objectContaining({ direction: 'to_english' }),
    );
    // Enter moved on to the next field: the English name in the Urdu app, the field after the pair
    // in the English app (where the Urdu name comes last).
    expect(language === 'ur' ? english : screen.getByTestId('next')).toHaveFocus();
  });

  it('the English name follows the word while it is being typed', async () => {
    const { english, urdu, user } = render(language);
    await user.click(urdu);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(english.value).toBe('Tariq');
    });
    await user.keyboard(' mehm');
    await waitFor(() => {
      expect(english.value).toBe('Tariq Mehm');
    });
  });

  it('Urdu typed with Urdu letters gets an English suggestion from the backend', async () => {
    const { english, urdu, user } = render(language);
    await user.click(urdu);
    await user.keyboard('ارشد ملتانی');
    await waitFor(() => {
      expect(english.value).toBe('Arshad Multani');
    });
    expect(urduApi.translitSuggest).toHaveBeenCalledWith({
      text: 'ارشد ملتانی',
      direction: 'to_english',
    });
  });

  it('the English name typed first gives the Urdu name', async () => {
    const { english, urdu, user } = render(language);
    await user.click(english);
    await user.keyboard('Arshad Multani');
    await waitFor(() => {
      expect(urdu.value).toBe('ارشد ملتانی');
    });
  });

  it('once he edits the English name it is never filled again', async () => {
    const { english, urdu, user } = render(language);
    await user.click(urdu);
    await user.keyboard('tariq ');
    await waitFor(() => {
      expect(english.value).toBe('Tariq');
    });
    await user.click(english);
    await user.keyboard('{End} M.');
    expect(english.value).toBe('Tariq M.');
    await user.click(urdu);
    await user.keyboard('{End}mehmood{Enter}');
    expect(urdu.value).toBe('طارق محمود');
    expect(english.value).toBe('Tariq M.');
  });

  it('once he edits the Urdu name it is never filled again', async () => {
    const { english, urdu, user } = render(language);
    await user.click(english);
    await user.keyboard('Arshad Multani');
    await waitFor(() => {
      expect(urdu.value).toBe('ارشد ملتانی');
    });
    await user.click(urdu);
    await user.keyboard('{End} ا');
    await user.click(english);
    await user.keyboard('{End}x');
    expect(urdu.value).toBe('ارشد ملتانی ا');
  });

  it('a name that arrives filled (a saved record) is never overwritten', async () => {
    const { english, urdu, user } = render(language, {
      initial: { urdu: 'ارشد', english: 'Arshad' },
    });
    await user.click(urdu);
    await user.keyboard('{End} ملتانی');
    expect(english.value).toBe('Arshad');
    await user.click(english);
    await user.keyboard('{End}x');
    expect(urdu.value).toBe('ارشد ملتانی');
  });

  it('a spelling that cannot be worked out leaves the other name as it is, with no message', async () => {
    vi.mocked(urduApi.translitSuggest).mockRejectedValue(fail({ code: 'ERR_NOT_IMPLEMENTED' }));
    const { english, urdu, user } = render(language);
    await user.click(urdu);
    await user.keyboard('ارشد');
    await act(async () => {
      await Promise.resolve();
    });
    expect(english.value).toBe('');
    expect(screen.queryByRole('alert')).not.toBeInTheDocument();
  });

  it('a backend message sits under the right field, in place of the hint', () => {
    const { urdu, english } = render(language, { urduError: true, englishError: true });
    expect(urdu).toHaveAttribute('aria-invalid', 'true');
    expect(noteOf(urdu)).toBe(message('en', 'err-required'));
    expect(english).toHaveAttribute('aria-invalid', 'true');
    expect(noteOf(english)).toBe(message('en', 'err-overflow'));
  });

  it('says what each field takes, and flags a wrong script at once', async () => {
    const { urdu, english, user } = render(language);
    // The name to type says how; the one that fills says so (the roles swap in the English app).
    expect(noteOf(urdu)).toBe(
      message(
        language,
        language === 'ur' ? 'translit-pair-urdu-hint' : 'translit-pair-urdu-hint-second',
        { key: 'Alt+U' },
      ),
    );
    expect(noteOf(english)).toBe(
      message(
        language,
        language === 'ur' ? 'translit-pair-english-hint' : 'translit-pair-english-hint-first',
      ),
    );
    await user.click(english);
    await user.keyboard('ارشد');
    expect(english).toHaveAttribute('aria-invalid', 'true');
    expect(noteOf(english)).toBe(message(language, 'err-name-en-script'));
    // An English word waiting to become Urdu is not a mistake while it is being typed.
    await user.click(urdu);
    await user.keyboard('arsh');
    expect(urdu).not.toHaveAttribute('aria-invalid', 'true');
    // Once the field is left with English letters still in it, they are.
    await user.tab();
    await waitFor(() => {
      expect(urdu).toHaveAttribute('aria-invalid', 'true');
    });
    expect(noteOf(urdu)).toBe(message(language, 'err-name-ur-script'));
  });

  it('learns the spelling he picks over the first one', async () => {
    const { urdu, user } = render(language);
    await user.click(urdu);
    await user.keyboard('tariq');
    await waitFor(() => {
      expect(options()).toHaveLength(2);
    });
    await user.keyboard('{ArrowDown}{Enter}');
    expect(urdu.value).toBe('تارق');
    expect(urduApi.translitLearn).toHaveBeenCalledWith({
      roman: 'tariq',
      urdu: 'تارق',
      kind: 'name',
    });
  });

  it('a keyboard-only run: type the name, Enter, check the English name, Enter, on to the next field', async () => {
    const { urdu, english, user } = render(language);
    await user.click(urdu);
    await user.keyboard('arshad ');
    await user.keyboard('multani{Enter}');
    await waitFor(() => {
      expect(english.value).toBe('Arshad Multani');
    });
    expect(urdu.value).toBe('ارشد ملتانی');
    // Enter moved on to the next field of the form, which is the English name (Urdu app) or the
    // field after the pair (English app, where the Urdu name is last).
    expect(language === 'ur' ? english : screen.getByTestId('next')).toHaveFocus();
  });
});
