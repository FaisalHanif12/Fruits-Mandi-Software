// What a chart is drawn from (PRD FR-18.3). The screen makes the figures into numbers for the
// drawing (`chartValueOfPaisa` in shared/format) and into text for every label (`formatPaisa`); the
// chart adds up nothing and writes no amount of its own (CLAUDE.md rule 1).

/** One series of bars: a name for the legend, a height and a written figure for each category. */
export interface BarSeries {
  /** The words of the legend, already in the app language. */
  name: string;
  /** The height of the bar of each category, for the drawing only. `null` is no bar at all. */
  values: readonly (number | null)[];
  /** The figure written above (or in the tooltip of) each bar, from `formatPaisa` or the backend. */
  labels: readonly string[];
  /**
   * The stack this series belongs to when the chart is stacked. Series with the same name are one stack;
   * series with different names stand side by side. Default: one stack for all (`stacked`).
   */
  stack?: string | undefined;
  /**
   * A scenario, not a result: drawn as an empty bar with a dashed outline, and in the legend as a
   * dashed box, so it is never taken for a figure of the books. Default: false.
   */
  scenario?: boolean | undefined;
}

export interface BarChartData {
  /** The names along the axis, in the order of the table (already in the app language). */
  categories: readonly string[];
  series: readonly BarSeries[];
  /** One stack of bars for each category (the parts of a total), or bars side by side. */
  stacked: boolean;
  /**
   * Many thin bars (the days of a month): the figures are not written on the bars and the axis shows
   * only the names that fit. The tooltip still has every figure. Default: false.
   */
  dense?: boolean | undefined;
}

/** The look of a chart, read from the design tokens at the time it is drawn (Code Standards F9). */
export interface ChartStyle {
  direction: 'rtl' | 'ltr';
  fontFamily: string;
  /** One colour for each series, in order; repeated when there are more series. */
  colors: readonly string[];
  ink: string;
  inkMuted: string;
  border: string;
  surface: string;
}
