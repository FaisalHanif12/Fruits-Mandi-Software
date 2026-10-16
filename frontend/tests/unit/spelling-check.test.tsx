// The spelling check of names (plan 05 v1.2, PRD FR-7.66, E49): a word made by the spelling rules
// alone is marked in amber, a save asks the user to confirm or fix it, and a refusal from the
// backend (ERR_SPELLING_UNCONFIRMED) shows the same dialog. In the Urdu and in the English app,
// against a fake of the real translit command shapes.

import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useSpellingFlow } from '../../src/shared/hooks';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import { NamePair, type NamePairValue } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import {
  NO_MARKS,
  confirmSpelling,
  confirmedSpelling,
  hasPendingSpelling,
  isInitial,
  pendingSpelling,
  rulesOnlyWords,
  withMark,
  wordsStillIn,
} from '../../src/shared/urdu/spelling';
import { fail } from '../helpers/app';
import { message, renderIn } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/urdu/api');

/** A rare name the word list does not know: its Urdu spelling is made by the rules alone. */
const RARE_URDU = 'ظہیر';
const RARE_ENGLISH = 'Zaheer';

interface Fake {
  /** Words (English lower case, or Urdu) whose spellings come from the rules alone. */
  rulesOnly?: readonly string[];
}

function fakeCommand({ rulesOnly = [] }: Fake = {}) {
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        {
          toUrdu: { zaheer: [RARE_URDU], ahmad: ['احمد'] },
          toEnglish: { [RARE_URDU]: RARE_ENGLISH, احمد: 'Ahmad' },
          rulesOnly,
        },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
}

type Run = (confirmed: string[]) => Promise<string>;

function Harness({
  run,
  initial,
  onFix,
}: {
  run: Run;
  initial?: NamePairValue | undefined;
  onFix?: () => void;
}) {
  const [names, setNames] = useState<NamePairValue>(initial ?? { urdu: '', english: '' });
  const [outcome, setOutcome] = useState('');
  const spelling = useSpellingFlow({
    onFix: () => {
      onFix?.();
      document.querySelector<HTMLInputElement>('[role="combobox"]')?.focus();
    },
  });
  useEscapeDispatcher();
  return (
    <form {...formNavigationProps}>
      <NamePair
        value={names}
        onChange={setNames}
        urduLabel={message('en', 'common-loading')}
        englishLabel={message('en', 'common-retry')}
      />
      <button
        type="button"
        data-testid="save"
        onClick={() => {
          spelling
            .guard(names, setNames, run)
            .then((result) => {
              setOutcome(result ?? 'fixing');
            })
            .catch(() => {
              setOutcome('failed');
            });
        }}
      >
        {message('en', 'common-back')}
      </button>
      <p data-testid="outcome">{outcome}</p>
      {spelling.dialog}
    </form>
  );
}

function setup(language: Language, props: { run?: Run; initial?: NamePairValue | undefined } = {}) {
  const run = vi.fn<Run>(props.run ?? (() => Promise.resolve('saved')));
  renderIn(language, <Harness run={run} initial={props.initial} />);
  const urdu = screen.getByRole<HTMLInputElement>('combobox');
  const english = screen.getByLabelText<HTMLInputElement>(message('en', 'common-retry'));
  return { run, urdu, english, user: userEvent.setup() };
}

const outcome = () => screen.getByTestId('outcome').textContent;
const dialog = () => screen.queryByRole('alertdialog');
const marks = () => screen.queryAllByRole('status');

beforeEach(() => {
  vi.clearAllMocks();
  fakeCommand({ rulesOnly: ['zaheer'] });
});

describe('spelling helpers', () => {
  it('marks only the words of a suggestion that the rules alone made, never an initial', () => {
    expect(
      rulesOnlyWords({
        text: 'کوٹ ظہیر ا',
        source: 'rules',
        words: [
          { text: 'کوٹ', source: 'word_list' },
          { text: 'ظہیر', source: 'rules' },
          { text: 'ا', source: 'rules' },
        ],
      }),
    ).toEqual(['ظہیر']);
    expect(rulesOnlyWords(undefined)).toEqual([]);
    expect(isInitial('M')).toBe(true);
    expect(isInitial('Ma')).toBe(false);
  });

  it('keeps marks next to the text, and only the words still in it count', () => {
    let value: NamePairValue = { urdu: `احمد ${RARE_URDU}`, english: 'Ahmad Zaheer' };
    value = { ...value, spelling: withMark(value.spelling, 'urdu', RARE_URDU) };
    expect(pendingSpelling(value)).toEqual({ urdu: [RARE_URDU], english: [] });
    expect(hasPendingSpelling(value)).toBe(true);
    // The word was changed by hand: nothing to check any more.
    expect(hasPendingSpelling({ ...value, urdu: 'احمد ظہیرا' })).toBe(false);
    expect(wordsStillIn('احمد ظہیر احمد', ['احمد'])).toEqual(['احمد']);
  });

  it('confirming moves a word from "to check" to "confirmed" and it goes with later saves', () => {
    const value: NamePairValue = {
      urdu: RARE_URDU,
      english: RARE_ENGLISH,
      spelling: withMark(NO_MARKS, 'urdu', RARE_URDU),
    };
    const confirmed = confirmSpelling(value, [RARE_URDU]);
    expect(hasPendingSpelling(confirmed)).toBe(false);
    expect(confirmedSpelling(confirmed)).toEqual([RARE_URDU]);
    // A word that left the name is no longer sent.
    expect(confirmedSpelling({ ...confirmed, urdu: 'احمد' })).toEqual([]);
    // A confirmed word is not marked again.
    expect(withMark(confirmed.spelling, 'urdu', RARE_URDU)).toBe(confirmed.spelling);
  });
});

