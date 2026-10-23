// CLAUDE.md rule 7, Code Standards F31: no user-facing text in code.
// Every word on screen comes from the Fluent files, in Urdu and English.
//
// Fails on:
// - text written between JSX tags: `<h1>Mandi</h1>`, `<p>بکری</p>`, `<td>10</td>`;
// - a string with letters shown as a JSX child, also through a branch, a constant, or an
//   array or object looked up by index: `<p>{'Save'}</p>`, `{on ? 'Yes' : 'No'}`,
//   `{on && 'Saved'}`, `{`Sale ${n}`}`, `const label = 'Save'; ... {label}`,
//   `const words = ['Save', 'Cancel']; ... {words[n]}`, `{TITLES['a']}`, and `{label()}`
//   where label is a local function that returns a word string;
// - a word string as a Fluent variable: `l10n.getString('x', { v: 'Hello' })`,
//   `<Localized vars={{ v: 'Hello' }}>`;
// - words joined by `+`: `<p>{'Saved ' + name}</p>`, `showToast('success', 'Saved ' + name)`;
// - a word string handed to a function that shows it: `showToast('success', 'Saved')`,
//   `alert('Saved')`, `confirm('Sure?')`;
// - a word string written into the page from code: `document.title = 'Mandi'`,
//   `el.textContent = 'Saved'`, `el.innerText`, `el.innerHTML`, `el.setAttribute('title', 'x')`;
// - a string with letters in ANY JSX attribute, also through a branch, except the attributes
//   that only ever hold code: CODE_ATTRIBUTES, data-*, and any prop whose TypeScript type is a
//   fixed set of string values (`variant: 'primary' | 'secondary'`, `tone`), because such a
//   value is a choice made in code, never text. So `title`, `alt`, `placeholder`, `aria-label`
//   and `string` props such as `<Button ariaLabel="Save">` all fail.
// Numbers and separators go through components (`<Num>`, `<Code>`), not literal text.
//
// Use `<Localized id="common-app-name">` or `l10n.getString('...')` instead.

import { ESLintUtils } from '@typescript-eslint/utils';
import ts from 'typescript';
import { attributeName, branches } from './class-names.js';

/** @typedef {import('@typescript-eslint/utils').TSESTree.Node} Node */
/** @typedef {import('@typescript-eslint/utils').TSESLint.Scope.Scope} Scope */

const createRule = ESLintUtils.RuleCreator(
  (name) =>
    `https://github.com/FaisalHanif12/Fruits-Mandi-Software/blob/main/frontend/eslint-rules/${name}.js`,
);

/**
 * Attributes whose value is code, never text a user reads or hears. Each one is either a DOM
 * attribute with a fixed set of keyword values, an identifier, or a prop of our own components
 * that takes a code value (`variant`, Fluent's message `id`, `labelId`, `titleId`, `helpId`, `finishId`,
 * `toastId`, `saveId`, `failedId` and `typeLabelId`: Fluent message IDs of a form, `path` and `leaveTo`: the address of a screen, `lotKey`: the row id of a lot, `fieldsShown` and `extraFields`: field
 * paths such as `lots[0].sales[2].qty`, `hidden`: menu item IDs, `theme`: AG Grid's "legacy").
 */
const CODE_ATTRIBUTES = new Set([
  'className',
  'class',
  'id',
  'key',
  'type',
  'lang',
  'dir',
  'role',
  'href',
  'src',
  'name',
  'htmlFor',
  'variant',
  'fieldsShown',
  'extraFields',
  'labelId',
  'titleId',
  'helpId',
  'failedId',
  'typeLabelId',
  'path',
  'leaveTo',
  'finishId',
  'toastId',
  'saveId',
  'lotKey',
  'scope',
  'hidden',
  'theme',
  'autoComplete',
  'inputMode',
  'enterKeyHint',
  'target',
  'rel',
  'method',
  'form',
  'tabIndex',
  'aria-hidden',
  'aria-live',
  'aria-atomic',
  'aria-relevant',
  'aria-pressed',
  'aria-expanded',
  'aria-selected',
  'aria-checked',
  'aria-current',
  'aria-controls',
  'aria-labelledby',
  'aria-describedby',
  'aria-owns',
  'aria-haspopup',
  'aria-orientation',
  'aria-sort',
  'aria-busy',
  'aria-invalid',
  'aria-modal',
  'aria-disabled',
  'aria-readonly',
  'aria-required',
]);

