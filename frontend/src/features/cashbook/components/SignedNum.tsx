// A signed amount from the backend: a partner's total, a khata's balance, the running cash. A
// negative one reads in the due colour and with its minus sign, so colour is never the only signal
// (NFR-USE-12). The sign comes from the backend's amount; nothing is worked out here.

import { isNegativePaisa, type Paisa } from '../../../shared/format';
import { Num } from '../../../shared/ui';

interface SignedNumProps {
  paisa: Paisa;
  /** Written at the size of the figure around it (a `Fact` card), not at the 16 px of a number. */
  inherit?: boolean;
}

export function SignedNum({ paisa, inherit = false }: SignedNumProps) {
  return (
    <span className={isNegativePaisa(paisa) ? 'text-due' : undefined}>
      {inherit ? <Num inherit paisa={paisa} /> : <Num paisa={paisa} />}
    </span>
  );
}
