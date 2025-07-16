use crate::database::{QueryResult, QueryValue};
use crate::ui::theme::get_table_colors;
use egui::{ScrollArea, Sense, Ui};

/// Result table for displaying query results
pub struct ResultTable {
    result: Option<QueryResult>,
    selected_row: Option<usize>,
    selected_column: Option<usize>,
    show_row_numbers: bool,
    max_cell_width: f32,
    page_size: usize,
    current_page: usize,
}

impl ResultTable {
    pub fn new() -> Self {
        Self {
            result: None,
            selected_row: None,
            selected_column: None,
            show_row_numbers: true,
            max_cell_width: 200.0,
            page_size: 100,
            current_page: 0,
        }
    }

    pub fn render(&mut self, ui: &mut Ui) {
        ui.vertical(|ui| {
            // Toolbar
            self.render_toolbar(ui);
            ui.separator();

            // Table content
            if let Some(result) = self.result.clone() {
                self.render_table(ui, &result);
            } else {
                ui.centered_and_justified(|ui| {
                    ui.label("No query results to display");
                });
            }
        });
    }

    fn render_toolbar(&mut self, ui: &mut Ui) {
        ui.horizontal(|ui| {
            if let Some(result) = &self.result {
                ui.label(format!("Rows: {}", result.row_count()));
                ui.separator();
                ui.label(format!("Columns: {}", result.column_count()));

                if let Some(execution_time) = result.execution_time {
                    ui.separator();
                    ui.label(format!("Time: {:.2}ms", execution_time.as_millis()));
                }

                if let Some(rows_affected) = result.rows_affected {
                    ui.separator();
                    ui.label(format!("Affected: {}", rows_affected));
                }
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                ui.checkbox(&mut self.show_row_numbers, "Row Numbers");

                if ui.button("Export").clicked() {
                    // TODO: Implement export functionality
                }

                if ui.button("Copy").clicked() {
                    // TODO: Implement copy functionality
                }
            });
        });

