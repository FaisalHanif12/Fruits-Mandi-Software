// A name with its tag after it (PRD 10.2, FR-19.13): "اکرم ٹول ٹیکس K". The name is in the print
// language already; the tag is English capital letters, stored once and written as it is, so it is
// isolated and reads left to right in an Urdu line (NFR-URD-13). No tag, no extra space.

import { Code } from '../ui/Code';
import { ScriptText } from '../ui/ScriptText';

interface PrintNameProps {
  name: string;
  /** The tag, absent when the account has none. */
  tag?: string | undefined;
}

export function PrintName({ name, tag }: PrintNameProps) {
  return (
    <>
      <ScriptText>{name}</ScriptText>
      {tag === undefined || tag === '' ? null : (
        <>
          {' '}
          <Code>{tag}</Code>
        </>
      )}
    </>
  );
}
