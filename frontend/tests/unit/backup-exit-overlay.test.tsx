// The window-close overlay (plan 18, PRD FR-20.3): while the copy at exit runs, the backend holds
// the first close request and sends the event `backup-exit`. The overlay says why the window has not
// closed, in the Urdu and in the English app, and goes when the copy is done. It listens once and
// stops listening when it goes. A payload it does not know is ignored.

import { LocalizationProvider, ReactLocalization } from '@fluent/react';
import { act, render, screen, waitFor } from '@testing-library/react';
import { beforeEach, describe, expect, it, vi } from 'vitest';
import { BackupExitOverlay } from '../../src/features/backup/components/BackupExitOverlay';
import { bundles, LANGUAGES, type Language } from '../../src/i18n';
import * as events from '../../src/shared/api/events';
import { message, visible } from '../helpers/render';

vi.mock('../../src/shared/api/events');

beforeEach(() => {
  vi.resetAllMocks();
});

describe.each(LANGUAGES)('the window-close overlay (%s)', (language) => {
  /** Renders the overlay and keeps the handler it gave to the event listener. */
  function mount(lang: Language) {
    const stop = vi.fn();
    let handler: (payload: unknown) => void = () => undefined;
    vi.mocked(events.listenEvent).mockImplementation((_name, next) => {
      handler = next;
      return Promise.resolve(stop);
    });
    const l10n = new ReactLocalization([bundles[lang]]);
    const view = render(
      <LocalizationProvider l10n={l10n}>
        <BackupExitOverlay />
      </LocalizationProvider>,
    );
    return {
      view,
      stop,
      send: (payload: unknown) => {
        act(() => {
          handler(payload);
        });
      },
    };
  }

  it('listens for backup-exit, shows why the window waits, and goes when the copy is done', async () => {
    const { send } = mount(language);
    await waitFor(() => {
      expect(events.listenEvent).toHaveBeenCalledWith('backup-exit', expect.any(Function));
    });
    expect(screen.queryByRole('alertdialog')).toBeNull();
    send({ phase: 'running' });
    const dialog = await screen.findByRole('alertdialog');
    expect(visible(dialog.textContent)).toContain(message(language, 'backup-exit-title'));
    expect(visible(dialog.textContent)).toContain(message(language, 'backup-exit-help'));
    send({ phase: 'done' });
    await waitFor(() => {
      expect(screen.queryByRole('alertdialog')).toBeNull();
    });
  });

  it('ignores a payload it does not know, and stops listening when it goes', async () => {
    const { view, send, stop } = mount(language);
    await waitFor(() => {
      expect(events.listenEvent).toHaveBeenCalled();
    });
    send({ phase: 'sleeping' });
    send('running');
    send(null);
    expect(screen.queryByRole('alertdialog')).toBeNull();
    view.unmount();
    await waitFor(() => {
      expect(stop).toHaveBeenCalled();
    });
  });
});