const HAS_LETTER = /\p{L}/u;

/**
 * The string a `const` holds, when the identifier refers to `const x = 'text'`.
 * @param {Node} node an Identifier
 * @param {Scope} scope
 * @returns {Node | undefined}
 */
function constantValue(node, scope) {
  if (node.type !== 'Identifier') return undefined;
  /** @type {Scope | null} */
  let current = scope;
  while (current !== null) {
    const variable = current.set.get(node.name);
    if (variable !== undefined) {
      const definition = variable.defs[0];
      if (
        definition?.type === 'Variable' &&
        definition.parent.kind === 'const' &&
        definition.node.init !== null
      ) {
        return definition.node.init;
      }
      return undefined;
    }
    current = current.upper;
  }
  return undefined;
}

/**
 * The local function an identifier names: `function label() {...}` or
 * `const label = () => ...`. Undefined for anything else (imports, parameters, `let`).
 * @param {Node} node
 * @param {Scope} scope
 * @returns {Node | undefined}
 */
function localFunction(node, scope) {
  if (node.type !== 'Identifier') return undefined;
  /** @type {Scope | null} */
  let current = scope;
  while (current !== null) {
    const variable = current.set.get(node.name);
    if (variable !== undefined) {
      const definition = variable.defs[0];
      if (definition?.type === 'FunctionName') return definition.node;
      if (definition?.type === 'Variable' && definition.parent.kind === 'const') {
        const init = definition.node.init;
        if (init?.type === 'ArrowFunctionExpression' || init?.type === 'FunctionExpression') {
          return init;
        }
      }
      return undefined;
    }
    current = current.upper;
  }
  return undefined;
}

const FUNCTION_TYPES = new Set([
  'FunctionDeclaration',
  'FunctionExpression',
  'ArrowFunctionExpression',
]);

/**
 * The expressions a function returns: the body of `() => 'x'`, and the argument of every
 * `return` in its body (not in functions nested inside it).
 * @param {Node} fn
 * @returns {Node[]}
 */
function returnedValues(fn) {
  if (fn.type === 'ArrowFunctionExpression' && fn.expression) return [fn.body];
  /** @type {Node[]} */
  const values = [];
  /** @param {unknown} node */
  const visit = (node) => {
    if (typeof node !== 'object' || node === null || !('type' in node)) return;
    const current = /** @type {Node} */ (node);
    if (current.type === 'ReturnStatement') {
      if (current.argument !== null) values.push(current.argument);
      return;
    }
    if (FUNCTION_TYPES.has(current.type)) return;
    for (const [key, child] of Object.entries(current)) {
      if (key === 'parent') continue;
      if (Array.isArray(child)) child.forEach(visit);
      else visit(child);
    }
  };
  if ('body' in fn && fn.body !== null && typeof fn.body === 'object') visit(fn.body);
  return values;
}

/** How many `const` hops are followed, so a chain of constants can never loop for ever. */
const MAX_DEPTH = 5;

/**
 * True when the expression ends up as a string (or template string) with letters: directly,
 * through a branch, through a `const`, or as an element of an array or a value of an object
 * looked up by index (`words[n]`, `TITLES['a']`, `TITLES.a`).
 * @param {Node} node
 * @param {Scope} scope
 * @param {number} [depth]
 * @returns {boolean}
 */
