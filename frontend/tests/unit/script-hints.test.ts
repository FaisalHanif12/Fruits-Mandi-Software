// On-screen hints for names and for the wizard (Code Standards 4.1 layer 1, PRD FR-7.28). They help
// at once and are never trusted: the backend checks every name again.

import { describe, expect, it } from 'vitest';
import { firstHintedField, hintFor } from '../../src/features/auth/wizard-hints';
import { requiredHintId, scriptHintId } from '../../src/shared/text/scriptHint';

describe('scriptHintId', () => {
  it.each([
    ['ارشد ملتانی', 'ur', null],
    ['Arshad', 'ur', 'err-name-ur-script'],
    ['ارشد Multani', 'ur', 'err-name-ur-script'],
    ['', 'ur', null],
    ['12 - (3) & /', 'ur', null],
    ['Arshad Multani', 'en', null],
    ['ارشد', 'en', 'err-name-en-script'],
    ["Maher & Sons (1) a-b.c'd/e", 'en', null],
    ['Maher ملتانی', 'en', 'err-name-en-script'],
    ['', 'en', null],
  ] as const)('%s as %s gives %s', (text, script, expected) => {
    expect(scriptHintId(text, script)).toBe(expected);
  });

  it('takes the Urdu letter variants and presentation forms as Urdu', () => {
    expect(scriptHintId('اكرم', 'en')).toBe('err-name-en-script');
    expect(scriptHintId('ﻻ', 'en')).toBe('err-name-en-script');
  });
});

describe('requiredHintId', () => {
  it('flags empty text and spaces alone, not text', () => {
    expect(requiredHintId('')).toBe('err-required');
    expect(requiredHintId('   ')).toBe('err-required');
    expect(requiredHintId('a')).toBeNull();
  });
});

describe('hintFor (first-run wizard)', () => {
  it('shows a wrong script at once, and an empty required field only after a try', () => {
    expect(hintFor('firm_name_ur', 'Maher', false)).toBe('err-name-ur-script');
    expect(hintFor('firm_name_en', 'مہر', false)).toBe('err-name-en-script');
    expect(hintFor('firm_name_en', '', false)).toBeNull();
    expect(hintFor('firm_name_en', '', true)).toBe('err-required');
  });

  it('leaves phone and address free to stay empty', () => {
    expect(hintFor('phone', '', true)).toBeNull();
    expect(hintFor('address', '', true)).toBeNull();
  });

  it('asks for the Owner name, user name and both passwords', () => {
    for (const field of [
      'owner_display_name',
      'owner_username',
      'password',
      'password_repeat',
    ] as const) {
      expect(hintFor(field, '', true)).toBe('err-required');
      expect(hintFor(field, 'x', true)).toBeNull();
    }
  });
});

describe('firstHintedField', () => {
  const empty = {
    firm_name_ur: '',
    firm_name_en: '',
    mandi_ur: '',
    mandi_en: '',
    phone: '',
    address: '',
    owner_display_name: '',
    owner_username: '',
    password: '',
    password_repeat: '',
  };

  it('is the first field of the step, in the order typed', () => {
    expect(firstHintedField(0, empty)).toBe('firm_name_en');
    expect(firstHintedField(1, empty)).toBe('owner_display_name');
    expect(firstHintedField(2, empty)).toBeNull();
  });

  it('is null when the step is fine', () => {
    expect(
      firstHintedField(0, {
        ...empty,
        firm_name_en: 'Maher',
        firm_name_ur: 'مہر',
        mandi_en: 'Mandi',
        mandi_ur: 'منڈی',
      }),
    ).toBeNull();
  });
});
