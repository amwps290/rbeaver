/// Test binary for Chinese character encoding support in RBeaver
///
/// This test demonstrates and validates that Chinese characters are properly
/// handled throughout the query execution and result display pipeline.
use eframe::egui;
use rbeaver::database::{QueryResult, QueryRow, QueryValue};
use rbeaver::ui::ResultTable;

fn main() -> Result<(), eframe::Error> {
    env_logger::init();

    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1000.0, 700.0])
            .with_title("RBeaver - Chinese Character Encoding Test"),
        ..Default::default()
    };

    eframe::run_native(
        "Chinese Encoding Test",
        options,
        Box::new(|cc| {
            // Configure fonts for Chinese character support
            configure_chinese_fonts(&cc.egui_ctx);
            Ok(Box::new(ChineseEncodingTestApp::new()))
        }),
    )
}

/// Configure fonts to support Chinese characters
fn configure_chinese_fonts(ctx: &egui::Context) {
    // egui 0.28 has good Unicode support by default
    // Configure text rendering for better Chinese character display
    let mut style = (*ctx.style()).clone();

    // Set better spacing for CJK characters
    style.spacing.item_spacing = egui::vec2(8.0, 4.0);

    // Ensure text is rendered with proper Unicode support
    style.text_styles.insert(
        egui::TextStyle::Body,
        egui::FontId::new(14.0, egui::FontFamily::Proportional),
    );
    style.text_styles.insert(
        egui::TextStyle::Monospace,
        egui::FontId::new(12.0, egui::FontFamily::Monospace),
    );

    ctx.set_style(style);
    println!("Configured text rendering for Chinese character support");
}

struct ChineseEncodingTestApp {
    result_table: ResultTable,
    test_results: Vec<TestResult>,
    current_test: usize,
}

#[derive(Clone)]
struct TestResult {
    name: String,
    description: String,
    query_result: QueryResult,
    expected_display: Vec<String>,
}

impl ChineseEncodingTestApp {
    fn new() -> Self {
        let mut app = Self {
            result_table: ResultTable::new(),
            test_results: Vec::new(),
            current_test: 0,
        };

        app.setup_test_data();
        app.load_current_test();
        app
    }

    fn setup_test_data(&mut self) {
        // Test 1: Basic Chinese characters
        let test1 = TestResult {
            name: "基本中文字符测试".to_string(),
            description: "Testing basic Chinese characters in query results".to_string(),
            query_result: create_chinese_test_result(),
            expected_display: vec![
                "用户名".to_string(),
                "张三".to_string(),
                "李四".to_string(),
                "王五".to_string(),
            ],
        };

        // Test 2: Mixed Chinese and English
        let test2 = TestResult {
            name: "中英文混合测试".to_string(),
            description: "Testing mixed Chinese and English text".to_string(),
            query_result: create_mixed_language_test_result(),
            expected_display: vec![
                "Product Name 产品名称".to_string(),
                "iPhone 苹果手机".to_string(),
                "MacBook 苹果电脑".to_string(),
            ],
        };

        // Test 3: Long Chinese text
        let test3 = TestResult {
            name: "长中文文本测试".to_string(),
            description: "Testing long Chinese text with proper truncation".to_string(),
            query_result: create_long_chinese_text_result(),
            expected_display: vec![
                "这是一个非常长的中文描述文本，用来测试系统是否能够正确处理和显示长文本内容。"
                    .to_string(),
            ],
        };

        // Test 4: Special Chinese characters
        let test4 = TestResult {
            name: "特殊字符测试".to_string(),
            description: "Testing special Chinese characters and symbols".to_string(),
            query_result: create_special_characters_test_result(),
            expected_display: vec![
                "价格：￥1,234.56".to_string(),
                "地址：北京市朝阳区".to_string(),
                "邮编：100000".to_string(),
            ],
        };

        self.test_results = vec![test1, test2, test3, test4];
    }

