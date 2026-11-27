// The check behind `pnpm i18n:check`, part two (plan 81, NFR-URD-1, NFR-URD-22): every message ID
// that the code asks for exists in BOTH the Urdu and the English files. A missing ID shows as a raw
// ID on the screen or on a print (for example `print-partner-month-col-advances` in the header of
// a printed statement), and in Urdu mode that is English text.
//
// i18n-check-lib.mjs only compares the two languages with each other. It cannot see an ID that the
// code uses and that neither language has. This file reads the code:
//
// 1. The frontend (frontend/src, read with the TypeScript compiler):
//    - a string that is written in the place of an ID (`<Localized id="...">`, `getString('...')`,
//      `titleId: '...'`);
//    - a template such as `` `settings-check-${kind}-bad` ``: every key it can produce is worked
//      out from the types (a union of strings), through `.replaceAll('_', '-')` and small helper
//      functions, and each one must exist. A template whose keys cannot be worked out is an error
//      unless it is in ENUMERATED (the keys listed) or REVIEWED (with the reason). A helper that
//      builds `${prefix}-${value}` is read at each call, so its prefix is known.
// 2. The backend (backend/**/src, Rust): the IDs it sends to the screens and the print windows
//    (`title_id`, `label_id`, `Cell::Label { id }`, constants of the ids.rs files, any string that
//    starts like a family of messages the files have, with the list BACKEND_LITERALS_OK of the
//    strings that are not messages) and the
//    templates it builds with `format!` (`report-{code}-title`, `print-partner-month-col-{key}` ...),
//    with their keys read from the Rust code itself.
//
// Example: `print-partner-month-col-advances` is built in statement_month_print.rs from the column
// key "advances". The check finds the key and fails because no Fluent file has that message.

import { existsSync, readFileSync, readdirSync, statSync } from 'node:fs';
import { createRequire } from 'node:module';
import path from 'node:path';
import { Message, Term, parse } from '@fluent/syntax';

const LANGUAGES = ['ur', 'en'];

/** An ID as written in the Fluent files: words joined by `-`, with `_` allowed inside a word. */
const ID_SHAPE = /^[a-z][a-z0-9]*(?:-[a-z0-9_]+)+$/;
/** The start of an ID template: `settings-check-` before the first `${`. */
const HEAD_SHAPE = /^[a-z][a-z0-9]*(?:-[a-z0-9_]+)*-?$/;

/**
 * Templates in the frontend that look like message IDs and are not, or whose missing messages are
 * handled by the code (a fallback word), each with the reason. The key is the template as written
 * (without the file), so the list does not move when a line moves.
 */
export const REVIEWED = new Map([
  // Not message IDs: an element id of the page, a React key, a file name, a test id.
  ['`mandi-support-${stamp}.txt`', 'a file name'],
  ['`gallery-${language}`', 'a test id of the dev gallery'],
  ['`goods-lines-heading-${kind}`', 'an element id'],
  ['`new-${String(search.party.id)}`', 'a React key'],
  ['`line-details-${row.rowId}`', 'an element id'],
  ['`chukta-${String(hit.chukta_id ?? hit.id)}`', 'the React key of a picker row'],
  ['`kharch-${part}`', 'a file name of the Excel export'],
  ['`${accountScope(draft)}-${draft.type}`', 'a React key of the account picker'],
  ['`${charge.side}-${charge.charge_type}`', 'the key of a table row'],
  ['`analytics-${entry.id}`', 'a file name of the Excel export'],
  // A key the backend sends and the app may not know: the caller gives a fallback word
  // (shared/edit/value-text.ts: getString(id, null, fallback)), so no raw ID is ever shown.
  ["`edit-field-${key.replaceAll('_', '-')}`", 'the caller gives a fallback word'],
]);

/**
 * Templates whose keys are numbers or other values the types cannot list. The keys are listed here
 * and every one must exist. The key is the template as written.
 */
