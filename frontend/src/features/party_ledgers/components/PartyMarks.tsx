// The words that mark a party on the head of his statement and of his slip, never colour alone
// (NFR-USE-12): a person in the dead amount is marked ڈوبی (PRD FR-10.10), another inactive account
// says so. A party that is active and not dead has no mark.

import { Localized } from '@fluent/react';

interface PartyMarksProps {
  isDead: boolean;
  isActive: boolean;
}

export function PartyMarks({ isDead, isActive }: PartyMarksProps) {
  if (isDead) {
    return (
      <Localized id="dead-tag">
        <span className="type-label text-due" />
      </Localized>
    );
  }
  return isActive ? null : (
    <Localized id="masters-status-inactive">
      <span className="type-label text-ink-muted" />
    </Localized>
  );
}
