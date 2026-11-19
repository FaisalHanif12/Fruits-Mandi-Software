// Plan 41, PRD FR-7.69, FR-7.63, FR-10.7, E75: the short form of "add it here" for a master list.
// One form for every kind, with only the fields a new record needs: both names (NamePair) for most
// kinds, plus the group and how it is sold for an item, the approximate weight for a packing, the
// city for an اڈا, the mark text for a مارکہ (its owner is the lot's seller), the Urdu and English
// text for a narration, the job and the phone for staff. The code is sent empty, so the backend
// gives the next one. Records that look like the name show under it and one can be taken instead.
// Both languages, against the fake of the masters and the search.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { QuickAddMaster } from '../../src/features/masters/components/QuickAddMaster';
import type { MasterKind, MasterRecord } from '../../src/features/masters/types';
import * as mastersApi from '../../src/features/masters/api';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import * as searchApi from '../../src/shared/search/api';
import type { SearchHit } from '../../src/shared/search';
import * as urduApi from '../../src/shared/urdu/api';
import { fakeMasters, record } from '../helpers/masters';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';
import { fail, press } from '../helpers/app';

vi.mock('../../src/features/masters/api');
vi.mock('../../src/shared/search/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;

interface HarnessProps {
  kind: MasterKind;
  typed: string;
  owner?: { id: number; code: string; name_ur: string; name_en: string } | null;
  onSaved?: (saved: MasterRecord) => void;
  onFullForm?: (text: string) => void;
  onPickExisting?: (hit: SearchHit) => void;
  onClosed?: () => void;
}

function Harness({
  kind,
  typed,
  owner,
  onSaved,
  onFullForm,
  onPickExisting,
  onClosed,
}: HarnessProps) {
  const [open, setOpen] = useState(true);
  useEscapeDispatcher();
  return (
    <QuickAddMaster
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) onClosed?.();
      }}
      kind={kind}
      typed={typed}
      owner={owner}
      onSaved={(saved) => {
        onSaved?.(saved);
      }}
      onFullForm={onFullForm}
      onPickExisting={onPickExisting}
    />
  );
}

function setup(language: Language, props: HarnessProps) {
  act(() => {
    useLanguageStore.setState({ language });
  });
  renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Harness {...props} />
    </QueryClientProvider>,
  );
  return userEvent.setup();
}

const field = (language: Language, id: string) =>
  screen.findByLabelText<HTMLInputElement>(named(message(language, id)));

/** The title of the dialog of a kind. */
const title = (language: Language, kind: MasterKind) =>
  named(message(language, `masters-${kind.replace('_', '-')}-new`));

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  fakeMasters({
    city: [record('city', 1, '0001', 'لاہور', 'Lahore')],
    item: [record('item', 5, '0005', 'کیلا', 'Banana')],
  });
});

