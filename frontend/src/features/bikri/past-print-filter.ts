// What the screen پچھلی بکری builds for its print and its Excel (plan 72 P8): the window (`from`, `to`),
// today's section (`with_today`), the filters and the switch "show every بکری" (`all`). It is the input of
// `bikri_list_print_data` without the print language and the paper, which the preview chooses; the Excel
// export takes the same fields. The types of the commands are the generated ones.

import type { BikriListPrintInput } from '../../bindings/BikriListPrintInput';

export type PastPrintFilter = Omit<BikriListPrintInput, 'language' | 'paper'>;
