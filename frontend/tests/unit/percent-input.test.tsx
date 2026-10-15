// PercentInput (plan 11): a percent typed as text, never read as a number (CLAUDE.md rule 1). The
// percent sign sits after the field in both languages, and a field in a list can carry its name
// for a screen reader when its label is the column heading.

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { Field, PercentInput } from '../../src/shared/ui';
import { message, renderIn } from '../helpers/render';

const LABEL = String(JSON.parse('"Share of Partner A"'));

function Harness({
  onValue,
  hideLabel = false,
}: {
  onValue: (text: string) => void;
  hideLabel?: boolean;
}) {
  const [value, setValue] = useState('');
  return (
    <Field label={LABEL} hideLabel={hideLabel}>
      {(control) => (
        <PercentInput
          {...control}
          value={value}
          label={LABEL}
          onChange={(text) => {
            setValue(text);
            onValue(text);
          }}
        />
      )}
    </Field>
  );
}

describe.each(LANGUAGES)('PercentInput (%s)', (language) => {
  it('sends every key as text, keeps decimals and odd text for the backend to judge, and groups nothing', async () => {
    const onValue = vi.fn();
    renderIn(language, <Harness onValue={onValue} />);
    const input = screen.getByRole('textbox', { name: LABEL });
    await userEvent.setup().type(input, '33.33');
    expect(onValue).toHaveBeenLastCalledWith('33.33');
    // Text that is not a percent is not dropped here: the backend says what is wrong.
    await userEvent.setup().clear(input);
    await userEvent.setup().type(input, '1,5x');
    expect(onValue).toHaveBeenLastCalledWith('1,5x');
  });

  it('is written left to right with the percent sign after it, in both languages', () => {
    renderIn(language, <Harness onValue={vi.fn()} />);
    const input = screen.getByRole('textbox');
    expect(input).toHaveAttribute('dir', 'ltr');
    const wrapper = input.parentElement;
    expect(wrapper).toHaveAttribute('dir', 'ltr');
    expect(wrapper?.textContent).toBe(message(language, 'common-percent-sign'));
  });

  it('keeps the label for a screen reader when the column heading is the visible label', () => {
    renderIn(language, <Harness onValue={vi.fn()} hideLabel />);
    expect(screen.getByRole('textbox', { name: LABEL })).toBeInTheDocument();
    expect(screen.getByText(LABEL, { selector: 'label' })).toHaveClass('sr-only');
  });
});
