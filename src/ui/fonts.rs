use egui::{Context, FontData, FontDefinitions, FontFamily, FontId, TextStyle};
use std::path::Path;
use std::sync::Arc;

/// Font loading error types
#[derive(Debug)]
pub enum FontError {
    IoError(std::io::Error),
    InvalidFont(String),
    NoFontsFound,
}

impl std::fmt::Display for FontError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FontError::IoError(e) => write!(f, "Font I/O error: {}", e),
            FontError::InvalidFont(msg) => write!(f, "Invalid font: {}", msg),
            FontError::NoFontsFound => write!(f, "No suitable Chinese fonts found"),
        }
    }
}

impl std::error::Error for FontError {}

impl From<std::io::Error> for FontError {
    fn from(error: std::io::Error) -> Self {
        FontError::IoError(error)
    }
}

/// Setup Chinese fonts for the egui context
pub fn setup_chinese_fonts(ctx: &Context) -> Result<(), FontError> {
    log::info!("Setting up Chinese font support");

    let mut fonts = FontDefinitions::default();
    let mut chinese_font_loaded = false;

    // Try to load Chinese fonts from system paths
    let chinese_font_paths = get_chinese_font_paths();

    for font_path in chinese_font_paths {
        if let Ok(font_data) = load_font_from_path(&font_path) {
            log::info!("Successfully loaded Chinese font from: {}", font_path);

            // Add the font to the font definitions
            fonts
                .font_data
                .insert("chinese_font".to_owned(), Arc::new(font_data));

            // Add Chinese font to the proportional family (for UI text)
            fonts
                .families
                .entry(FontFamily::Proportional)
                .or_default()
                .insert(0, "chinese_font".to_owned());

            // Add Chinese font to the monospace family (for code/query editor)
            fonts
                .families
                .entry(FontFamily::Monospace)
                .or_default()
                .insert(0, "chinese_font".to_owned());

            chinese_font_loaded = true;
            break;
        } else {
            log::debug!("Failed to load font from: {}", font_path);
        }
    }

    if !chinese_font_loaded {
        log::warn!("No Chinese fonts found, using default fonts with Unicode support");
        // Even without Chinese fonts, egui 0.32 has good Unicode support
        // We'll configure it for better CJK character rendering
    }

    // Apply font definitions to context
    ctx.set_fonts(fonts);

    // Configure text styles for better Chinese character rendering
    configure_text_styles(ctx);

    if chinese_font_loaded {
        log::info!("Chinese font support configured successfully");
    } else {
        log::info!("Using default Unicode fonts for Chinese character support");
    }

    Ok(())
}

/// Get potential Chinese font paths based on the operating system
fn get_chinese_font_paths() -> Vec<String> {
    let mut paths = Vec::new();

    #[cfg(target_os = "linux")]
    {
        // Common Chinese font paths on Linux
        paths.extend_from_slice(&[
            "/usr/share/fonts/truetype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/noto/NotoSerifCJK-Regular.ttc",
            "/usr/share/fonts/opentype/noto/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/truetype/dejavu/DejaVuSans.ttf",
            "/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf",
            "/usr/share/fonts/truetype/wqy/wqy-microhei.ttc",
            "/usr/share/fonts/truetype/wqy/wqy-zenhei.ttc",
            "/usr/share/fonts/truetype/arphic/uming.ttc",
            "/usr/share/fonts/truetype/arphic/ukai.ttc",
            "/usr/share/fonts/truetype/droid/DroidSansFallback.ttf",
            "/usr/share/fonts/truetype/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/opentype/source-han-sans/SourceHanSansCN-Regular.otf",
            "/usr/share/fonts/opentype/source-han-serif/SourceHanSerifCN-Regular.otf",
            "/usr/share/fonts/noto-cjk/NotoSerifCJK-Black.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Thin.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-DemiLight.ttc",
            "/usr/share/fonts/noto-cjk/NotoSerifCJK-Light.ttc",
            "/usr/share/fonts/noto-cjk/NotoSerifCJK-Bold.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Medium.ttc",
            "/usr/share/fonts/noto-cjk/NotoSerifCJK-SemiBold.ttc",
            "/usr/share/fonts/noto-cjk/NotoSerifCJK-Regular.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Bold.ttc",
            "/usr/share/fonts/noto-cjk/NotoSerifCJK-Medium.ttc",
            "/usr/share/fonts/noto-cjk/NotoSerifCJK-ExtraLight.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Light.ttc",
            "/usr/share/fonts/noto-cjk/NotoSansCJK-Black.ttc",
        ]);
    }

    #[cfg(target_os = "windows")]
    {
        // Common Chinese font paths on Windows
        paths.extend_from_slice(&[
            "C:\\Windows\\Fonts\\msyh.ttc",    // Microsoft YaHei
            "C:\\Windows\\Fonts\\msyhbd.ttc",  // Microsoft YaHei Bold
            "C:\\Windows\\Fonts\\simsun.ttc",  // SimSun
            "C:\\Windows\\Fonts\\simhei.ttf",  // SimHei
            "C:\\Windows\\Fonts\\simkai.ttf",  // KaiTi
            "C:\\Windows\\Fonts\\simfang.ttf", // FangSong
            "C:\\Windows\\Fonts\\msjh.ttc",    // Microsoft JhengHei
            "C:\\Windows\\Fonts\\msjhbd.ttc",  // Microsoft JhengHei Bold
        ]);
    }

    #[cfg(target_os = "macos")]
    {
        // Common Chinese font paths on macOS
        paths.extend_from_slice(&[
            "/System/Library/Fonts/PingFang.ttc",
            "/System/Library/Fonts/STHeiti Light.ttc",
            "/System/Library/Fonts/STHeiti Medium.ttc",
            "/System/Library/Fonts/Hiragino Sans GB.ttc",
            "/Library/Fonts/Arial Unicode MS.ttf",
            "/System/Library/Fonts/Apple LiGothic Medium.ttf",
            "/System/Library/Fonts/Apple LiSung Light.ttf",
        ]);
    }

    // Convert to String and filter existing files
    paths
        .into_iter()
        .map(|s| s.to_string())
        .filter(|path| Path::new(path).exists())
        .collect()
}