const MONTHS = Array.from({ length: 12 }, (_, index) => String(index + 1));
export const ENUMERATED = new Map([
  ['`report-month-${String(parts.month)}`', ['report-month-', MONTHS]],
  ['`report-month-${String(Number(month))}`', ['report-month-', MONTHS]],
  ['`report-month-${String(month)}`', ['report-month-', MONTHS]],
  // A cashbook row's kind is the posting type of its journal entry: the types the backend lists in
  // POSTING_TYPES (reports/ids.rs), without the goods arrived that posts nothing.
  ['`cashbook-kind-${kindId(row.posting_type)}`', ['cashbook-kind-', 'posting_types']],
  // The series of a voucher a statement line came from: the values of VOUCHER_SERIES and KIND_SERIES
  // in partners/statement-text.ts.
  [
    '`vouchers-number-${series}`',
    [
      'vouchers-number-',
      [
        'shop-expense',
        'capital-in',
        'capital-out',
        'personal-expense',
        'personal-return',
        'khata-out',
        'khata-in',
        'money-lent',
        'money-lent-return',
        'general',
      ],
    ],
  ],
]);

/** Strings in the frontend that look like IDs and are not messages (an element id, a key). */
export const LITERALS_OK = new Set([]);

// ------------------------------------------------------------------------------------------------
// The IDs the Fluent files define
// ------------------------------------------------------------------------------------------------

/**
 * Every message ID of each language, from all of its files.
 * @param {string} i18nDir
 * @returns {Record<string, Set<string>>}
 */
export function definedIds(i18nDir) {
  /** @type {Record<string, Set<string>>} */
  const defined = {};
  for (const language of LANGUAGES) {
    const ids = new Set();
    const dir = path.join(i18nDir, language);
    for (const name of readdirSync(dir).filter((file) => file.endsWith('.ftl'))) {
      const resource = parse(readFileSync(path.join(dir, name), 'utf8'), { withSpans: false });
      for (const entry of resource.body) {
        if (entry instanceof Message) ids.add(entry.id.name);
        else if (entry instanceof Term) ids.add(`-${entry.id.name}`);
      }
    }
    defined[language] = ids;
  }
  return defined;
}

/** The languages in which `id` is not defined. */
function missingIn(defined, id) {
  return LANGUAGES.filter((language) => !defined[language]?.has(id));
}

/** True when some defined message starts with `head` (an ID family such as `print-partner-month-`). */
function familyOf(defined, head) {
  const cut = head.endsWith('-') ? head : head.slice(0, head.lastIndexOf('-') + 1);
  if (cut === '') return false;
  for (const language of LANGUAGES) {
    for (const id of defined[language] ?? []) {
      if (id.startsWith(cut)) return true;
    }
  }
  return false;
}

// ------------------------------------------------------------------------------------------------
// The frontend
// ------------------------------------------------------------------------------------------------

function listFiles(dir, accept, out = []) {
  for (const name of readdirSync(dir)) {
    if (name === 'node_modules' || name === 'target' || name === '.git') continue;
    const file = path.join(dir, name);
    if (statSync(file).isDirectory()) listFiles(file, accept, out);
    else if (accept(file)) out.push(file);
  }
  return out;
}

function escapeRegExp(text) {
  return text.replace(/[.*+?^${}()|[\]\\]/g, '\\$&');
}

/** The names of the functions whose first argument is a message ID. */
const ID_CALLEES = new Set(['getString', 'text', 'say', 't', 'messageText', 'string']);

/**
 * Lists the Rust code holds that the frontend's templates depend on.
 * @param {string} root
 * @returns {Record<string, string[]>}
 */
function backendLists(root) {
  const lists = { posting_types: [] };
  try {
    const idsRs = readFileSync(
      path.join(root, 'backend/crates/reports/src/reports/ids.rs'),
      'utf8',
    );
    const block = /const POSTING_TYPES: &\[&str\] = &\[([^\]]*)\];/.exec(idsRs)?.[1] ?? '';
    lists.posting_types = [...block.matchAll(/"([a-z_]+)"/g)]
      .map((match) => match[1] ?? '')
      // The goods arrived is a document with no posting: no row of the cash book has it.
      .filter((type) => type !== 'garden_arrival')
      .map((type) => type.replaceAll('_', '-'));
  } catch {
    // A small repository made by a test has no backend: the list stays empty.
  }
  return lists;
}

/**
 * Reads frontend/src with the TypeScript compiler and checks every ID and every ID template.
 * @param {string} root the repository root
 * @param {Record<string, Set<string>>} defined
 * @param {{ typescriptFrom?: string }} [options] where to find the `typescript` package
 * @returns {string[]} one line per problem
 */
