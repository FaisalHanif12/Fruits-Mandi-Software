// The pieces that trips share (plan 16b, PRD FR-12.29, FR-12.30, C23, C25), in the Urdu and the
// English app: the shares and sources of the partners, the source of one entry, the parts of a
// payment before it is saved, and the parts of money that comes in. Each part is the backend's
// number with its working; the pieces add nothing up (CLAUDE.md rule 1).

import { screen, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { describe, expect, it, vi } from 'vitest';
import type { MoneySource } from '../../src/bindings/MoneySource';
import type { EntryParts } from '../../src/bindings/EntryParts';
import { LANGUAGES } from '../../src/i18n';
import {
  EntryPartsCard,
  EntrySources,
  PartnerSharesEditor,
  ShareAmounts,
  type VentureShareRow,
} from '../../src/shared/ui';
import { money } from '../helpers/accounts';
import { message, renderIn, visible } from '../helpers/render';
import {
  PARTNER_A,
  PARTNER_B,
  PARTNER_C,
  e34Credits,
  e34Parts,
  tripPartners,
} from '../helpers/trips';
import { refusal } from '../helpers/ui-error';
import { useLanguageStore } from '../../src/shared/i18n';
import { act } from '@testing-library/react';

const named = (text: string) => (name: string) => visible(name) === text;

describe.each(LANGUAGES)('shared venture pieces (%s)', (language) => {
  const name = (en: string, ur: string) => (language === 'ur' ? ur : en);

  function setLanguage() {
    act(() => {
      useLanguageStore.setState({ language });
    });
  }

  function Editor({ onChange }: { onChange: (rows: VentureShareRow[]) => void }) {
    const [rows, setRows] = useState<VentureShareRow[]>([
      { partner: PARTNER_A, share: '40', source: 'shop' },
      { partner: PARTNER_B, share: '', source: 'shop' },
      { partner: PARTNER_C, share: '60', source: 'shop' },
    ]);
    const update = (next: VentureShareRow[]) => {
      setRows(next);
      onChange(next);
    };
    return (
      <PartnerSharesEditor
        rows={rows}
        error={refusal([{ code: 'ERR_NOT_A_NUMBER', field: 'partners[2].share' }])}
        onShareChange={(index, share) => {
          update(rows.map((row, place) => (place === index ? { ...row, share } : row)));
        }}
        onSourceChange={(index, source) => {
          update(rows.map((row, place) => (place === index ? { ...row, source } : row)));
        }}
      />
    );
  }

  it('the shares editor has a share and a source for each partner; no share means not included', async () => {
    setLanguage();
    const onChange = vi.fn();
    renderIn(language, <Editor onChange={onChange} />);
    const user = userEvent.setup();
    const list = screen.getByRole('list', {
      name: named(message(language, 'ventures-shares-label')),
    });
    expect(within(list).getAllByRole('listitem')).toHaveLength(3);
    // Partner B has no share: told in words, and there is no source choice for him.
    expect(within(list).getAllByText(named(message(language, 'ventures-not-in')))).toHaveLength(1);
    expect(
      screen.queryByRole('group', {
        name: named(message(language, 'ventures-source-of', { name: name('Partner B', 'ب') })),
      }),
    ).toBeNull();
    // A problem of the backend shows under the share it points at.
    expect(
      screen.getByRole('textbox', {
        name: named(message(language, 'ventures-share-of', { name: name('Partner C', 'ج') })),
      }),
    ).toHaveAttribute('aria-invalid', 'true');
    await user.click(
      within(
        screen.getByRole('group', {
          name: named(message(language, 'ventures-source-of', { name: name('Partner C', 'ج') })),
        }),
      ).getByRole('button', { name: message(language, 'ventures-source-pocket') }),
    );
    expect(onChange).toHaveBeenLastCalledWith([
      expect.objectContaining({ share: '40', source: 'shop' }),
      expect.objectContaining({ share: '', source: 'shop' }),
      expect.objectContaining({ share: '60', source: 'pocket' }),
    ]);
    // A share typed for B brings his source choice in.
    await user.type(
      screen.getByRole('textbox', {
        name: named(message(language, 'ventures-share-of', { name: name('Partner B', 'ب') })),
      }),
      '10',
    );
    expect(
      screen.getByRole('group', {
        name: named(message(language, 'ventures-source-of', { name: name('Partner B', 'ب') })),
      }),
    ).toBeInTheDocument();
  });

  it('the source of one entry starts from the trip and a change is reported by partner', async () => {
    setLanguage();
    const onChange = vi.fn();
    function Rig() {
      const [value, setValue] = useState<Record<number, MoneySource>>({});
      return (
        <EntrySources
          partners={tripPartners(['shop', 'pocket', 'shop'])}
          value={value}
          onChange={(id, source) => {
            onChange(id, source);
            setValue({ ...value, [id]: source });
          }}
        />
      );
    }
    renderIn(language, <Rig />);
    const user = userEvent.setup();
    const pressed = (partner: string, source: 'shop' | 'pocket') =>
      within(
        screen.getByRole('group', {
          name: named(message(language, 'ventures-source-of', { name: partner })),
        }),
      ).getByRole('button', { name: message(language, `ventures-source-${source}`) });
    expect(pressed(name('Partner B', 'ب'), 'pocket')).toHaveAttribute('aria-pressed', 'true');
    await user.click(pressed(name('Partner B', 'ب'), 'shop'));
    expect(onChange).toHaveBeenCalledWith(2, 'shop');
    expect(pressed(name('Partner B', 'ب'), 'shop')).toHaveAttribute('aria-pressed', 'true');
  });

  it('the parts card shows each part with its source in words, and the backend’s two sums', () => {
    setLanguage();
    const parts: EntryParts = {
      parts: e34Parts(['shop', 'pocket', 'shop']),
      shop_total_paisa: money(7_280),
      pocket_total_paisa: money(3_120),
    };
    renderIn(
      language,
      <EntryPartsCard parts={parts} title={message(language, 'trips-parts-title')} />,
    );
    const list = screen.getByRole('list', { name: named(message(language, 'trips-parts-title')) });
    expect(within(list).getAllByRole('listitem')).toHaveLength(3);
    expect(within(list).getAllByText(named(message(language, 'ventures-part-shop')))).toHaveLength(
      2,
    );
    expect(
      within(list).getAllByText(named(message(language, 'ventures-part-pocket'))),
    ).toHaveLength(1);
    const text = visible(document.body.textContent);
    for (const figure of ['4,160', '3,120', '7,280']) expect(text).toContain(figure);
  });

  it('with nothing worked out the card says what to do; with only pocket parts it says nothing posts', () => {
    setLanguage();
    const { unmount } = renderIn(
      language,
      <EntryPartsCard parts={null} title={message(language, 'trips-parts-title')} />,
    );
    expect(screen.getByText(named(message(language, 'ventures-parts-empty')))).toBeInTheDocument();
    unmount();
    renderIn(
      language,
      <EntryPartsCard
        parts={{
          parts: e34Parts(['pocket', 'pocket', 'pocket']),
          shop_total_paisa: money(0),
          pocket_total_paisa: money(10_400),
        }}
        title={message(language, 'trips-parts-title')}
      />,
    );
    expect(
      screen.getByText(named(message(language, 'ventures-parts-no-posting'))),
    ).toBeInTheDocument();
  });

  it('the credits list shows each partner’s part of the money that comes in', () => {
    setLanguage();
    renderIn(
      language,
      <ShareAmounts credits={e34Credits()} label={message(language, 'trips-sale-credits-title')} />,
    );
    const text = visible(document.body.textContent);
    for (const figure of ['3,384', '2,538']) expect(text).toContain(figure);
  });
});