describe.each(LANGUAGES)('The spelling check (%s)', (language) => {
  it('marks a word made by the rules alone in amber, under its name, with the words to check', async () => {
    const { urdu, english, user, run } = setup(language);
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(english.value).toBe(RARE_ENGLISH);
    });
    const note = within(urdu.closest('[dir]')?.parentElement ?? document.body).getByRole('status');
    expect(note.textContent).toContain(message(language, 'translit-spelling-check'));
    expect(note.textContent).toContain(RARE_URDU);
    expect(note).toHaveClass('bg-warn-soft');
    // The English name is exactly what was typed: nothing to check there.
    expect(marks()).toHaveLength(1);
    expect(run).not.toHaveBeenCalled();
  });

  it('a word from the word list is not marked', async () => {
    const { urdu, english, user } = setup(language);
    await user.click(urdu);
    await user.keyboard('ahmad ');
    await waitFor(() => {
      expect(english.value).toBe('Ahmad');
    });
    expect(marks()).toHaveLength(0);
  });

  it('changing the marked word by hand takes the mark away', async () => {
    const { urdu, english, user } = setup(language);
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(marks()).toHaveLength(1);
    });
    await user.click(urdu);
    await user.keyboard('{Backspace}{Backspace}ی ');
    await waitFor(() => {
      expect(marks()).toHaveLength(0);
    });
    expect(english.value).not.toBe('');
  });

  it('an English name made from Urdu letters by the rules alone is marked too', async () => {
    fakeCommand({ rulesOnly: [RARE_URDU] });
    const { urdu, english, user } = setup(language);
    await user.click(urdu);
    await user.keyboard(RARE_URDU);
    await waitFor(() => {
      expect(english.value).toBe(RARE_ENGLISH);
    });
    const note = await screen.findByRole('status');
    expect(note.textContent).toContain(message(language, 'translit-spelling-check'));
    expect(note.textContent).toContain(RARE_ENGLISH);
  });

  it('an Urdu name made from the English name by the rules alone is marked', async () => {
    const { urdu, english, user } = setup(language);
    await user.click(english);
    await user.keyboard(RARE_ENGLISH);
    await waitFor(() => {
      expect(urdu.value).toBe(RARE_URDU);
    });
    const note = await screen.findByRole('status');
    expect(note.textContent).toContain(RARE_URDU);
  });

  it('save with a marked word asks first; confirming repeats nothing and sends the word', async () => {
    const { urdu, user, run } = setup(language);
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(marks()).toHaveLength(1);
    });
    await user.click(screen.getByTestId('save'));
    const box = await screen.findByRole('alertdialog');
    expect(within(box).getByText(message(language, 'translit-spelling-title'))).toBeInTheDocument();
    expect(
      within(box).getByText(message(language, 'translit-spelling-urdu-name')),
    ).toBeInTheDocument();
    // The name is shown with the word marked.
    expect(box.querySelector('mark')?.textContent).toBe(RARE_URDU);
    expect(run).not.toHaveBeenCalled();
    await user.click(
      within(box).getByRole('button', {
        name: new RegExp(message(language, 'translit-spelling-confirm')),
      }),
    );
    await waitFor(() => {
      expect(outcome()).toBe('saved');
    });
    expect(run).toHaveBeenCalledTimes(1);
    expect(run).toHaveBeenCalledWith([RARE_URDU]);
    expect(dialog()).not.toBeInTheDocument();
    // The mark is gone: the word is confirmed.
    expect(marks()).toHaveLength(0);
    // A second save does not ask again, and still sends the word.
    await user.click(screen.getByTestId('save'));
    await waitFor(() => {
      expect(run).toHaveBeenCalledTimes(2);
    });
    expect(run).toHaveBeenLastCalledWith([RARE_URDU]);
    expect(dialog()).not.toBeInTheDocument();
  });

  it('going back to fix does not save and puts the cursor in the Urdu name', async () => {
    const { urdu, user, run } = setup(language);
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(marks()).toHaveLength(1);
    });
    await user.click(screen.getByTestId('save'));
    const box = await screen.findByRole('alertdialog');
    await user.click(
      within(box).getByRole('button', { name: new RegExp(message(language, 'common-back')) }),
    );
    await waitFor(() => {
      expect(outcome()).toBe('fixing');
    });
    expect(run).not.toHaveBeenCalled();
    await waitFor(() => {
      expect(urdu).toHaveFocus();
    });
    // The mark stays until he fixes it or confirms it.
    expect(marks()).toHaveLength(1);
  });

  it('Esc is the same as going back', async () => {
    const { urdu, user, run } = setup(language);
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(marks()).toHaveLength(1);
    });
    await user.click(screen.getByTestId('save'));
    await screen.findByRole('alertdialog');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(outcome()).toBe('fixing');
    });
    expect(run).not.toHaveBeenCalled();
  });

  it('a name with no marked word saves at once', async () => {
    const { urdu, user, run } = setup(language);
    await user.click(urdu);
    await user.keyboard('ahmad ');
    await waitFor(() => {
      expect(run).not.toHaveBeenCalled();
    });
    await user.click(screen.getByTestId('save'));
    await waitFor(() => {
      expect(outcome()).toBe('saved');
    });
    expect(run).toHaveBeenCalledWith([]);
    expect(dialog()).not.toBeInTheDocument();
  });

  it('ERR_SPELLING_UNCONFIRMED from the backend shows the same dialog, and confirming saves again', async () => {
    fakeCommand();
    let calls = 0;
    const { urdu, user, run } = setup(language, {
      run: (confirmed) => {
        calls += 1;
        return calls === 1
          ? Promise.reject(
              fail({
                code: 'ERR_SPELLING_UNCONFIRMED',
                field: 'name_ur',
                values: { words: RARE_URDU },
              }),
            )
          : Promise.resolve(`saved with ${confirmed.join(' ')}`);
      },
    });
    await user.click(urdu);
    await user.keyboard(RARE_URDU);
    await waitFor(() => {
      expect(urduApi.translitSuggest).toHaveBeenCalled();
    });
    await user.click(screen.getByTestId('save'));
    const box = await screen.findByRole('alertdialog');
    expect(box.querySelector('mark')?.textContent).toBe(RARE_URDU);
    await user.click(
      within(box).getByRole('button', {
        name: new RegExp(message(language, 'translit-spelling-confirm')),
      }),
    );
    await waitFor(() => {
      expect(outcome()).toBe(`saved with ${RARE_URDU}`);
    });
    expect(run).toHaveBeenNthCalledWith(1, []);
    expect(run).toHaveBeenNthCalledWith(2, [RARE_URDU]);
  });

  it('a second refusal for the word he already confirmed is an error, not another dialog', async () => {
    fakeCommand();
    const { urdu, user, run } = setup(language, {
      run: () =>
        Promise.reject(
          fail({
            code: 'ERR_SPELLING_UNCONFIRMED',
            field: 'name_ur',
            values: { words: RARE_URDU },
          }),
        ),
    });
    await user.click(urdu);
    await user.keyboard(RARE_URDU);
    await user.click(screen.getByTestId('save'));
    const box = await screen.findByRole('alertdialog');
    await user.click(
      within(box).getByRole('button', {
        name: new RegExp(message(language, 'translit-spelling-confirm')),
      }),
    );
    await waitFor(() => {
      expect(outcome()).toBe('failed');
    });
    expect(run).toHaveBeenCalledTimes(2);
    expect(dialog()).not.toBeInTheDocument();
  });

  it('another kind of refusal goes straight on to the screen, with no dialog', async () => {
    fakeCommand();
    const { urdu, user } = setup(language, {
      run: () => Promise.reject(fail({ code: 'ERR_NAME_UR_SCRIPT', field: 'name_ur' })),
    });
    await user.click(urdu);
    await user.keyboard('ahmad ');
    await user.click(screen.getByTestId('save'));
    await waitFor(() => {
      expect(outcome()).toBe('failed');
    });
    expect(dialog()).not.toBeInTheDocument();
  });

  it('the marks live in the value, so a screen that comes back with the value shows them again', () => {
    setup(language, {
      initial: {
        urdu: RARE_URDU,
        english: RARE_ENGLISH,
        spelling: withMark(NO_MARKS, 'urdu', RARE_URDU),
      },
    });
    const note = screen.getByRole('status');
    expect(note.textContent).toContain(RARE_URDU);
  });

  it('the dialog is written in the language of the app, with the name in its own script', async () => {
    const { urdu, user } = setup(language);
    await user.click(urdu);
    await user.keyboard('zaheer ');
    await waitFor(() => {
      expect(marks()).toHaveLength(1);
    });
    await user.click(screen.getByTestId('save'));
    const box = await screen.findByRole('alertdialog');
    expect(box).toHaveAttribute('lang', language);
    expect(within(box).getByText(message(language, 'translit-spelling-body'))).toBeInTheDocument();
    const name = box.querySelector('bdi[lang="ur"]');
    expect(name).toHaveAttribute('dir', 'rtl');
    await act(async () => {
      await Promise.resolve();
    });
  });
});
