// The local lint rules that block CLAUDE.md rules 1, 7 and 8 (plan 01 T3.4, D1-A4).

import { RuleTester } from '@typescript-eslint/rule-tester';
import path from 'node:path';
import { afterAll, describe, it } from 'vitest';
import noJsxLiterals from '../../eslint-rules/no-jsx-literals.js';
import noPaisaArithmetic from '../../eslint-rules/no-paisa-arithmetic.js';
import noPhysicalDirection from '../../eslint-rules/no-physical-direction.js';
import noRawDesignValues from '../../eslint-rules/no-raw-design-values.js';

RuleTester.afterAll = afterAll;
RuleTester.describe = describe;
RuleTester.it = it;
RuleTester.itOnly = it.only;

const tsconfigRootDir = path.resolve(import.meta.dirname, 'fixtures');

const ruleTester = new RuleTester({
  languageOptions: {
    parserOptions: {
      ecmaFeatures: { jsx: true },
      // A small project with empty file.ts and react.tsx; RuleTester puts each test's code into them.
      project: './tsconfig.json',
      tsconfigRootDir,
    },
  },
});

// The same opaque Paisa as src/shared/format/money.ts.
const PAISA =
  "declare const paisaBrand: unique symbol;\ninterface Paisa { readonly [paisaBrand]: 'Paisa' }\ndeclare const a: Paisa;\ndeclare const b: Paisa;\n";

describe('mandi/no-paisa-arithmetic (rule 1)', () => {
  ruleTester.run('no-paisa-arithmetic', noPaisaArithmetic, {
    valid: [
      { code: `${PAISA}const sorted = a < b; const same = a === b;` },
      {
        // Passing Paisa on to a formatter is the allowed path.
        code: `${PAISA}declare function formatPaisa(value: Paisa): string; const text = formatPaisa(a);`,
      },
      { code: 'declare const qty: number; const next = qty + 1; Math.round(qty);' },
      { code: `${PAISA}const list: Paisa[] = [a, b]; list.sort((x, y) => (x < y ? -1 : 1));` },
      { code: `${PAISA}const same = a as Paisa; const maybe = b as Paisa | null;` },
      { code: 'declare const qty: number; const text = String(qty); const n = Number(`${qty}`);' },
    ],
    invalid: [
      {
        code: `${PAISA}const total = a + b;`,
        errors: [{ messageId: 'operator', data: { operator: '+' } }],
      },
      {
        code: `${PAISA}const c = a * 6;`,
        errors: [{ messageId: 'operator', data: { operator: '*' } }],
      },
      {
        code: `${PAISA}const c = 100 / a;`,
        errors: [{ messageId: 'operator', data: { operator: '/' } }],
      },
      {
        code: `${PAISA}const c = a % 100;`,
        errors: [{ messageId: 'operator', data: { operator: '%' } }],
      },
      {
        code: `${PAISA}const c = a - b;`,
        errors: [{ messageId: 'operator', data: { operator: '-' } }],
      },
      {
        code: `${PAISA}let t = a; t += b;`,
        errors: [{ messageId: 'operator', data: { operator: '+=' } }],
      },
      {
        code: `${PAISA}let t = a; t++;`,
        errors: [{ messageId: 'operator', data: { operator: '++' } }],
      },
      {
        code: `${PAISA}const c = -a;`,
        errors: [{ messageId: 'operator', data: { operator: '-' } }],
      },
      {
        code: `${PAISA}const c = Math.round(a);`,
        errors: [{ messageId: 'call', data: { name: 'Math.round' } }],
      },
      {
        code: `${PAISA}const c = Math.floor(a);`,
        errors: [{ messageId: 'call', data: { name: 'Math.floor' } }],
      },
      {
        code: `${PAISA}const c = a.toFixed(2);`,
        errors: [{ messageId: 'call', data: { name: 'toFixed' } }],
      },
      {
        code: `${PAISA}const c = parseFloat(a);`,
        errors: [{ messageId: 'call', data: { name: 'parseFloat' } }],
      },
      // Bypasses found by robustness (findings 01, item 2): each fails at the conversion.
      {
        code: `${PAISA}const c = Number(a) + 1;`,
        errors: [{ messageId: 'call', data: { name: 'Number' } }],
      },
      { code: `${PAISA}const c = (a as number) * 2;`, errors: [{ messageId: 'cast' }] },
      { code: `${PAISA}const c = a as unknown as number;`, errors: [{ messageId: 'cast' }] },
      {
        code: `${PAISA}const c = parseFloat(String(a));`,
        errors: [{ messageId: 'call', data: { name: 'String' } }],
      },
      {
        code: `${PAISA}const c = parseInt(a, 10);`,
        errors: [{ messageId: 'call', data: { name: 'parseInt' } }],
      },
      {
        code: `${PAISA}const c = Number.parseFloat(a);`,
        errors: [{ messageId: 'call', data: { name: 'Number.parseFloat' } }],
      },
      {
        code: `${PAISA}const c = a.toString();`,
        errors: [{ messageId: 'call', data: { name: 'toString' } }],
      },
      {
        code: `${PAISA}const c = a.valueOf() + 1;`,
        errors: [{ messageId: 'call', data: { name: 'valueOf' } }],
      },
      {
        code: `${PAISA}const c = \`\${a}\`;`,
        errors: [{ messageId: 'call', data: { name: 'template string' } }],
      },
      {
        code: `${PAISA}const c = <number>a;`,
        filename: 'file.ts',
        languageOptions: { parserOptions: { ecmaFeatures: { jsx: false } } },
        errors: [{ messageId: 'cast' }],
      },
      {
        code: `${PAISA}declare const maybe: Paisa | null; const c = (maybe ?? a) * 2;`,
        errors: [{ messageId: 'operator', data: { operator: '*' } }],
      },
      {
        code: `${PAISA}interface Bill { buyerTotal: Paisa; laga: Paisa } declare const bill: Bill; const t = bill.buyerTotal + bill.laga;`,
        errors: [{ messageId: 'operator', data: { operator: '+' } }],
      },
    ],
  });
});

