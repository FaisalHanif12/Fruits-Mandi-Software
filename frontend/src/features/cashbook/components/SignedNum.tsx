// A signed amount from the backend: a partner's total, a khata's balance, the running cash. A
// negative one reads in the due colour and with its minus sign, so colour is never the only signal
// (NFR-USE-12). The sign comes from the backend's amount; nothing is worked out here.

import { isNegativePaisa, type Paisa } from '../../../shared/format';
import { Num } from '../../../shared/ui';

export function SignedNum({ paisa }: { paisa: Paisa }) {
  return (
    <span className={isNegativePaisa(paisa) ? 'text-due' : undefined}>
      <Num paisa={paisa} />
    </span>
  );
}
