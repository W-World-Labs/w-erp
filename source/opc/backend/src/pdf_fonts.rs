//! Embedded CJK fonts for PDF generation.
//!
//! Noto Sans SC (OFL-1.1) is compiled into the binary via `include_bytes!` so
//! invoice/quote PDFs never depend on host system fonts. Subset TTFs cover
//! Simplified Chinese + Latin + common currency symbols.

use genpdfi::fonts::{FontData, FontFamily};

/// Bundled Noto Sans SC Regular derivative; see assets/fonts/NOTICE.md.
const FONT_REGULAR: &[u8] = include_bytes!("../assets/fonts/NotoSansSC-Regular.ttf");
/// Bundled Noto Sans SC Bold derivative; see assets/fonts/NOTICE.md.
const FONT_BOLD: &[u8] = include_bytes!("../assets/fonts/NotoSansSC-Bold.ttf");

/// Load the embedded Noto Sans SC family for genpdfi documents.
///
/// Italic variants fall back to regular/bold (SC has no true italic in this subset).
pub fn pdf_font_family() -> Result<FontFamily<FontData>, genpdfi::error::Error> {
    let regular = FontData::new(FONT_REGULAR.to_vec(), None)?;
    let bold = FontData::new(FONT_BOLD.to_vec(), None)?;
    Ok(FontFamily {
        regular: regular.clone(),
        bold: bold.clone(),
        italic: regular,
        bold_italic: bold,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_noto_sans_sc_loads() {
        let family = pdf_font_family().expect("embedded Noto Sans SC must load");
        // Sanity: regular metrics font must expose a CJK glyph.
        assert!(family.regular.has_glyph('发'));
        assert!(family.regular.has_glyph('A'));
        assert!(family.bold.has_glyph('票'));
    }
}
