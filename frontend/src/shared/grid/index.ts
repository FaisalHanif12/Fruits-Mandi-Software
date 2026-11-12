// The shared entry grid (entry-grid skill, Code Standards F17, F18, F27). Screens declare
// columns and a row store; they never write key handlers.
export { EntryGrid } from './EntryGrid';
export { REMOVE_COLUMN_ID, REMOVE_COLUMN_WIDTH, type RemoveQuestion } from './removeRow';
export {
  ListGrid,
  type ListCellControl,
  type ListColumn,
  type ListContext,
  type ListGroups,
  type ListHeading,
  type ListMonths,
  type ListPaging,
  type ListSort,
} from './ListGrid';
export { Pager } from './Pager';
export { useListPaging } from './useListPaging';
export { PagedList } from './PagedList';
export { ListRegion, SHORT_LIST_ROWS, isLongList, listHeightMode } from './ListRegion';
export { columnsWithMore, type MoreColumnsPlan } from './moreColumns';
export { addRowAndFocus, createEntryStore, type EntryState, type EntryStore } from './entryStore';
export { AddRowButton, type AddRowLimit } from './AddRowButton';
export { useAddRowLink, type AddRowLink } from './addLink';
export {
  COLUMN_WIDTHS,
  FIT_MIN,
  FIT_MIN_WIDTHS,
  FIT_WEIGHTS,
  LIST_COLUMN_WIDTHS,
  LIST_GROW_MIN_WIDTH,
  isListNumberLike,
  isNumberLike,
  type ListColumnType,
} from './columnTypes';
export { isEditable, isEnterStop, nextEnterStop, type EnterStep } from './enterOrder';
export { syncEditedText } from './editEnd';
export { leftTextKey } from './leftText';
export {
  isReadOnlyType,
  type CellRef,
  type ChoiceSpec,
  type EntryColumn,
  type EntryColumnType,
  type EntryRow,
  type GridInterceptors,
  type PickedRecord,
  type PickerSpec,
} from './types';