function isWordString(node, scope, depth = 0) {
  if (depth > MAX_DEPTH) return false;
  return branches(node).some((value) => {
    if (value.type === 'Literal') {
      return typeof value.value === 'string' && HAS_LETTER.test(value.value);
    }
    if (value.type === 'TemplateLiteral') {
      return value.quasis.some((quasi) => HAS_LETTER.test(quasi.value.cooked ?? quasi.value.raw));
    }
    // 'Saved ' + name: the words are still written in code.
    if (value.type === 'BinaryExpression' && value.operator === '+') {
      return (
        isWordString(value.left, scope, depth + 1) || isWordString(value.right, scope, depth + 1)
      );
    }
    if (value.type === 'ArrayExpression') {
      return value.elements.some(
        (element) =>
          element !== null &&
          element.type !== 'SpreadElement' &&
          isWordString(element, scope, depth + 1),
      );
    }
    if (value.type === 'MemberExpression') {
      return hasWordValues(value.object, scope, depth + 1);
    }
    // {label()} where label is a local function that returns a word string.
    if (value.type === 'CallExpression') {
      const fn = localFunction(value.callee, scope);
      return (
        fn !== undefined &&
        returnedValues(fn).some((returned) => isWordString(returned, scope, depth + 1))
      );
    }
    const constant = constantValue(value, scope);
    return constant !== undefined && constant !== value && isWordString(constant, scope, depth + 1);
  });
}

/**
 * True when an array or object (or a `const` holding one) contains a word string, so indexing
 * into it can show a word.
 * @param {Node} node
 * @param {Scope} scope
 * @param {number} depth
 * @returns {boolean}
 */
function hasWordValues(node, scope, depth) {
  if (depth > MAX_DEPTH) return false;
  const target = node.type === 'Identifier' ? constantValue(node, scope) : node;
  if (target === undefined) return false;
  if (target.type === 'TSAsExpression' || target.type === 'TSSatisfiesExpression') {
    return hasWordValues(target.expression, scope, depth + 1);
  }
  if (target.type === 'ArrayExpression') {
    return isWordString(target, scope, depth + 1);
  }
  if (target.type === 'ObjectExpression') {
    return objectHasWord(target, scope, depth + 1);
  }
  return false;
}

/**
 * True when any value of the object literal is a word string.
 * @param {import('@typescript-eslint/utils').TSESTree.ObjectExpression} object
 * @param {Scope} scope
 * @param {number} [depth]
 * @returns {boolean}
 */
function objectHasWord(object, scope, depth = 0) {
  return object.properties.some(
    (property) =>
      property.type === 'Property' &&
      property.value.type !== 'AssignmentPattern' &&
      isWordString(property.value, scope, depth + 1),
  );
}

/** Properties of DOM objects that put text on screen or in the title bar. */
const TEXT_PROPERTIES = new Set([
  'title',
  'textContent',
  'innerText',
  'innerHTML',
  'outerHTML',
  'placeholder',
  'ariaLabel',
  'alt',
  'label',
]);

/** Attributes set through setAttribute() that a user reads or hears. */
const TEXT_ATTRIBUTE_NAMES = new Set(['title', 'aria-label', 'placeholder', 'alt', 'label']);

/**
 * Functions that put a text argument in front of the user, and which argument it is (0 is the
 * first). `showToast(tone, text)` is ours; alert, confirm and prompt are the browser's.
 */
const TEXT_SINKS = new Map([
  ['showToast', 1],
  ['alert', 0],
  ['confirm', 0],
  ['prompt', 0],
]);

/** Props that pass variables into a Fluent message: `<Localized vars={...}>`. */
const FLUENT_VARIABLE_PROPS = new Set(['vars']);

