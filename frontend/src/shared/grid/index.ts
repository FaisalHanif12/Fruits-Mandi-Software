// The shared entry grid (entry-grid skill, Code Standards F17, F18, F27). Screens declare
// columns and a row store; they never write key handlers.
export { EntryGrid } from './EntryGrid';
export { createEntryStore, type EntryState, type EntryStore } from './entryStore';
export { COLUMN_WIDTHS, isNumberLike } from './columnTypes';
export { isEditable, isEnterStop, nextEnterStop, type EnterStep } from './enterOrder';
export type { CellRef, EntryColumn, EntryColumnType, EntryRow } from './types';
