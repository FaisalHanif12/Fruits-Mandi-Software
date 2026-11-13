// The chips of the states the lists share (plan 75, item 4): one look for each, so "Saved" is the same
// on every list and "Cancelled" is the same everywhere. The word comes from Fluent in each list; here is
// only the icon and the tone that go with it.

import { Ban, Check, CircleDot, Lock } from 'lucide-react';
import type { StatusChipLook } from './StatusChip';

/** A saved entry: in order. */
export const STATUS_SAVED: StatusChipLook = { tone: 'primary', icon: Check };

/** A cancelled entry: amber, it needs a look, and its reversal is in the books. */
export const STATUS_CANCELLED: StatusChipLook = { tone: 'warn', icon: Ban };

/** Something still open, such as a garden that takes entries. */
export const STATUS_OPEN: StatusChipLook = { tone: 'primary', icon: CircleDot };

/** Something finished, such as a closed garden: quiet. */
export const STATUS_CLOSED: StatusChipLook = { tone: 'muted', icon: Lock };
