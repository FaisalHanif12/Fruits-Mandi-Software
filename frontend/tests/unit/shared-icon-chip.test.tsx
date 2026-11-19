// The optional picture and guide of the shared pieces (plan 75 §7.6): a soft icon chip before the title of a
// `Card` and of a `ListPage`, at the end of the label row of a `Fact` that is a stat card, `Balance` written at
// the size of the text around it, and a `SegmentedControl` whose choices open a guide card when they are
// pointed at or reached with the keyboard. Every prop is optional: nothing changes where it is not given.
// The words are real messages of the app (the lint rule keeps text out of code, also here).

import { Localized, useLocalization } from '@fluent/react';
import { screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { Wallet } from 'lucide-react';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { Balance, Card, Fact, ListPage, Num, SegmentedControl } from '../../src/shared/ui';
import { paisaForTest } from '../helpers/paisa';
import { message, renderIn } from '../helpers/render';

describe.each(LANGUAGES)(
  'the soft icon chip and the guide of the shared pieces (%s)',
  (language) => {
    it('a Card with an icon has the chip before its title, in one row; without one it is as it was', () => {
      renderIn(
        language,
        <>
          <Card title={<Localized id="common-back" />} icon={Wallet}>
            <p />
          </Card>
          <Card title={<Localized id="common-retry" />}>
            <p />
          </Card>
        </>,
      );
      const heading = screen.getByRole('heading', { name: message(language, 'common-back') });
      const chip = heading.previousElementSibling;
      expect(chip).toHaveAttribute('aria-hidden', 'true');
      expect(chip).toHaveClass('bg-primary-soft', 'text-primary');
      expect(heading.parentElement).toHaveClass('flex', 'items-center');
      // The card without an icon has its heading directly in the card, as before.
      const plain = screen.getByRole('heading', { name: message(language, 'common-retry') });
      expect(plain.parentElement?.tagName).toBe('SECTION');
    });

    it('a ListPage with an icon has the chip before its name, and the name is still the heading', () => {
      renderIn(
        language,
        <ListPage titleId="common-back" icon={Wallet}>
          <p />
        </ListPage>,
      );
      const heading = screen.getByRole('heading', {
        level: 1,
        name: message(language, 'common-back'),
      });
      expect(heading.parentElement?.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
    });

    it('a stat Fact with an icon has the chip at the end of its label row; a plain Fact ignores the icon', () => {
      renderIn(
        language,
        <dl>
          <Fact stat icon={Wallet} label={<Localized id="common-back" />}>
            <Num>{'12,000'}</Num>
          </Fact>
          <Fact icon={Wallet} label={<Localized id="common-retry" />}>
            <Num>{'5'}</Num>
          </Fact>
        </dl>,
      );
      const stat = screen.getByText(message(language, 'common-back')).closest('dt');
      expect(stat?.querySelector('svg')).toHaveAttribute('aria-hidden', 'true');
      expect(stat).toHaveClass('flex', 'justify-between');
      expect(
        screen.getByText(message(language, 'common-retry')).closest('dt')?.querySelector('svg'),
      ).toBeNull();
    });

    it('a Balance written at the size of the text around it passes that on to its number', () => {
      renderIn(
        language,
        <>
          <Balance inherit balance={{ amount_paisa: paisaForTest(288_000), side: 'dr' }} />
          <Balance balance={{ amount_paisa: paisaForTest(100), side: 'zero' }} />
        </>,
      );
      expect(screen.getByText('2,880')).toHaveClass('number-run');
      expect(screen.getByText('1')).toHaveClass('type-number');
    });

    function Subject({ onPick }: { onPick: (value: string) => void }) {
      const { l10n } = useLocalization();
      const [value, setValue] = useState('a');
      return (
        <SegmentedControl
          groupLabel={l10n.getString('common-back')}
          value={value}
          onChange={(next) => {
            setValue(next);
            onPick(next);
          }}
          options={[
            {
              value: 'a',
              label: <Localized id="common-retry" />,
              detailsLabel: l10n.getString('reports-title'),
              details: (
                <p className="p-4">
                  <Localized id="reports-help" />
                </p>
              ),
            },
            { value: 'b', label: <Localized id="common-back" /> },
          ]}
        />
      );
    }

    it('a choice with details opens its card for the keyboard and keeps its order and its click; one without has none', async () => {
      const picked = vi.fn();
      renderIn(language, <Subject onPick={picked} />);
      const user = userEvent.setup();
      const first = screen.getByRole('button', { name: message(language, 'common-retry') });
      const second = screen.getByRole('button', { name: message(language, 'common-back') });
      await user.tab();
      expect(first).toHaveFocus();
      await waitFor(() => {
        expect(screen.getByText(message(language, 'reports-help'))).toBeInTheDocument();
      });
      // Tab goes on to the next choice in order, which has no card, and the card of the first closes.
      await user.tab();
      expect(second).toHaveFocus();
      await waitFor(() => {
        expect(screen.queryByText(message(language, 'reports-help'))).toBeNull();
      });
      await user.click(second);
      expect(picked).toHaveBeenCalledWith('b');
      expect(second).toHaveAttribute('aria-pressed', 'true');
    });
  },
);
