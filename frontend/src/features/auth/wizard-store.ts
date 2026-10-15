// What the first-run wizard has collected (plan 04, PRD NFR-INS-2). It lives in a store above the
// pages, so moving between steps and switching language lose nothing (Code Standards F39). The
// passwords are cleared the moment the setup succeeds.

import { create } from 'zustand';
import type { SpellingMarks } from '../../shared/urdu/spelling';

export const WIZARD_FIELDS = [
  'firm_name_ur',
  'firm_name_en',
  'mandi_ur',
  'mandi_en',
  'phone',
  'address',
  'owner_display_name',
  'owner_username',
  'password',
  'password_repeat',
] as const;

export type WizardField = (typeof WIZARD_FIELDS)[number];
export type WizardValues = Record<WizardField, string>;

/** The fields of each step, in the order the user types them. */
export const STEP_FIELDS: readonly (readonly WizardField[])[] = [
  ['firm_name_en', 'firm_name_ur', 'mandi_en', 'mandi_ur', 'phone', 'address'],
  ['owner_display_name', 'owner_username', 'password', 'password_repeat'],
  [],
];

const EMPTY: WizardValues = {
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

/** The two names of the first step that have a spelling check (PRD FR-7.66). */
export type WizardNameKey = 'firm' | 'mandi';

interface WizardState {
  step: number;
  values: WizardValues;
  /** The words of the firm and mandi names to check, and the ones he confirmed (FR-7.66). */
  marks: Partial<Record<WizardNameKey, SpellingMarks>>;
  /** Shown once on step 3. Never saved anywhere else. */
  recoveryCode: string | null;
  setStep: (step: number) => void;
  setValue: (field: WizardField, text: string) => void;
  setMarks: (name: WizardNameKey, marks: SpellingMarks | undefined) => void;
  finishSetup: (recoveryCode: string) => void;
  reset: () => void;
}

export const useWizardStore = create<WizardState>()((set) => ({
  step: 0,
  values: EMPTY,
  marks: {},
  recoveryCode: null,
  setStep: (step) => {
    set({ step });
  },
  setValue: (field, text) => {
    set((state) => ({ values: { ...state.values, [field]: text } }));
  },
  setMarks: (name, marks) => {
    set((state) => ({ marks: { ...state.marks, [name]: marks } }));
  },
  finishSetup: (recoveryCode) => {
    set((state) => ({
      step: 2,
      recoveryCode,
      values: { ...state.values, password: '', password_repeat: '' },
    }));
  },
  reset: () => {
    set({ step: 0, values: EMPTY, marks: {}, recoveryCode: null });
  },
}));

/**
 * The earliest step that has a field an error points at, or null when no field is named. The
 * wizard goes back to it, so every message is shown at the field it belongs to.
 */
export function earliestStepWithError(fields: readonly (string | null)[]): number | null {
  let earliest: number | null = null;
  for (const field of fields) {
    if (field === null) continue;
    const index = STEP_FIELDS.findIndex((step) => (step as readonly string[]).includes(field));
    if (index >= 0 && (earliest === null || index < earliest)) earliest = index;
  }
  return earliest;
}
