use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
};

use ecow::EcoVec;
use typst::{
    Library, LibraryExt, World,
    diag::{FileError, FileResult, SourceDiagnostic},
    foundations::{Bytes, Datetime},
    layout::PagedDocument,
    syntax::{FileId, Source, VirtualPath},
    text::{Font, FontBook},
    utils::LazyHash,
};
use typst_kit::fonts::{FontSearcher, FontSlot};
use typst_pdf::PdfOptions;

use crate::{error::StatementPdfError, render::TypstDocument};

pub(crate) fn compile_pdf(document: TypstDocument) -> Result<Vec<u8>, StatementPdfError> {
    let world = StatementWorld::new(document);
    let document = typst::compile::<PagedDocument>(&world)
        .output
        .map_err(|diagnostics| StatementPdfError::Typst(format_diagnostics(diagnostics)))?;

    typst_pdf::pdf(&document, &PdfOptions::default())
        .map_err(|diagnostics| StatementPdfError::Pdf(format_diagnostics(diagnostics)))
}

struct StatementWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<FontSlot>,
    main: FileId,
    sources: BTreeMap<PathBuf, Source>,
    files: BTreeMap<PathBuf, Bytes>,
}

impl StatementWorld {
    fn new(document: TypstDocument) -> Self {
        let main = file_id("/main.typ");
        let statement_json = file_id("/statement.json");
        let fonts = FontSearcher::new()
            .include_system_fonts(true)
            .include_embedded_fonts(true)
            .search();

        let mut sources = BTreeMap::new();
        sources.insert(
            rooted_path("/main.typ"),
            Source::new(main, document.main_source),
        );

        let mut files = BTreeMap::new();
        files.insert(
            rooted_path("/statement.json"),
            Bytes::new(document.statement_json.into_bytes()),
        );

        for (path, bytes) in document.assets {
            files.insert(rooted_path(path), Bytes::new(bytes));
        }

        // Intern the JSON path up front so `json("statement.json")` resolves to
        // the same normalized virtual location when Typst joins it from /main.typ.
        let _ = statement_json;

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(fonts.book),
            fonts: fonts.fonts,
            main,
            sources,
            files,
        }
    }
}

impl World for StatementWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }

    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
    }

    fn main(&self) -> FileId {
        self.main
    }

    fn source(&self, id: FileId) -> FileResult<Source> {
        let path = id.vpath().as_rooted_path();
        self.sources
            .get(path)
            .cloned()
            .ok_or_else(|| FileError::NotFound(path.to_path_buf()))
    }

    fn file(&self, id: FileId) -> FileResult<Bytes> {
        let path = id.vpath().as_rooted_path();
        self.files
            .get(path)
            .cloned()
            .ok_or_else(|| FileError::NotFound(path.to_path_buf()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).and_then(FontSlot::get)
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Datetime::from_ymd(1970, 1, 1)
    }
}

fn file_id(path: impl AsRef<Path>) -> FileId {
    FileId::new(None, VirtualPath::new(path))
}

fn rooted_path(path: impl AsRef<Path>) -> PathBuf {
    VirtualPath::new(path).as_rooted_path().to_path_buf()
}

fn format_diagnostics(diagnostics: EcoVec<SourceDiagnostic>) -> String {
    diagnostics
        .iter()
        .map(|diagnostic| {
            let mut message = diagnostic.message.to_string();

            for hint in &diagnostic.hints {
                message.push_str("\nhelp: ");
                message.push_str(hint);
            }

            message
        })
        .collect::<Vec<_>>()
        .join("\n")
}
