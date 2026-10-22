// What the first-run wizard has collected (plan 04, PRD NFR-INS-2). It lives in a store above the
// pages, so moving between steps and switching language lose nothing (Code Standards F39). The
// passwords are cleared the moment the setup succeeds.

import { create } from 'zustand';
import type { SpellingMarks } from '../../shared/urdu/spelling';
import { useBackupKeyStore } from '../backup';
import { usePartnerSetupStore } from '../partners';

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
  [],
  [],
];

/**
 * The steps in order: 0 firm, 1 Owner, 2 recovery code, 3 backup key, 4 partners (NFR-INS-2).
 * The backup key step (plan 18, PRD FR-20.10) needs the signed-in Owner, so it comes after the
 * recovery code, like the partners step.
 */
export const STEP_CODE = 2;
export const STEP_KEY = 3;
export const STEP_PARTNERS = 4;

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
  /**
   * True when the Owner was signed in right after the setup, so the partners step can save
   * (every partners command needs a signed-in Owner). Without it the wizard ends at sign-in.
   */
  signedIn: boolean;
  setStep: (step: number) => void;
  setValue: (field: WizardField, text: string) => void;
  setMarks: (name: WizardNameKey, marks: SpellingMarks | undefined) => void;
  finishSetup: (recoveryCode: string, signedIn: boolean) => void;
  /** On to the backup key step. The recovery code is shown once, so it is dropped here. */
  toKey: () => void;
  /** On to the partners step. The backup key is shown once, so the key step is over. */
  toPartners: () => void;
  reset: () => void;
}

export const useWizardStore = create<WizardState>()((set) => ({
  step: 0,
  values: EMPTY,
  marks: {},
  recoveryCode: null,
  signedIn: false,
  setStep: (step) => {
    set({ step });
  },
  setValue: (field, text) => {
    set((state) => ({ values: { ...state.values, [field]: text } }));
  },
  setMarks: (name, marks) => {
    set((state) => ({ marks: { ...state.marks, [name]: marks } }));
  },
  finishSetup: (recoveryCode, signedIn) => {
    set((state) => ({
      step: STEP_CODE,
      recoveryCode,
      signedIn,
      values: { ...state.values, password: '', password_repeat: '' },
    }));
  },
  toKey: () => {
    set({ step: STEP_KEY, recoveryCode: null });
  },
  toPartners: () => {
    set({ step: STEP_PARTNERS, recoveryCode: null });
  },
  reset: () => {
    set({ step: 0, values: EMPTY, marks: {}, recoveryCode: null, signedIn: false });
    // What was typed in the partners step is gone with the wizard, and so is the backup key
    // (it is shown once; the backup screen can make a new one).
    usePartnerSetupStore.getState().clear();
    useBackupKeyStore.getState().clear();
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
