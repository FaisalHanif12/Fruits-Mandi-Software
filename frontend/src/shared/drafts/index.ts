// Drafts of unsaved screens (plan 13, PRD FR-12.10). Used by the screens that keep a draft.
export { draftsDiscard, draftsGet, draftsSave, type DraftRow, type DraftSaved } from './api';

/** A screen sends its draft this often while the user types (PRD FR-12.10, NFR-REL-2). */
export const DRAFT_SAVE_INTERVAL_MS = 5_000;