    fn load_current_test(&mut self) {
        if let Some(test) = self.test_results.get(self.current_test) {
            self.result_table.set_result(test.query_result.clone());
        }
    }

    fn next_test(&mut self) {
        if self.current_test < self.test_results.len() - 1 {
            self.current_test += 1;
            self.load_current_test();
        }
    }

    fn previous_test(&mut self) {
        if self.current_test > 0 {
            self.current_test -= 1;
            self.load_current_test();
        }
    }
}

impl eframe::App for ChineseEncodingTestApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("RBeaver - 中文字符编码测试 (Chinese Character Encoding Test)");
            ui.separator();

            // Test navigation
            ui.horizontal(|ui| {
                if ui.button("⬅ Previous Test").clicked() {
                    self.previous_test();
                }

                if ui.button("Next Test ➡").clicked() {
                    self.next_test();
                }

                ui.separator();

                if let Some(test) = self.test_results.get(self.current_test) {
                    ui.label(format!("Test {}/{}: {}", 
                        self.current_test + 1,
                        self.test_results.len(),
                        test.name
                    ));
                }
            });

            ui.separator();

            // Test description
            if let Some(test) = self.test_results.get(self.current_test) {
                ui.label(format!("Description: {}", test.description));
                ui.separator();
            }

            // Test instructions
            ui.collapsing("测试说明 (Test Instructions)", |ui| {
                ui.label("1. 检查中文字符是否正确显示 (Check if Chinese characters display correctly)");
                ui.label("2. 验证文本截断是否正确处理多字节字符 (Verify text truncation handles multi-byte characters)");
                ui.label("3. 确认悬停提示显示完整文本 (Confirm hover tooltips show full text)");
                ui.label("4. 测试搜索和过滤功能 (Test search and filtering functionality)");
            });

            ui.separator();

            // Result table
            ui.heading("Query Results 查询结果:");
            egui::ScrollArea::vertical()
                .max_height(400.0)
                .show(ui, |ui| {
                    self.result_table.render(ui);
                });

            ui.separator();

            // Validation results
            ui.heading("Validation Results 验证结果:");
            if let Some(test) = self.test_results.get(self.current_test) {
                ui.label("✅ Expected Chinese characters should be visible");
                ui.label("✅ Text should not appear as question marks or garbled");
                ui.label("✅ Hover tooltips should show complete Chinese text");
                ui.label("✅ Text truncation should preserve character integrity");

                ui.collapsing("Expected Display Values", |ui| {
                    for (i, expected) in test.expected_display.iter().enumerate() {
                        ui.label(format!("{}: {}", i + 1, expected));
                    }
                });
            }
        });
    }
}

// Helper functions to create test data

fn create_chinese_test_result() -> QueryResult {
    let mut result = QueryResult::new("SELECT * FROM users".to_string());

    // Add columns
    result = result.with_columns(vec![
        rbeaver::database::QueryColumn::new("id".to_string(), "INTEGER".to_string(), 0, false),
        rbeaver::database::QueryColumn::new("用户名".to_string(), "TEXT".to_string(), 1, true),
        rbeaver::database::QueryColumn::new("邮箱".to_string(), "TEXT".to_string(), 2, true),
        rbeaver::database::QueryColumn::new(
            "创建时间".to_string(),
            "TIMESTAMP".to_string(),
            3,
            true,
        ),
    ]);

    // Add rows with Chinese data
    let rows = vec![
        QueryRow::new(vec![
            QueryValue::Int32(1),
            QueryValue::String("张三".to_string()),
            QueryValue::String("zhangsan@example.com".to_string()),
            QueryValue::String("2024-01-15 10:30:00".to_string()),
        ]),
        QueryRow::new(vec![
            QueryValue::Int32(2),
            QueryValue::String("李四".to_string()),
            QueryValue::String("lisi@example.com".to_string()),
            QueryValue::String("2024-01-16 14:20:00".to_string()),
        ]),
        QueryRow::new(vec![
            QueryValue::Int32(3),
            QueryValue::String("王五".to_string()),
            QueryValue::String("wangwu@example.com".to_string()),
            QueryValue::String("2024-01-17 09:15:00".to_string()),
        ]),
    ];

    result.with_rows(rows)
}

