// The one way a record chosen in a picker is kept on a screen (PRD 10.2, Code Standards F38): its
// id and code, both names, and its tag (نشان) when it has one. Every screen that keeps a chosen
// account, from a hit of the picker or from a saved record, builds it here, so the tag can never
// be lost on the way and a language switch still shows the name with its tag.

/** What a picker hit, a record reference or an account row says about a record. */
interface PickedSource {
  readonly id: number;
  readonly code: string;
  readonly name_ur: string;
  readonly name_en: string;
  readonly tag?: string | null | undefined;
  /** True for an item sold by weight, as the search hit of an item says (PRD FR-12.41, D3). */
  readonly sold_by_weight?: boolean | undefined;
}

/**
 * The record as a screen keeps it. `tag` is left out when there is none, which is also the shape
 * of a `RecordRef` from the backend, so it can go wherever one is expected. `soldByWeight` is there
 * only for an item that is sold by weight, so the shape of every other record stays the same.
 */
export interface PickedAccount {
  readonly id: number;
  readonly code: string;
  readonly name_ur: string;
  readonly name_en: string;
  readonly tag?: string;
  readonly soldByWeight?: true;
}

export function pickedOf(source: PickedSource): PickedAccount {
  const base = {
    id: source.id,
    code: source.code,
    name_ur: source.name_ur,
    name_en: source.name_en,
    ...(source.sold_by_weight === true ? { soldByWeight: true as const } : {}),
  };
  return source.tag === undefined || source.tag === null || source.tag === ''
    ? base
    : { ...base, tag: source.tag };
}
