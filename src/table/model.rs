//! Finite placement of current read-only table cells, before any content factory.
use glory::ui::Result;
pub const MAX_TABLE_ROWS: usize = 256;
pub const MAX_TABLE_COLUMNS: usize = 64;
pub const MAX_TABLE_CELLS: usize = 4096;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) struct Span {
    pub rows: u16,
    pub columns: u16,
}
pub(super) struct Grid {
    pub columns: usize,
    pub cells: Vec<Vec<u16>>,
}
fn invalid(message: &'static str) -> glory::ui::Error {
    glory::render::Error::InvalidProps(message).into()
}
pub(super) fn grid(rows: &[Vec<Span>]) -> Result<Grid> {
    if rows.len() > MAX_TABLE_ROWS || rows.iter().map(Vec::len).sum::<usize>() > MAX_TABLE_CELLS
    {
        return Err(invalid("table exceeds its live row/cell budget"));
    }
    let mut occupied = [0u16; MAX_TABLE_COLUMNS];
    let mut columns = 0;
    let mut placed = Vec::with_capacity(rows.len());
    for (index, row) in rows.iter().enumerate() {
        let mut cursor = 0;
        let mut cells = Vec::with_capacity(row.len());
        for span in row {
            if span.rows == 0
                || span.columns == 0
                || usize::from(span.rows) > rows.len() - index
            {
                return Err(invalid("table span must remain within its row group"));
            }
            while cursor < MAX_TABLE_COLUMNS && occupied[cursor] > 0 {
                cursor += 1;
            }
            let end = cursor
                .checked_add(usize::from(span.columns))
                .ok_or_else(|| invalid("table column capacity"))?;
            if end > MAX_TABLE_COLUMNS || occupied[cursor..end].iter().any(|value| *value > 0) {
                return Err(invalid(
                    "table cell overlaps a row span or exceeds 64 columns",
                ));
            }
            cells.push(cursor as u16);
            occupied[cursor..end].fill(span.rows);
            columns = columns.max(end);
            cursor = end;
        }
        placed.push(cells);
        for value in &mut occupied {
            *value = value.saturating_sub(1);
        }
    }
    Ok(Grid {
        columns,
        cells: placed,
    })
}
#[cfg(test)]
mod tests {
    use super::*;
    fn span(rows: u16, columns: u16) -> Span {
        Span { rows, columns }
    }
    #[test]
    fn row_spans_skip_only_current_occupied_columns_without_filling_holes() {
        let value = grid(&[
            vec![span(3, 1), span(1, 2)],
            vec![span(1, 1), span(2, 1)],
            vec![span(1, 1)],
        ])
        .unwrap();
        assert_eq!(value.columns, 3);
        assert_eq!(value.cells, [vec![0, 1], vec![1, 2], vec![1]]);
    }
    #[test]
    fn malformed_or_excessive_current_tables_fail_before_rendering() {
        for rows in [
            vec![vec![span(0, 1)]],
            vec![vec![span(1, 0)]],
            vec![vec![span(2, 1)]],
            vec![vec![span(1, 65)]],
            vec![vec![span(1, 1)]; 257],
            vec![vec![span(1, 1); 17]; 256],
        ] {
            assert!(grid(&rows).is_err());
        }
    }
    #[test]
    fn independent_row_group_plans_do_not_leak_header_spans_into_body() {
        let head = grid(&[vec![span(2, 1), span(1, 1)], vec![span(1, 1)]]).unwrap();
        let body = grid(&[vec![span(1, 1), span(1, 1)]]).unwrap();
        assert_eq!(head.cells, [vec![0, 1], vec![1]]);
        assert_eq!(body.cells, [vec![0, 1]]);
    }
}
