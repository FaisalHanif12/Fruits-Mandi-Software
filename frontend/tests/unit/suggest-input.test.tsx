// SuggestInput: a text field that suggests what was used before, such as a truck number (PRD
// FR-7.49). It takes text that is not in the list, so typing never picks by itself, Enter takes
// what is typed unless an arrow moved onto a suggestion, and leaving the field keeps the text.

import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { formNavigationProps } from '../../src/shared/keyboard';
import { SuggestInput, type Suggestion } from '../../src/shared/ui';
import { renderIn } from '../helpers/render';

const SUGGESTIONS: readonly Suggestion[] = [
  { value: 'LES-1234', note: '09-10-2026' },
  { value: 'LES-5678', note: '08-10-2026' },
];

function Rig({
  suggestions = SUGGESTIONS,
  onCommit,
}: {
  suggestions?: readonly Suggestion[];
  onCommit: (text: string) => void;
}) {
  const [value, setValue] = useState('');
  return (
    <div {...formNavigationProps}>
      <SuggestInput
        id="truck"
        value={value}
        onChange={setValue}
        suggestions={suggestions}
        onCommit={onCommit}
      />
      <input data-testid="next" />
    </div>
  );
}

describe.each(LANGUAGES)('SuggestInput (%s)', (language) => {
  const field = () => screen.getByRole<HTMLInputElement>('combobox');

  it('shows the suggestions with their notes while typing, and typing picks nothing', async () => {
    const onCommit = vi.fn();
    renderIn(language, <Rig onCommit={onCommit} />);
    const user = userEvent.setup();
    await user.click(field());
    await user.keyboard('LES');
    expect(await screen.findAllByRole('option')).toHaveLength(2);
    expect(screen.getByText('09-10-2026')).toBeInTheDocument();
    expect(field().value).toBe('LES');
    expect(onCommit).not.toHaveBeenCalled();
  });

  it('↓ then Enter takes the highlighted suggestion, commits it once, and moves on', async () => {
    const onCommit = vi.fn();
    renderIn(language, <Rig onCommit={onCommit} />);
    const user = userEvent.setup();
    await user.click(field());
    await user.keyboard('LES{ArrowDown}{Enter}');
    await waitFor(() => {
      expect(field().value).toBe('LES-1234');
    });
    expect(onCommit).toHaveBeenCalledTimes(1);
    expect(onCommit).toHaveBeenCalledWith('LES-1234');
    // Leaving the field does not hand the same text over a second time.
    await user.tab();
    expect(onCommit).toHaveBeenCalledTimes(1);
  });

  it('Enter with nothing highlighted takes the text as typed, a new truck number, and moves to the next field', async () => {
    const onCommit = vi.fn();
    renderIn(language, <Rig onCommit={onCommit} />);
    const user = userEvent.setup();
    await user.click(field());
    await user.keyboard('NEW-1{Enter}');
    expect(field().value).toBe('NEW-1');
    expect(onCommit).toHaveBeenCalledWith('NEW-1');
    expect(screen.getByTestId('next')).toHaveFocus();
  });

  it('Esc closes the list first and keeps the text', async () => {
    renderIn(language, <Rig onCommit={vi.fn()} />);
    const user = userEvent.setup();
    await user.click(field());
    await user.keyboard('LES');
    await screen.findAllByRole('option');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByRole('option')).not.toBeInTheDocument();
    });
    expect(field().value).toBe('LES');
  });

  it('leaving the field keeps the text and commits it', async () => {
    const onCommit = vi.fn();
    renderIn(language, <Rig onCommit={onCommit} />);
    const user = userEvent.setup();
    await user.click(field());
    await user.keyboard('XYZ-9');
    await user.tab();
    expect(field().value).toBe('XYZ-9');
    expect(onCommit).toHaveBeenCalledWith('XYZ-9');
  });

  it('with no suggestions it is a plain field: no list, Enter moves on', async () => {
    const onCommit = vi.fn();
    renderIn(language, <Rig suggestions={[]} onCommit={onCommit} />);
    const user = userEvent.setup();
    await user.click(field());
    await user.keyboard('A1');
    expect(screen.queryByRole('option')).not.toBeInTheDocument();
    await user.keyboard('{Enter}');
    expect(screen.getByTestId('next')).toHaveFocus();
  });
});
