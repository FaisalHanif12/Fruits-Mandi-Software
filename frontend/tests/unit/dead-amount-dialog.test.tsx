// "ڈوبی رقم میں ڈالیں" and "دوبارہ فعال کریں" (plan 40, PRD FR-10.10, E71, E73): the box shows what the
// person owes (the backend's preview), asks for the date, the reason and a note, moves him, and keeps
// what was typed when the backend refuses. A person who cannot be moved is told why, in words, from
// an error code. Everything is done by keyboard. In the Urdu and in the English app.

import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { act, screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as appApi from '../../src/app/api';
import * as mastersApi from '../../src/features/masters/api';
import { DeadMarkDialog } from '../../src/features/masters/components/DeadMarkDialog';
import { DeadUndoDialog } from '../../src/features/masters/components/DeadUndoDialog';
import { LANGUAGES, type Language } from '../../src/i18n';
import { useLanguageStore } from '../../src/shared/i18n';
import { useEscapeDispatcher } from '../../src/shared/keyboard';
import { useToastStore } from '../../src/shared/ui';
import { DAY, fail } from '../helpers/app';
import { ARSHAD_DEAD, ARSHAD_PREVIEW, LINKED_PREVIEW, NOTHING_OWED_PREVIEW } from '../helpers/dead';
import { MARKED_ARSHAD, MARK_ARSHAD_REQUEST } from '../helpers/dead-real';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/app/api');
vi.mock('../../src/features/masters/api');

const NAMES = { name_ur: 'ارشد ملتانی', name_en: 'Arshad Multani', tag: null };

function Host({ children }: { children: React.ReactNode }) {
  useEscapeDispatcher();
  return <>{children}</>;
}

function shown(node: React.ReactNode, language: Language) {
  return renderIn(
    language,
    <QueryClientProvider
      client={new QueryClient({ defaultOptions: { queries: { retry: false } } })}
    >
      <Host>{node}</Host>
    </QueryClientProvider>,
  );
}

beforeEach(() => {
  vi.resetAllMocks();
  vi.mocked(appApi.dayStatus).mockResolvedValue(DAY);
  vi.mocked(mastersApi.deadPreview).mockResolvedValue(ARSHAD_PREVIEW);
  vi.mocked(mastersApi.deadMark).mockResolvedValue(MARKED_ARSHAD);
  vi.mocked(mastersApi.deadUndo).mockResolvedValue({ ok: true });
  act(() => {
    useToastStore.setState({ toasts: [] });
  });
});

describe.each(LANGUAGES)('moving a person to the dead amount (%s)', (language: Language) => {
  const m = (id: string, args: Record<string, string | number> = {}) => message(language, id, args);
  const named = (text: string) => (name: string) => visible(name) === text;
  const nameOf = language === 'ur' ? NAMES.name_ur : NAMES.name_en;

  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  function open(personKey: string | null = null, onClose = vi.fn()) {
    shown(
      <DeadMarkDialog request={{ accountId: 22, personKey, names: NAMES }} onClose={onClose} />,
      language,
    );
    return onClose;
  }

  it('shows what he owes, with today in the date box, and moves him with what was typed', async () => {
    const onClose = open();
    const dialog = await screen.findByRole('dialog');
    expect(visible(dialog.textContent)).toContain(nameOf);
    await within(dialog).findByText('18,684');
    expect(mastersApi.deadPreview).toHaveBeenCalledWith({
      account_id: 22,
      pukhta_person_key: null,
    });
    const date = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(m('dead-move-date')),
    });
    // Today is shown, visible and editable (NFR-USE-27), and the box has the focus to type in.
    expect(date.value).toBe('10-10-2026');
    await waitFor(() => {
      expect(date).toHaveFocus();
    });
    const user = userEvent.setup();
    await user.clear(date);
    await user.type(date, '11-10-2026');
    await user.click(within(dialog).getByRole('button', { name: named(m('dead-move-confirm')) }));
    await waitFor(() => {
      expect(mastersApi.deadMark).toHaveBeenCalledWith({
        account_id: 22,
        pukhta_person_key: null,
        entry_date: '11-10-2026',
        reason: 'died',
        note: '',
      });
    });
    await waitFor(() => {
      expect(onClose).toHaveBeenCalledWith(true);
    });
    expect(
      useToastStore
        .getState()
        .toasts.map((toast) => visible(typeof toast.message === 'string' ? toast.message : '')),
    ).toEqual([m('dead-move-saved', { name: nameOf })]);
  });

  it('sends exactly the request the backend sent in its own test for ارشد', async () => {
    const onClose = vi.fn();
    shown(
      <DeadMarkDialog
        request={{ accountId: MARK_ARSHAD_REQUEST.account_id, personKey: null, names: NAMES }}
        onClose={onClose}
      />,
      language,
    );
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText('18,684');
    const user = userEvent.setup();
    const date = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(m('dead-move-date')),
    });
    await user.clear(date);
    await user.type(date, MARK_ARSHAD_REQUEST.entry_date);
    await user.click(within(dialog).getByRole('combobox', { name: named(m('dead-move-note')) }));
    await user.keyboard(MARK_ARSHAD_REQUEST.note);
    await user.click(within(dialog).getByRole('button', { name: named(m('dead-move-confirm')) }));
    await waitFor(() => {
      expect(mastersApi.deadMark).toHaveBeenCalledWith(MARK_ARSHAD_REQUEST);
    });
  });

  it('is done with the keyboard alone: Enter in the last box, the note, moves him', async () => {
    const onClose = open();
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText('18,684');
    within(dialog)
      .getByRole('combobox', { name: named(m('dead-move-note')) })
      .focus();
    const user = userEvent.setup();
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(onClose).toHaveBeenCalledWith(true);
    });
    expect(mastersApi.deadMark).toHaveBeenCalledTimes(1);
  });

  it('sends the person key of a one-time person to both commands', async () => {
    const onClose = open('shahid|03000000000');
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText('18,684');
    expect(mastersApi.deadPreview).toHaveBeenCalledWith({
      account_id: 22,
      pukhta_person_key: 'shahid|03000000000',
    });
    await userEvent
      .setup()
      .click(within(dialog).getByRole('button', { name: named(m('dead-move-confirm')) }));
    await waitFor(() => {
      expect(onClose).toHaveBeenCalledWith(true);
    });
    expect(mastersApi.deadMark).toHaveBeenCalledWith(
      expect.objectContaining({ pukhta_person_key: 'shahid|03000000000' }),
    );
  });

  it('a refusal stays in the box, under its field, and what was typed is kept', async () => {
    vi.mocked(mastersApi.deadMark).mockRejectedValue(
      fail({ code: 'ERR_NOTE_REQUIRED', field: 'note' }),
    );
    const onClose = open();
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText('18,684');
    const user = userEvent.setup();
    const date = within(dialog).getByRole<HTMLInputElement>('textbox', {
      name: named(m('dead-move-date')),
    });
    await user.clear(date);
    await user.type(date, '11-10-2026');
    await user.click(within(dialog).getByRole('combobox', { name: named(m('dead-move-reason')) }));
    await user.click(await screen.findByRole('option', { name: named(m('dead-reason-other')) }));
    // The hint says a note is needed for the reason Other.
    expect(visible(dialog.textContent)).toContain(m('dead-move-note-hint'));
    await user.click(within(dialog).getByRole('button', { name: named(m('dead-move-confirm')) }));
    await waitFor(() => {
      expect(visible(dialog.textContent)).toContain(message(language, 'err-note-required'));
    });
    expect(onClose).not.toHaveBeenCalled();
    expect(date.value).toBe('11-10-2026');
    expect(mastersApi.deadMark).toHaveBeenCalledWith(
      expect.objectContaining({ reason: 'other', note: '' }),
    );
  });

  it('a refused date is told under the date box in words, never as a code', async () => {
    vi.mocked(mastersApi.deadMark).mockRejectedValue(
      fail({
        code: 'ERR_DATE_BEFORE_LAST_POSTING',
        field: 'entry_date',
        values: { last_posting_date: '10-10-2026' },
      }),
    );
    open();
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText('18,684');
    await userEvent
      .setup()
      .click(within(dialog).getByRole('button', { name: named(m('dead-move-confirm')) }));
    await waitFor(() => {
      expect(visible(dialog.textContent)).toContain(
        message(language, 'err-date-before-last-posting', { last_posting_date: '10-10-2026' }),
      );
    });
    expect(visible(dialog.textContent)).not.toContain('ERR_');
  });

  it('says his last entry day, so the date is not guessed', async () => {
    open();
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText('18,684');
    expect(visible(dialog.textContent)).toContain(
      m('dead-move-last-entry', { date: '10-10-2026' }),
    );
  });

  it('a person who cannot be moved is told why by the code, and the button stays off', async () => {
    vi.mocked(mastersApi.deadPreview).mockResolvedValue(NOTHING_OWED_PREVIEW);
    open();
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText((content) => visible(content) === m('dead-move-cannot'));
    expect(visible(dialog.textContent)).toContain(message(language, 'err-nothing-owed'));
    expect(
      within(dialog).getByRole('button', { name: named(m('dead-move-confirm')) }),
    ).toBeDisabled();
    expect(mastersApi.deadMark).not.toHaveBeenCalled();
  });

  it('names the linked account and its balance, and says it is not moved', async () => {
    vi.mocked(mastersApi.deadPreview).mockResolvedValue(LINKED_PREVIEW);
    open();
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText('2,000', { exact: false });
    const text = visible(dialog.textContent);
    expect(text).toContain(language === 'ur' ? 'ارشد بیوپاری' : 'Arshad Seller');
    expect(text).toContain('030110001');
    expect(text).toContain(language === 'ur' ? 'نہیں ڈالا جاتا' : 'not moved');
  });

  it('while his account is read there is a loading state, and a failure has a retry', async () => {
    vi.mocked(mastersApi.deadPreview).mockRejectedValueOnce(fail({ code: 'ERR_NOT_FOUND' }));
    open();
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText(m('dead-move-failed'));
    expect(
      within(dialog).getByRole('button', { name: named(m('dead-move-confirm')) }),
    ).toBeDisabled();
    await userEvent
      .setup()
      .click(
        within(dialog).getByRole('button', { name: named(message(language, 'common-retry')) }),
      );
    await within(dialog).findByText('18,684');
    expect(mastersApi.deadPreview).toHaveBeenCalledTimes(2);
  });

  it('Esc leaves without moving anyone', async () => {
    const onClose = open();
    const dialog = await screen.findByRole('dialog');
    await within(dialog).findByText('18,684');
    await userEvent.setup().keyboard('{Escape}');
    await waitFor(() => {
      expect(onClose).toHaveBeenCalledWith(false);
    });
    expect(mastersApi.deadMark).not.toHaveBeenCalled();
  });
});

