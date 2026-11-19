// The posting that Edit opens on a بکری (PRD FR-7.70, FR-12.18): the first that can be edited, else (a
// closed day) the first that is still standing. The saved view and the rows of پچھلی بکری both use it, so
// Edit goes to the same entry from either (plan 72).

import type { BikriDetail } from '../../bindings/BikriDetail';
import type { SavedPosting } from '../../bindings/SavedPosting';

export function firstEditablePosting(detail: BikriDetail): SavedPosting | undefined {
  return (
    detail.postings.find((posting) => posting.editable) ??
    detail.postings.find((posting) => !posting.reversed && posting.posting_type !== 'reversal')
  );
}