describe('mandi/no-jsx-literals (rule 7)', () => {
  ruleTester.run('no-jsx-literals', noJsxLiterals, {
    valid: [
      { code: 'const x = <h1 className="type-title" />;' },
      { code: 'declare const name: string; const x = <p>{name}</p>;' },
      { code: "const x = <p>{' '}</p>;" },
      { code: 'const x = <img alt="" />;' },
      { code: 'const x = <div lang="ur" dir="rtl" data-testid="app-version" />;' },
    ],
    invalid: [
      { code: 'const x = <h1>Mandi Software</h1>;', errors: [{ messageId: 'text' }] },
      { code: 'const x = <h1>منڈی سافٹ ویئر</h1>;', errors: [{ messageId: 'text' }] },
      { code: 'const x = <td>10</td>;', errors: [{ messageId: 'text' }] },
      { code: "const x = <p>{'Save'}</p>;", errors: [{ messageId: 'text' }] },
      {
        code: 'declare const n: number; const x = <p>{`Sale ${n}`}</p>;',
        errors: [{ messageId: 'text' }],
      },
      {
        code: 'const x = <input placeholder="گاہک کا کوڈ" />;',
        errors: [{ messageId: 'attribute' }],
      },
      { code: "const x = <button aria-label={'Close'} />;", errors: [{ messageId: 'attribute' }] },
      { code: 'const x = <span title="Version" />;', errors: [{ messageId: 'attribute' }] },
    ],
  });
});

