// A draft kept before v1.34 (plan 71 H1): its header held a truck, a زمیندار, an اڈا, a driver, a freight
// total, a store, a book, a bill date and a note, which the header does not have any more. The stores
// cannot hold them, so a test makes such a snapshot by putting them into the JSON text of a snapshot
// the stores made, the way an older version of the app saved it.

/** The old header fields as an older version of the app kept them (all text or a picked record). */
export interface OldHeaderFields {
  billDate?: string;
  book?: string;
  truckNo?: string;
  zamindar?: unknown;
  adda?: unknown;
  driverName?: string;
  driverPhone?: string;
  totalFreight?: string;
  store?: unknown;
  note?: string;
}

/** The text of a snapshot with the old header fields put back, every one that is not given empty. */
export function withOldHeader(snapshotText: string, fields: OldHeaderFields): string {
  const snapshot = JSON.parse(snapshotText) as { header: Record<string, unknown> };
  snapshot.header = {
    ...snapshot.header,
    billDate: '',
    book: '',
    truckNo: '',
    zamindar: null,
    adda: null,
    driverName: '',
    driverPhone: '',
    totalFreight: '',
    store: null,
    note: '',
    ...fields,
  };
  return JSON.stringify(snapshot);
}
