// A name or a word typed by a person, shown as it was typed (PRD FR-7.25), tagged with the script
// it is written in. An Urdu name inside an English sentence then gets the Urdu font, size and line
// height (the rule in styles/globals.css), and an English name inside an Urdu sentence gets the
// Latin ones. Fluent can fill it: `User: <name>{ $name }</name>` with `elems={{ name: <ScriptText /> }}`.

import type { ReactNode } from 'react';
import { scriptOf } from '../text/scriptHint';

interface ScriptTextProps {
  /** The typed text. Fluent passes the text between the tags here. */
  children?: ReactNode;
}

/** The text of the children, to find the script. Fluent gives text, or a list of text parts. */
function textOf(children: ReactNode): string {
  if (typeof children === 'string') return children;
  if (Array.isArray(children)) return (children as ReactNode[]).map(textOf).join('');
  return '';
}

export function ScriptText({ children }: ScriptTextProps) {
  return <bdi lang={scriptOf(textOf(children))}>{children}</bdi>;
}