export function checkFrontendIds(root, defined, options = {}) {
  // The TypeScript compiler of the real frontend, also when `root` is a small folder made by a test.
  // The keys of a template come from the types, and the types of the backend come from the generated
  // files. With that folder empty (pnpm gen:types is running or failed) every union is lost and the
  // check would list every template: say the one real reason instead.
  const bindingsDir = path.join(root, 'frontend', 'src', 'bindings');
  if (
    existsSync(bindingsDir) &&
    readdirSync(bindingsDir).filter((name) => name.endsWith('.ts')).length === 0
  ) {
    return [
      'frontend/src/bindings is empty: the keys of the ID templates cannot be worked out without the generated types. Run pnpm gen:types, then pnpm i18n:check again.',
    ];
  }
  const require = createRequire(
    options.typescriptFrom ?? path.join(root, 'frontend', 'package.json'),
  );
  const ts = require('typescript');
  const configPath = path.join(root, 'frontend', 'tsconfig.json');
  const config = ts.readConfigFile(configPath, ts.sys.readFile);
  const parsed = ts.parseJsonConfigFileContent(config.config, ts.sys, path.dirname(configPath));
  const program = ts.createProgram(parsed.fileNames, parsed.options);
  const checker = program.getTypeChecker();
  const srcDir = path.join(root, 'frontend', 'src') + path.sep;
  const skip = ['bindings', 'i18n', path.join('app', 'dev')].map(
    (part) => srcDir + part + path.sep,
  );

  const MAX_KEYS = 600;

  /** The values of a type when it is a union of string (or number) literals, else null. */
  function valuesOfType(type) {
    if (type.isUnion()) {
      const out = [];
      for (const member of type.types) {
        const values = valuesOfType(member);
        if (values === null) return null;
        out.push(...values);
      }
      return out;
    }
    if (type.isStringLiteral()) return [type.value];
    if (type.isNumberLiteral()) return [String(type.value)];
    if (type.flags & ts.TypeFlags.BooleanLiteral) return [checker.typeToString(type)];
    return null;
  }

  const product = (left, right) => {
    const out = [];
    for (const a of left) for (const b of right) out.push(a + b);
    return out.length > MAX_KEYS ? null : out;
  };

  /**
   * The strings an expression can have, or null when that cannot be worked out. `env` holds the
   * parameters of a helper function that is being read at one call.
   */
  function values(node, env, depth = 0) {
    if (depth > 8) return null;
    if (
      ts.isParenthesizedExpression(node) ||
      ts.isAsExpression(node) ||
      ts.isNonNullExpression(node) ||
      ts.isSatisfiesExpression(node)
    ) {
      return values(node.expression, env, depth + 1);
    }
    if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) return [node.text];
    if (ts.isNumericLiteral(node)) return [node.text];
    if (ts.isTemplateExpression(node)) {
      let acc = [node.head.text];
      for (const span of node.templateSpans) {
        const part = values(span.expression, env, depth + 1);
        if (part === null) return null;
        acc = product(acc, part);
        if (acc === null) return null;
        acc = acc.map((text) => text + span.literal.text);
      }
      return acc;
    }
    if (ts.isConditionalExpression(node)) {
      const a = values(node.whenTrue, env, depth + 1);
      const b = values(node.whenFalse, env, depth + 1);
      return a === null || b === null ? null : [...a, ...b];
    }
    if (ts.isBinaryExpression(node)) {
      const kind = node.operatorToken.kind;
      const nullish =
        kind === ts.SyntaxKind.QuestionQuestionToken || kind === ts.SyntaxKind.BarBarToken;
      // `side ?? 'buyer'`: the left side counts without its undefined and null.
      const a = nullish
        ? (valuesOfType(checker.getNonNullableType(checker.getTypeAtLocation(node.left))) ??
          values(node.left, env, depth + 1))
        : values(node.left, env, depth + 1);
      const b = values(node.right, env, depth + 1);
      if (a === null || b === null) return null;
      if (kind === ts.SyntaxKind.PlusToken) return product(a, b);
      if (nullish) return [...a, ...b];
      return null;
    }
    if (ts.isIdentifier(node)) {
      const symbol = checker.getSymbolAtLocation(node);
      if (symbol !== undefined && env.has(symbol)) return env.get(symbol);
      // A `const` of the same function: read what it was made from.
      const declaration = symbol?.valueDeclaration;
      if (
        declaration !== undefined &&
        ts.isVariableDeclaration(declaration) &&
        declaration.initializer !== undefined &&
        (declaration.parent.flags & ts.NodeFlags.Const) !== 0
      ) {
        const made = values(declaration.initializer, env, depth + 1);
        if (made !== null) {
          // Where the code has narrowed the variable (`where === 'cloud' ? ... : <here>`), only the
          // values the checker still allows at this place count.
          const here = valuesOfType(checker.getTypeAtLocation(node));
          const kept = here === null ? made : made.filter((value) => here.includes(value));
          return kept.length > 0 ? kept : made;
        }
      }
    }
    if (ts.isCallExpression(node)) {
      const callee = node.expression;
      if (ts.isPropertyAccessExpression(callee)) {
        const method = callee.name.text;
        const receiver = values(callee.expression, env, depth + 1);
        if (receiver === null) return null;
        if (method === 'toLowerCase') return receiver.map((text) => text.toLowerCase());
        if (method === 'toUpperCase') return receiver.map((text) => text.toUpperCase());
        if (method === 'replaceAll' || method === 'replace') {
          const find = node.arguments[0];
          const put = node.arguments[1];
          if (find === undefined || put === undefined || !ts.isStringLiteral(put)) return null;
          let pattern = null;
          if (ts.isStringLiteral(find)) pattern = new RegExp(escapeRegExp(find.text), 'g');
          else if (ts.isRegularExpressionLiteral(find)) {
            const close = find.text.lastIndexOf('/');
            pattern = new RegExp(find.text.slice(1, close), find.text.slice(close + 1));
          }
          if (pattern === null) return null;
          return receiver.map((text) => text.replace(pattern, put.text));
        }
        return null;
      }
      if (ts.isIdentifier(callee) && callee.text === 'String' && node.arguments[0] !== undefined) {
        return values(node.arguments[0], env, depth + 1);
      }
      // A helper of the project that only returns one expression: read it with the call's arguments.
      const declaration = checker.getResolvedSignature(node)?.declaration;
      if (
        declaration !== undefined &&
        (ts.isFunctionDeclaration(declaration) ||
          ts.isArrowFunction(declaration) ||
          ts.isFunctionExpression(declaration)) &&
        declaration.body !== undefined
      ) {
        const body = declaration.body;
        let returned = null;
        if (!ts.isBlock(body)) returned = body;
        else if (body.statements.length === 1 && ts.isReturnStatement(body.statements[0])) {
          returned = body.statements[0].expression ?? null;
        }
        if (returned !== null) {
          const inner = new Map(env);
          declaration.parameters.forEach((parameter, index) => {
            const symbol = checker.getSymbolAtLocation(parameter.name);
            const argument = node.arguments[index];
            if (symbol === undefined || argument === undefined) return;
            const set = values(argument, env, depth + 1);
            if (set !== null) inner.set(symbol, set);
          });
          const result = values(returned, inner, depth + 1);
          if (result !== null) return result;
        }
      }
    }
    return valuesOfType(checker.getTypeAtLocation(node));
  }

  const lineOf = (file, node) =>
    `${path.relative(root, file.fileName)}:${file.getLineAndCharacterOfPosition(node.getStart()).line + 1}`;

  /** True when the node is written in the place of a message ID. */
  function inIdPlace(node) {
    const parent = node.parent;
    if (parent === undefined) return false;
    if (ts.isJsxAttribute(parent) && parent.name.text === 'id') {
      const tag = parent.parent?.parent;
      const name =
        tag && (ts.isJsxOpeningElement(tag) || ts.isJsxSelfClosingElement(tag))
          ? tag.tagName.getText()
          : '';
      return name === 'Localized';
    }
    if (ts.isJsxExpression(parent) && ts.isJsxAttribute(parent.parent)) {
      return inIdPlace(parent);
    }
    if (ts.isCallExpression(parent) && parent.arguments[0] === node) {
      const callee = parent.expression;
      const name = ts.isIdentifier(callee)
        ? callee.text
        : ts.isPropertyAccessExpression(callee)
          ? callee.name.text
          : '';
      return ID_CALLEES.has(name);
    }
    if (ts.isPropertyAssignment(parent) && parent.initializer === node) {
      // `titleId`, `labelId`, `headerId`: not a plain `id`, which is mostly a route or a key.
      return /(Id|Ids)$/.test(parent.name.getText());
    }
    return false;
  }

  /** @type {string[]} */
  const errors = [];
  const seen = new Set();
  const report = (line, text) => {
    const key = `${line}|${text}`;
    if (!seen.has(key)) {
      seen.add(key);
      errors.push(`${line}: ${text}`);
    }
  };

  const sources = program
    .getSourceFiles()
    .filter(
      (file) =>
        !file.isDeclarationFile &&
        file.fileName.startsWith(srcDir) &&
        !skip.some((prefix) => file.fileName.startsWith(prefix)),
    );

  // Every call of a function of the project, by the function's declaration, so a template inside a
  // helper (`stateMessageId(state: string)`) is read with the argument of each call.
  /** @type {Map<import('typescript').Node, import('typescript').CallExpression[]>} */
  const callsOf = new Map();
  for (const file of program.getSourceFiles()) {
    if (file.isDeclarationFile || !file.fileName.startsWith(srcDir)) continue;
    const collect = (node) => {
      if (ts.isCallExpression(node)) {
        const declaration = checker.getResolvedSignature(node)?.declaration;
        if (declaration !== undefined) {
          const list = callsOf.get(declaration) ?? [];
          list.push(node);
          callsOf.set(declaration, list);
        }
      }
      ts.forEachChild(node, collect);
    };
    collect(file);
  }

  const isFunction = (node) =>
    ts.isFunctionDeclaration(node) || ts.isArrowFunction(node) || ts.isFunctionExpression(node);

  /** The function a node is written in, or null. */
  function enclosingFunction(node) {
    for (let up = node.parent; up !== undefined; up = up.parent) if (isFunction(up)) return up;
    return null;
  }

  /**
   * What an `includes` test in front of the template says about a parameter:
   * `(KNOWN as readonly string[]).includes(code) ? `audit-${code}` : null` gives `code` the
   * values of KNOWN.
   */
  function guardEnv(node) {
    const env = new Map();
    for (let child = node, up = node.parent; up !== undefined; child = up, up = up.parent) {
      let test = null;
      if (ts.isConditionalExpression(up) && up.whenTrue === child) test = up.condition;
      else if (ts.isIfStatement(up) && up.thenStatement === child) test = up.expression;
      if (
        test !== null &&
        ts.isCallExpression(test) &&
        ts.isPropertyAccessExpression(test.expression) &&
        test.expression.name.text === 'includes' &&
        test.arguments[0] !== undefined &&
        ts.isIdentifier(test.arguments[0])
      ) {
        let list = test.expression.expression;
        while (ts.isParenthesizedExpression(list) || ts.isAsExpression(list)) {
          list = list.expression;
        }
        const element = checker.getTypeAtLocation(list).getNumberIndexType();
        const symbol = checker.getSymbolAtLocation(test.arguments[0]);
        const found = element === undefined ? null : valuesOfType(element);
        if (found !== null && symbol !== undefined) env.set(symbol, found);
      }
    }
    return env;
  }

  /**
   * True when the template is not a plain ID the screen shows: the code asks `exists` first or
   * gives a fallback, so a missing message is expected and handled.
   */
  function isGuarded(node) {
    const parent = node.parent;
    if (ts.isCallExpression(parent)) {
      const callee = parent.expression;
      const name = ts.isPropertyAccessExpression(callee)
        ? callee.name.text
        : ts.isIdentifier(callee)
          ? callee.text
          : '';
      if (['exists', 'has', 'hasMessage'].includes(name)) return true;
      if (name === 'getString' && parent.arguments.length >= 3) return true;
    }
    if (ts.isVariableDeclaration(parent) && ts.isIdentifier(parent.name)) {
      const scope = enclosingFunction(node);
      const body = (scope ?? node.getSourceFile()).getText();
      return new RegExp(`\\.(exists|has|hasMessage)\\(\\s*${parent.name.text}\\s*\\)`).test(body);
    }
    return false;
  }

  /** The keys a template can make, or { unresolved } with the place that could not be worked out. */
  function keysOf(node) {
    const listed = ENUMERATED.get(node.getText());
    if (listed !== undefined) {
      const parts =
        typeof listed[1] === 'string' ? (backendLists(root)[listed[1]] ?? []) : listed[1];
      return { keys: parts.map((part) => listed[0] + part) };
    }
    const guard = guardEnv(node);
    const direct = values(node, guard);
    if (direct !== null) return { keys: direct };
    const scope = enclosingFunction(node);
    if (scope === null) return { unresolved: lineOf(node.getSourceFile(), node) };
    const calls = callsOf.get(scope) ?? [];
    if (calls.length === 0) return { unresolved: lineOf(node.getSourceFile(), node) };
    const keys = [];
    for (const call of calls) {
      const env = new Map(guard);
      scope.parameters.forEach((parameter, index) => {
        const symbol = checker.getSymbolAtLocation(parameter.name);
        const argument = call.arguments[index];
        if (symbol === undefined || argument === undefined) return;
        const found = values(argument, new Map());
        if (found !== null) env.set(symbol, found);
      });
      const made = values(node, env);
      if (made === null) return { unresolved: lineOf(call.getSourceFile(), call) };
      keys.push(...made);
    }
    return { keys };
  }

  for (const file of sources) {
    const visit = (node) => {
      if (ts.isTemplateExpression(node)) {
        const head = node.head.text;
        const source = node.getText();
        const idLike = HEAD_SHAPE.test(head) && head.includes('-');
        // `${prefix}-${value}` in a helper (`choice('masters-used-in', ['receipt', ...])`): the prefix
        // comes from the call. Read at each call; only keys that start like a family of messages and
        // are not defined are told (a key that is not an ID is none of our business).
        if (!idLike && head === '' && !REVIEWED.has(source) && !isGuarded(node)) {
          const made = keysOf(node);
          if (made.keys !== undefined) {
            for (const key of new Set(made.keys)) {
              if (
                ID_SHAPE.test(key) &&
                familyOf(defined, key.slice(0, key.lastIndexOf('-') + 1)) &&
                missingIn(defined, key).length > 0
              ) {
                report(
                  lineOf(file, node),
                  `${source} can be "${key}", which is missing in ${missingIn(defined, key).join(' and ')}`,
                );
              }
            }
          }
        }
        if (
          idLike &&
          (familyOf(defined, head) || inIdPlace(node)) &&
          !REVIEWED.has(source) &&
          !isGuarded(node)
        ) {
          const made = keysOf(node);
          if (made.keys === undefined) {
            report(
              lineOf(file, node),
              `${source}: the keys of this ID template cannot be worked out (seen at ${made.unresolved}); give the value a union type, or add the template to REVIEWED or ENUMERATED`,
            );
          } else {
            for (const key of new Set(made.keys)) {
              const gone = missingIn(defined, key);
              if (gone.length > 0) {
                report(
                  lineOf(file, node),
                  `${source} can be "${key}", which is missing in ${gone.join(' and ')}`,
                );
              }
            }
          }
        }
      } else if (ts.isStringLiteral(node) || ts.isNoSubstitutionTemplateLiteral(node)) {
        const text = node.text;
        // Only a string in the place of an ID: a class name or a kind that looks like an ID is not one.
        if (
          ID_SHAPE.test(text) &&
          !LITERALS_OK.has(text) &&
          inIdPlace(node) &&
          missingIn(defined, text).length > 0
        ) {
          report(
            lineOf(file, node),
            `"${text}" is missing in ${missingIn(defined, text).join(' and ')}`,
          );
        }
      }
      ts.forEachChild(node, visit);
    };
    visit(file);
  }
  return errors;
}

