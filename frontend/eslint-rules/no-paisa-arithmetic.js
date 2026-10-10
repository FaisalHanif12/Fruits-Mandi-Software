// CLAUDE.md rule 1, Code Standards F3, M3 and M12: the frontend never calculates money.
//
// A value typed `Paisa` (the branded type in src/shared/format/money.ts) may only be passed to
// display formatters and sort comparisons. This type-aware rule fails on:
// - arithmetic operators: `a + b`, `a - b`, `a * b`, `a / b`, `a % b`, `a ** b`, bit operators;
// - compound assignment and update: `total += line`, `amount++`;
// - unary minus or plus: `-amount`;
// - `Math.*(amount)` and `Math.max(...amounts)`, `amount.toFixed()`, `amount.toPrecision()`, `amount.toString()`;
// - conversions that would let the maths happen on a plain number or string: `Number(amount)`,
//   `String(amount)`, `parseFloat(amount)`, `parseInt(amount)`, `Number.parseFloat(amount)`,
//   `amount as number`, `<number>amount` and `${amount}` in a template string.
//   Example: `Number(a) + 1` and `parseFloat(String(a))` fail at the inner call.
// - Paisa flowing into any call or `new` argument whose parameter is not typed Paisa (after
//   generic inference): `new Number(amount)`, `JSON.stringify(amount)`, `console.log(amount)`,
//   `identity<unknown>(amount)`, and a spread of Paisa[] into such a rest parameter;
// - a function passed where it would receive Paisa into a parameter not typed Paisa:
//   `amounts.map(Number)`, `amounts.map(String)`, `amounts.forEach(console.log)`,
//   `amounts.map((p: unknown) => p)`.
// Allowed: Paisa into Paisa slots (`formatPaisa(a)`, `setTotal(a)` from useState<Paisa | null>,
// props typed Paisa), `amounts.sort(comparePaisa)`, `amounts.map((p) => formatPaisa(p))`.
// Comparisons (`<`, `===`) stay allowed, because sorting needs them.
//
// Example: `const total = bill.buyerTotal + bill.laga;` fails when either side is Paisa.
// The only file allowed to do this is src/shared/format/money.ts (set in eslint.config.js),
// where formatPaisa builds the display text.

import { ESLintUtils } from '@typescript-eslint/utils';
import { carriesPaisa, isPaisaType, leaksThroughCallback, parameterTypeAt } from './paisa-types.js';

/** @typedef {import('@typescript-eslint/utils').TSESTree.Node} Node */

const createRule = ESLintUtils.RuleCreator(
  (name) =>
    `https://github.com/FaisalHanif12/Fruits-Mandi-Software/blob/main/frontend/eslint-rules/${name}.js`,
);

