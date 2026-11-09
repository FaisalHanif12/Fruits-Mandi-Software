// Plan 64, PRD FR-7.69, FR-13.2, FR-13.3: the short form of a new چکتا buyer, the second add row of
// the buyer box of a بکری line. It asks his name in both scripts (one typed, the other filled,
// FR-7.24) and his phone, which is optional. He is saved at once through `chukta_add` and handed
// back to the field. A refusal is shown at its fields, both errors together. F10 saves and Esc
// closes with nothing saved. In the Urdu and in the English app, against the backend's real
// requests and answers.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { useState } from 'react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { LANGUAGES, type Language } from '../../src/i18n';
import { CommandError } from '../../src/shared/api';
import * as accountApi from '../../src/shared/accounts/api';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import { QuickAddChukta } from '../../src/shared/ui';
import * as urduApi from '../../src/shared/urdu/api';
import { press } from '../helpers/app';
import {
  CHUKTA_ADD_NASIR,
  CHUKTA_ADD_NASIR_REQUEST,
  CHUKTA_ADD_REFUSED,
  CHUKTA_ADD_REFUSED_REQUEST,
} from '../helpers/bikri-real';
import { EMPTY_PAIRS } from '../helpers/paged';
import { message, renderIn, visible } from '../helpers/render';
import { suggestionsFor } from '../helpers/translit';

vi.mock('../../src/shared/accounts/api');
vi.mock('../../src/shared/urdu/api');

const named = (text: string) => (name: string) => visible(name) === text;
/** An element whose own text is all of `text`: Fluent isolation marks in the page are not compared. */
const shown = (text: string) => (_content: string, element: Element | null) =>
  element !== null && element.children.length === 0 && visible(element.textContent) === text;

interface HarnessProps {
  typed: string;
  onSaved?: (buyer: unknown) => void;
  onClosed?: () => void;
}

function Harness({ typed, onSaved, onClosed }: HarnessProps) {
  const [open, setOpen] = useState(true);
  useEscapeDispatcher();
  return (
    <QuickAddChukta
      open={open}
      onOpenChange={(next) => {
        setOpen(next);
        if (!next) onClosed?.();
      }}
      typed={typed}
      onSaved={(buyer) => {
        onSaved?.(buyer);
      }}
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

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(urduApi.translitPairsList).mockResolvedValue(EMPTY_PAIRS);
  vi.mocked(urduApi.translitLearn).mockResolvedValue(null);
  // Both directions of the one name used here, from the word list (no spelling check).
  vi.mocked(urduApi.translitSuggest).mockImplementation(({ text, direction }) =>
    Promise.resolve(
      suggestionsFor(
        { toUrdu: { nasir: ['ناصر'] }, toEnglish: { ناصر: 'Nasir' } },
        text,
        direction,
      ),
    ),
  );
  vi.mocked(accountApi.chuktaAdd).mockResolvedValue(CHUKTA_ADD_NASIR);
});

