// A line of free text on a print (the firm's address, its mandi, a contact person's name) with every
// number in it drawn as a number: Inter with equal-width digits, written left to right, never in the
// font of the words around it (Code Standards F17, F25, PRD FR-7.32, NFR-URD-13). The text is as typed
// in Settings (PRD FR-7.25) and keeps its own direction (`dir="auto"`); only its digit runs, for
// example "12" or "12-A", are taken out and wrapped. The words are never changed.

import { Fragment } from 'react';
import { Num } from '../ui/Num';

/** A run of digits, with the signs a house or shop number uses between digits: `12`, `12-14`, `3/4`. */
const NUMBER = /(\d+(?:[-/.,]\d+)*)/;

export function PrintText({ text }: { text: string }) {
  // The capture group makes `split` keep the digit runs: they are at the odd places.
  const parts = text.split(NUMBER);
  return (
    <bdi dir="auto">
      {parts.map((part, index) =>
        index % 2 === 1 ? <Num key={index}>{part}</Num> : <Fragment key={index}>{part}</Fragment>,
      )}
    </bdi>
  );
}