// ------------------------------------------------------------------------------------------------
// The backend
// ------------------------------------------------------------------------------------------------

/**
 * Strings in the Rust code that look like message IDs and are not (a kind of print, an event name,
 * a key name, a file stem ...), each with the reason. Everything else that starts like a message of
 * the Fluent files must be one.
 */
export const BACKEND_LITERALS_OK = new Map([
  ['advances-open', 'a kind of print (print_report.rs)'],
  ['cash-sales-day', 'a kind of print'],
  ['cash-sales-month', 'a kind of print'],
  ['lent-open', 'a kind of print'],
  ['seller-bills', 'a kind of print'],
  ['seller-dues', 'a kind of print'],
  ['statement-rows', 'a kind of print'],
  ['total-dues', 'a kind of print, and a source name of the assistant'],
  ['seller-bill', 'the file stem of a PDF (core/printing.rs)'],
  ['backup-exit', 'the name of an event'],
  ['backup-restore-progress', 'the name of an event'],
  ['export-progress', 'the name of an event'],
  ['audit-chain-key', 'the name of an entry in the Credential Manager'],
  ['backup-key', 'the name of an entry in the Credential Manager'],
  ['day-auto-close', 'the name of a job'],
  ['seller-item', 'a name of the assistant understanding of a question'],
  ['seller-own', 'a name of the assistant understanding of a question'],
  ['total-worth', 'a kind of print (print_report.rs)'],
  ['dead-amount', 'a source key of Total Worth, part of the template total-worth-source-{}'],
]);

