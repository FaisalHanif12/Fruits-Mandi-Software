// Typed wrappers around the draft commands (plan 13; PRD FR-12.10, NFR-REL-2). A draft is the
// unsaved work of one screen, kept as JSON text so it comes back after a crash or a power cut.
// A draft is not a business record: it has no audit row, and a save removes it. Components never
// call invoke() (Code Standards F4).

import type { DraftDiscardInput } from '../../bindings/DraftDiscardInput';
import type { DraftGetInput } from '../../bindings/DraftGetInput';
import type { DraftRow } from '../../bindings/DraftRow';
import type { DraftSaved } from '../../bindings/DraftSaved';
import type { DraftSaveInput } from '../../bindings/DraftSaveInput';
import { callCommand } from '../api';

export type { DraftRow, DraftSaved };

/** Keeps the draft of a screen, replacing the one before it. */
export function draftsSave(screen: string, payloadJson: string): Promise<DraftSaved> {
  const input: DraftSaveInput = { screen, payload_json: payloadJson };
  return callCommand<DraftSaved>('drafts_save', { input });
}

/** The kept draft of a screen, or null when there is none. */
export function draftsGet(screen: string): Promise<DraftRow | null> {
  const input: DraftGetInput = { screen };
  return callCommand<DraftRow | null>('drafts_get', { input });
}

/** Throws the draft of a screen away. Nothing happens when there is none. */
export function draftsDiscard(screen: string): Promise<null> {
  const input: DraftDiscardInput = { screen };
  return callCommand<null>('drafts_discard', { input });
}