        // Pagination controls
        if let Some(result) = &self.result {
            if self.page_size > 0 && result.row_count() > self.page_size {
                ui.horizontal(|ui| {
                    let total_pages = (result.row_count() + self.page_size - 1) / self.page_size;

                    if ui.button("◀◀").clicked() {
                        self.current_page = 0;
                    }

                    if ui.button("◀").clicked() && self.current_page > 0 {
                        self.current_page -= 1;
                    }

                    ui.label(format!("Page {} of {}", self.current_page + 1, total_pages));

                    if ui.button("▶").clicked() && self.current_page < total_pages - 1 {
                        self.current_page += 1;
                    }

                    if ui.button("▶▶").clicked() {
                        self.current_page = total_pages - 1;
                    }

                    ui.separator();
                    ui.label("Page size:");
                    ui.add(egui::DragValue::new(&mut self.page_size).range(10..=1000));
                });
            }
        }
    }

    fn render_table(&mut self, ui: &mut Ui, result: &QueryResult) {
        let _colors = get_table_colors();

        ScrollArea::both()
            .auto_shrink([false, false])
            .id_source("result_table_scroll")
            .show(ui, |ui| {
                egui_extras::TableBuilder::new(ui)
                    .striped(true)
                    .resizable(true)
                    .cell_layout(egui::Layout::left_to_right(egui::Align::Center))
                    .columns(
                        egui_extras::Column::auto().resizable(true),
                        if self.show_row_numbers { 1 } else { 0 } + result.column_count(),
                    )
                    .header(25.0, |mut header| {
                        // Row number header
                        if self.show_row_numbers {
                            header.col(|ui| {
                                ui.strong("#");
                            });
                        }

                        // Column headers
                        for (i, column) in result.columns.iter().enumerate() {
                            header.col(|ui| {
                                let response = ui.button(&column.name);
                                if response.clicked() {
                                    self.selected_column = Some(i);
                                    // TODO: Implement column sorting
                                }

                                // Show data type on hover
                                response.on_hover_text(&column.data_type);
                            });
                        }
                    })
                    .body(|mut body| {
                        // Ensure page_size is never 0 to prevent division by zero
                        let page_size = if self.page_size == 0 {
                            100
                        } else {
                            self.page_size
                        };
                        let start_row = self.current_page * page_size;
                        let end_row = (start_row + page_size).min(result.row_count());

                        for row_idx in start_row..end_row {
                            let actual_row_idx = row_idx;
                            body.row(20.0, |mut row| {
                                // Row number
                                if self.show_row_numbers {
                                    row.col(|ui| {
                                        let response = ui.selectable_label(
                                            self.selected_row == Some(actual_row_idx),
                                            format!("{}", actual_row_idx + 1),
                                        );
                                        if response.clicked() {
                                            self.selected_row = Some(actual_row_idx);
                                        }
                                    });
                                }

                                // Data cells
                                if let Some(query_row) = result.rows.get(actual_row_idx) {
                                    for (col_idx, value) in query_row.values.iter().enumerate() {
                                        row.col(|ui| {
                                            let is_selected = self.selected_row
                                                == Some(actual_row_idx)
                                                && self.selected_column == Some(col_idx);

                                            let text = self.format_cell_value(value);
                                            let response = ui.selectable_label(is_selected, text);

                                            if response.clicked() {
                                                self.selected_row = Some(actual_row_idx);
                                                self.selected_column = Some(col_idx);
                                            }

                                            // Show full value on hover for long text
                                            if value.to_display_string().len() > 50 {
                                                response.on_hover_text(&value.to_display_string());
                                            }
                                        });
                                    }
                                }
                            });
                        }
                    });
            });
    }

    fn format_cell_value(&self, value: &QueryValue) -> String {
        let display = value.to_display_string();

        // Truncate long values
        if display.len() > 100 {
            format!("{}...", &display[..97])
        } else {
            display
        }
    }

    pub fn set_result(&mut self, result: QueryResult) {
        self.result = Some(result);
        self.selected_row = None;
        self.selected_column = None;
        self.current_page = 0;
    }

    pub fn set_page_size(&mut self, page_size: usize) {
        // Ensure page_size is never 0
        self.page_size = if page_size == 0 { 100 } else { page_size };
        self.current_page = 0; // Reset to first page when changing page size
    }

    pub fn clear(&mut self) {
        self.result = None;
        self.selected_row = None;
        self.selected_column = None;
        self.current_page = 0;
    }

    pub fn get_selected_value(&self) -> Option<&QueryValue> {
        if let (Some(result), Some(row_idx), Some(col_idx)) =
            (&self.result, self.selected_row, self.selected_column)
        {
            result.get_value(row_idx, col_idx)
        } else {
            None
        }
    }

    pub fn export_to_csv(&self) -> Option<String> {
        if let Some(result) = &self.result {
            let mut csv = String::new();

            // Headers
            let headers: Vec<String> = result
                .columns
                .iter()
                .map(|col| format!("\"{}\"", col.name.replace("\"", "\"\"")))
                .collect();
            csv.push_str(&headers.join(","));
            csv.push('\n');

            // Data rows
            for row in &result.rows {
                let values: Vec<String> = row
                    .values
                    .iter()
                    .map(|val| {
                        let display = val.to_display_string();
                        format!("\"{}\"", display.replace("\"", "\"\""))
                    })
                    .collect();
                csv.push_str(&values.join(","));
                csv.push('\n');
            }

            Some(csv)
        } else {
            None
        }
    }
}

impl Default for ResultTable {
    fn default() -> Self {
        Self {
            result: None,
            selected_row: None,
            selected_column: None,
            show_row_numbers: true,
            max_cell_width: 200.0,
            page_size: 100, // Ensure page_size is never 0
            current_page: 0,
        }
    }
}
