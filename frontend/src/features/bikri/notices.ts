// The short notes of the بکری screen (plan 95 items 1 and 2, plan 97 items 2 and 4, PRD FR-12.52) stay for the one
// `NOTICE_MS` of `shared/hooks` (5 seconds), and each shows again when the user repeats the action that gives it
// (`useTimedNotices`, with the screen's count from `NoticePulseContext`). There is no second value here, so no note can
// outlast another. A red mark on a cell is not a note: it stays until the problem is fixed.
export { NOTICE_MS } from '../../shared/hooks';