describe('mandi/no-physical-direction (rule 8)', () => {
  ruleTester.run('no-physical-direction', noPhysicalDirection, {
    valid: [
      {
        code: 'const x = <div className="ms-2 me-2 ps-4 pe-4 text-start text-end border-s rounded-e start-0 end-0" />;',
      },
      { code: 'const x = <div className="mx-auto px-4 mt-3 mb-4 inset-x-0 border-x" />;' },
      {
        code: "const x = <div className=\"rtl:-scale-x-100\" style={{ marginInlineStart: 'var(--space-2)', textAlign: 'end', direction: 'ltr' }} />;",
      },
      { code: 'const x = <div className="leftover-class" />;' },
    ],
    invalid: [
      {
        code: 'const x = <div className="ml-2" />;',
        errors: [{ messageId: 'className', data: { token: 'ml-2' } }],
      },
      {
        code: 'const x = <div className="md:-mr-4" />;',
        errors: [{ messageId: 'className', data: { token: 'mr-4' } }],
      },
      {
        code: 'const x = <div className="pl-3 pr-3" />;',
        errors: [{ messageId: 'className' }, { messageId: 'className' }],
      },
      {
        code: 'const x = <div className="left-0" />;',
        errors: [{ messageId: 'className', data: { token: 'left-0' } }],
      },
      {
        code: 'const x = <div className="text-right" />;',
        errors: [{ messageId: 'className', data: { token: 'text-right' } }],
      },
      {
        code: 'const x = <div className="float-left" />;',
        errors: [{ messageId: 'className', data: { token: 'float-left' } }],
      },
      {
        code: 'const x = <div className="border-l-2" />;',
        errors: [{ messageId: 'className', data: { token: 'border-l-2' } }],
      },
      {
        code: 'const x = <div className="rounded-tr-1" />;',
        errors: [{ messageId: 'className', data: { token: 'rounded-tr-1' } }],
      },
      {
        code: "declare const on: boolean; const x = <div className={on ? 'ms-2' : 'ml-2'} />;",
        errors: [{ messageId: 'className', data: { token: 'ml-2' } }],
      },
      {
        code: 'const x = <div style={{ marginLeft: 4 }} />;',
        errors: [{ messageId: 'style', data: { key: 'marginLeft' } }],
      },
      {
        code: 'const x = <div style={{ right: 0 }} />;',
        errors: [{ messageId: 'style', data: { key: 'right' } }],
      },
      {
        code: "const x = <div style={{ borderLeftWidth: '1px' }} />;",
        errors: [{ messageId: 'style' }],
      },
      {
        code: "const x = <div style={{ textAlign: 'left' }} />;",
        errors: [{ messageId: 'style', data: { key: 'textAlign' } }],
      },
    ],
  });
});

describe('mandi/no-raw-design-values (Code Standards F9)', () => {
  ruleTester.run('no-raw-design-values', noRawDesignValues, {
    valid: [
      {
        code: 'const x = <div className="p-3 bg-paper text-ur-2 rounded-1 bg-(--color-surface)" />;',
      },
      { code: "const x = <div style={{ color: 'var(--color-ink)', padding: 0 }} />;" },
    ],
    invalid: [
      {
        code: 'const x = <div className="p-[13px]" />;',
        errors: [{ messageId: 'arbitrary', data: { token: 'p-[13px]' } }],
      },
      { code: 'const x = <div className="bg-[#2e7d32]" />;', errors: [{ messageId: 'arbitrary' }] },
      {
        code: "const x = <div style={{ color: '#2e7d32' }} />;",
        errors: [{ messageId: 'raw', data: { value: '#2e7d32' } }],
      },
      {
        code: 'const x = <div style={{ padding: 13 }} />;',
        errors: [{ messageId: 'style', data: { key: 'padding' } }],
      },
      {
        code: "const x = <div style={{ fontSize: '15px' }} />;",
        errors: [{ messageId: 'raw', data: { value: '15px' } }],
      },
    ],
  });
});

