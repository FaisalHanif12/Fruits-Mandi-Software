// The one merged list of a name field (plan 13, PRD FR-7.29): rows of known records come before the
// Urdu spellings. RomanUrduInput takes them as `extras`; SuggestInput as rows with `onPick`. The
// rows are never taken by Space or by leaving the field, only by Enter or a click on them, and none
// is highlighted until an arrow moves onto it, so Enter on a new name still moves on. With no
// extras the field works exactly as before (roman-urdu-input.test.tsx).

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { formNavigationProps, useEscapeDispatcher } from '../../src/shared/keyboard';
import {
  Field,
  RomanUrduInput,
  SuggestInput,
  type ListExtra,
  type Suggestion,
} from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/urdu/api');

const PEOPLE = [
  { key: 'shahid', ur: 'شاہد', en: 'Shahid' },
  { key: 'nasir', ur: 'ناصر', en: 'Nasir' },
];

function Rig({ onPick, rows = PEOPLE }: { onPick: (key: string) => void; rows?: typeof PEOPLE }) {
  const [value, setValue] = useState('');
  useEscapeDispatcher();
  const extras: ListExtra[] = rows.map((row) => ({
    key: row.key,
    content: (
      <span>
        {row.ur} {row.en}
      </span>
    ),
    onPick: () => {
      onPick(row.key);
    },
  }));
  return (
    <form {...formNavigationProps}>
      <Field label={message('en', 'common-back')}>
        {(control) => (
          <RomanUrduInput {...control} value={value} onChange={setValue} extras={extras} />
        )}
      </Field>
      <input data-testid="next" />
    </form>
  );
}

const options = () => screen.queryAllByRole('option');

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor({ toUrdu: { shah: ['شاہ'], shahid: ['شاہد', 'شاھد'] } }, text, direction),
    ),
  );
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
});

describe.each(LANGUAGES)(
  'RomanUrduInput with rows before the spellings (%s)',
  (language: Language) => {
    async function setup(onPick = vi.fn()) {
      renderIn(language, <Rig onPick={onPick} />);
      const input = screen.getByRole<HTMLInputElement>('combobox');
      const user = userEvent.setup();
      await user.click(input);
      return { input, user, onPick };
    }

    it('lists the rows first and then the spellings, with the first spelling highlighted', async () => {
      const { user } = await setup();
      await user.keyboard('shahid');
      await waitFor(() => {
        expect(options()).toHaveLength(4);
      });
      const texts = options().map((option) => visible(option.textContent));
      expect(texts[0]).toContain('شاہد');
      expect(texts[0]).toContain('Shahid');
      expect(texts[1]).toContain('Nasir');
      expect(texts.slice(2)).toEqual(['شاہد', 'شاھد']);
      expect(options().map((option) => option.getAttribute('aria-selected'))).toEqual([
        'false',
        'false',
        'true',
        'false',
      ]);
    });

    it('↑ from the first spelling goes up to the rows; Enter takes the row and moves on', async () => {
      const { input, user, onPick } = await setup();
      await user.keyboard('shahid');
      await waitFor(() => {
        expect(options()).toHaveLength(4);
      });
      await user.keyboard('{ArrowUp}{ArrowUp}');
      expect(options()[0]).toHaveAttribute('aria-selected', 'true');
      await user.keyboard('{Enter}');
      expect(onPick).toHaveBeenCalledWith('shahid');
      // The text typed is not turned into a spelling, and focus went on to the next field.
      expect(input.value).toBe('shahid');
      expect(screen.getByTestId('next')).toHaveFocus();
    });

    it('Enter takes the highlighted spelling when no row was moved to', async () => {
      const { input, user, onPick } = await setup();
      await user.keyboard('shahid');
      await waitFor(() => {
        expect(options()).toHaveLength(4);
      });
      await user.keyboard('{Enter}');
      expect(onPick).not.toHaveBeenCalled();
      expect(input.value).toBe('شاہد');
      expect(screen.getByTestId('next')).toHaveFocus();
    });

    it('Space takes the first spelling even if a row was moved to; it never takes a row', async () => {
      const { input, user, onPick } = await setup();
      await user.keyboard('shahid');
      await waitFor(() => {
        expect(options()).toHaveLength(4);
      });
      await user.keyboard('{ArrowUp}{ArrowUp}{ }');
      expect(onPick).not.toHaveBeenCalled();
      expect(input.value).toBe('شاہد ');
    });

    it('with no spelling for the word, Enter is a plain Enter, and ↓ then Enter takes a row', async () => {
      const { input, user, onPick } = await setup();
      await user.keyboard('zzz');
      await waitFor(() => {
        expect(options()).toHaveLength(2);
      });
      expect(options().every((option) => option.getAttribute('aria-selected') === 'false')).toBe(
        true,
      );
      // Space types a space: there is no spelling to take.
      await user.keyboard(' ');
      expect(input.value).toBe('zzz ');
      await user.keyboard('{ArrowDown}{Enter}');
      expect(onPick).toHaveBeenCalledWith('shahid');
    });

    it('a new name is not replaced: Enter with nothing moved to leaves the field and takes no row', async () => {
      const { input, user, onPick } = await setup();
      await user.keyboard('zzz{Enter}');
      expect(onPick).not.toHaveBeenCalled();
      expect(input.value).toBe('zzz');
      expect(screen.getByTestId('next')).toHaveFocus();
    });

    it('leaving the field takes no row', async () => {
      const { user, onPick } = await setup();
      await user.keyboard('zzz');
      await waitFor(() => {
        expect(options()).toHaveLength(2);
      });
      await user.tab();
      expect(onPick).not.toHaveBeenCalled();
    });

    it('a click on a row takes it and the field keeps focus', async () => {
      const { input, user, onPick } = await setup();
      await user.keyboard('zzz');
      await waitFor(() => {
        expect(options()).toHaveLength(2);
      });
      const second = options()[1];
      if (second === undefined) throw new Error('no row');
      await user.click(second);
      expect(onPick).toHaveBeenCalledWith('nasir');
      expect(input).toHaveFocus();
    });

    it('Esc closes the list, rows and spellings together', async () => {
      const { user } = await setup();
      await user.keyboard('shahid');
      await waitFor(() => {
        expect(options()).toHaveLength(4);
      });
      await user.keyboard('{Escape}');
      expect(options()).toHaveLength(0);
    });

    it('the rows show while the field has text and no English word (an Urdu name already typed)', async () => {
      const { user } = await setup();
      await user.keyboard('شاہد');
      expect(options()).toHaveLength(2);
      // None is highlighted, so Enter moves on.
      expect(options().every((option) => option.getAttribute('aria-selected') === 'false')).toBe(
        true,
      );
      await user.keyboard('{Enter}');
      expect(screen.getByTestId('next')).toHaveFocus();
    });
  },
);