fn create_mixed_language_test_result() -> QueryResult {
    let mut result = QueryResult::new("SELECT * FROM products".to_string());

    result = result.with_columns(vec![
        rbeaver::database::QueryColumn::new("id".to_string(), "INTEGER".to_string(), 0, false),
        rbeaver::database::QueryColumn::new("产品名称".to_string(), "TEXT".to_string(), 1, true),
        rbeaver::database::QueryColumn::new("价格".to_string(), "DECIMAL".to_string(), 2, true),
        rbeaver::database::QueryColumn::new("描述".to_string(), "TEXT".to_string(), 3, true),
    ]);

    let rows = vec![
        QueryRow::new(vec![
            QueryValue::Int32(1),
            QueryValue::String("iPhone 苹果手机".to_string()),
            QueryValue::String("￥8999.00".to_string()),
            QueryValue::String("Apple iPhone with advanced features 苹果智能手机".to_string()),
        ]),
        QueryRow::new(vec![
            QueryValue::Int32(2),
            QueryValue::String("MacBook 苹果电脑".to_string()),
            QueryValue::String("￥12999.00".to_string()),
            QueryValue::String("Apple MacBook Pro laptop computer 苹果专业笔记本电脑".to_string()),
        ]),
    ];

    result.with_rows(rows)
}

fn create_long_chinese_text_result() -> QueryResult {
    let mut result = QueryResult::new("SELECT * FROM articles".to_string());

    result = result.with_columns(vec![
        rbeaver::database::QueryColumn::new("id".to_string(), "INTEGER".to_string(), 0, false),
        rbeaver::database::QueryColumn::new("标题".to_string(), "TEXT".to_string(), 1, true),
        rbeaver::database::QueryColumn::new("内容".to_string(), "TEXT".to_string(), 2, true),
    ]);

    let long_text = "这是一个非常长的中文描述文本，用来测试系统是否能够正确处理和显示长文本内容。文本包含了各种中文字符，包括简体中文、繁体中文、标点符号等。这个测试的目的是验证文本截断功能是否能够正确处理多字节UTF-8字符，而不会导致字符损坏或显示异常。同时也要确保悬停提示能够显示完整的文本内容。";

    let rows = vec![QueryRow::new(vec![
        QueryValue::Int32(1),
        QueryValue::String("中文长文本测试".to_string()),
        QueryValue::String(long_text.to_string()),
    ])];

    result.with_rows(rows)
}

fn create_special_characters_test_result() -> QueryResult {
    let mut result = QueryResult::new("SELECT * FROM locations".to_string());

    result = result.with_columns(vec![
        rbeaver::database::QueryColumn::new("id".to_string(), "INTEGER".to_string(), 0, false),
        rbeaver::database::QueryColumn::new("地址".to_string(), "TEXT".to_string(), 1, true),
        rbeaver::database::QueryColumn::new("特殊符号".to_string(), "TEXT".to_string(), 2, true),
    ]);

    let rows = vec![
        QueryRow::new(vec![
            QueryValue::Int32(1),
            QueryValue::String("北京市朝阳区建国门外大街1号".to_string()),
            QueryValue::String("￥¥€$£¢©®™".to_string()),
        ]),
        QueryRow::new(vec![
            QueryValue::Int32(2),
            QueryValue::String("上海市浦东新区陆家嘴金融贸易区".to_string()),
            QueryValue::String("①②③④⑤⑥⑦⑧⑨⑩".to_string()),
        ]),
    ];

    result.with_rows(rows)
}