// Reviewer finding 3 (plan 01): text, classes and styles hidden in constants, branches, lookup
// tables, component props and spread arguments.
describe('reviewer finding 3: values outside JSX literals', () => {
  ruleTester.run('no-physical-direction', noPhysicalDirection, {
    valid: [
      { code: "type Align = 'text-left'; const o = { 'ml-2': 1 }; export { o };" },
      { code: "import { x } from './left-panel'; export { x };" },
      { code: "const s = { direction: 'ltr', textAlign: 'end', insetInlineStart: 0 };" },
    ],
    invalid: [
      {
        code: "const cls = 'ml-2 text-left';",
        errors: [
          { messageId: 'className', data: { token: 'ml-2' } },
          { messageId: 'className', data: { token: 'text-left' } },
        ],
      },
      {
        code: "const VARIANT_CLASSES = { primary: 'bg-primary', quiet: 'pr-4' };",
        errors: [{ messageId: 'className', data: { token: 'pr-4' } }],
      },
      {
        code: 'declare const v: string; const c = `${v} mr-2`;',
        errors: [{ messageId: 'className', data: { token: 'mr-2' } }],
      },
      {
        code: "const align = { textAlign: 'left' } as const;",
        errors: [{ messageId: 'style', data: { key: 'textAlign' } }],
      },
      {
        code: "declare const on: boolean; const x = <span style={{ textAlign: on ? 'right' : 'end' }} />;",
        errors: [{ messageId: 'style', data: { key: 'textAlign' } }],
      },
      {
        code: "const s = { transformOrigin: 'left top', backgroundPosition: 'right' };",
        errors: [
          { messageId: 'style', data: { key: 'transformOrigin' } },
          { messageId: 'style', data: { key: 'backgroundPosition' } },
        ],
      },
      {
        code: 'const box = { marginLeft: 8 };',
        errors: [{ messageId: 'style', data: { key: 'marginLeft' } }],
      },
    ],
  });

  ruleTester.run('no-raw-design-values', noRawDesignValues, {
    valid: [
      { code: "const path = 'lots[0].sales[2].qty'; const ref = 'E-20261010-0001';" },
      {
        code: "const s = { retry: 3, width: 0, color: 'var(--color-ink)' }; const c = 'h-(--control-h)';",
      },
    ],
    invalid: [
      {
        code: "const cls = 'p-[13px]';",
        errors: [{ messageId: 'arbitrary', data: { token: 'p-[13px]' } }],
      },
      { code: "const c = { primary: 'bg-[#1D3D6E]' };", errors: [{ messageId: 'arbitrary' }] },
      {
        code: "const color = '#2e7d32';",
        errors: [{ messageId: 'raw', data: { value: '#2e7d32' } }],
      },
      { code: "const size = '15px';", errors: [{ messageId: 'raw', data: { value: '15px' } }] },
      {
        code: 'const s = { fontSize: 15 };',
        errors: [{ messageId: 'style', data: { key: 'fontSize' } }],
      },
      {
        code: 'declare const big: boolean; const s = { padding: big ? 16 : 0 };',
        errors: [{ messageId: 'style', data: { key: 'padding' } }],
      },
    ],
  });

  ruleTester.run('no-jsx-literals', noJsxLiterals, {
    valid: [
      {
        code: 'const x = <div role="group" aria-hidden="true" aria-live="polite" data-testid="app-version" lang="ur" dir="rtl" id="common-app-name" />;',
      },
      { code: "const id = 'common-retry'; const x = <p id={id} />;" },
      {
        code: "declare function Chip(p: { tone: 'on-primary' | 'on-light' }): null; declare const on: boolean; const x = <><Chip tone=\"on-light\" /><Chip tone={on ? 'on-primary' : 'on-light'} /></>;",
      },
      { code: 'declare const n: number; const x = <p>{n}</p>;' },
    ],
    invalid: [
      {
        code: "declare const on: boolean; const x = <p>{on ? 'Yes' : 'No'}</p>;",
        errors: [{ messageId: 'text' }],
      },
      {
        code: "declare const on: boolean; const x = <p>{on && 'Saved'}</p>;",
        errors: [{ messageId: 'text' }],
      },
      { code: "const label = 'Save'; const x = <p>{label}</p>;", errors: [{ messageId: 'text' }] },
      {
        code: 'declare function Button(p: { ariaLabel: string }): null; const x = <Button ariaLabel="Save" />;',
        errors: [{ messageId: 'attribute', data: { name: 'ariaLabel' } }],
      },
      {
        code: "declare const on: boolean; const x = <div title={on ? 'Open' : 'Shut'} />;",
        errors: [{ messageId: 'attribute', data: { name: 'title' } }],
      },
      {
        code: "const hint = 'Type the code'; const x = <input placeholder={hint} />;",
        errors: [{ messageId: 'attribute', data: { name: 'placeholder' } }],
      },
    ],
  });

  ruleTester.run('no-paisa-arithmetic', noPaisaArithmetic, {
    valid: [{ code: 'declare const qty: number[]; const most = Math.max(...qty);' }],
    invalid: [
      {
        code: `${PAISA}declare const list: Paisa[]; const most = Math.max(...list);`,
        errors: [{ messageId: 'call', data: { name: 'Math.max' } }],
      },
      {
        code: `${PAISA}declare const pair: [Paisa, Paisa]; const low = Math.min(...pair);`,
        errors: [{ messageId: 'call', data: { name: 'Math.min' } }],
      },
    ],
  });
});

