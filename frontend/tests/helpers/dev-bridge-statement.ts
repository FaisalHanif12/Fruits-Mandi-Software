// A stand-in for the Tauri bridge for the development page /dev/statement-print (plan 74 T3): the statement print
// box and the print window with a double of the backend's answers (statement-print.ts). Never part of a build:
// only the development route imports it. Flags in the hash: `?window&language=ur|en&paper=a4|a5&data=example|
// thirty|empty` shows the page the way the hidden print window makes it; without `window` the page has the
// button "اسٹیٹمنٹ پرنٹ" and its box. `&down` makes the statement command fail.

import type { PrintFrameInput } from '../../src/bindings/PrintFrameInput';
import type { PrintStatementInput } from '../../src/bindings/PrintStatementInput';
import type { StatementPrintDataInput } from '../../src/bindings/StatementPrintDataInput';
import { printFrameFor } from './print-frame-real';
import { statementAnswer } from './statement-print';

/** The date of the stand-in's day. */
export const STATEMENT_DEV_TODAY = '2026-11-12';

function flag(name: string): boolean {
  return new URLSearchParams(window.location.hash.split('?')[1] ?? '').has(name);
}

type Handler = (input: Record<string, unknown>) => unknown;

const handlers: Record<string, Handler> = {
  day_status: () => ({ today: STATEMENT_DEV_TODAY, closed_up_to: null, today_open: true }),
  reports_print_header: (input) => printFrameFor(input as unknown as PrintFrameInput),
  party_ledgers_statement_print_data: (input) => {
    if (flag('down')) {
      throw Object.assign(new Error('ERR_INTERNAL'), {
        ok: false,
        errors: [{ code: 'ERR_INTERNAL', field: null, values: {} }],
        ref: 'E-DEV',
      });
    }
    return statementAnswer(input as unknown as StatementPrintDataInput);
  },
  party_ledgers_statement_print: (input) => {
    const { to } = input as unknown as PrintStatementInput;
    return { printed: [], pdf_paths: to === 'pdf' ? ['statement-0020-ur.pdf'] : null };
  },
};

/** Puts the stand-in where Tauri's `invoke` looks for the bridge. */
export function installStatementDevBridge(): void {
  Object.defineProperty(window, '__TAURI_INTERNALS__', {
    configurable: true,
    value: {
      invoke: (command: string, args: unknown): Promise<unknown> =>
        new Promise((resolve, reject) => {
          window.setTimeout(() => {
            const handler = handlers[command];
            if (handler === undefined) {
              reject(new Error(`no stand-in for ${command}`));
              return;
            }
            try {
              const input =
                typeof args === 'object' && args !== null && 'input' in args
                  ? (args as { input: Record<string, unknown> }).input
                  : {};
              resolve(handler(input));
            } catch (failure: unknown) {
              reject(failure instanceof Error ? failure : new Error('failed'));
            }
          }, 20);
        }),
    },
  });
}