function SuggestRig({
  onPick,
  onCommit,
}: {
  onPick: (key: string) => void;
  onCommit: (text: string) => void;
}) {
  const [value, setValue] = useState('');
  const suggestions: Suggestion[] = PEOPLE.map((row) => ({
    value: row.key,
    content: <span>{row.en}</span>,
    onPick: () => {
      onPick(row.key);
    },
  }));
  return (
    <div {...formNavigationProps}>
      <SuggestInput
        id="phone"
        value={value}
        onChange={setValue}
        suggestions={suggestions}
        onCommit={onCommit}
      />
      <input data-testid="next" />
    </div>
  );
}

describe.each(LANGUAGES)(
  'SuggestInput with rows that stand for a record (%s)',
  (language: Language) => {
    it('shows the row’s own content, and taking a row calls its onPick, not the field’s onChange', async () => {
      const onPick = vi.fn();
      const onCommit = vi.fn();
      renderIn(language, <SuggestRig onPick={onPick} onCommit={onCommit} />);
      const user = userEvent.setup();
      const input = screen.getByRole<HTMLInputElement>('combobox');
      await user.click(input);
      await user.keyboard('03');
      const rows = await screen.findAllByRole('option');
      expect(rows.map((row) => visible(row.textContent))).toEqual(['Shahid', 'Nasir']);
      await user.keyboard('{ArrowDown}{Enter}');
      expect(onPick).toHaveBeenCalledWith('shahid');
      // The text of the field is the user's own; the list did not put the row's value in it.
      expect(input.value).toBe('03');
      expect(onCommit).not.toHaveBeenCalledWith('shahid');
    });

    it('Enter with nothing highlighted takes the text as typed and moves on', async () => {
      const onPick = vi.fn();
      renderIn(language, <SuggestRig onPick={onPick} onCommit={vi.fn()} />);
      const user = userEvent.setup();
      await user.click(screen.getByRole('combobox'));
      await user.keyboard('03{Enter}');
      expect(onPick).not.toHaveBeenCalled();
      expect(screen.getByTestId('next')).toHaveFocus();
    });
  },
);
