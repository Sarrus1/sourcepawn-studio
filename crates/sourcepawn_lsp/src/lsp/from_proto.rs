use anyhow::format_err;
use base_db::FilePosition;
use ide::{LineCol, WideLineCol};
use lsp_types::Url;
use paths::AbsPathBuf;
use rowan::{TextRange, TextSize};
use vfs::FileId;

use crate::{
    line_index::{LineIndex, PositionEncoding},
    server::GlobalStateSnapshot,
};

pub(crate) fn abs_path(url: &lsp_types::Url) -> anyhow::Result<AbsPathBuf> {
    let path = url
        .to_file_path()
        .map_err(|()| anyhow::format_err!("url is not a file"))?;
    Ok(AbsPathBuf::try_from(path).unwrap())
}

pub(crate) fn vfs_path(url: &lsp_types::Url) -> anyhow::Result<vfs::VfsPath> {
    abs_path(url).map(vfs::VfsPath::from)
}

pub(crate) fn offset(
    line_index: &LineIndex,
    position: lsp_types::Position,
) -> anyhow::Result<TextSize> {
    let line_col = match line_index.encoding {
        PositionEncoding::Utf8 => LineCol {
            line: position.line,
            col: position.character,
        },
        PositionEncoding::Wide(enc) => {
            let line_col = WideLineCol {
                line: position.line,
                col: position.character,
            };
            line_index
                .index
                .to_utf8(enc, line_col)
                .ok_or_else(|| format_err!("Invalid wide col offset"))?
        }
    };
    let text_size = line_index
        .index
        .offset(line_col)
        .ok_or_else(|| format_err!("Invalid offset"))?;
    Ok(text_size)
}

pub(crate) fn text_range(
    line_index: &LineIndex,
    range: lsp_types::Range,
) -> anyhow::Result<TextRange> {
    let start = offset(line_index, range.start)?;
    let end = offset(line_index, range.end)?;
    if end < start {
        Err(format_err!("Invalid Range"))
    } else {
        Ok(TextRange::new(start, end))
    }
}

pub(crate) fn file_id(snap: &GlobalStateSnapshot, uri: &Url) -> anyhow::Result<FileId> {
    snap.url_to_file_id(uri)
}

pub(crate) fn file_position(
    snap: &GlobalStateSnapshot,
    params: lsp_types::TextDocumentPositionParams,
) -> anyhow::Result<FilePosition> {
    let file_id = file_id(snap, &params.text_document.uri)?;
    Ok(FilePosition {
        file_id,
        position: params.position,
    })
}
