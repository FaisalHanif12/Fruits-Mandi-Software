// Umar's case of the demo of 08-11-2026 (D8, Bug 1, plan 61), opened on the real screen. The
// backend's answers and the lot are in bikri-umar-data.ts.

import { screen } from '@testing-library/react';
import userEvent from '@testing-library/user-event';
import type { BikriDraft } from '../../src/bindings/BikriDraft';
import type { BikriPreview } from '../../src/bindings/BikriPreview';
import { usePendingRestore } from '../../src/features/bikri/draft-offer';
import { createBikriDraft } from '../../src/features/bikri/draft-store';
import { parseSnapshot, serialiseDraft } from '../../src/features/bikri/draft-snapshot';
import type { Language } from '../../src/i18n';
import { fakeAccountSearch, fakeAccounts } from './accounts';
import { renderApp } from './app';
import { fakeBikri } from './bikri';
import { EHSAN, UMAR_BUYERS, fillUmarsLot, umarAnswer } from './bikri-umar-data';
import { message } from './render';

export { umarAnswer } from './bikri-umar-data';

/** Makes the next screen the sale book opens as one holding Umar's lot. Call before opening it. */
export function seedUmarsLot(): void {
  const stores = createBikriDraft('10-10-2026');
  fillUmarsLot(stores);
  const snapshot = parseSnapshot(serialiseDraft(stores));
  if (snapshot === null) throw new Error('the seeded draft is not a snapshot');
  usePendingRestore.getState().offer(snapshot);
}

/** Opens the sale book with the three buyers of Umar's case known to the pickers. */
export async function openUmarsBook(
  language: Language,
  answer: (draft: BikriDraft) => BikriPreview | Promise<BikriPreview> = umarAnswer,
) {
  const backend = fakeBikri(answer);
  fakeAccounts([EHSAN, ...UMAR_BUYERS]);
  fakeAccountSearch([EHSAN, ...UMAR_BUYERS]);
  const app = await renderApp('/bikri/book');
  await screen.findByRole('heading', { level: 1, name: message(language, 'glossary-bikri-book') });
  return { ...app, backend, user: userEvent.setup() };
}