// Reviewer recheck finding C (plan 01): words that reach the screen from outside JSX literals.
describe('reviewer recheck finding C: words outside JSX', () => {
  ruleTester.run('no-jsx-literals', noJsxLiterals, {
    valid: [
      {
        code: "const KEYS = ['app_info', 'ERR_INTERNAL']; declare const n: number; const k = KEYS[n];",
      },
      { code: 'const SIZES = [1024, 1366]; declare const n: number; const x = <p>{SIZES[n]}</p>;' },
      {
        code: "declare const l10n: { getString(id: string, args?: Record<string, string | number>): string }; declare const v: string; document.title = l10n.getString('common-app-name', { version: v, count: 3 });",
      },
      {
        code: 'declare const version: string; declare const reference: string; const x = <p vars={{ version, ref: reference, count: 2 }} />;',
      },
      { code: "declare const el: HTMLElement; el.dir = 'rtl'; el.setAttribute('lang', 'ur');" },
      {
        code: "declare const l10n: { getString(id: string): string }; function label() { return l10n.getString('common-retry'); } const count = () => 3; const x = <p>{label()}{count()}</p>;",
      },
    ],
    invalid: [
      {
        code: "const words = ['Save', 'Cancel']; declare const n: number; const x = <p>{words[n]}</p>;",
        errors: [{ messageId: 'text' }],
      },
      {
        code: "const TITLES = { a: 'Sale Book' }; const x = <p>{TITLES['a']}</p>;",
        errors: [{ messageId: 'text' }],
      },
      {
        code: "const TITLES = { a: 'Sale Book' } as const; const x = <p>{TITLES.a}</p>;",
        errors: [{ messageId: 'text' }],
      },
      {
        code: "declare const n: number; const x = <p>{['Yes', 'No'][n]}</p>;",
        errors: [{ messageId: 'text' }],
      },
      {
        code: "declare const l10n: { getString(id: string, args?: Record<string, string>): string }; const t = l10n.getString('x', { v: 'Hello world' });",
        errors: [{ messageId: 'fluentArgs' }],
      },
      {
        code: "declare function getString(id: string, args?: Record<string, string>): string; const greeting = 'Hello'; const t = getString('x', { v: greeting });",
        errors: [{ messageId: 'fluentArgs' }],
      },
      {
        code: "const x = <p vars={{ v: 'Hello world' }} />;",
        errors: [{ messageId: 'fluentArgs' }],
      },
      {
        code: "function label() { return 'Save'; } const x = <p>{label()}</p>;",
        errors: [{ messageId: 'text' }],
      },
      {
        code: "declare const on: boolean; const label = () => (on ? 'Open' : 'Shut'); const x = <p>{label()}</p>;",
        errors: [{ messageId: 'text' }],
      },
      {
        code: "document.title = 'Mandi';",
        errors: [{ messageId: 'property', data: { name: 'title' } }],
      },
      {
        code: "declare const el: HTMLElement; el.textContent = 'Saved';",
        errors: [{ messageId: 'property', data: { name: 'textContent' } }],
      },
      {
        code: "declare const el: HTMLElement; declare const on: boolean; el.innerText = on ? 'Open' : 'Shut';",
        errors: [{ messageId: 'property', data: { name: 'innerText' } }],
      },
      {
        code: "declare const el: HTMLElement; el.setAttribute('aria-label', 'Save');",
        errors: [{ messageId: 'property', data: { name: 'aria-label' } }],
      },
    ],
  });
});

// Reviewer recheck 2, finding A2: Paisa leaking through callbacks and any/unknown parameters.
const PAISA_API = `${PAISA}declare const list: Paisa[];
declare function formatPaisa(amount: Paisa): string;
declare function comparePaisa(x: Paisa, y: Paisa): number;
declare function useState<S>(initial: S | (() => S)): [S, (value: S | ((previous: S) => S)) => void];
declare function identity<T>(value: T): T;
`;