export default createRule({
  name: 'no-jsx-literals',
  meta: {
    type: 'problem',
    docs: {
      description: 'Disallow user-facing text written in JSX; all text comes from Fluent.',
    },
    messages: {
      text: 'Text written in code: "{{text}}". Every word on screen comes from the Fluent files in ur and en (CLAUDE.md rule 7, Code Standards F31).',
      attribute:
        'Text written in code in the "{{name}}" attribute. Use a Fluent message instead (CLAUDE.md rule 7, Code Standards F31).',
      fluentArgs:
        'Text written in code as a Fluent variable. Variables carry values (numbers, names, codes); words belong in the message itself (CLAUDE.md rule 7, Code Standards F31).',
      sink: 'Text written in code passed to "{{name}}", which shows it. Use a Fluent message instead (CLAUDE.md rule 7, Code Standards F31).',
      property:
        'Text written in code into "{{name}}". Use a Fluent message instead (CLAUDE.md rule 7, Code Standards F31).',
    },
    schema: [],
  },
  defaultOptions: [],
  create(context) {
    const services = ESLintUtils.getParserServices(context, true);
    const checker = services.program?.getTypeChecker();

    /**
     * True when TypeScript expects only fixed string values here, for example
     * `'on-primary' | 'on-light'`. Such a value is code. Without type information: false.
     * @param {Node} node
     * @returns {boolean}
     */
    const expectsStringChoice = (node) => {
      if (checker === undefined) return false;
      const tsNode = services.esTreeNodeToTSNodeMap.get(node);
      if (!ts.isExpression(tsNode)) return false;
      const type = checker.getContextualType(tsNode);
      if (type === undefined) return false;
      const parts = (type.isUnion() ? type.types : [type]).filter(
        (part) => (part.flags & (ts.TypeFlags.Undefined | ts.TypeFlags.Null)) === 0,
      );
      return parts.length > 0 && parts.every((part) => part.isStringLiteral());
    };

    return {
      JSXText(node) {
        const text = node.value.trim();
        if (text !== '') {
          context.report({ node, messageId: 'text', data: { text: text.slice(0, 40) } });
        }
      },
      JSXExpressionContainer(node) {
        const parent = node.parent;
        const isChild = parent.type === 'JSXElement' || parent.type === 'JSXFragment';
        if (!isChild || node.expression.type === 'JSXEmptyExpression') return;
        if (isWordString(node.expression, context.sourceCode.getScope(node))) {
          context.report({
            node,
            messageId: 'text',
            data: { text: context.sourceCode.getText(node).slice(0, 40) },
          });
        }
      },
      JSXAttribute(node) {
        const name = attributeName(node);
        if (CODE_ATTRIBUTES.has(name) || name.startsWith('data-') || node.value === null) return;
        const value =
          node.value.type === 'JSXExpressionContainer' ? node.value.expression : node.value;
        if (value.type === 'JSXEmptyExpression' || expectsStringChoice(value)) return;
        const scope = context.sourceCode.getScope(node);
        if (FLUENT_VARIABLE_PROPS.has(name) && value.type === 'ObjectExpression') {
          if (objectHasWord(value, scope)) {
            context.report({ node, messageId: 'fluentArgs' });
          }
          return;
        }
        if (isWordString(value, scope)) {
          context.report({ node, messageId: 'attribute', data: { name } });
        }
      },
      CallExpression(node) {
        const { callee } = node;
        const name =
          callee.type === 'Identifier'
            ? callee.name
            : callee.type === 'MemberExpression' && callee.property.type === 'Identifier'
              ? callee.property.name
              : '';
        const scope = context.sourceCode.getScope(node);
        // l10n.getString('id', { name: 'Hello world' })
        if (name === 'getString') {
          const args = node.arguments[1];
          if (args?.type === 'ObjectExpression' && objectHasWord(args, scope)) {
            context.report({ node: args, messageId: 'fluentArgs' });
          }
          return;
        }
        // showToast('success', 'Saved'), alert('Saved')
        const sink = TEXT_SINKS.get(name);
        if (sink !== undefined) {
          const shown = node.arguments[sink];
          if (shown !== undefined && shown.type !== 'SpreadElement' && isWordString(shown, scope)) {
            context.report({ node: shown, messageId: 'sink', data: { name } });
          }
          return;
        }
        // element.setAttribute('aria-label', 'Save')
        if (name === 'setAttribute') {
          const [attribute, value] = node.arguments;
          if (
            attribute?.type === 'Literal' &&
            typeof attribute.value === 'string' &&
            TEXT_ATTRIBUTE_NAMES.has(attribute.value) &&
            value !== undefined &&
            value.type !== 'SpreadElement' &&
            isWordString(value, scope)
          ) {
            context.report({ node, messageId: 'property', data: { name: attribute.value } });
          }
        }
      },
      AssignmentExpression(node) {
        // document.title = 'Mandi', element.textContent = 'Saved'
        const { left } = node;
        if (left.type !== 'MemberExpression' || left.property.type !== 'Identifier') return;
        const name = left.property.name;
        if (
          TEXT_PROPERTIES.has(name) &&
          isWordString(node.right, context.sourceCode.getScope(node))
        ) {
          context.report({ node, messageId: 'property', data: { name } });
        }
      },
    };
  },
});