const ARITHMETIC_OPERATORS = new Set([
  '+',
  '-',
  '*',
  '/',
  '%',
  '**',
  '<<',
  '>>',
  '>>>',
  '&',
  '|',
  '^',
]);
const NUMBER_TEXT_METHODS = new Set([
  'toFixed',
  'toPrecision',
  'toExponential',
  'toString',
  'toLocaleString',
  'valueOf',
]);
const CONVERT_FUNCTIONS = new Set(['Number', 'String', 'BigInt', 'parseFloat', 'parseInt']);
export default createRule({
  name: 'no-paisa-arithmetic',
  meta: {
    type: 'problem',
    docs: {
      description:
        'Disallow arithmetic on the Paisa money type outside shared/format/money.ts (the backend owns all money).',
    },
    messages: {
      operator:
        "Money maths in the frontend: '{{operator}}' on a Paisa value. The backend calculates every amount; show what it returns (CLAUDE.md rule 1, Code Standards F3).",
      call: "Money maths in the frontend: '{{name}}' on a Paisa value. Only formatPaisa in shared/format/money.ts may touch Paisa (Code Standards F3, M3).",
      cast: 'A Paisa value is cast to another type. Casting it would let money maths through; only formatPaisa in shared/format/money.ts may touch Paisa (Code Standards F3).',
      flow: "A Paisa value goes into '{{name}}', where the parameter is not typed Paisa, so the amount stops being money and maths becomes possible. Pass Paisa only where Paisa is expected (Code Standards F3, M12).",
      callback:
        "'{{name}}' would receive Paisa values into a parameter that is not typed Paisa. Use comparePaisa, formatPaisa or an inline function that keeps the value Paisa (Code Standards F3, M12).",
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    const services = ESLintUtils.getParserServices(context);
    const checker = services.program.getTypeChecker();

    /** @param {Node} node */
    const isPaisa = (node) => isPaisaType(services.getTypeAtLocation(node), checker);

    /**
     * True when any argument is Paisa, including a spread of a Paisa list:
     * `Math.max(...amounts)` where amounts is Paisa[].
     * @param {import('@typescript-eslint/utils').TSESTree.CallExpressionArgument[]} args
     */
    const anyPaisaArgument = (args) =>
      args.some((arg) => {
        if (arg.type !== 'SpreadElement') return isPaisa(arg);
        const element = services.getTypeAtLocation(arg.argument).getNumberIndexType();
        return element !== undefined && isPaisaType(element, checker);
      });

    /**
     * Checks where each argument of a call or `new` goes (the flow checks of the header).
     * @param {import('@typescript-eslint/utils').TSESTree.CallExpression | import('@typescript-eslint/utils').TSESTree.NewExpression} node
     */
    const checkArguments = (node) => {
      if (node.arguments.length === 0) return;
      const tsNode = /** @type {import('typescript').CallLikeExpression} */ (
        services.esTreeNodeToTSNodeMap.get(node)
      );
      const signature = checker.getResolvedSignature(tsNode);
      if (signature === undefined) return;
      const name = context.sourceCode.getText(node.callee).slice(0, 40);
      node.arguments.forEach((argument, index) => {
        const spread = argument.type === 'SpreadElement';
        const valueNode = spread ? argument.argument : argument;
        const argumentType = services.getTypeAtLocation(valueNode);
        const slot = parameterTypeAt(signature, index, checker);
        const valueType = spread
          ? (argumentType.getNumberIndexType() ?? argumentType)
          : argumentType;
        if (carriesPaisa(valueType, checker)) {
          if (slot === undefined || !carriesPaisa(slot, checker)) {
            context.report({ node: argument, messageId: 'flow', data: { name } });
          }
          return;
        }
        if (!spread && slot !== undefined && leaksThroughCallback(slot, argumentType, checker)) {
          context.report({
            node: argument,
            messageId: 'callback',
            data: { name: context.sourceCode.getText(argument).slice(0, 40) },
          });
        }
      });
    };

    return {
      NewExpression(node) {
        checkArguments(node);
      },
      BinaryExpression(node) {
        if (!ARITHMETIC_OPERATORS.has(node.operator)) return;
        const leftIsPaisa = node.left.type !== 'PrivateIdentifier' && isPaisa(node.left);
        if (leftIsPaisa || isPaisa(node.right)) {
          context.report({ node, messageId: 'operator', data: { operator: node.operator } });
        }
      },
      AssignmentExpression(node) {
        if (
          node.operator === '=' ||
          node.operator === '&&=' ||
          node.operator === '||=' ||
          node.operator === '??='
        ) {
          return;
        }
        if (isPaisa(node.left) || isPaisa(node.right)) {
          context.report({ node, messageId: 'operator', data: { operator: node.operator } });
        }
      },
      UpdateExpression(node) {
        if (isPaisa(node.argument)) {
          context.report({ node, messageId: 'operator', data: { operator: node.operator } });
        }
      },
      UnaryExpression(node) {
        if (
          (node.operator === '-' || node.operator === '+' || node.operator === '~') &&
          isPaisa(node.argument)
        ) {
          context.report({ node, messageId: 'operator', data: { operator: node.operator } });
        }
      },
      TSAsExpression(node) {
        if (isPaisa(node.expression) && !isPaisa(node)) {
          context.report({ node, messageId: 'cast' });
        }
      },
      TSTypeAssertion(node) {
        if (isPaisa(node.expression) && !isPaisa(node)) {
          context.report({ node, messageId: 'cast' });
        }
      },
      TemplateLiteral(node) {
        if (node.expressions.some((expression) => isPaisa(expression))) {
          context.report({ node, messageId: 'call', data: { name: 'template string' } });
        }
      },
      CallExpression(node) {
        const { callee } = node;
        if (callee.type === 'Identifier' && CONVERT_FUNCTIONS.has(callee.name)) {
          if (anyPaisaArgument(node.arguments)) {
            context.report({ node, messageId: 'call', data: { name: callee.name } });
          }
          return;
        }
        if (callee.type !== 'MemberExpression' || callee.property.type !== 'Identifier') return;
        const method = callee.property.name;
        const objectName = callee.object.type === 'Identifier' ? callee.object.name : '';
        if (objectName === 'Math' || objectName === 'Number') {
          if (anyPaisaArgument(node.arguments)) {
            context.report({ node, messageId: 'call', data: { name: `${objectName}.${method}` } });
          }
          return;
        }
        if (NUMBER_TEXT_METHODS.has(method) && isPaisa(callee.object)) {
          context.report({ node, messageId: 'call', data: { name: method } });
        }
      },
      // The flow checks run after the named checks above, for calls they did not report.
      'CallExpression:exit'(node) {
        const { callee } = node;
        const named =
          (callee.type === 'Identifier' && CONVERT_FUNCTIONS.has(callee.name)) ||
          (callee.type === 'MemberExpression' &&
            callee.object.type === 'Identifier' &&
            (callee.object.name === 'Math' || callee.object.name === 'Number'));
        if (!named) checkArguments(node);
      },
    };
  },
});