/** The words after the first dash of every defined ID, as the families the Fluent files have. */
function prefixesOf(defined) {
  const prefixes = new Set();
  for (const language of LANGUAGES) {
    for (const id of defined[language] ?? []) {
      const first = /^([a-z0-9]+)-/.exec(id);
      if (first?.[1] !== undefined) prefixes.add(first[1]);
    }
  }
  return prefixes;
}

/** Rust source files of the backend that are not tests, with the text before `#[cfg(test)]`. */
function backendSources(root) {
  return listFiles(path.join(root, 'backend'), (file) => {
    if (!file.endsWith('.rs')) return false;
    const parts = file.split(path.sep);
    return !parts.includes('tests') && !parts.includes('benches') && !parts.includes('examples');
  }).map((file) => {
    const full = readFileSync(file, 'utf8');
    const cut = full.indexOf('#[cfg(test)]');
    return { file, rel: path.relative(root, file), text: cut >= 0 ? full.slice(0, cut) : full };
  });
}

/**
 * Every ID written as a string in the Rust code (a `title_id`, a label, a column heading, a
 * constant of ids.rs ...) exists in both languages.
 * @param {string} root the repository root
 * @param {Record<string, Set<string>>} defined
 * @returns {string[]}
 */
export function checkBackendIds(root, defined) {
  const prefixes = prefixesOf(defined);
  /** @type {string[]} */
  const errors = [];
  for (const { rel, text } of backendSources(root)) {
    text.split('\n').forEach((line, index) => {
      if (line.trimStart().startsWith('//')) return;
      for (const match of line.matchAll(/"([a-z][a-z0-9]*(?:-[a-z0-9_]+)+)"/g)) {
        const id = match[1];
        if (id === undefined || BACKEND_LITERALS_OK.has(id)) continue;
        // An ID in the place of an ID (`title_id: "..."`, `Cell::Label { id: "..." }`), whatever its
        // first word is, or any string that starts like a family of messages the files have.
        const place = /(_id|\bid)\s*:\s*"|Label\s*\{\s*id/.test(line);
        if (!place && !prefixes.has(id.split('-')[0] ?? '')) continue;
        const gone = missingIn(defined, id);
        if (gone.length > 0) {
          errors.push(
            `${rel}:${index + 1}: "${id}" is missing in ${gone.join(' and ')} (if it is not a message ID, add it to BACKEND_LITERALS_OK with the reason)`,
          );
        }
      }
    });
  }
  return errors;
}