describe.each(LANGUAGES)('the short form of a master list (%s)', (language) => {
  // The name typed, in the script of the screen.
  const typedName = language === 'ur' ? 'بجلی' : 'Electricity';
  const nameField = language === 'ur' ? 'masters-field-name-ur' : 'masters-field-name-en';

  it.each([
    ['city', ['masters-field-name-ur', 'masters-field-name-en']],
    ['store', ['masters-field-name-ur', 'masters-field-name-en']],
    ['expense_head', ['masters-field-name-ur', 'masters-field-name-en']],
    ['packing', ['masters-field-name-ur', 'masters-field-name-en', 'masters-packing-weight']],
    ['staff', ['masters-field-name-ur', 'masters-field-name-en', 'masters-staff-phone']],
  ] as const)(
    '%s asks only its own fields, with the typed name in the name box',
    async (kind, ids) => {
      setup(language, { kind, typed: typedName });
      const dialog = await screen.findByRole('dialog', { name: title(language, kind) });
      for (const id of ids) {
        expect(within(dialog).getByLabelText(named(message(language, id)))).toBeInTheDocument();
      }
      // No code box: the code is given by itself.
      expect(
        within(dialog).queryByLabelText(named(message(language, 'masters-field-code'))),
      ).toBeNull();
      expect((await field(language, nameField)).value).toBe(typedName);
    },
  );

  it('the cursor starts in the first box that is still empty, so the typed name is not in the way', async () => {
    setup(language, { kind: 'city', typed: typedName });
    const other = await field(
      language,
      language === 'ur' ? 'masters-field-name-en' : 'masters-field-name-ur',
    );
    await waitFor(() => {
      expect(other).toHaveFocus();
    });
  });

  it('F10 saves with an empty code, and the saved record is handed back', async () => {
    const onSaved = vi.fn();
    const onClosed = vi.fn();
    setup(language, { kind: 'expense_head', typed: typedName, onSaved, onClosed });
    await field(language, nameField);
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    const [kind, draft] = vi.mocked(mastersApi.mastersSave).mock.calls[0] ?? [];
    expect(kind).toBe('expense_head');
    expect(draft).toMatchObject({
      id: null,
      code: '',
      names: language === 'ur' ? { urdu: typedName } : { english: typedName },
    });
    await waitFor(() => {
      expect(onSaved).toHaveBeenCalledTimes(1);
    });
    // The fake gives the next code, as the backend does.
    expect(onSaved.mock.calls[0]?.[0]).toMatchObject({ kind: 'expense_head', code: '0001' });
    await waitFor(() => {
      expect(onClosed).toHaveBeenCalled();
    });
  });

  it('Esc closes it and saves nothing', async () => {
    const onClosed = vi.fn();
    const user = setup(language, { kind: 'city', typed: typedName, onClosed });
    await field(language, nameField);
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(onClosed).toHaveBeenCalled();
    });
    expect(mastersApi.mastersSave).not.toHaveBeenCalled();
  });

  it('an item asks the group and how it is sold, which starts at نگ', async () => {
    setup(language, { kind: 'item', typed: typedName });
    const dialog = await screen.findByRole('dialog', { name: title(language, 'item') });
    const choice = (id: string) =>
      visible(
        within(dialog).getByRole('combobox', { name: named(message(language, id)) }).textContent,
      );
    expect(choice('masters-item-group')).toBe(message(language, 'masters-item-group-fruit'));
    expect(choice('masters-item-sold-by')).toBe(message(language, 'masters-sold-by-unit'));
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1]).toMatchObject({
      code: '',
      text: { item_group: 'fruit', sold_by: 'unit' },
    });
  });

  it('a packing sends its approximate weight as typed', async () => {
    const user = setup(language, { kind: 'packing', typed: typedName });
    await user.type(await field(language, 'masters-packing-weight'), '20');
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1]).toMatchObject({
      text: { approx_kg: '20' },
    });
  });

  it('a staff member asks the job (a munshi at first) and the phone', async () => {
    const user = setup(language, { kind: 'staff', typed: typedName });
    await user.type(await field(language, 'masters-staff-phone'), '0300-1234567');
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1]).toMatchObject({
      text: { job: 'munshi', phone: '0300-1234567' },
    });
  });

  it('an اڈا asks its city, which is optional', async () => {
    setup(language, { kind: 'adda', typed: typedName });
    expect(
      await screen.findByRole('combobox', { name: named(message(language, 'masters-adda-city')) }),
    ).toBeInTheDocument();
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1]).toMatchObject({
      refs: { city: null },
    });
  });

  it('a مارکہ is its mark text; the lot’s seller is shown as its owner and sent with it', async () => {
    const owner = { id: 7, code: '030110001', name_ur: 'رحیم', name_en: 'Rahim' };
    setup(language, { kind: 'marka', typed: 'AR', owner });
    const mark = await field(language, 'masters-marka-text');
    expect(mark.value).toBe('AR');
    // No names for a mark: its text is not translated.
    expect(screen.queryByLabelText(named(message(language, 'masters-field-name-ur')))).toBeNull();
    expect((await field(language, 'masters-marka-owner')).value).toBe(
      language === 'ur' ? owner.name_ur : owner.name_en,
    );
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    const draft = vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1];
    expect(draft).toMatchObject({ text: { mark_text: 'AR' } });
    expect(draft?.refs['owner']?.id).toBe(7);
  });

  it('a narration is the Urdu and the English text, with no spelling check', async () => {
    setup(language, {
      kind: 'narration',
      typed: language === 'ur' ? 'بجلی کا بل' : 'Electricity bill',
    });
    expect(
      await screen.findByLabelText(named(message(language, 'masters-narration-text-ur'))),
    ).toBeInTheDocument();
    expect(
      screen.getByLabelText(named(message(language, 'masters-narration-text-en'))),
    ).toBeInTheDocument();
    press('F10');
    await waitFor(() => {
      expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1]).toMatchObject({
      lists: { used_in: [] },
    });
  });

  it('a name that looks like one already there shows it under the name, and it can be taken instead', async () => {
    const onPickExisting = vi.fn();
    const user = setup(language, {
      kind: 'item',
      typed: language === 'ur' ? 'کیلا' : 'Banan',
      onPickExisting,
    });
    const similar = await screen.findByRole('heading', {
      name: named(message(language, 'search-quick-similar-title')),
    });
    const panel = similar.closest('section');
    if (panel === null) throw new Error('no panel');
    expect(visible(panel.textContent)).toContain(language === 'ur' ? 'کیلا' : 'Banana');
    expect(visible(panel.textContent)).toContain('0005');
    await user.click(
      within(panel).getByRole('button', { name: named(message(language, 'search-quick-pick')) }),
    );
    expect(onPickExisting).toHaveBeenCalledWith(expect.objectContaining({ id: 5, code: '0005' }));
    // Taking it saves nothing.
    expect(mastersApi.mastersSave).not.toHaveBeenCalled();
  });

  it('no similar panel when nothing is like the name', async () => {
    vi.mocked(searchApi.searchMasters).mockResolvedValue([]);
    setup(language, { kind: 'item', typed: typedName });
    await field(language, nameField);
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 300));
    });
    expect(
      screen.queryByRole('heading', {
        name: named(message(language, 'search-quick-similar-title')),
      }),
    ).toBeNull();
  });

  it('the link "Full form" hands over the name typed so far', async () => {
    const onFullForm = vi.fn();
    const user = setup(language, { kind: 'city', typed: typedName, onFullForm });
    await user.click(
      await screen.findByRole('button', {
        name: named(message(language, 'search-quick-full-form')),
      }),
    );
    expect(onFullForm).toHaveBeenCalledWith(typedName);
    // Without the app’s full form there is no link.
  });

  it('there is no link "Full form" where the app has none', async () => {
    setup(language, { kind: 'city', typed: typedName });
    await field(language, nameField);
    expect(
      screen.queryByRole('button', { name: named(message(language, 'search-quick-full-form')) }),
    ).toBeNull();
  });

  it('a refusal of the backend shows at the field and nothing typed is lost', async () => {
    vi.mocked(mastersApi.mastersSave).mockRejectedValueOnce(
      fail({ code: 'ERR_NAME_REQUIRED', field: language === 'ur' ? 'name_ur' : 'name_en' }),
    );
    const onSaved = vi.fn();
    const user = setup(language, { kind: 'city', typed: typedName, onSaved });
    const name = await field(language, nameField);
    await user.click(name);
    await user.keyboard('{End} x');
    press('F10');
    await waitFor(() => {
      expect(name).toHaveAttribute('aria-invalid', 'true');
    });
    expect(onSaved).not.toHaveBeenCalled();
    expect(name.value).toBe(`${typedName} x`);
  });
});

