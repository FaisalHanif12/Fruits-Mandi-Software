// The text side of the partner setup (no money, no adding): which text a share input shows, which
// rows are sent, and whether anything is unsaved. The backend reads every share and adds them up
// (partners_check_shares, partners_set_shares), and is the final word.

import { formatBps } from '../../shared/format';
import type { PartnerRow, ShareInput } from './types';

/** The partners whose shares are set (PRD FR-17.1): the active ones, in the order of the list. */
export function activePartners(partners: readonly PartnerRow[]): PartnerRow[] {
  return partners.filter((partner) => partner.is_active);
}

/**
 * The text a share input shows before the user types in it. Once any active partner has a share,
 * everyone shows his saved share, and a partner with none shows "0". Before any share was ever
 * set, the inputs are empty, so a fresh form is not full of zeros the user did not type.
 */
export function savedShareText(partner: PartnerRow, partners: readonly PartnerRow[]): string {
  const anyShare = activePartners(partners).some((other) => other.share_bps > 0);
  return anyShare ? formatBps(partner.share_bps) : '';
}

/** The text on screen for a partner: what the user typed, else his saved share. */
export function shareText(
  partner: PartnerRow,
  partners: readonly PartnerRow[],
  edits: Readonly<Record<number, string>>,
): string {
  return edits[partner.id] ?? savedShareText(partner, partners);
}

/** The rows sent to the check and to the save: every active partner once, with his typed text. */
export function shareInputs(
  partners: readonly PartnerRow[],
  edits: Readonly<Record<number, string>>,
): ShareInput[] {
  return activePartners(partners).map((partner) => ({
    partner_id: partner.id,
    share: shareText(partner, partners, edits),
  }));
}

/** True when any share text differs from the saved one, or the date was typed. */
export function hasUnsavedShares(
  partners: readonly PartnerRow[],
  edits: Readonly<Record<number, string>>,
  fromEdit: string | null,
): boolean {
  if (fromEdit !== null) return true;
  return activePartners(partners).some(
    (partner) =>
      edits[partner.id] !== undefined && edits[partner.id] !== savedShareText(partner, partners),
  );
}