/// Load font data from a file path
fn load_font_from_path(path: &str) -> Result<FontData, FontError> {
    log::debug!("Attempting to load font from: {}", path);

    let font_bytes = std::fs::read(path)?;

    if font_bytes.is_empty() {
        return Err(FontError::InvalidFont("Font file is empty".to_string()));
    }

    // Validate that this looks like a font file
    if !is_valid_font_data(&font_bytes) {
        return Err(FontError::InvalidFont(
            "Invalid font file format".to_string(),
        ));
    }

    Ok(FontData::from_owned(font_bytes))
}

/// Basic validation to check if the data looks like a font file
fn is_valid_font_data(data: &[u8]) -> bool {
    if data.len() < 4 {
        return false;
    }

    // Check for common font file signatures
    let signature = &data[0..4];

    // TTF/OTF signatures
    matches!(
        signature,
        b"\x00\x01\x00\x00" |  // TTF
        b"OTTO" |              // OTF
        b"ttcf" |              // TTC (TrueType Collection)
        b"wOFF" |              // WOFF
        b"wOF2" // WOFF2
    )
}

/// Configure text styles for better Chinese character rendering
fn configure_text_styles(ctx: &Context) {
    let mut style = (*ctx.style()).clone();

    // Set better spacing for CJK characters
    style.spacing.item_spacing = egui::vec2(8.0, 6.0);
    style.spacing.button_padding = egui::vec2(12.0, 6.0);

    // Configure text styles with appropriate sizes for Chinese characters
    style.text_styles.insert(
        TextStyle::Heading,
        FontId::new(20.0, FontFamily::Proportional),
    );
    style
        .text_styles
        .insert(TextStyle::Body, FontId::new(14.0, FontFamily::Proportional));
    style.text_styles.insert(
        TextStyle::Monospace,
        FontId::new(13.0, FontFamily::Monospace),
    );
    style.text_styles.insert(
        TextStyle::Button,
        FontId::new(14.0, FontFamily::Proportional),
    );
    style.text_styles.insert(
        TextStyle::Small,
        FontId::new(12.0, FontFamily::Proportional),
    );

    ctx.set_style(style);
}

/// Test if Chinese characters can be rendered properly
pub fn test_chinese_rendering(ctx: &Context) -> bool {
    // This is a simple test - in a real application you might want to
    // render some Chinese text and check if it displays correctly
    let fonts = ctx.fonts(|f| f.clone());

    // Check if we have fonts that can handle Chinese characters
    let test_chars = ['中', '文', '测', '试'];
    let font_id = FontId::new(14.0, FontFamily::Proportional);

    for &ch in &test_chars {
        // Check if the character has a reasonable width (indicating it can be rendered)
        let char_width = fonts.glyph_width(&font_id, ch);
        if char_width <= 0.0 {
            log::warn!(
                "Chinese character '{}' may not render properly (width: {})",
                ch,
                char_width
            );
            return false;
        }
    }

    log::info!("Chinese character rendering test passed");
    true
}
