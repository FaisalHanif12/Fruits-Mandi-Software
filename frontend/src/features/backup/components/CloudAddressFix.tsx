// "Remove the path" under the cloud address (plan 53 Section 10, ERR_BACKUP_CLOUD_ADDRESS_PATH): the
// address was written with more than the host, for example `https://acct.r2.cloudflarestorage.com/mandi`.
// The backend refuses it and says which address to use (`address`), what it cut off (`path`) and, when
// the path is one bucket-like word, that word (`bucket`). One press puts the address right in its box and,
// when the bucket box is still empty, the bucket name in its box. Both are visible and can be changed,
// and nothing is saved until the Owner saves (NFR-USE-27); the screen only copies what the backend said.

import { Localized } from '@fluent/react';
import type { UiError } from '../../../bindings/UiError';
import { errorsForField } from '../../../shared/api';
import { Button } from '../../../shared/ui';

/** The code of the refusal this fix answers. */
export const CLOUD_ADDRESS_PATH_CODE = 'ERR_BACKUP_CLOUD_ADDRESS_PATH';

/** What the backend said about an address with a path: the address to use, and a bucket name if any. */
export interface CloudAddressPath {
  address: string;
  bucket: string | null;
}

/** The refusal of this kind on a field, with its values as text, or null when there is none. */
export function cloudAddressPathOf(
  error: UiError | null | undefined,
  field: string,
): CloudAddressPath | null {
  const item = errorsForField(error, field).find((candidate) => {
    // The code of a backup refusal is a wire word the shared error type does not list: compared as text.
    const code: string = candidate.code;
    return code === CLOUD_ADDRESS_PATH_CODE;
  });
  if (item === undefined) return null;
  const address = item.values['address'];
  const bucket = item.values['bucket'];
  if (typeof address !== 'string' || address === '') return null;
  return { address, bucket: typeof bucket === 'string' && bucket !== '' ? bucket : null };
}

interface CloudAddressFixProps {
  fix: CloudAddressPath;
  /** The bucket box is empty, so the bucket name of the refusal can go into it. */
  bucketEmpty: boolean;
  onFix: (address: string, bucket: string | null) => void;
}

export function CloudAddressFix({ fix, bucketEmpty, onFix }: CloudAddressFixProps) {
  const fillsBucket = fix.bucket !== null && bucketEmpty;
  return (
    <div className="flex flex-wrap items-center gap-3">
      <Button
        onClick={() => {
          onFix(fix.address, fillsBucket ? fix.bucket : null);
        }}
      >
        <Localized id="backup-fix-address-path" />
      </Button>
      {fillsBucket ? (
        <Localized id="backup-fix-address-bucket" vars={{ bucket: fix.bucket ?? '' }}>
          <p className="type-body text-ink-muted" />
        </Localized>
      ) : null}
    </div>
  );
}
