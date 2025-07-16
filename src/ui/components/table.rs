use egui::Ui;

/// Custom table component for data display
pub struct DataTable {
    headers: Vec<String>,
    rows: Vec<Vec<String>>,
    selected_row: Option<usize>,
    sortable: bool,
}

impl DataTable {
    pub fn new() -> Self {
        Self {
            headers: Vec::new(),
            rows: Vec::new(),
            selected_row: None,
            sortable: true,
        }
    }

    pub fn with_headers(mut self, headers: Vec<String>) -> Self {
        self.headers = headers;
        self
    }

    pub fn with_rows(mut self, rows: Vec<Vec<String>>) -> Self {
        self.rows = rows;
        self
    }

    pub fn sortable(mut self, sortable: bool) -> Self {
        self.sortable = sortable;
        self
    }

    pub fn render(&mut self, ui: &mut Ui) {
        // TODO: Implement custom table rendering
        ui.label("Custom table component - TODO");
    }

    pub fn get_selected_row(&self) -> Option<usize> {
        self.selected_row
    }

    pub fn set_selected_row(&mut self, row: Option<usize>) {
        self.selected_row = row;
    }
}

impl Default for DataTable {
    fn default() -> Self {
        Self::new()
    }
}
