// The collection list of one day on screen: reads its print data and draws the template. It is the
// body of the hidden print window (mode `window`) and also the in-app preview (mode `preview`), so
// what the user previews is what is printed. It calls ONE command, `reports_collection_print_data`.

import type { CollectionPrintData } from '../../../bindings/CollectionPrintData';
import type { CollectionPrintDataInput } from '../../../bindings/CollectionPrintDataInput';
import { PrintView, type PrintDescription, type PrintReady } from '../../../shared/print';
import { useCollectionPrintData } from '../../reports';
import { CollectionListTemplate } from './CollectionListTemplate';

interface CollectionPrintViewProps {
  input: CollectionPrintDataInput;
  /** `window`: the hidden print window. `preview`: inside the app, which keeps its own language. */
  mode: 'window' | 'preview';
  onReady?: (ready: PrintReady) => void;
  onFailed?: (codes: string[]) => void;
}

/** Empty when no buyer is planned for the day; the buyers' names are the sample for the fonts. */
function describeCollection(data: CollectionPrintData): PrintDescription {
  return {
    empty: data.groups.length === 0,
    sample: data.groups.flatMap((group) => group.rows.map((row) => row.name)).join(' '),
  };
}

export function CollectionPrintView({ input, mode, onReady, onFailed }: CollectionPrintViewProps) {
  const query = useCollectionPrintData(input);
  return (
    <PrintView
      query={query}
      describe={describeCollection}
      {...(onReady === undefined ? {} : { onReady })}
      {...(onFailed === undefined ? {} : { onFailed })}
      render={(data, ref) => (
        <CollectionListTemplate ref={ref} data={data} applyRoot={mode === 'window'} />
      )}
    />
  );
}
