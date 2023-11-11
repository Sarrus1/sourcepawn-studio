use ide::WideEncoding;
use lsp_types::PositionEncodingKind;

use crate::line_index::PositionEncoding;

pub fn negotiated_encoding(caps: &lsp_types::ClientCapabilities) -> PositionEncoding {
    let client_encodings = match &caps.general {
        Some(general) => general.position_encodings.as_deref().unwrap_or_default(),
        None => &[],
    };

    for enc in client_encodings {
        if enc == &PositionEncodingKind::UTF8 {
            return PositionEncoding::Utf8;
        } else if enc == &PositionEncodingKind::UTF32 {
            return PositionEncoding::Wide(WideEncoding::Utf32);
        }
        // NB: intentionally prefer just about anything else to utf-16.
    }

    PositionEncoding::Wide(WideEncoding::Utf16)
}
