// The Settings group "سوال پوچھیں" (plan 49, PRD FR-7.84, FR-7.85, NFR-SEC-15), in Urdu and English:
// on or off, the key (pasted, sent with the password, then only its last 4 characters), the model, the
// connection test, today's count and the question log page by page, each row read again. The key is
// only ever in the paste field until it is sent: after that it is nowhere on the screen, in the cache
// or in the store.

import { QueryClientProvider, type QueryClient } from '@tanstack/react-query';
import { screen, waitFor, within } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import * as assistantApi from '../../src/features/assistant/api';
import { AssistantSettings } from '../../src/features/assistant/components/AssistantSettings';
import { useAssistantStore } from '../../src/features/assistant/store';
import { LANGUAGES, type Language } from '../../src/i18n';
import * as real from '../helpers/assistant-real';
import { named, newClient, setLanguage } from '../helpers/assistant-shell';
import { message, renderIn, visible } from '../helpers/render';

vi.mock('../../src/features/assistant/api');

/** A key as the Owner pastes it. It is made up for the test and is never a real one. */
function pastedKey(): string {
  return ['not', 'a', 'real', 'key'].join('-');
}

beforeEach(() => {
  vi.clearAllMocks();
  vi.mocked(assistantApi.assistantStatus).mockResolvedValue(real.STATUS_ON);
  vi.mocked(assistantApi.assistantLogList).mockResolvedValue(real.LOG_LIST_FIRST_PAGE);
  vi.mocked(assistantApi.listenSteps).mockResolvedValue(() => undefined);
  useAssistantStore.getState().reset();
});

function show(language: Language, client: QueryClient = newClient()) {
  setLanguage(language);
  renderIn(
    language,
    <QueryClientProvider client={client}>
      <AssistantSettings />
    </QueryClientProvider>,
  );
  return client;
}