/** The series of the voucher a statement line came from (`vouchers-number-<series>`). */
const VOUCHER_SERIES = [
  'receipt',
  'payment',
  'advance',
  'shop-expense',
  'personal-expense',
  'personal-return',
  'money-lent',
  'money-lent-return',
  'capital-in',
  'capital-out',
  'khata-out',
  'khata-in',
  'general',
];

/**
 * The templates the Rust code builds IDs with: `format!("report-{}-title", code)`. Each is listed
 * here with the keys it can make, read from the Rust code itself (the report codes, the view
 * letters, the posting types, the column keys). A new template that is not listed fails the check.
 * @param {string} root the repository root
 * @param {Record<string, Set<string>>} defined
 * @returns {string[]}
 */
export function checkBackendTemplates(root, defined) {
  const read = (relative) => readFileSync(path.join(root, relative), 'utf8');
  const all = (text, pattern) => [...text.matchAll(pattern)].map((match) => match[1]);

  const reportCodes = all(
    read('backend/crates/reports/src/reports/model.rs'),
    /ReportId::(R\d+) => "R\d+"/g,
  );
  const viewLetters = all(
    read('backend/crates/reports/src/analytics/model.rs'),
    /AnalyticsView::([A-Z]) => "[A-Z]"/g,
  );
  const idsRs = read('backend/crates/reports/src/reports/ids.rs');
  const postingBlock = /const POSTING_TYPES: &\[&str\] = &\[([^\]]*)\];/.exec(idsRs)?.[1] ?? '';
  const postingTypes = all(postingBlock, /"([a-z_]+)"/g);
  const personTypes = all(
    read('backend/crates/core/src/links.rs'),
    /PersonType::[A-Za-z]+ => "([a-z_]+)"/g,
  );
  const monthColumns = all(
    read('backend/crates/reports/src/reports/statement_month_print.rs'),
    /\bcolumn\(\s*"([a-z_]+)"/g,
  );

  const worthSources = all(
    read('backend/crates/reports/src/dashboard/total_worth.rs'),
    /TotalWorthSource::[A-Za-z]+ => "([a-z-]+)"/g,
  );

  const problems = [];
  if (worthSources.length < 15)
    problems.push('could not read the sources of dashboard/total_worth.rs');
  if (reportCodes.length < 30)
    problems.push('could not read the report codes from reports/model.rs');
  if (viewLetters.length < 12)
    problems.push('could not read the view letters from analytics/model.rs');
  if (postingTypes.length < 20) problems.push('could not read POSTING_TYPES from reports/ids.rs');
  if (personTypes.length < 4) problems.push('could not read the person types from core/links.rs');
  if (monthColumns.length < 8)
    problems.push('could not read the columns of statement_month_print.rs');
  if (problems.length > 0) return problems.map((text) => `i18n ids: ${text}`);

  /** The keys of each template, by the template with its placeholders as `{}`. */
  const known = new Map([
    // R38 has the title of its own screen (ids.rs title_id).
    [
      'report-{}-title',
      reportCodes.map((code) =>
        code === 'R38' ? 'dead-report-title' : `report-${code.toLowerCase()}-title`,
      ),
    ],
    [
      'analytics-view-{}-title',
      viewLetters.map((letter) => `analytics-view-${letter.toLowerCase()}-title`),
    ],
    ['report-month-{}', MONTHS.map((month) => `report-month-${month}`)],
    [
      'report-doc-{}',
      [
        ...postingTypes.map((type) => `report-doc-${type.replaceAll('_', '-')}`),
        'report-doc-other',
      ],
    ],
    ['total-worth-source-{}', worthSources.map((source) => `total-worth-source-${source}`)],
    [
      'print-partner-month-col-{}',
      monthColumns.map((key) => `print-partner-month-col-${key.replaceAll('_', '-')}`),
    ],
    ['vouchers-number-{}', VOUCHER_SERIES.map((series) => `vouchers-number-${series}`)],
    [
      'cashbook-person-type-{}',
      personTypes.map((type) => `cashbook-person-type-${type.replaceAll('_', '-')}`),
    ],
  ]);
  /** Templates that are not message IDs (a file name, a window label, a job name). */
  const notMessages = new Set([
    'c-{}-{}',
    'pre-migration-{}-{}-{}.db',
    'snapshot-{}.db',
    'verify-{}.db',
    'verify-{}',
    'mandi-local-{}.{}',
    'mandi-{}.{}',
    'lot-{}',
    's-{}-{}',
    'person-{}',
    'print-ready-{}',
    'print-error-{}',
    'print-{}',
    'report-{}',
    // File names of the exports and the prints, and the random token of a fix-and-close session.
    'mandi-support-{}.txt',
    'report-{}{}-{}-{}.pdf',
    'analytics-{}-{}-{}.pdf',
    'bikri-list-{}-{}.pdf',
    'party-statement-{}-{}-{}.pdf',
    'fix-{}',
  ]);

  const errors = [];
  const seen = new Set();
  for (const { rel, text } of backendSources(root)) {
    for (const match of text.matchAll(/format!\(\s*"([a-z][a-z0-9]*-[^"]*\{[^"]*)"/g)) {
      const template = (match[1] ?? '').replace(/\{[^}]*\}/g, '{}');
      if (notMessages.has(template) || seen.has(template)) continue;
      seen.add(template);
      const keys = known.get(template);
      if (keys === undefined) {
        errors.push(
          `${rel}: the ID template "${template}" is not known to scripts/i18n-ids-lib.mjs: list its keys in checkBackendTemplates (or in notMessages if it is not a message)`,
        );
        continue;
      }
      for (const key of keys) {
        const gone = missingIn(defined, key);
        if (gone.length > 0)
          errors.push(
            `${rel}: the template "${template}" makes "${key}", which is missing in ${gone.join(' and ')}`,
          );
      }
    }
  }
  // A listed template that no longer exists in the code is a stale entry: say so, so the list stays true.
  for (const template of known.keys()) {
    if (!seen.has(template))
      errors.push(
        `i18n ids: the template "${template}" is listed in checkBackendTemplates and is not in the Rust code any more`,
      );
  }
  return errors;
}

/**
 * Part two of `pnpm i18n:check`: every ID the code asks for is defined in both languages.
 * @param {string} root the repository root
 * @returns {string[]}
 */
export function checkMessageIds(root) {
  const defined = definedIds(path.join(root, 'frontend', 'src', 'i18n'));
  return [
    ...checkFrontendIds(root, defined),
    ...checkBackendIds(root, defined),
    ...checkBackendTemplates(root, defined),
  ];
}