describe.each(LANGUAGES)('the short form of a new چکتا buyer (%s)', (language) => {
  const typedName = language === 'ur' ? 'ناصر' : 'Nasir';

  it('asks both names and the phone, with the typed name already in its box', async () => {
    setup(language, { typed: typedName });
    const dialog = await screen.findByRole('dialog', {
      name: named(message(language, 'chukta-add-title')),
    });
    expect(dialog).toBeInTheDocument();
    expect(await field(language, 'accounts-field-name-ur')).toBeInTheDocument();
    expect(await field(language, 'accounts-field-name-en')).toBeInTheDocument();
    expect(await field(language, 'accounts-field-phone1')).toBeInTheDocument();
    const own = await field(
      language,
      language === 'ur' ? 'accounts-field-name-ur' : 'accounts-field-name-en',
    );
    expect(own.value).toBe(typedName);
  });

  it('fills the other script by itself, so only one name is typed (FR-7.24)', async () => {
    setup(language, { typed: typedName });
    const other = await field(
      language,
      language === 'ur' ? 'accounts-field-name-en' : 'accounts-field-name-ur',
    );
    await waitFor(() => {
      expect(other.value).toBe(language === 'ur' ? 'Nasir' : 'ناصر');
    });
  });

  it('digits typed in the buyer box start the phone, not the name', async () => {
    setup(language, { typed: '0300-1111111' });
    expect((await field(language, 'accounts-field-phone1')).value).toBe('0300-1111111');
    expect((await field(language, 'accounts-field-name-ur')).value).toBe('');
    expect((await field(language, 'accounts-field-name-en')).value).toBe('');
  });

  it('F10 saves at once with both names and the phone as typed, and hands the buyer back', async () => {
    const onSaved = vi.fn();
    const onClosed = vi.fn();
    const user = setup(language, { typed: typedName, onSaved, onClosed });
    const other = await field(
      language,
      language === 'ur' ? 'accounts-field-name-en' : 'accounts-field-name-ur',
    );
    await waitFor(() => {
      expect(other.value).not.toBe('');
    });
    await user.type(await field(language, 'accounts-field-phone1'), '0300-1111111');
    press('F10');
    await waitFor(() => {
      expect(accountApi.chuktaAdd).toHaveBeenCalledTimes(1);
    });
    // The very request the backend's own test wrote.
    expect(vi.mocked(accountApi.chuktaAdd).mock.calls[0]?.[0]).toEqual(CHUKTA_ADD_NASIR_REQUEST);
    await waitFor(() => {
      expect(onSaved).toHaveBeenCalledWith(CHUKTA_ADD_NASIR);
    });
    await waitFor(() => {
      expect(onClosed).toHaveBeenCalled();
    });
  });

  it('sends no phone when he gave none', async () => {
    setup(language, { typed: typedName });
    const other = await field(
      language,
      language === 'ur' ? 'accounts-field-name-en' : 'accounts-field-name-ur',
    );
    await waitFor(() => {
      expect(other.value).not.toBe('');
    });
    press('F10');
    await waitFor(() => {
      expect(accountApi.chuktaAdd).toHaveBeenCalledTimes(1);
    });
    expect(vi.mocked(accountApi.chuktaAdd).mock.calls[0]?.[0]).toEqual({
      name_ur: 'ناصر',
      name_en: 'Nasir',
      phone: null,
    });
  });

  it("shows the backend's two refusals at their fields, keeps the form and saves nothing", async () => {
    vi.mocked(accountApi.chuktaAdd).mockRejectedValue(new CommandError(CHUKTA_ADD_REFUSED));
    const onSaved = vi.fn();
    const onClosed = vi.fn();
    const user = setup(language, { typed: 'Nasir', onSaved, onClosed });
    // What the real request of the refusal holds: an English name, no Urdu name, a short phone.
    expect(CHUKTA_ADD_REFUSED_REQUEST).toEqual({ name_en: 'Nasir', name_ur: '', phone: '12' });
    // The word list knows no spelling of this name here, so the Urdu name stays empty.
    vi.mocked(urduApi.translitSuggest).mockResolvedValue({ items: [] });
    await user.type(await field(language, 'accounts-field-phone1'), '12');
    press('F10');
    expect(
      await screen.findByText(shown(message(language, 'err-chukta-name-required'))),
    ).toBeTruthy();
    expect(screen.getByText(shown(message(language, 'err-phone-invalid')))).toBeTruthy();
    expect(onSaved).not.toHaveBeenCalled();
    expect(onClosed).not.toHaveBeenCalled();
    expect(screen.getByRole('dialog')).toBeInTheDocument();
  });

  it('Esc closes it and saves nothing', async () => {
    const onClosed = vi.fn();
    const user = setup(language, { typed: typedName, onClosed });
    await field(language, 'accounts-field-phone1');
    await user.keyboard('{Escape}');
    await waitFor(() => {
      expect(onClosed).toHaveBeenCalled();
    });
    expect(accountApi.chuktaAdd).not.toHaveBeenCalled();
  });
});
