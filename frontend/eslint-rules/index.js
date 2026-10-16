// Local ESLint rules for the Mandi Software. Each one turns a CLAUDE.md rule into a lint error.
// Tests: tests/unit/eslint-rules.test.ts.

import noJsxLiterals from './no-jsx-literals.js';
import noPaisaArithmetic from './no-paisa-arithmetic.js';
import noPhysicalDirection from './no-physical-direction.js';
import noRawDesignValues from './no-raw-design-values.js';

const plugin = {
  meta: { name: 'mandi' },
  rules: {
    'no-paisa-arithmetic': noPaisaArithmetic,
    'no-jsx-literals': noJsxLiterals,
    'no-physical-direction': noPhysicalDirection,
    'no-raw-design-values': noRawDesignValues,
  },
};

export default plugin;
