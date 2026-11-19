// The shared pieces of a clean list page (plan 38 G1, G3, G4, G5, G6): the form grid, the filter bar
// with "More filters", the period box, the tick for closed records and the info button. Urdu and
// English. Nothing here is money.

import { act, render, screen, waitFor } from '@testing-library/react';
import { Localized } from '@fluent/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it } from 'vitest';
import { AppProviders, createQueryClient } from '../../src/app/providers';
import { LANGUAGES, type Language } from '../../src/i18n';
import { usePeriod } from '../../src/shared/hooks';
import { useLanguageStore } from '../../src/shared/i18n';
import {
  FilterBar,
  FormGrid,
  InfoHint,
  PeriodBox,
  ShowClosedTick,
  TextInput,
  Field,
} from '../../src/shared/ui';
import { message, visible } from '../helpers/render';

function mount(node: React.ReactElement) {
  return render(<AppProviders queryClient={createQueryClient()}>{node}</AppProviders>);
}

type Box = 'more-set' | 'none';

const BAR_SCREEN = 'test.bar';

function Bar({ rare }: { rare: Box }) {
  const [text, setText] = useState('');
  const [rareText, setRareText] = useState(rare === 'more-set' ? 'x' : '');
  const [closed, setClosed] = useState(false);
  const period = usePeriod(BAR_SCREEN, 'month', '2026-10-10');
  return (
    <FilterBar
      screen={BAR_SCREEN}
      search={{ value: text, onChange: setText }}
      more={
        <Field label={<Localized id="common-pager-size" />}>
          {(control) => <TextInput {...control} value={rareText} onChange={setRareText} />}
        </Field>
      }
      moreSet={rareText === '' ? 0 : 1}
      extra={<ShowClosedTick checked={closed} onChange={setClosed} />}
    >
      {period.ready ? <PeriodBox period={period} /> : null}
    </FilterBar>
  );
}

describe.each(LANGUAGES)('the filter bar (%s)', (language: Language) => {
  beforeEach(() => {
    // The open or shut state of "More filters" is kept on this computer; a test starts shut.
    window.localStorage.clear();
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  it('has a search box, the period, and the rare filters shut behind "More filters"', async () => {
    const user = userEvent.setup();
    mount(<Bar rare="none" />);
    expect(
      await screen.findByRole('textbox', { name: message(language, 'common-search-label') }),
    ).toBeInTheDocument();
    // The period of a register starts on this month, and the box is a choice, not two empty dates.
    expect(
      await screen.findByRole('combobox', { name: message(language, 'common-period-label') }),
    ).toHaveTextContent(message(language, 'common-period-month'));
    expect(
      screen.queryByRole('textbox', { name: message(language, 'common-pager-size') }),
    ).not.toBeInTheDocument();
    await user.click(
      screen.getByRole('button', { name: message(language, 'common-filters-more') }),
    );
    expect(
      await screen.findByRole('textbox', { name: message(language, 'common-pager-size') }),
    ).toBeInTheDocument();
    // The button now says "Fewer filters" and is reached again by the keyboard.
    expect(
      screen.getByRole('button', { name: message(language, 'common-filters-less') }),
    ).toBeInTheDocument();
  });

  it('opens "More filters" with the keyboard and keeps it open while a hidden filter is set', async () => {
    const user = userEvent.setup();
    mount(<Bar rare="more-set" />);
    // A hidden filter that is set never filters in secret: the part is open and says how many.
    expect(
      await screen.findByRole('textbox', { name: message(language, 'common-pager-size') }),
    ).toBeInTheDocument();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'common-filters-set', { count: 1 }),
    );
    // It cannot be shut while the filter is set.
    await user.click(
      screen.getByRole('button', { name: message(language, 'common-filters-less') }),
    );
    expect(
      screen.getByRole('textbox', { name: message(language, 'common-pager-size') }),
    ).toBeInTheDocument();
  });

  it('choosing "Choose the dates" shows From and To with the days of the period', async () => {
    const user = userEvent.setup();
    mount(<Bar rare="none" />);
    const choice = await screen.findByRole('combobox', {
      name: message(language, 'common-period-label'),
    });
    await user.click(choice);
    await user.click(
      await screen.findByRole('option', { name: message(language, 'common-period-custom') }),
    );
    const from = await screen.findByRole('textbox', {
      name: message(language, 'common-period-from'),
    });
    const to = screen.getByRole('textbox', { name: message(language, 'common-period-to') });
    // They start from the days of this month, to be edited.
    expect(from).toHaveValue('01-10-2026');
    expect(to).toHaveValue('10-10-2026');
  });

  it('the tick for closed and cancelled records starts off and is a real checkbox', async () => {
    const user = userEvent.setup();
    mount(<Bar rare="none" />);
    const tick = await screen.findByRole('checkbox', {
      name: message(language, 'common-show-closed'),
    });
    expect(tick).not.toBeChecked();
    await user.click(tick);
    expect(tick).toBeChecked();
  });
});

describe.each(LANGUAGES)('the info button (%s)', (language: Language) => {
  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  it('opens its words with the keyboard and closes them with Esc', async () => {
    const user = userEvent.setup();
    const words = `${message(language, 'common-info-label')}!`;
    mount(
      <div>
        <InfoHint>
          <span>{words}</span>
        </InfoHint>
      </div>,
    );
    const button = screen.getByRole('button', { name: message(language, 'common-info-label') });
    button.focus();
    await user.keyboard('{Enter}');
    const bubble = await screen.findByText(words);
    expect(bubble).toBeVisible();
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(screen.queryByText(words)).not.toBeInTheDocument();
    });
  });
});

describe('the form grid', () => {
  it('is a container with columns that grow with the free width, never with the window', () => {
    const { container } = render(
      <FormGrid columns={3}>
        <span className="first" />
        <span className="second" />
      </FormGrid>,
    );
    const wrapper = container.firstElementChild;
    expect(wrapper?.className).toContain('@container');
    const grid = wrapper?.querySelector('.first')?.parentElement;
    expect(grid?.className).toContain('grid-cols-1');
    expect(grid?.className).toContain('@form2:grid-cols-2');
    expect(grid?.className).toContain('@form3:grid-cols-3');
    expect(grid?.className).not.toContain('@form4');
  });
});
