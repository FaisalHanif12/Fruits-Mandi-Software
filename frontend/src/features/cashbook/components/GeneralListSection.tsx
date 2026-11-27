// The saved جنرل entries under the form (plan 44, plan 100 F, PRD FR-7.70, FR-19.11): 30 days at a
// time, the latest first, with the box that finds a number or the words of a note, so an older entry can
// be found and fixed. The row that has focus is the one the Edit and Cancel buttons work on; Enter on a
// row opens it. A cancelled entry is left out until the tick is on. The total under the amount column is
// the backend's.

import { GENERAL_ENTRY_TYPES } from '../entry-list-columns';
import { GENERAL_LIST_COLUMNS } from '../list-columns';
import { EntryListSection } from './EntryListSection';

export function GeneralListSection({ today }: { today: string }) {
  return (
    <EntryListSection
      screen="cashbook.general.list"
      titleId="cashbook-general-list-title"
      types={GENERAL_ENTRY_TYPES}
      accountId={null}
      source={{ kind: 'journal' }}
      columns={GENERAL_LIST_COLUMNS}
      today={today}
      totalsField="narration"
      words={{
        labelId: 'cashbook-general-list-label',
        failedId: 'cashbook-general-list-failed',
        emptyId: 'cashbook-general-list-empty',
        emptyHelpId: 'cashbook-general-list-empty-help',
      }}
    />
  );
}
