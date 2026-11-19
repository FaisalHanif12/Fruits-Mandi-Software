// Display-only formatting (Tech Stack 5.3). Nothing here calculates money (CLAUDE.md rule 1).
export {
  chartValueOfPaisa,
  comparePaisa,
  formatPaisa,
  formatPaisaValue,
  groupDigits,
  isNegativePaisa,
  isPositivePaisa,
  isZeroPaisa,
  paisaOfErrorValue,
  type FormatPaisaOptions,
  type Grouping,
  type Paisa,
} from './money';
export { addDays, dayOfText, isIsoDay } from './dayMath';
export {
  DAYS_WINDOW,
  canDaysWindowComeNearer,
  canDaysWindowGoBack,
  daysWindowEnd,
  daysWindowOf,
  stepDaysWindow,
} from './daysWindow';
export {
  addMonths,
  canMonthComeNearer,
  canMonthGoBack,
  displayMonth,
  isIsoMonth,
  monthBounds,
  monthOf,
  monthOfText,
  monthParts,
  stepMonth,
  windowMonth,
} from './months';
export { formatBps } from './percent';
export { GRADE_CHOICES, GRADE_WORD_ID, gradedItem, type GradeChoice } from './grade';
export { formatGrams } from './weight';
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
export {
  PERIOD_KINDS,
  periodMessageId,
  periodRange,
  type PeriodKind,
  type PeriodRange,
} from './period';
