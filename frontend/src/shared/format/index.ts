// Display-only formatting (Tech Stack 5.3). Nothing here calculates money (CLAUDE.md rule 1).
export {
  comparePaisa,
  formatPaisa,
  formatPaisaValue,
  groupDigits,
  isNegativePaisa,
  isPositivePaisa,
  isZeroPaisa,
  type FormatPaisaOptions,
  type Grouping,
  type Paisa,
} from './money';
export { formatBps } from './percent';
export { DEFAULT_PRINT_DIGITS, printDigits, toUrduDigits, type PrintDigits } from './digits';
export { DEFAULT_GROUPING, useGrouping, useGroupingStore } from './grouping';
export {
  formatDateParts,
  isoToDisplayDate,
  readDateText,
  shiftDateText,
  todayText,
  utcToLocalDisplay,
  utcToLocalTime,
} from './dates';