describe.each(LANGUAGES)('making a person active again (%s)', (language: Language) => {
  const m = (id: string, args: Record<string, string | number> = {}) => message(language, id, args);
  const named = (text: string) => (name: string) => visible(name) === text;
  const nameOf = language === 'ur' ? NAMES.name_ur : NAMES.name_en;

  beforeEach(() => {
    act(() => {
      useLanguageStore.setState({ language });
    });
  });

  it('needs a reason, sends it with the move, and says he is active again', async () => {
    const onClose = vi.fn();
    shown(
      <DeadUndoDialog request={{ markId: ARSHAD_DEAD.mark_id, names: NAMES }} onClose={onClose} />,
      language,
    );
    const dialog = await screen.findByRole('alertdialog');
    const user = userEvent.setup();
    // An empty reason sends nothing.
    await user.click(within(dialog).getByRole('button', { name: named(m('dead-undo-confirm')) }));
    expect(mastersApi.deadUndo).not.toHaveBeenCalled();
    await user.type(within(dialog).getByRole('combobox'), 'The family paid.');
    await user.click(within(dialog).getByRole('button', { name: named(m('dead-undo-confirm')) }));
    await waitFor(() => {
      expect(mastersApi.deadUndo).toHaveBeenCalledWith({
        mark_id: ARSHAD_DEAD.mark_id,
        reason: 'The family paid.',
      });
    });
    await waitFor(() => {
      expect(onClose).toHaveBeenCalledWith(true);
    });
    expect(
      useToastStore
        .getState()
        .toasts.map((toast) => visible(typeof toast.message === 'string' ? toast.message : '')),
    ).toEqual([m('dead-undo-saved', { name: nameOf })]);
  });

  it('a refusal stays in the box with the reason that was typed', async () => {
    vi.mocked(mastersApi.deadUndo).mockRejectedValue(
      fail({ code: 'ERR_NOT_FOUND', field: 'mark_id' }),
    );
    const onClose = vi.fn();
    shown(
      <DeadUndoDialog request={{ markId: ARSHAD_DEAD.mark_id, names: NAMES }} onClose={onClose} />,
      language,
    );
    const dialog = await screen.findByRole('alertdialog');
    const user = userEvent.setup();
    const box = within(dialog).getByRole<HTMLInputElement>('combobox');
    await user.type(box, 'Mistake');
    await user.keyboard('{Enter}');
    await waitFor(() => {
      expect(visible(dialog.textContent)).toContain(message(language, 'err-not-found'));
    });
    expect(box.value).toBe('Mistake');
    expect(onClose).not.toHaveBeenCalled();
  });
});
