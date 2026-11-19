// Records of a master list that look like the one being typed, under its name, before the save
// (PRD FR-7.63, FR-10.7, E75): code, name in both scripts and the second line that tells two of the
// same name apart. One can be taken instead of making a new one. The search is the backend's
// (both scripts, look-alike letters, small mistakes); the screen never decides what is similar. It
// only warns: the save is never blocked by it.

import { Localized } from '@fluent/react';
import { TriangleAlert } from 'lucide-react';
import { displayName, otherScriptName, useLanguage } from '../../../shared/i18n';
import type { SearchHit } from '../../../shared/search';
import { Code, LinkButton } from '../../../shared/ui';

interface SimilarMastersProps {
  matches: readonly SearchHit[];
  /** Takes the record instead of making a new one. */
  onPick?: ((hit: SearchHit) => void) | undefined;
}

export function SimilarMasters({ matches, onPick }: SimilarMastersProps) {
  const language = useLanguage();
  if (matches.length === 0) return null;
  return (
    <section className="flex flex-col gap-2 rounded-2 border border-warn-border bg-warn-soft p-3">
      <h3 className="flex items-center gap-2 type-body font-semibold text-warn">
        <TriangleAlert aria-hidden="true" className="size-icon-md shrink-0" />
        <Localized id="search-quick-similar-title" />
      </h3>
      <ul className="flex flex-col gap-1">
        {matches.map((hit) => {
          const extra = displayName({ name_ur: hit.extra_ur, name_en: hit.extra_en }, language);
          return (
            <li
              key={hit.id}
              className="flex flex-wrap items-baseline gap-x-4 gap-y-1 type-body text-ink"
            >
              {hit.code === '' ? null : <Code>{hit.code}</Code>}
              <span className="font-semibold">{displayName(hit, language)}</span>
              <span className="text-ink-muted">{otherScriptName(hit, language)}</span>
              {extra === '' ? null : <span className="text-ink-muted">{extra}</span>}
              {onPick === undefined ? null : (
                <span className="ms-auto">
                  <LinkButton
                    onClick={() => {
                      onPick(hit);
                    }}
                  >
                    <Localized id="search-quick-pick" />
                  </LinkButton>
                </span>
              )}
            </li>
          );
        })}
      </ul>
    </section>
  );
}
