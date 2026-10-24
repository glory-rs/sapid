//! Optional finite charts. Data, scales and geometry remain host independent;
//! drawing uses Glory's existing portable vector protocol.
pub use crate::chart_aux::{Legend, LegendItem, Tooltip, legend, tooltip};
pub use crate::chart_model::{
    ChartBrush, ChartCategory, ChartData, ChartDomain, ChartError, ChartSeries,
    ChartSeriesKind, MAX_CHART_POINTS, MAX_CHART_SERIES, MAX_CHART_TEXT_BYTES, MAX_CHART_VALUE,
};
pub use crate::chart_scene::{
    CartesianKind, CartesianOptions, CartesianScene, ChartFrame, ChartLabel, ChartLocation,
    ChartPath, ChartTheme, ChartTick,
};
pub use crate::chart_widget::{
    Chart, ChartPointDetails, ChartSelection, area_chart, bar_chart, chart, composite_chart,
    line_chart,
};
pub use crate::plot_model::{
    CandleDatum, FlowData, FlowLink, HierarchyDatum, MAX_FLOW_LINKS, MAX_MATRIX_AXIS,
    MAX_MATRIX_CELLS, MAX_PLOT_DEPTH, MAX_PLOT_RECORDS, MatrixCell, MatrixData, PlotData,
    PlotKind, PointDatum, TargetDatum, ValueDatum,
};
pub use crate::plot_scene::{
    PlotLabel, PlotMark, PlotOptions, PlotRange, PlotScaleStop, PlotScene,
};
pub use crate::plot_widget::{
    MAX_PLOT_KEY_BYTES, Plot, bars_list, bubble_chart, bullet_chart, candlestick_chart,
    donut_chart, funnel_chart, gauge_chart, heatmap, matrix_chart, pie_chart, plot,
    radar_chart, radial_bar_chart, sankey_chart, scatter_chart, sparkline, sunburst_chart,
    treemap, waffle_chart,
};
