//! Deterministic current-snapshot hierarchy and DAG layouts; no retained source,
//! prior order, force simulation, or visited-node history.
use crate::charts::{ChartError, ChartTheme};
use crate::plot_model::*;
use crate::plot_scene::{PlotOptions, PlotScene, rect, ring, stroke};
use glory::render::{GraphicPoint as Point, PathCommand as Cmd};
use std::{
    collections::BTreeMap,
    f64::consts::{FRAC_PI_2, TAU},
};
#[derive(Clone, Copy)]
struct Box2 {
    x: f64,
    y: f64,
    w: f64,
    h: f64,
}
struct Hierarchy<'a> {
    nodes: &'a [HierarchyDatum],
    children: Vec<Vec<usize>>,
    weights: Vec<f64>,
    depths: Vec<usize>,
    theme: &'a ChartTheme,
}
impl Hierarchy<'_> {
    fn treemap(
        &self,
        scene: &mut PlotScene,
        indices: &[usize],
        area: Box2,
        depth: usize,
    ) -> Result<(), ChartError> {
        let total = indices.iter().map(|i| self.weights[*i]).sum::<f64>();
        if total <= 0. {
            return Ok(());
        }
        let mut offset = 0.;
        for index in indices {
            let weight = self.weights[*index];
            if weight <= 0. {
                continue;
            }
            let fraction = weight / total;
            let child = if depth.is_multiple_of(2) {
                Box2 {
                    x: area.x + offset * area.w,
                    y: area.y,
                    w: area.w * fraction,
                    h: area.h,
                }
            } else {
                Box2 {
                    x: area.x,
                    y: area.y + offset * area.h,
                    w: area.w,
                    h: area.h * fraction,
                }
            };
            offset += fraction;
            let node = &self.nodes[*index];
            let mut color = self.theme.series[*index % 8];
            if !self.children[*index].is_empty() {
                color.3 = 80;
            }
            scene.layer(
                &node.key,
                rect(child.x, child.y, child.w, child.h),
                Some(color),
                Some(stroke(self.theme.grid, 1.)),
            )?;
            scene.mark(
                &node.key,
                &node.label,
                format!("Visible leaf weight: {weight}"),
                if self.children[*index].is_empty() {
                    child.x + child.w / 2.
                } else {
                    child.x + 2.
                },
                if self.children[*index].is_empty() {
                    child.y + child.h / 2.
                } else {
                    child.y + 2.
                },
                self.theme.series[*index % 8],
            )?;
            let gap = 3f64.min(child.w / 4.).min(child.h / 4.);
            self.treemap(
                scene,
                &self.children[*index],
                Box2 {
                    x: child.x + gap,
                    y: child.y + gap,
                    w: (child.w - 2. * gap).max(0.),
                    h: (child.h - 2. * gap).max(0.),
                },
                depth + 1,
            )?;
        }
        Ok(())
    }
    fn sunburst(
        &self,
        scene: &mut PlotScene,
        indices: &[usize],
        start: f64,
        sweep: f64,
        depth: usize,
        band: f64,
    ) -> Result<(), ChartError> {
        let total = indices.iter().map(|i| self.weights[*i]).sum::<f64>();
        if total <= 0. {
            return Ok(());
        }
        let (mut angle, cx, cy) = (start, scene.frame.width / 2., scene.frame.height / 2.);
        for index in indices {
            let weight = self.weights[*index];
            if weight <= 0. {
                continue;
            }
            let part = sweep * weight / total;
            let node = &self.nodes[*index];
            let (inner, outer) = (depth as f64 * band, (depth + 1) as f64 * band);
            let color = self.theme.series[*index % 8];
            scene.layer(
                &node.key,
                ring(cx, cy, inner, outer, angle, part),
                Some(color),
                Some(stroke(self.theme.grid, 1.)),
            )?;
            let middle = angle + part / 2.;
            scene.mark(
                &node.key,
                &node.label,
                format!("Visible leaf weight: {weight}"),
                cx + (inner + outer) / 2. * libm::cos(middle),
                cy + (inner + outer) / 2. * libm::sin(middle),
                color,
            )?;
            self.sunburst(scene, &self.children[*index], angle, part, depth + 1, band)?;
            angle += part;
        }
        Ok(())
    }
}
pub(crate) fn hierarchy(
    scene: &mut PlotScene,
    nodes: &[HierarchyDatum],
    kind: PlotKind,
    options: &PlotOptions,
    theme: &ChartTheme,
) -> Result<(), ChartError> {
    let indices = nodes
        .iter()
        .enumerate()
        .map(|(i, r)| (r.key.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    let mut children = vec![Vec::new(); nodes.len()];
    let mut roots = Vec::new();
    let mut depths = vec![0; nodes.len()];
    let mut active = vec![true; nodes.len()];
    for (i, node) in nodes.iter().enumerate() {
        let mut cursor = Some(i);
        while let Some(j) = cursor {
            active[i] &= !options.hidden.contains(&nodes[j].key);
            depths[i] += 1;
            cursor = nodes[j]
                .parent
                .as_ref()
                .and_then(|p| indices.get(p.as_str()).copied());
        }
        if let Some(parent) = &node.parent {
            children[*indices.get(parent.as_str()).expect("admitted parent")].push(i);
        } else {
            roots.push(i);
        }
    }
    let mut weights = vec![0.; nodes.len()];
    let mut order = (0..nodes.len()).collect::<Vec<_>>();
    order.sort_by_key(|i| std::cmp::Reverse(depths[*i]));
    for i in order {
        if active[i] {
            weights[i] = nodes[i]
                .weight
                .unwrap_or_else(|| children[i].iter().map(|j| weights[*j]).sum());
        }
    }
    let model = Hierarchy {
        nodes,
        children,
        weights,
        depths,
        theme,
    };
    if kind == PlotKind::Treemap {
        model.treemap(
            scene,
            &roots,
            Box2 {
                x: 16.,
                y: 16.,
                w: scene.frame.width - 32.,
                h: scene.frame.height - 32.,
            },
            0,
        )?;
    } else {
        let maximum = model
            .depths
            .iter()
            .enumerate()
            .filter(|(i, _)| active[*i])
            .map(|(_, d)| *d)
            .max()
            .unwrap_or(1);
        let band = (scene.frame.width.min(scene.frame.height) - 32.) / 2. / maximum as f64;
        model.sunburst(scene, &roots, -FRAC_PI_2, TAU, 0, band)?;
    }
    Ok(())
}
pub(crate) fn flows(
    scene: &mut PlotScene,
    data: &FlowData,
    options: &PlotOptions,
    theme: &ChartTheme,
) -> Result<(), ChartError> {
    let indices = data
        .nodes
        .iter()
        .enumerate()
        .map(|(i, r)| (r.key.as_str(), i))
        .collect::<BTreeMap<_, _>>();
    let active = data
        .nodes
        .iter()
        .map(|n| !options.hidden.contains(&n.key))
        .collect::<Vec<_>>();
    let links = data
        .links
        .iter()
        .filter_map(|e| {
            let from = *indices.get(e.source.as_str()).expect("admitted source");
            let to = *indices.get(e.target.as_str()).expect("admitted target");
            (active[from] && active[to]).then_some((e, from, to))
        })
        .collect::<Vec<_>>();
    let mut incoming = vec![0f64; data.nodes.len()];
    let mut outgoing = incoming.clone();
    let mut indegree = vec![0usize; data.nodes.len()];
    let mut children = vec![Vec::new(); data.nodes.len()];
    for (e, from, to) in &links {
        incoming[*to] += e.value;
        outgoing[*from] += e.value;
        indegree[*to] += 1;
        children[*from].push(*to);
    }
    let mut levels = vec![0usize; data.nodes.len()];
    let mut ready = (0..data.nodes.len())
        .filter(|i| active[*i] && indegree[*i] == 0)
        .collect::<Vec<_>>();
    while let Some(i) = ready.pop() {
        for child in &children[i] {
            levels[*child] = levels[*child].max(levels[i] + 1);
            indegree[*child] -= 1;
            if indegree[*child] == 0 {
                ready.push(*child);
            }
        }
    }
    let columns = levels
        .iter()
        .enumerate()
        .filter(|(i, _)| active[*i])
        .map(|(_, l)| *l + 1)
        .max()
        .unwrap_or(1);
    let mut ranks = vec![Vec::new(); columns];
    for (i, level) in levels.iter().enumerate() {
        if active[i] {
            ranks[*level].push(i);
        }
    }
    let height = scene.frame.height - 32.;
    let mut scale = f64::INFINITY;
    for rank in &ranks {
        let n = rank.len().max(1) as f64;
        let gap = (height / n / 4.).min(8.);
        let zero = (height / n / 2.).min(8.);
        let zeros = rank
            .iter()
            .filter(|i| incoming[**i].max(outgoing[**i]) == 0.)
            .count();
        let total = rank
            .iter()
            .map(|i| incoming[*i].max(outgoing[*i]))
            .sum::<f64>();
        if total > 0. {
            scale = scale.min((height - gap * (n - 1.) - zero * zeros as f64) / total);
        }
    }
    if !scale.is_finite() {
        scale = 0.;
    }
    let mut boxes = vec![
        Box2 {
            x: 0.,
            y: 0.,
            w: 0.,
            h: 0.
        };
        data.nodes.len()
    ];
    for (level, rank) in ranks.iter().enumerate() {
        let n = rank.len().max(1) as f64;
        let gap = (height / n / 4.).min(8.);
        let zero = (height / n / 2.).min(8.);
        let mut y = 16.;
        for i in rank {
            let weight = incoming[*i].max(outgoing[*i]);
            let h = if weight == 0. { zero } else { weight * scale };
            let x = 16.
                + (scene.frame.width - 48.) * level as f64
                    / (columns.saturating_sub(1)).max(1) as f64;
            boxes[*i] = Box2 { x, y, w: 16., h };
            y += h + gap;
        }
    }
    let mut used_in = vec![0.; data.nodes.len()];
    let mut used_out = used_in.clone();
    for (e, from, to) in &links {
        let thickness = e.value * scale;
        if thickness <= 0. {
            continue;
        }
        let a = boxes[*from];
        let b = boxes[*to];
        let x1 = a.x + a.w;
        let x2 = b.x;
        let mid = (x1 + x2) / 2.;
        let y1 = a.y + used_out[*from];
        let y2 = b.y + used_in[*to];
        used_out[*from] += thickness;
        used_in[*to] += thickness;
        let mut color = theme.series[*from % 8];
        color.3 = 112;
        scene.layer(
            format!("edge-{}-{}", e.key.len(), e.key),
            vec![
                Cmd::Move(Point::new(x1, y1)),
                Cmd::Cubic {
                    first: Point::new(mid, y1),
                    second: Point::new(mid, y2),
                    end: Point::new(x2, y2),
                },
                Cmd::Line(Point::new(x2, y2 + thickness)),
                Cmd::Cubic {
                    first: Point::new(mid, y2 + thickness),
                    second: Point::new(mid, y1 + thickness),
                    end: Point::new(x1, y1 + thickness),
                },
                Cmd::Close,
            ],
            Some(color),
            None,
        )?;
    }
    for (i, node) in data.nodes.iter().enumerate() {
        if !active[i] {
            continue;
        }
        let b = boxes[i];
        let color = theme.series[i % 8];
        scene.layer(
            format!("node-{}-{}", node.key.len(), node.key),
            rect(b.x, b.y, b.w, b.h),
            Some(color),
            None,
        )?;
        scene.mark(
            &node.key,
            &node.label,
            format!("Incoming {}, outgoing {}", incoming[i], outgoing[i]),
            b.x + b.w / 2.,
            b.y + b.h / 2.,
            color,
        )?;
    }
    Ok(())
}
