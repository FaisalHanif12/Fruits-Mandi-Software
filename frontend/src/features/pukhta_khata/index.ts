// The public face of the pukhta_khata module (Tech Stack 5.3): the register screen, and the pieces
// the sale book uses for the one merged list of known one-time people (PRD FR-7.29).
export { MakeRegularDialog } from './components/MakeRegularDialog';
export { PukhtaPersonPicker } from './components/PukhtaPersonPicker';
export {
  NO_MARKS_TEXT,
  PukhtaPersonFields,
  readMarks,
  type PukhtaPersonValue,
} from './components/PukhtaPersonFields';
export { PukhtaPersonRow } from './components/PukhtaPersonRow';
export { PukhtaRegisterPage } from './pages/PukhtaRegisterPage';
export { usePukhtaPeople } from './hooks';
export { PUKHTA_BUYER_CODE, PUKHTA_SELLER_CODE, isPukhtaCode } from './codes';
export { pukhtaMakeRegular } from './api';
