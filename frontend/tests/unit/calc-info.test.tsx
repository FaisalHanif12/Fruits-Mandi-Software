// How was this worked out? (PRD FR-7.43, Code Standards F42, B30): the small ⓘ shows the working
// the backend sent, in the app language, and never builds a working itself. The backend sends each
// value with its kind (money, percent in basis points, count, level); the screen writes it as text.

import { fireEvent, screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { describe, expect, it } from 'vitest';
import { LANGUAGES } from '../../src/i18n';
import { CalcInfo, type Working } from '../../src/shared/ui';
import { money } from '../helpers/accounts';
import { message, renderIn, visible } from '../helpers/render';

const PERCENT: Working = {
  message_id: 'working-percent',
  args: [
    { name: 'base', value: { kind: 'money', paisa: money(12000) } },
    { name: 'bps', value: { kind: 'percent', bps: 600 } },
    { name: 'result', value: { kind: 'money', paisa: money(720) } },
    { name: 'level', value: { kind: 'level', level: 'default' } },
  ],
  rule_id: 1,
};

describe.each(LANGUAGES)('CalcInfo (%s)', (language) => {
  const label = () => message(language, 'calc-info-label');
  const open = async (working: Working) => {
    renderIn(language, <CalcInfo working={working} />);
    await userEvent.setup().click(screen.getByRole('button', { name: label() }));
    await screen.findByText(message(language, 'calc-info-title'));
  };

  it('shows nothing when the backend sent no working', () => {
    renderIn(language, <CalcInfo working={null} />);
    expect(screen.queryByRole('button')).not.toBeInTheDocument();
  });

  it('opens on a click and shows the working with the real numbers, money grouped and a percent from its basis points', async () => {
    await open(PERCENT);
    expect(visible(document.body.textContent)).toContain(
      message(language, 'working-percent', { base: '12,000', bps: '6', result: '720' }),
    );
  });

  it('a percent with decimals is written as a person writes it', async () => {
    await open({
      ...PERCENT,
      args: [
        { name: 'base', value: { kind: 'money', paisa: money(12000) } },
        { name: 'bps', value: { kind: 'percent', bps: 250 } },
        { name: 'result', value: { kind: 'money', paisa: money(300) } },
      ],
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'working-percent', { base: '12,000', bps: '2.5', result: '300' }),
    );
  });

  it('a count and a per-unit value', async () => {
    await open({
      message_id: 'working-per-unit',
      args: [
        { name: 'qty', value: { kind: 'count', value: 20 } },
        { name: 'value', value: { kind: 'money', paisa: money(8) } },
        { name: 'result', value: { kind: 'money', paisa: money(160) } },
      ],
      rule_id: 3,
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'working-per-unit', { qty: '20', value: '8', result: '160' }),
    );
  });

  it('a per-Nag charge of a lot says the quantity was the quantity received', async () => {
    await open({
      message_id: 'working-per-unit',
      args: [
        { name: 'qty_received', value: { kind: 'count', value: 50 } },
        { name: 'value', value: { kind: 'money', paisa: money(5) } },
        { name: 'result', value: { kind: 'money', paisa: money(250) } },
        { name: 'level', value: { kind: 'level', level: 'default' } },
      ],
      rule_id: 9,
    });
    expect(visible(document.body.textContent)).toContain(
      message(language, 'working-per-unit-received', {
        qty_received: '50',
        value: '5',
        result: '250',
      }),
    );
  });

  it('the working picks its own direction, so a sum reads left to right in Urdu too (NFR-URD-13)', async () => {
    await open(PERCENT);
    const working = screen.getByText(
      (content) =>
        visible(content) ===
        message(language, 'working-percent', { base: '12,000', bps: '6', result: '720' }),
    );
    expect(working).toHaveAttribute('dir', 'auto');
  });

  it('names the level of the rule that was used, as its word', async () => {
    await open(PERCENT);
    expect(visible(document.body.textContent)).toContain(
      message(language, 'calc-info-rule', { level: message(language, 'rules-level-default') }),
    );
  });

  it('Ctrl+I on the button opens it too', async () => {
    renderIn(language, <CalcInfo working={PERCENT} />);
    const button = screen.getByRole('button', { name: label() });
    button.focus();
    fireEvent.keyDown(button, { code: 'KeyI', key: 'i', ctrlKey: true });
    expect(await screen.findByText(message(language, 'calc-info-title'))).toBeInTheDocument();
  });

  it('a working this build has no words for says so, and never shows the raw ID', async () => {
    await open({ message_id: 'working-from-the-future', args: [], rule_id: null });
    expect(await screen.findByText(message(language, 'calc-info-unknown'))).toBeInTheDocument();
    expect(document.body.textContent).not.toContain('working-from-the-future');
  });

  it('every kind of working has its words in both languages', () => {
    for (const id of [
      'working-percent',
      'working-per-unit',
      'working-per-lot',
      'working-manual',
      'working-none',
    ]) {
      expect(
        message(language, id, { base: '1', bps: '2', qty: '3', value: '4', result: '5' }),
      ).not.toBe('');
    }
  });
});