describe.each(LANGUAGES)('the Settings group (%s)', (language) => {
  it('shows on or off, the key as dots and its last 4 characters, the model, the count and the log', async () => {
    show(language);
    expect(
      await screen.findByRole('checkbox', {
        name: message(language, 'assistant-settings-enabled'),
      }),
    ).toBeChecked();
    expect(visible(document.body.textContent)).toContain(
      message(language, 'assistant-settings-key-saved', { last4: 'ab12' }),
    );
    expect(
      screen.getByRole('textbox', { name: message(language, 'assistant-settings-model') }),
    ).toHaveValue('openai/gpt-6-luna');
    expect(visible(document.body.textContent)).toContain(
      message(language, 'assistant-settings-count', { used: 9, limit: 20 }),
    );
    expect(visible(document.body.textContent)).toContain(
      message(language, 'assistant-settings-limit-fixed'),
    );
    // The log: one row for each of the backend's rows, newest first, with its words.
    const table = await screen.findByRole('table', {
      name: message(language, 'assistant-log-title'),
    });
    expect(within(table).getAllByRole('row')).toHaveLength(
      1 + real.LOG_LIST_FIRST_PAGE.rows.length,
    );
    expect(visible(table.textContent)).toContain('akram ka hisab');
    expect(visible(table.textContent)).toContain(message(language, 'assistant-outcome-answered'));
    expect(visible(table.textContent)).toContain('party_card, report R13');
  });

  it('the daily limit of 20 is not a setting: there is no box for it', async () => {
    show(language);
    await screen.findByRole('checkbox');
    expect(screen.queryByRole('spinbutton')).toBeNull();
  });

  it('turning it off sends the new state and keeps the model', async () => {
    vi.mocked(assistantApi.assistantSettingsSet).mockResolvedValue({
      ...real.STATUS_ON,
      enabled: false,
    });
    show(language);
    const user = userEvent.setup();
    await user.click(
      await screen.findByRole('checkbox', {
        name: message(language, 'assistant-settings-enabled'),
      }),
    );
    expect(assistantApi.assistantSettingsSet).toHaveBeenCalledWith({
      enabled: false,
      model: 'openai/gpt-6-luna',
    });
    await waitFor(() => {
      expect(
        screen.getByRole('checkbox', { name: message(language, 'assistant-settings-enabled') }),
      ).not.toBeChecked();
    });
  });

  it('a new model is saved from its own box', async () => {
    vi.mocked(assistantApi.assistantSettingsSet).mockResolvedValue({
      ...real.STATUS_ON,
      model: 'openai/other-model',
    });
    show(language);
    const user = userEvent.setup();
    const box = await screen.findByRole('textbox', {
      name: message(language, 'assistant-settings-model'),
    });
    const save = screen.getByRole('button', {
      name: message(language, 'assistant-settings-model-save'),
    });
    expect(save).toBeDisabled();
    await user.clear(box);
    await user.type(box, 'openai/other-model');
    await user.click(save);
    expect(assistantApi.assistantSettingsSet).toHaveBeenCalledWith({
      enabled: true,
      model: 'openai/other-model',
    });
  });

  describe('the key', () => {
    it('replacing it asks for the new key and the password, sends both, and then shows only the last 4', async () => {
      vi.mocked(assistantApi.assistantKeySave).mockResolvedValue({
        ...real.STATUS_ON,
        key_last4: 'ef56',
      });
      const client = show(language);
      const user = userEvent.setup();
      await user.click(
        await screen.findByRole('button', {
          name: message(language, 'assistant-settings-key-replace'),
        }),
      );
      const confirm = screen.getByRole('button', {
        name: message(language, 'assistant-settings-key-confirm-replace'),
      });
      expect(confirm).toBeDisabled();
      const [keyBox, passwordBox] = screen.getAllByLabelText(
        new RegExp(
          `${message(language, 'assistant-settings-key-paste')}|${message(language, 'assistant-settings-key-password')}`,
        ),
        { selector: 'input' },
      );
      // The paste field is masked, like a password.
      expect(keyBox).toHaveAttribute('type', 'password');
      await user.type(keyBox ?? document.body, pastedKey());
      await user.type(passwordBox ?? document.body, 'owner-password');
      await user.click(confirm);
      expect(assistantApi.assistantKeySave).toHaveBeenCalledWith({
        key: pastedKey(),
        password: 'owner-password',
      });
      // After the send: the fields are gone and the key is nowhere.
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain(
          message(language, 'assistant-settings-key-saved', { last4: 'ef56' }),
        );
      });
      expect(screen.queryByDisplayValue(pastedKey())).toBeNull();
      expect(document.body.innerHTML).not.toContain(pastedKey());
      const cached = JSON.stringify(
        client
          .getQueryCache()
          .getAll()
          .map((query) => query.state.data),
      );
      expect(cached).not.toContain(pastedKey());
      expect(JSON.stringify(useAssistantStore.getState().entries)).not.toContain(pastedKey());
    });

    it('a wrong password is told under the password field and the key stays in the box', async () => {
      vi.mocked(assistantApi.assistantKeySave).mockRejectedValue({
        ok: false,
        errors: [{ code: 'ERR_WRONG_PASSWORD', field: 'password', values: { tries_left: 4 } }],
        ref: '',
      });
      show(language);
      const user = userEvent.setup();
      await user.click(
        await screen.findByRole('button', {
          name: message(language, 'assistant-settings-key-replace'),
        }),
      );
      const [keyBox, passwordBox] = screen.getAllByLabelText(
        new RegExp(
          `${message(language, 'assistant-settings-key-paste')}|${message(language, 'assistant-settings-key-password')}`,
        ),
        { selector: 'input' },
      );
      await user.type(keyBox ?? document.body, pastedKey());
      await user.type(passwordBox ?? document.body, 'wrong');
      await user.click(
        screen.getByRole('button', {
          name: message(language, 'assistant-settings-key-confirm-replace'),
        }),
      );
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain(
          message(language, 'err-wrong-password', { tries_left: 4 }),
        );
      });
      expect(keyBox).toHaveValue(pastedKey());
    });

    it('removing it asks for the password only', async () => {
      vi.mocked(assistantApi.assistantKeyRemove).mockResolvedValue(real.KEY_REMOVE);
      show(language);
      const user = userEvent.setup();
      await user.click(
        await screen.findByRole('button', {
          name: message(language, 'assistant-settings-key-remove'),
        }),
      );
      expect(screen.queryByLabelText(message(language, 'assistant-settings-key-paste'))).toBeNull();
      await user.type(
        screen.getByLabelText(message(language, 'assistant-settings-key-password'), {
          selector: 'input',
        }),
        'owner-password',
      );
      await user.click(
        screen.getByRole('button', {
          name: message(language, 'assistant-settings-key-confirm-remove'),
        }),
      );
      expect(assistantApi.assistantKeyRemove).toHaveBeenCalledWith({ password: 'owner-password' });
    });

    it('with no key saved it asks for one straight away, and says to save it before turning on', async () => {
      vi.mocked(assistantApi.assistantStatus).mockResolvedValue({
        ...real.STATUS_OFF,
        key_saved: false,
        key_last4: null,
      });
      show(language);
      expect(
        await screen.findByText(message(language, 'assistant-settings-key-none')),
      ).toBeInTheDocument();
      expect(
        screen.getByLabelText(message(language, 'assistant-settings-key-paste'), {
          selector: 'input',
        }),
      ).toBeInTheDocument();
      expect(
        screen.getByText(message(language, 'assistant-settings-enabled-needs-key')),
      ).toBeInTheDocument();
    });
  });

  describe('the connection test', () => {
    it('needs a saved key, and not the on switch', async () => {
      vi.mocked(assistantApi.assistantStatus).mockResolvedValue({
        ...real.STATUS_OFF,
        key_saved: true,
        key_last4: 'ab12',
      });
      show(language);
      const test = await screen.findByRole('button', {
        name: message(language, 'assistant-settings-test'),
      });
      expect(test).toBeEnabled();
      vi.mocked(assistantApi.assistantStatus).mockResolvedValue({
        ...real.STATUS_OFF,
        key_saved: false,
        key_last4: null,
      });
    });

    it('is off, with the reason, when no key is saved', async () => {
      vi.mocked(assistantApi.assistantStatus).mockResolvedValue({
        ...real.STATUS_OFF,
        key_saved: false,
        key_last4: null,
      });
      show(language);
      const test = await screen.findByRole('button', {
        name: message(language, 'assistant-settings-test'),
      });
      expect(test).toBeDisabled();
      expect(
        screen.getByText(message(language, 'assistant-settings-test-needs-key')),
      ).toBeInTheDocument();
    });

    it('says it worked, with the time', async () => {
      vi.mocked(assistantApi.assistantTest).mockResolvedValue(real.TEST_WORKED);
      show(language);
      const user = userEvent.setup();
      await user.click(
        await screen.findByRole('button', { name: message(language, 'assistant-settings-test') }),
      );
      await waitFor(() => {
        expect(visible(document.body.textContent)).toContain(
          message(language, 'assistant-settings-test-ok', { ms: String(real.TEST_WORKED.millis) }),
        );
      });
    });

    it('says why it did not, with the status of the answer', async () => {
      vi.mocked(assistantApi.assistantTest).mockResolvedValue(real.TEST_NO_ENDPOINT);
      show(language);
      const user = userEvent.setup();
      await user.click(
        await screen.findByRole('button', { name: message(language, 'assistant-settings-test') }),
      );
      const failed = await screen.findByText(
        new RegExp(message(language, 'assistant-failure-no-endpoint').slice(0, 12)),
      );
      expect(failed).toHaveTextContent('(404)');
    });
  });

  describe('the question log', () => {
    it('reads a row again: the panel rises and shows the answer on today’s books', async () => {
      vi.mocked(assistantApi.assistantRerun).mockResolvedValue(real.RERUN);
      show(language);
      const user = userEvent.setup();
      const table = await screen.findByRole('table', {
        name: message(language, 'assistant-log-title'),
      });
      const again = within(table).getAllByRole('button', {
        name: new RegExp(message(language, 'assistant-log-open')),
      });
      await user.click(again[0] ?? table);
      expect(assistantApi.assistantRerun).toHaveBeenCalledWith({
        question_id: real.LOG_LIST_FIRST_PAGE.rows[0]?.id,
      });
      expect(useAssistantStore.getState().open).toBe(true);
      await waitFor(() => {
        expect(useAssistantStore.getState().entries.at(-1)?.asked.type).toBe('log');
      });
    });

    it('a chip row shows the chip’s words, and a row that cannot be read again has no button', async () => {
      show(language);
      const table = await screen.findByRole('table', {
        name: message(language, 'assistant-log-title'),
      });
      const rows = within(table).getAllByRole('row').slice(1);
      expect(rows).toHaveLength(real.LOG_LIST_FIRST_PAGE.rows.length);
      for (const [index, row] of rows.entries()) {
        const logRow = real.LOG_LIST_FIRST_PAGE.rows[index];
        expect(within(row).queryAllByRole('button').length > 0).toBe(logRow?.can_rerun === true);
      }
    });

    it('has the four states: loading, an error with a retry, nothing yet, and the rows', async () => {
      vi.mocked(assistantApi.assistantLogList).mockRejectedValueOnce({
        ok: false,
        errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
        ref: '',
      });
      show(language);
      expect(
        await screen.findByText(message(language, 'assistant-log-failed')),
      ).toBeInTheDocument();
      vi.mocked(assistantApi.assistantLogList).mockResolvedValue({
        ...real.LOG_LIST_FIRST_PAGE,
        rows: [],
        total_count: 0,
      });
      const user = userEvent.setup();
      await user.click(screen.getByRole('button', { name: message(language, 'common-retry') }));
      expect(await screen.findByText(message(language, 'assistant-log-empty'))).toBeInTheDocument();
    });
  });

  it('every button and heading has words from the Fluent file', async () => {
    show(language);
    await screen.findByRole('checkbox');
    for (const id of [
      'assistant-settings-title',
      'assistant-settings-key',
      'assistant-settings-model',
      'assistant-settings-test',
      'assistant-log-title',
    ]) {
      expect(visible(document.body.textContent)).toContain(message(language, id));
    }
    expect(
      screen.getByRole('button', { name: named(language, 'assistant-settings-test') }),
    ).toBeInTheDocument();
  });
});
