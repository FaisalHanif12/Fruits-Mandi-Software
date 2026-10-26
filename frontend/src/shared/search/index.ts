// Search for pickers (plan 06, Tech Stack 4.7). The Picker in shared/ui is built on these.
export { findMaster, searchMasters, searchRecent, searchTouch } from './api';
export {
  PICKER_LOAD_AHEAD_ROWS,
  PICKER_PAGE_SIZE,
  PICKER_RESULT_LIMIT,
  PICKER_SEARCH_DELAY_MS,
  masterSource,
  useFetchHits,
  usePickerHits,
  useTouchRecord,
  type PickerSource,
} from './hooks';
export type {
  SearchHit,
  SearchInput,
  SearchScope,
  SearchScopeInput,
  SearchTouchInput,
} from './types';
