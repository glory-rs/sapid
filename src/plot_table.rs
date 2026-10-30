//! Exact values alongside current plot details. Table chunks respect the
//! existing intrinsic table budget and never omit a record from the snapshot.
use crate::plot_model::*;
use crate::plot_scene::PlotScene;
use std::collections::BTreeMap;
#[derive(Clone, Debug, PartialEq)]
pub(crate) struct PlotTable {
    pub caption: String,
    pub columns: Vec<&'static str>,
    pub rows: Vec<Vec<String>>,
}
pub(crate) fn tables(data: &PlotData, scene: &PlotScene) -> Vec<PlotTable> {
    #[cfg(test)]
    FORMAT_PASSES.with(|count| count.set(count.get() + 1));
    let details = scene
        .marks
        .iter()
        .map(|m| (m.key.as_str(), m.value_text.as_str()))
        .collect::<BTreeMap<_, _>>();
    let detail = |key: &str| {
        details
            .get(key)
            .copied()
            .unwrap_or("No visible mark")
            .to_owned()
    };
    let mut tables = match data {
        PlotData::Values(values) => vec![PlotTable {
            caption: "Plot values".into(),
            columns: vec!["Label", "Value", "Visible plot detail"],
            rows: values
                .iter()
                .map(|r| {
                    vec![
                        r.label.clone(),
                        r.value.map_or_else(|| "No value".into(), |v| v.to_string()),
                        detail(&r.key),
                    ]
                })
                .collect(),
        }],
        PlotData::Points(values) | PlotData::SampledPoints { points: values, .. } => {
            vec![PlotTable {
                caption: match data {
                    PlotData::SampledPoints { source_count, .. } => {
                        format!("Plot points ({} of {source_count} sampled)", values.len())
                    }
                    _ => "Plot points".into(),
                },
                columns: vec!["Label", "X", "Y", "Size"],
                rows: values
                    .iter()
                    .map(|r| {
                        vec![
                            r.label.clone(),
                            r.x.to_string(),
                            r.y.to_string(),
                            r.size.to_string(),
                        ]
                    })
                    .collect(),
            }]
        }
        PlotData::Candles(values) => vec![PlotTable {
            caption: "OHLC values".into(),
            columns: vec!["Label", "Open", "High", "Low", "Close", "Change"],
            rows: values
                .iter()
                .map(|r| {
                    vec![
                        r.label.clone(),
                        r.open.to_string(),
                        r.high.to_string(),
                        r.low.to_string(),
                        r.close.to_string(),
                        if r.close > r.open {
                            "Up"
                        } else if r.close < r.open {
                            "Down"
                        } else {
                            "Unchanged"
                        }
                        .into(),
                    ]
                })
                .collect(),
        }],
        PlotData::Matrix(data) => {
            let rows = data
                .rows
                .iter()
                .map(|r| (r.key.as_str(), r.label.as_str()))
                .collect::<BTreeMap<_, _>>();
            let columns = data
                .columns
                .iter()
                .map(|c| (c.key.as_str(), c.label.as_str()))
                .collect::<BTreeMap<_, _>>();
            vec![PlotTable {
                caption: "Matrix values".into(),
                columns: vec!["Row", "Column", "Value"],
                rows: data
                    .cells
                    .iter()
                    .map(|r| {
                        vec![
                            rows[r.row.as_str()].into(),
                            columns[r.column.as_str()].into(),
                            r.value.to_string(),
                        ]
                    })
                    .collect(),
            }]
        }
        PlotData::Targets(values) => vec![PlotTable {
            caption: "Target values".into(),
            columns: vec![
                "Label",
                "Minimum",
                "Maximum",
                "Value",
                "Target",
                "Band boundaries",
            ],
            rows: values
                .iter()
                .map(|r| {
                    vec![
                        r.label.clone(),
                        r.min.to_string(),
                        r.max.to_string(),
                        r.value.to_string(),
                        r.target
                            .map_or_else(|| "No target".into(), |v| v.to_string()),
                        r.bands
                            .iter()
                            .map(f64::to_string)
                            .collect::<Vec<_>>()
                            .join(", "),
                    ]
                })
                .collect(),
        }],
        PlotData::Hierarchy(values) => {
            let labels = values
                .iter()
                .map(|r| (r.key.as_str(), r.label.as_str()))
                .collect::<BTreeMap<_, _>>();
            vec![PlotTable {
                caption: "Hierarchy values".into(),
                columns: vec!["Label", "Parent", "Leaf weight", "Visible plot detail"],
                rows: values
                    .iter()
                    .map(|r| {
                        vec![
                            r.label.clone(),
                            r.parent
                                .as_ref()
                                .map_or_else(|| "Root".into(), |p| labels[p.as_str()].into()),
                            r.weight.map_or_else(
                                || "Derived from descendants".into(),
                                |v| v.to_string(),
                            ),
                            detail(&r.key),
                        ]
                    })
                    .collect(),
            }]
        }
        PlotData::Flows(data) => {
            let labels = data
                .nodes
                .iter()
                .map(|r| (r.key.as_str(), r.label.as_str()))
                .collect::<BTreeMap<_, _>>();
            let mut incoming = data
                .nodes
                .iter()
                .map(|r| (r.key.as_str(), 0f64))
                .collect::<BTreeMap<_, _>>();
            let mut outgoing = incoming.clone();
            for e in &data.links {
                *incoming
                    .get_mut(e.target.as_str())
                    .expect("admitted target") += e.value;
                *outgoing
                    .get_mut(e.source.as_str())
                    .expect("admitted source") += e.value;
            }
            vec![
                PlotTable {
                    caption: "Flow nodes".into(),
                    columns: vec!["Label", "Incoming", "Outgoing", "Visible plot detail"],
                    rows: data
                        .nodes
                        .iter()
                        .map(|r| {
                            vec![
                                r.label.clone(),
                                incoming[r.key.as_str()].to_string(),
                                outgoing[r.key.as_str()].to_string(),
                                detail(&r.key),
                            ]
                        })
                        .collect(),
                },
                PlotTable {
                    caption: "Flow links".into(),
                    columns: vec!["Source", "Target", "Value"],
                    rows: data
                        .links
                        .iter()
                        .map(|e| {
                            vec![
                                labels[e.source.as_str()].into(),
                                labels[e.target.as_str()].into(),
                                e.value.to_string(),
                            ]
                        })
                        .collect(),
                },
            ]
        }
    };
    let mut chunks = Vec::new();
    for table in tables.drain(..) {
        if table.rows.is_empty() {
            chunks.push(table);
            continue;
        }
        let count = table.rows.len();
        for (index, rows) in table.rows.chunks(128).enumerate() {
            chunks.push(PlotTable {
                caption: if count > 128 {
                    format!(
                        "{} ({}–{})",
                        table.caption,
                        index * 128 + 1,
                        (index * 128 + rows.len()).min(count)
                    )
                } else {
                    table.caption.clone()
                },
                columns: table.columns.clone(),
                rows: rows.to_vec(),
            });
        }
    }
    chunks
}

#[cfg(test)]
thread_local! { pub(super) static FORMAT_PASSES: std::cell::Cell<usize> = const { std::cell::Cell::new(0) }; }