// PRD FR-7.24, FR-7.66: the name is filled in the other script by itself, as it is when typed. A name
// that arrives pre-filled from the picker (what was typed there) must do the same, so the user only
// checks both names. A spelling made by the rules alone is marked "check the spelling".
describe.each(LANGUAGES)(
  'the other name is filled by itself in the short form (%s)',
  (language) => {
    beforeEach(() => {
      vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
        Promise.resolve(
          suggestionsFor(
            {
              toUrdu: { electricity: ['بجلی'], nadeem: ['ندیم'] },
              toEnglish: { بجلی: 'Bijli', ندیم: 'Nadeem' },
              rulesOnly: ['nadeem', 'ندیم'],
            },
            text,
            direction,
          ),
        ),
      );
    });

    const urduBox = () => field(language, 'masters-field-name-ur');
    const englishBox = () => field(language, 'masters-field-name-en');

    it('a name typed in English letters fills the Urdu name from the word list', async () => {
      setup(language, { kind: 'expense_head', typed: 'Electricity' });
      expect((await englishBox()).value).toBe('Electricity');
      await waitFor(async () => {
        expect((await urduBox()).value).toBe('بجلی');
      });
      // From the word list, so nothing is marked.
      expect(screen.queryByText(message(language, 'translit-spelling-check'))).toBeNull();
    });

    it('a name typed in Urdu letters fills the English name', async () => {
      setup(language, { kind: 'expense_head', typed: 'بجلی' });
      expect((await urduBox()).value).toBe('بجلی');
      await waitFor(async () => {
        expect((await englishBox()).value).toBe('Bijli');
      });
    });

    it('a spelling made by the rules alone is marked "check the spelling" (FR-7.66)', async () => {
      setup(language, { kind: 'city', typed: 'Nadeem' });
      await waitFor(async () => {
        expect((await urduBox()).value).toBe('ندیم');
      });
      expect(
        await screen.findByText(message(language, 'translit-spelling-check')),
      ).toBeInTheDocument();
    });

    it('an item fills too, and the save sends both names', async () => {
      setup(language, { kind: 'item', typed: 'Electricity' });
      await waitFor(async () => {
        expect((await urduBox()).value).toBe('بجلی');
      });
      press('F10');
      await waitFor(() => {
        expect(mastersApi.mastersSave).toHaveBeenCalledTimes(1);
      });
      expect(vi.mocked(mastersApi.mastersSave).mock.calls[0]?.[1]).toMatchObject({
        names: { urdu: 'بجلی', english: 'Electricity' },
      });
    });

    it('a narration is free text: it fills, and nothing is marked', async () => {
      setup(language, { kind: 'narration', typed: 'Nadeem' });
      await waitFor(async () => {
        expect((await field(language, 'masters-narration-text-ur')).value).toBe('ندیم');
      });
      expect(screen.queryByText(message(language, 'translit-spelling-check'))).toBeNull();
    });
  },
);
