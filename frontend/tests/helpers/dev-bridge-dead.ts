// A stand-in for the backend for the development page /dev/dead-amount (plan 40): the screens of the
// dead amount (R38, the move box, the party card) with the worked numbers of E2E-44, so they can be
// opened in a plain browser tab and photographed in both languages. It answers what the shell and
// these screens ask, with the dashboard's answers for the shell and its own for the dead amount;
// every other command is refused with ERR_NOT_IMPLEMENTED. These are the contract's shapes with the
// worked numbers, not the backend's own answers. Never part of a build: only the development route
// imports it.

import { balance, cardOf, money, account } from './accounts';
import {
  PAGE_ARSHAD,
  PAGE_BOTH,
  ARSHAD_DEAD,
  ARSHAD_PREVIEW,
  LINKED_PREVIEW,
  NOTHING_OWED_PREVIEW,
} from './dead';
import { dashboardHandlers, inputOf, refusal, type Input } from './dev-bridge-dashboard';
import { devFlag } from './dev-bridge';
import { deadPage } from './dead';

const AKRAM = account(21, 'buyer', '03040001', 'اکرم ٹول ٹیکس', 'Akram Toll Tax', {
  balance: balance(2880, 'dr'),
});
const ARSHAD = account(22, 'buyer', '03040002', 'ارشد ملتانی', 'Arshad Multani', {
  balance: balance(14684, 'dr'),
  is_active: false,
});

const handlers: Record<string, (input: Input) => unknown> = {
  ...dashboardHandlers,
  assist_party_card: (input) => {
    const id = Number(input['account_id']);
    if (id === 22)
      return cardOf(ARSHAD, {
        dead: ARSHAD_DEAD,
        warnings: [{ kind: 'inactive', amount_paisa: null }],
      });
    return cardOf(AKRAM, { dead: null });
  },
  masters_dead_preview: () => {
    if (devFlag('notowed')) return NOTHING_OWED_PREVIEW;
    return devFlag('linked') ? LINKED_PREVIEW : ARSHAD_PREVIEW;
  },
  masters_dead_mark: () => ({ mark_id: 7, owed_paisa: money(18684) }),
  masters_dead_undo: () => ({ ok: true }),
  reports_dead_amount: (input) => {
    if (devFlag('down')) throw refusal('ERR_INTERNAL');
    if (devFlag('none')) return deadPage([], { owed: 0, recovered: 0, still: 0 });
    return input['filter'] === 'all' || devFlag('both') ? PAGE_BOTH : PAGE_ARSHAD;
  },
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installDeadDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      transformCallback: () => 0,
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const handler = handlers[command];
            if (handler === undefined) {
              reject(refusal('ERR_NOT_IMPLEMENTED'));
              return;
            }
            try {
              resolve(handler(inputOf(args)));
            } catch (error) {
              reject(error instanceof Error ? error : refusal('ERR_INTERNAL'));
            }
          }, 20);
        }),
    },
  });
}