describe('reviewer recheck 2 finding A2: Paisa flows only into Paisa slots', () => {
  ruleTester.run('no-paisa-arithmetic', noPaisaArithmetic, {
    valid: [
      { code: `${PAISA_API}const text = formatPaisa(a);` },
      { code: `${PAISA_API}const sorted = [...list].sort(comparePaisa);` },
      { code: `${PAISA_API}const [total, setTotal] = useState<Paisa | null>(null); setTotal(a);` },
      { code: `${PAISA_API}const texts = list.map((p) => formatPaisa(p));` },
      { code: `${PAISA_API}const same = identity(a); const kept = list.filter((p) => p === a);` },
      {
        code: `${PAISA_API}declare function Row(props: { total: Paisa }): null; Row({ total: a });`,
      },
      {
        code: 'declare const qty: number[]; const t = qty.map(Number); console.log(JSON.stringify(qty));',
      },
    ],
    invalid: [
      {
        code: `${PAISA_API}const n1 = [a].map(Number)[0] ?? 0;`,
        errors: [{ messageId: 'callback', data: { name: 'Number' } }],
      },
      {
        code: `${PAISA_API}const n2 = new Number(a).valueOf();`,
        errors: [{ messageId: 'flow', data: { name: 'Number' } }],
      },
      {
        code: `${PAISA_API}const n3 = list.map(Number).reduce((x, y) => x + y, 0);`,
        errors: [{ messageId: 'callback', data: { name: 'Number' } }],
      },
      {
        code: `${PAISA_API}const n4 = Number(JSON.stringify(a));`,
        errors: [{ messageId: 'flow', data: { name: 'JSON.stringify' } }],
      },
      {
        code: `${PAISA_API}const texts = list.map(String);`,
        errors: [{ messageId: 'callback', data: { name: 'String' } }],
      },
      {
        code: `${PAISA_API}list.forEach(console.log);`,
        errors: [{ messageId: 'callback', data: { name: 'console.log' } }],
      },
      {
        code: `${PAISA_API}const loose = list.map((p: unknown) => p);`,
        errors: [{ messageId: 'callback' }],
      },
      {
        code: `${PAISA_API}console.log(a);`,
        errors: [{ messageId: 'flow', data: { name: 'console.log' } }],
      },
      {
        code: `${PAISA_API}const u = identity<unknown>(a);`,
        errors: [{ messageId: 'flow', data: { name: 'identity' } }],
      },
      {
        code: `${PAISA_API}declare function log(...values: unknown[]): void; log(...list);`,
        errors: [{ messageId: 'flow', data: { name: 'log' } }],
      },
      {
        code: `${PAISA_API}const all = new Set<unknown>(list);`,
        errors: [{ messageId: 'flow', data: { name: 'Set' } }],
      },
    ],
  });
});

// Plan 01 review finding 01-M1: objects that hold Paisa, taken apart or turned into text.
describe('01-M1: Object.values, Object.entries and JSON.stringify on objects with Paisa', () => {
  const BILL = `${PAISA_API}interface Bill { total: Paisa; laga: Paisa; note: string }\ndeclare const bill: Bill;\n`;
  ruleTester.run('no-paisa-arithmetic', noPaisaArithmetic, {
    valid: [
      { code: `${BILL}const text = formatPaisa(bill.total);` },
      {
        code: 'declare const qty: { n: number }; const v = Object.values(qty); const s = JSON.stringify(qty);',
      },
      { code: `${BILL}const keys = Object.keys(bill);` },
      {
        code: 'declare const row: { name: string; codes: string[] }; const e = Object.entries(row);',
      },
    ],
    invalid: [
      {
        code: `${BILL}const all = Object.values(bill);`,
        errors: [{ messageId: 'objectFlow', data: { name: 'Object.values', property: 'bill' } }],
      },
      {
        code: `${BILL}const pairs = Object.entries(bill);`,
        errors: [{ messageId: 'objectFlow', data: { name: 'Object.entries', property: 'bill' } }],
      },
      {
        code: `${BILL}const text = JSON.stringify(bill);`,
        errors: [{ messageId: 'objectFlow', data: { name: 'JSON.stringify', property: 'bill' } }],
      },
      {
        // Deeper: a list of lines inside a bill, and an optional amount.
        code: `${PAISA_API}declare const posting: { lines: { amount: Paisa | null }[] }; const t = JSON.stringify(posting);`,
        errors: [{ messageId: 'objectFlow' }],
      },
      {
        code: `${BILL}const bills: Bill[] = [bill]; const t = JSON.stringify(bills);`,
        errors: [{ messageId: 'objectFlow' }],
      },
    ],
  });
});
