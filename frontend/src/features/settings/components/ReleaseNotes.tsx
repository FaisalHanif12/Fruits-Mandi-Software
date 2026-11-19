// The notes of a new version ("what is new", NFR-INS-3), as the release file writes them: a line that
// starts with "- " is a bullet, a line that starts with "## " is a small heading, anything else is a
// paragraph. The words are the release text, not app text: they come in the language they were
// written in, so the block carries that language and direction (FR-7.25).

import type { ReactNode } from 'react';
import type { Language } from '../../../bindings/Language';

const BULLET = '- ';
const HEADING = '## ';

interface Block {
  kind: 'bullets' | 'heading' | 'paragraph';
  lines: string[];
}

/** Groups the lines: consecutive bullets are one list. */
export function notesBlocks(text: string): Block[] {
  const blocks: Block[] = [];
  for (const raw of text.split('\n')) {
    const line = raw.trim();
    if (line === '') continue;
    if (line.startsWith(BULLET)) {
      const item = line.slice(BULLET.length).trim();
      const last = blocks.at(-1);
      if (last?.kind === 'bullets') last.lines.push(item);
      else blocks.push({ kind: 'bullets', lines: [item] });
    } else if (line.startsWith(HEADING)) {
      blocks.push({ kind: 'heading', lines: [line.slice(HEADING.length).trim()] });
    } else {
      blocks.push({ kind: 'paragraph', lines: [line] });
    }
  }
  return blocks;
}

interface ReleaseNotesProps {
  text: string;
  /** The language the notes are written in. */
  language: Language;
}

export function ReleaseNotes({ text, language }: ReleaseNotesProps) {
  const direction = language === 'ur' ? 'rtl' : 'ltr';
  const nodes: ReactNode[] = notesBlocks(text).map((block, blockIndex) => {
    const key = `${block.kind}-${String(blockIndex)}`;
    if (block.kind === 'bullets') {
      return (
        <ul key={key} className="list-disc ps-6">
          {block.lines.map((line, lineIndex) => (
            <li key={`${key}-${String(lineIndex)}`} className="type-body">
              {line}
            </li>
          ))}
        </ul>
      );
    }
    if (block.kind === 'heading') {
      return (
        <h4 key={key} className="type-control">
          {block.lines.join(' ')}
        </h4>
      );
    }
    return (
      <p key={key} className="type-body">
        {block.lines.join(' ')}
      </p>
    );
  });
  return (
    <div lang={language} dir={direction} className="flex flex-col gap-2">
      {nodes}
    </div>
  );
}
