//! Repository tooling for `estimate`, run as `cargo xtask <task>`.
//!
//! `cargo xtask doc-lint` checks that the doc comment of every public item in `estimate` and
//! `estimate-derive` follows the template in AGENTS.md ("Doc comments"): a one-line summary,
//! optional details, then `# Arguments`, `# Returns`, `# Errors` and `# Examples` as the item
//! needs them. rustdoc and clippy cannot check that structure on stable Rust, so this does.
//!
//! The check is syntactic. That is enough because `unreachable_pub` is denied: every `pub` item
//! in a `pub` module really is public API.

use std::cmp::Ordering;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::ExitCode;

use proc_macro2::Span;
use syn::{
    Attribute, Expr, ExprLit, Field, FnArg, ImplItem, Item, ItemImpl, ItemMod, ItemTrait, Lit, Meta, Pat, ReturnType,
    Signature, TraitItem, Type, Visibility,
};

/// Library crates whose public API `doc-lint` checks: (crate name, root file under the workspace).
const CRATES: [(&str, &str); 2] = [
    ("estimate", "estimate/src/lib.rs"),
    ("estimate_derive", "estimate-derive/src/lib.rs"),
];

/// The standard doc sections, in the order they must appear.
const SECTIONS: [&str; 6] = ["Arguments", "Returns", "Errors", "Panics", "Safety", "Examples"];

/// Sections only a function can have.
const FN_SECTIONS: [&str; 3] = ["Arguments", "Returns", "Errors"];

/// Near-miss headings, and the standard section each one should be.
const NEAR_MISSES: [(&str, &str); 9] = [
    ("Example", "Examples"),
    ("Argument", "Arguments"),
    ("Args", "Arguments"),
    ("Parameters", "Arguments"),
    ("Params", "Arguments"),
    ("Return", "Returns"),
    ("Return value", "Returns"),
    ("Error", "Errors"),
    ("Panic", "Panics"),
];

/// Runs the task named on the command line.
fn main() -> ExitCode {
    if std::env::args().nth(1).as_deref() != Some("doc-lint") {
        eprintln!("usage: cargo xtask doc-lint");
        return ExitCode::FAILURE;
    }
    match doc_lint() {
        Ok(0) => ExitCode::SUCCESS,
        Ok(count) => {
            eprintln!("doc-lint: {count} problem(s). See \"Doc comments\" in AGENTS.md.");
            ExitCode::FAILURE
        }
        Err(failure) => {
            eprintln!("doc-lint: {failure}");
            ExitCode::FAILURE
        }
    }
}

/// Checks every crate in [`CRATES`] and prints one line per problem.
///
/// Returns the number of problems found.
fn doc_lint() -> Result<usize, Failure> {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .ok_or(Failure::NoWorkspace)?;
    let mut checker = Checker::default();
    for (name, lib) in CRATES {
        checker.crate_root(name, &root.join(lib))?;
    }
    for diagnostic in &checker.diagnostics {
        println!("{}", diagnostic.relative_to(root));
    }
    Ok(checker.diagnostics.len())
}

/// Why `doc-lint` could not run (as opposed to a problem it found).
#[derive(Debug)]
enum Failure {
    /// `CARGO_MANIFEST_DIR` has no parent directory.
    NoWorkspace,
    /// A source file could not be read.
    Io(PathBuf, io::Error),
    /// A source file is not valid Rust.
    Parse(PathBuf, syn::Error),
    /// `mod name;` has neither `name.rs` nor `name/mod.rs`.
    NoModule(PathBuf),
}

impl fmt::Display for Failure {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NoWorkspace => write!(f, "cannot find the workspace root"),
            Self::Io(path, error) => write!(f, "{}: {error}", path.display()),
            Self::Parse(path, error) => {
                let start = error.span().start();
                write!(f, "{}:{}: {error}", path.display(), start.line)
            }
            Self::NoModule(path) => write!(f, "{}: no `.rs` or `/mod.rs` file for this module", path.display()),
        }
    }
}

/// One problem in one doc comment.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Diagnostic {
    /// The source file.
    file: PathBuf,
    /// 1-based line of the item's name.
    line: usize,
    /// Path of the item, e.g. `estimate::manifold::Manifold::boxplus`.
    item: String,
    /// What is wrong.
    message: String,
}

impl Diagnostic {
    /// Formats as `file:line: item: message`, with `file` relative to `root`.
    fn relative_to(&self, root: &Path) -> String {
        let file = self.file.strip_prefix(root).unwrap_or(&self.file);
        format!("{}:{}: {}: {}", file.display(), self.line, self.item, self.message)
    }
}

/// What an item is, which decides the sections its docs need.
#[derive(Clone, Copy)]
enum Kind<'a> {
    /// A function or method: its signature decides `# Arguments`, `# Returns` and `# Errors`.
    Fn(&'a Signature),
    /// A trait, struct, enum, union, type alias or proc macro: needs `# Examples`.
    Type,
    /// A module: needs `# Examples` once it declares a public item.
    Module {
        /// Whether the module declares a public item, submodules and re-exports included.
        has_items: bool,
    },
    /// A field, variant, constant, static or associated item: summary only.
    Member,
}

/// Where in the module tree an item is.
struct Scope<'a> {
    /// The source file being read.
    file: &'a Path,
    /// The directory holding the files of this module's `mod name;` children.
    dir: PathBuf,
    /// Path of the module, e.g. `estimate::manifold`.
    path: String,
}

impl Scope<'_> {
    /// Path of an item named `name` in this module.
    fn join(&self, name: &str) -> String {
        format!("{}::{name}", self.path)
    }
}

/// Walks a crate's public module tree and collects [`Diagnostic`]s.
#[derive(Default)]
struct Checker {
    /// Problems found so far.
    diagnostics: Vec<Diagnostic>,
}

impl Checker {
    /// Checks the crate whose root file is `lib`.
    fn crate_root(&mut self, name: &str, lib: &Path) -> Result<(), Failure> {
        let file = parse(lib)?;
        let dir = lib.parent().map_or_else(PathBuf::new, Path::to_path_buf);
        let scope = Scope {
            file: lib,
            dir,
            path: name.to_owned(),
        };
        self.module(&scope, 1, &doc_lines(&file.attrs), &file.items)
    }

    /// Checks a module's own docs, then every public item in it.
    fn module(&mut self, scope: &Scope<'_>, line: usize, docs: &[String], items: &[Item]) -> Result<(), Failure> {
        let has_items = items.iter().any(is_public_item);
        self.check(scope.file, line, &scope.path, docs, Kind::Module { has_items });
        for item in items {
            self.item(scope, item)?;
        }
        Ok(())
    }

    /// Checks one item, and the public members inside it.
    fn item(&mut self, scope: &Scope<'_>, item: &Item) -> Result<(), Failure> {
        if is_skipped(item_attrs(item)) {
            return Ok(());
        }
        match item {
            Item::Fn(f) if is_pub(&f.vis) => {
                let kind = if is_proc_macro(&f.attrs) {
                    Kind::Type
                } else {
                    Kind::Fn(&f.sig)
                };
                self.named(
                    scope.file,
                    &f.sig.ident,
                    &scope.join(&f.sig.ident.to_string()),
                    &f.attrs,
                    kind,
                );
            }
            Item::Struct(s) if is_pub(&s.vis) => {
                let path = scope.join(&s.ident.to_string());
                self.named(scope.file, &s.ident, &path, &s.attrs, Kind::Type);
                self.fields(scope.file, &path, s.fields.iter().filter(|field| is_pub(&field.vis)));
            }
            Item::Union(u) if is_pub(&u.vis) => {
                let path = scope.join(&u.ident.to_string());
                self.named(scope.file, &u.ident, &path, &u.attrs, Kind::Type);
                self.fields(
                    scope.file,
                    &path,
                    u.fields.named.iter().filter(|field| is_pub(&field.vis)),
                );
            }
            Item::Enum(e) if is_pub(&e.vis) => {
                let path = scope.join(&e.ident.to_string());
                self.named(scope.file, &e.ident, &path, &e.attrs, Kind::Type);
                for variant in e.variants.iter().filter(|variant| !is_skipped(&variant.attrs)) {
                    let variant_path = format!("{path}::{}", variant.ident);
                    self.named(scope.file, &variant.ident, &variant_path, &variant.attrs, Kind::Member);
                    self.fields(scope.file, &variant_path, variant.fields.iter());
                }
            }
            Item::Trait(t) if is_pub(&t.vis) => self.trait_items(scope, t),
            Item::TraitAlias(t) if is_pub(&t.vis) => {
                self.named(
                    scope.file,
                    &t.ident,
                    &scope.join(&t.ident.to_string()),
                    &t.attrs,
                    Kind::Type,
                );
            }
            Item::Type(t) if is_pub(&t.vis) => {
                self.named(
                    scope.file,
                    &t.ident,
                    &scope.join(&t.ident.to_string()),
                    &t.attrs,
                    Kind::Type,
                );
            }
            Item::Const(c) if is_pub(&c.vis) => {
                self.named(
                    scope.file,
                    &c.ident,
                    &scope.join(&c.ident.to_string()),
                    &c.attrs,
                    Kind::Member,
                );
            }
            Item::Static(s) if is_pub(&s.vis) => {
                self.named(
                    scope.file,
                    &s.ident,
                    &scope.join(&s.ident.to_string()),
                    &s.attrs,
                    Kind::Member,
                );
            }
            Item::Impl(i) if i.trait_.is_none() => self.impl_items(scope, i),
            Item::Mod(m) if is_pub(&m.vis) => self.submodule(scope, m)?,
            _ => {}
        }
        Ok(())
    }

    /// Checks a public trait and every item in it.
    fn trait_items(&mut self, scope: &Scope<'_>, t: &ItemTrait) {
        let path = scope.join(&t.ident.to_string());
        self.named(scope.file, &t.ident, &path, &t.attrs, Kind::Type);
        for item in &t.items {
            match item {
                TraitItem::Fn(f) if !is_skipped(&f.attrs) => {
                    let item_path = format!("{path}::{}", f.sig.ident);
                    self.named(scope.file, &f.sig.ident, &item_path, &f.attrs, Kind::Fn(&f.sig));
                }
                TraitItem::Type(ty) if !is_skipped(&ty.attrs) => {
                    let item_path = format!("{path}::{}", ty.ident);
                    self.named(scope.file, &ty.ident, &item_path, &ty.attrs, Kind::Member);
                }
                TraitItem::Const(c) if !is_skipped(&c.attrs) => {
                    let item_path = format!("{path}::{}", c.ident);
                    self.named(scope.file, &c.ident, &item_path, &c.attrs, Kind::Member);
                }
                _ => {}
            }
        }
    }

    /// Checks the public functions and constants of an inherent `impl` block.
    fn impl_items(&mut self, scope: &Scope<'_>, block: &ItemImpl) {
        if is_skipped(&block.attrs) {
            return;
        }
        let owner = match &*block.self_ty {
            Type::Path(ty) => ty.path.segments.last().map(|segment| segment.ident.to_string()),
            _ => None,
        }
        .unwrap_or_else(|| "impl".to_owned());
        let path = scope.join(&owner);
        for item in &block.items {
            match item {
                ImplItem::Fn(f) if is_pub(&f.vis) && !is_skipped(&f.attrs) => {
                    let item_path = format!("{path}::{}", f.sig.ident);
                    self.named(scope.file, &f.sig.ident, &item_path, &f.attrs, Kind::Fn(&f.sig));
                }
                ImplItem::Const(c) if is_pub(&c.vis) && !is_skipped(&c.attrs) => {
                    let item_path = format!("{path}::{}", c.ident);
                    self.named(scope.file, &c.ident, &item_path, &c.attrs, Kind::Member);
                }
                _ => {}
            }
        }
    }

    /// Checks a public submodule, reading `name.rs` or `name/mod.rs` for `mod name;`.
    fn submodule(&mut self, scope: &Scope<'_>, m: &ItemMod) -> Result<(), Failure> {
        let name = m.ident.to_string();
        let dir = scope.dir.join(&name);
        let path = scope.join(&name);
        let mut docs = doc_lines(&m.attrs);
        if let Some((_, items)) = &m.content {
            let inline = Scope {
                file: scope.file,
                dir,
                path,
            };
            return self.module(&inline, line(m.ident.span()), &docs, items);
        }
        let file_path = module_file(&scope.dir, &name)?;
        let file = parse(&file_path)?;
        docs.extend(doc_lines(&file.attrs));
        let child = Scope {
            file: &file_path,
            dir,
            path,
        };
        self.module(&child, 1, &docs, &file.items)
    }

    /// Checks the named fields among `fields` as members of the item at `path`.
    fn fields<'f>(&mut self, file: &Path, path: &str, fields: impl Iterator<Item = &'f Field>) {
        for field in fields.filter(|field| !is_skipped(&field.attrs)) {
            if let Some(ident) = &field.ident {
                self.named(file, ident, &format!("{path}::{ident}"), &field.attrs, Kind::Member);
            }
        }
    }

    /// Checks the docs of an item whose name is `ident`.
    fn named(&mut self, file: &Path, ident: &syn::Ident, path: &str, attrs: &[Attribute], kind: Kind<'_>) {
        self.check(file, line(ident.span()), path, &doc_lines(attrs), kind);
    }

    /// Lints `docs` and records each problem against `item`.
    fn check(&mut self, file: &Path, line: usize, item: &str, docs: &[String], kind: Kind<'_>) {
        for message in lint(docs, kind) {
            self.diagnostics.push(Diagnostic {
                file: file.to_path_buf(),
                line,
                item: item.to_owned(),
                message,
            });
        }
    }
}

/// Reads and parses one source file.
fn parse(path: &Path) -> Result<syn::File, Failure> {
    let source = fs::read_to_string(path).map_err(|error| Failure::Io(path.to_path_buf(), error))?;
    syn::parse_file(&source).map_err(|error| Failure::Parse(path.to_path_buf(), error))
}

/// The file of `mod name;` declared in a module whose children live in `dir`.
fn module_file(dir: &Path, name: &str) -> Result<PathBuf, Failure> {
    let flat = dir.join(format!("{name}.rs"));
    let nested = dir.join(name).join("mod.rs");
    [flat, nested]
        .into_iter()
        .find(|candidate| candidate.is_file())
        .ok_or_else(|| Failure::NoModule(dir.join(name)))
}

/// 1-based line where `span` starts.
fn line(span: Span) -> usize {
    span.start().line
}

/// Whether `vis` is plain `pub`.
const fn is_pub(vis: &Visibility) -> bool {
    matches!(vis, Visibility::Public(_))
}

/// Whether `item` is public, which makes its module need an example.
///
/// Submodules and re-exports count, so a crate root of `pub mod`s still needs an example. A
/// module that declares nothing public (a stub) has nothing to show yet.
fn is_public_item(item: &Item) -> bool {
    let vis = match item {
        Item::Fn(i) => &i.vis,
        Item::Struct(i) => &i.vis,
        Item::Union(i) => &i.vis,
        Item::Enum(i) => &i.vis,
        Item::Trait(i) => &i.vis,
        Item::TraitAlias(i) => &i.vis,
        Item::Type(i) => &i.vis,
        Item::Const(i) => &i.vis,
        Item::Static(i) => &i.vis,
        Item::Mod(i) => &i.vis,
        Item::Use(i) => &i.vis,
        _ => return false,
    };
    is_pub(vis) && !is_skipped(item_attrs(item))
}

/// The attributes of any item.
fn item_attrs(item: &Item) -> &[Attribute] {
    match item {
        Item::Const(i) => &i.attrs,
        Item::Enum(i) => &i.attrs,
        Item::ExternCrate(i) => &i.attrs,
        Item::Fn(i) => &i.attrs,
        Item::ForeignMod(i) => &i.attrs,
        Item::Impl(i) => &i.attrs,
        Item::Macro(i) => &i.attrs,
        Item::Mod(i) => &i.attrs,
        Item::Static(i) => &i.attrs,
        Item::Struct(i) => &i.attrs,
        Item::Trait(i) => &i.attrs,
        Item::TraitAlias(i) => &i.attrs,
        Item::Type(i) => &i.attrs,
        Item::Union(i) => &i.attrs,
        Item::Use(i) => &i.attrs,
        _ => &[],
    }
}

/// Whether an item is outside the public docs: `#[cfg(test)]` or `#[doc(hidden)]`.
fn is_skipped(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| match &attr.meta {
        Meta::List(list) => {
            let tokens = list.tokens.to_string();
            (list.path.is_ident("cfg") && tokens == "test") || (list.path.is_ident("doc") && tokens == "hidden")
        }
        _ => false,
    })
}

/// Whether a function is a proc macro, whose docs describe the macro rather than its signature.
fn is_proc_macro(attrs: &[Attribute]) -> bool {
    attrs.iter().any(|attr| {
        ["proc_macro", "proc_macro_derive", "proc_macro_attribute"]
            .iter()
            .any(|name| attr.path().is_ident(name))
    })
}

/// The lines of an item's doc comment (`///`, `//!` and `#[doc = "…"]`), in order.
///
/// The one space after `///` is stripped, so indentation inside the comment is kept.
fn doc_lines(attrs: &[Attribute]) -> Vec<String> {
    let mut lines = Vec::new();
    for attr in attrs.iter().filter(|attr| attr.path().is_ident("doc")) {
        if let Meta::NameValue(pair) = &attr.meta
            && let Expr::Lit(ExprLit {
                lit: Lit::Str(text), ..
            }) = &pair.value
        {
            for line in text.value().split('\n') {
                let line = line.strip_suffix('\r').unwrap_or(line);
                lines.push(line.strip_prefix(' ').unwrap_or(line).to_owned());
            }
        }
    }
    lines
}

/// A doc comment split into sections at its headings.
struct Doc {
    /// The text before the first heading, then one entry per heading.
    sections: Vec<Section>,
}

/// One heading and what follows it, up to the next heading.
struct Section {
    /// Heading text, e.g. `Examples`. Empty for the text before the first heading.
    title: String,
    /// Number of `#`s in the heading.
    level: usize,
    /// Lines outside code blocks.
    lines: Vec<String>,
    /// Info string of each code block, e.g. `""`, `"text"` or `"rust,no_run"`.
    blocks: Vec<String>,
}

impl Section {
    /// An empty section under a heading.
    const fn new(title: String, level: usize) -> Self {
        Self {
            title,
            level,
            lines: Vec::new(),
            blocks: Vec::new(),
        }
    }
}

impl Doc {
    /// Splits `lines` at headings outside code blocks, reporting an unclosed code block.
    fn parse(lines: &[String], out: &mut Vec<String>) -> Self {
        let mut sections = vec![Section::new(String::new(), 0)];
        let mut fence: Option<String> = None;
        for line in lines {
            let trimmed = line.trim();
            if let Some(marker) = &fence {
                if closes(trimmed, marker) {
                    fence = None;
                }
                continue;
            }
            if let Some((marker, info)) = opens(trimmed) {
                if let Some(section) = sections.last_mut() {
                    section.blocks.push(info);
                }
                fence = Some(marker);
            } else if let Some((level, title)) = heading(line) {
                sections.push(Section::new(title, level));
            } else if let Some(section) = sections.last_mut() {
                section.lines.push(line.clone());
            }
        }
        if fence.is_some() {
            out.push("unclosed code block".to_owned());
        }
        Self { sections }
    }

    /// The sections under headings, in order.
    fn headed(&self) -> impl Iterator<Item = &Section> {
        self.sections.iter().skip(1)
    }

    /// The section titled `title`, if any.
    fn section(&self, title: &str) -> Option<&Section> {
        self.headed().find(|section| section.title == title)
    }
}

/// If `line` opens a code block, its fence marker and info string.
fn opens(line: &str) -> Option<(String, String)> {
    let fence_char = line.chars().next().filter(|c| *c == '`' || *c == '~')?;
    let marker: String = line.chars().take_while(|c| *c == fence_char).collect();
    let info = line.get(marker.len()..)?;
    (marker.len() >= 3).then(|| (marker, info.trim().to_owned()))
}

/// Whether `line` closes the code block opened by `marker`.
fn closes(line: &str, marker: &str) -> bool {
    let fence_char = marker.chars().next();
    line.len() >= marker.len() && line.chars().all(|c| Some(c) == fence_char)
}

/// If `line` is a Markdown heading, its level and text.
fn heading(line: &str) -> Option<(usize, String)> {
    let level = line.chars().take_while(|c| *c == '#').count();
    let text = line.get(level..)?.strip_prefix(' ')?;
    (level > 0).then(|| (level, text.trim().to_owned()))
}

/// The words of a code block's info string, e.g. `rust` and `no_run` for `rust,no_run`.
fn info_words(info: &str) -> impl Iterator<Item = &str> {
    info.split([',', ' ', '\t'])
        .map(str::trim)
        .filter(|word| !word.is_empty())
}

/// Whether a code block with this info string runs as a doctest.
fn is_runnable(info: &str) -> bool {
    info_words(info).all(|word| word == "rust" || word == "no_run" || word.starts_with("edition"))
}

/// Every problem with one item's doc comment.
fn lint(docs: &[String], kind: Kind<'_>) -> Vec<String> {
    let mut out = Vec::new();
    if docs.iter().all(|line| line.trim().is_empty()) {
        out.push("missing doc comment".to_owned());
        return out;
    }
    summary(docs, &mut out);
    let doc = Doc::parse(docs, &mut out);
    headings(&doc, &mut out);
    code_blocks(&doc, &mut out);
    inline_math(&doc, &mut out);
    match kind {
        Kind::Fn(signature) => function(signature, &doc, &mut out),
        Kind::Type => {
            not_a_function(&doc, &mut out);
            examples(&doc, &mut out);
        }
        Kind::Module { has_items } => {
            not_a_function(&doc, &mut out);
            if has_items {
                examples(&doc, &mut out);
            }
        }
        Kind::Member => not_a_function(&doc, &mut out),
    }
    out
}

/// The doc starts with a one-line summary that ends in `.`, `?` or `!`.
fn summary(docs: &[String], out: &mut Vec<String>) {
    let paragraph: Vec<&str> = docs
        .iter()
        .map(|line| line.trim())
        .take_while(|line| !line.is_empty())
        .collect();
    let Some(first) = paragraph.first() else {
        out.push("doc must start with a one-line summary, not a blank line".to_owned());
        return;
    };
    if first.starts_with('#') || opens(first).is_some() {
        out.push("doc must start with a one-line summary, not a heading or code block".to_owned());
        return;
    }
    if paragraph.len() > 1 {
        out.push("summary must be one line; put details in a second paragraph".to_owned());
    }
    if paragraph.last().is_some_and(|last| !last.ends_with(['.', '?', '!'])) {
        out.push("summary must end with '.', '?' or '!'".to_owned());
    }
}

/// Headings are level 1, spelled as in [`SECTIONS`], in that order, with `# Examples` last.
fn headings(doc: &Doc, out: &mut Vec<String>) {
    // Rank in SECTIONS of the last standard heading seen.
    let mut previous: Option<usize> = None;
    for section in doc.headed() {
        let title = section.title.as_str();
        if section.level != 1 {
            out.push(format!("use a level-1 heading: `# {title}`"));
        }
        let fix = NEAR_MISSES
            .iter()
            .copied()
            .chain(SECTIONS.iter().map(|standard| (*standard, *standard)))
            .find(|(miss, standard)| miss.eq_ignore_ascii_case(title) && *standard != title);
        if let Some((_, standard)) = fix {
            out.push(format!("`# {title}` should be `# {standard}`"));
            continue;
        }
        let Some(position) = SECTIONS.iter().position(|standard| *standard == title) else {
            continue;
        };
        if let Some(before) = previous.and_then(|rank| SECTIONS.get(rank).map(|before| (rank, before))) {
            match position.cmp(&before.0) {
                Ordering::Equal => out.push(format!("duplicate `# {title}`")),
                Ordering::Less => out.push(format!("`# {title}` must come before `# {}`", before.1)),
                Ordering::Greater => {}
            }
        }
        previous = Some(position);
    }
    let last = doc.headed().last().map(|section| section.title.as_str());
    if doc.section("Examples").is_some() && last != Some("Examples") {
        out.push("`# Examples` must be the last section".to_owned());
    }
}

/// No code block is `ignore` (it never compiles) or `text` (equations go in ```math blocks).
fn code_blocks(doc: &Doc, out: &mut Vec<String>) {
    let has_word = |target: &str| {
        doc.sections
            .iter()
            .flat_map(|section| &section.blocks)
            .any(|info| info_words(info).any(|word| word == target))
    };
    if has_word("ignore") {
        out.push("```ignore blocks never run; make it a doctest".to_owned());
    }
    if has_word("text") {
        out.push("```text is not allowed: equations go in a ```math block, or inline as $...$".to_owned());
    }
}

/// Inline `$...$` math is balanced, and Markdown leaves its TeX intact.
///
/// rustdoc's Markdown has no math mode, so a backslash before ASCII punctuation (`\,`, `\{`, `\|`,
/// a `\\` line break) is an escape: the backslash is dropped before KaTeX sees it. Display math
/// belongs in a ```math block, where rustdoc keeps the TeX verbatim, not a `$$` paragraph.
fn inline_math(doc: &Doc, out: &mut Vec<String>) {
    for section in &doc.sections {
        for paragraph in section.lines.split(|line| line.trim().is_empty()) {
            let prose = without_code_spans(&paragraph.join(" "));
            let first = paragraph.first().map_or("", |line| line.trim());
            if prose.contains("$$") {
                out.push(format!("display math goes in a ```math block, not $$...$$: {first}"));
                continue;
            }
            let pieces: Vec<&str> = prose.split('$').collect();
            if pieces.len().is_multiple_of(2) {
                out.push(format!("unbalanced `$` in the paragraph starting: {first}"));
                continue;
            }
            for math in pieces.iter().skip(1).step_by(2) {
                if let Some(escape) = markdown_escape(math) {
                    out.push(format!(
                        "Markdown drops the backslash of `{escape}` in ${math}$; rewrite it or use a ```math block"
                    ));
                }
            }
        }
    }
}

/// `text` with its `code spans` removed, so a `$` in code is not read as math.
fn without_code_spans(text: &str) -> String {
    text.split('`').step_by(2).collect()
}

/// The first backslash escape of ASCII punctuation in `math`, e.g. `\,`.
fn markdown_escape(math: &str) -> Option<String> {
    let mut chars = math.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\\'
            && let Some(next) = chars.next_if(char::is_ascii_punctuation)
        {
            return Some(format!("\\{next}"));
        }
    }
    None
}

/// `# Arguments`, `# Returns` and `# Errors` match the signature, and there is an example.
fn function(signature: &Signature, doc: &Doc, out: &mut Vec<String>) {
    let parameters = parameters(signature, out);
    match (parameters.is_empty(), doc.section("Arguments")) {
        (false, None) => out.push(format!("missing `# Arguments` for {}", quoted(&parameters))),
        (true, Some(_)) => out.push("takes no arguments besides `self`; remove `# Arguments`".to_owned()),
        (false, Some(section)) => arguments(section, &parameters, out),
        (true, None) => {}
    }
    match (returns_value(&signature.output), doc.section("Returns").is_some()) {
        (true, false) => out.push("returns a value but has no `# Returns`".to_owned()),
        (false, true) => out.push("returns `()`; remove `# Returns`".to_owned()),
        _ => {}
    }
    match (returns_result(&signature.output), doc.section("Errors").is_some()) {
        (true, false) => out.push("returns `Result` but has no `# Errors`".to_owned()),
        (false, true) => out.push("does not return `Result`; remove `# Errors`".to_owned()),
        _ => {}
    }
    examples(doc, out);
}

/// Names of a function's parameters, excluding `self`.
fn parameters(signature: &Signature, out: &mut Vec<String>) -> Vec<String> {
    let mut names = Vec::new();
    for input in &signature.inputs {
        if let FnArg::Typed(typed) = input {
            if let Pat::Ident(pattern) = &*typed.pat {
                names.push(pattern.ident.to_string());
            } else {
                out.push("bind each parameter to a name so `# Arguments` can document it".to_owned());
            }
        }
    }
    names
}

/// `# Arguments` has one "* `name` - description" entry per parameter, in signature order.
fn arguments(section: &Section, parameters: &[String], out: &mut Vec<String>) {
    let mut documented = Vec::new();
    for line in &section.lines {
        // Blank lines and indented continuations of an entry.
        if line.trim().is_empty() || line.starts_with(' ') {
            continue;
        }
        let entry = line
            .strip_prefix("* `")
            .and_then(|rest| rest.split_once('`'))
            .and_then(|(name, rest)| Some((name, rest.strip_prefix(" - ")?)));
        match entry {
            Some((name, description)) => {
                if description.trim().is_empty() {
                    out.push(format!("`# Arguments` entry `{name}` needs a description"));
                }
                documented.push(name.to_owned());
            }
            None => out.push(format!(
                "`# Arguments` entries must read \"* `name` - description\": {line}"
            )),
        }
    }
    let mut complete = true;
    for parameter in parameters.iter().filter(|parameter| !documented.contains(parameter)) {
        out.push(format!("parameter `{parameter}` is missing from `# Arguments`"));
        complete = false;
    }
    for name in documented.iter().filter(|name| !parameters.contains(name)) {
        out.push(format!("`# Arguments` documents `{name}`, which is not a parameter"));
        complete = false;
    }
    if complete && documented != parameters {
        out.push(format!(
            "`# Arguments` must follow signature order: {}",
            quoted(parameters)
        ));
    }
}

/// `# Examples` exists and has a code block that runs.
fn examples(doc: &Doc, out: &mut Vec<String>) {
    match doc.section("Examples") {
        None => out.push("missing `# Examples` with a runnable doctest".to_owned()),
        Some(section) if !section.blocks.iter().any(|info| is_runnable(info)) => {
            out.push("`# Examples` needs a runnable code block (```, ```rust or ```no_run)".to_owned());
        }
        Some(_) => {}
    }
}

/// Only functions have `# Arguments`, `# Returns` or `# Errors`.
fn not_a_function(doc: &Doc, out: &mut Vec<String>) {
    for title in FN_SECTIONS {
        if doc.section(title).is_some() {
            out.push(format!("only functions have `# {title}`"));
        }
    }
}

/// Whether a function returns something other than `()`.
fn returns_value(output: &ReturnType) -> bool {
    match output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => !matches!(&**ty, Type::Tuple(tuple) if tuple.elems.is_empty()),
    }
}

/// Whether a function returns a `Result` (any type whose last path segment is `Result`).
fn returns_result(output: &ReturnType) -> bool {
    match output {
        ReturnType::Default => false,
        ReturnType::Type(_, ty) => match &**ty {
            Type::Path(path) => path
                .path
                .segments
                .last()
                .is_some_and(|segment| segment.ident == "Result"),
            _ => false,
        },
    }
}

/// `names` as "`a`, `b`".
fn quoted(names: &[String]) -> String {
    names
        .iter()
        .map(|name| format!("`{name}`"))
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A doc comment that satisfies every rule for `fn(state, delta) -> Result<_, _>`.
    const GOOD_FN: &str = r"Moves a state along a tangent vector.

Details about the step.

# Arguments

* `state` - The state x.
* `delta` - The step δ, which
  may span lines.

# Returns

x ⊞ δ.

# Errors

`E` if it fails.

# Examples

```
assert_eq!(1 + 1, 2);
```";

    /// The signature `GOOD_FN` documents.
    const SIGNATURE: &str = "pub fn step(state: f64, delta: f64) -> Result<f64, E>";

    fn lines(doc: &str) -> Vec<String> {
        doc.split('\n').map(str::to_owned).collect()
    }

    fn lint_fn(doc: &str, signature: &str) -> Vec<String> {
        let item: syn::ItemFn = syn::parse_str(&format!("{signature} {{}}")).unwrap();
        lint(&lines(doc), Kind::Fn(&item.sig))
    }

    fn assert_one(found: &[String], expected: &str) {
        assert!(
            found.iter().any(|message| message.contains(expected)),
            "expected a message containing {expected:?}, got {found:#?}"
        );
    }

    #[test]
    fn good_function_passes() {
        assert_eq!(lint_fn(GOOD_FN, SIGNATURE), Vec::<String>::new());
    }

    #[test]
    fn missing_docs() {
        assert_eq!(lint(&[], Kind::Member), ["missing doc comment"]);
    }

    #[test]
    fn summary_rules() {
        let multi = GOOD_FN.replacen("vector.", "vector\nand more.", 1);
        assert_one(&lint_fn(&multi, SIGNATURE), "summary must be one line");
        let unpunctuated = GOOD_FN.replacen("vector.", "vector:", 1);
        assert_one(&lint_fn(&unpunctuated, SIGNATURE), "must end with");
        assert_one(&lint(&lines("# Heading\n\nText."), Kind::Member), "not a heading");
        assert_one(&lint(&lines("\nText."), Kind::Member), "not a blank line");
        assert_eq!(lint(&lines("Is it?"), Kind::Member), Vec::<String>::new());
    }

    #[test]
    fn heading_spelling_level_and_order() {
        let singular = GOOD_FN.replace("# Examples", "# Example");
        assert_one(&lint_fn(&singular, SIGNATURE), "`# Example` should be `# Examples`");
        let shouty = GOOD_FN.replace("# Returns", "# RETURNS");
        assert_one(&lint_fn(&shouty, SIGNATURE), "should be `# Returns`");
        let nested = GOOD_FN.replace("# Errors", "## Errors");
        assert_one(&lint_fn(&nested, SIGNATURE), "level-1");
        let swapped = GOOD_FN
            .replace("# Returns", "# Tmp")
            .replace("# Errors", "# Returns")
            .replace("# Tmp", "# Errors");
        assert_one(&lint_fn(&swapped, SIGNATURE), "`# Returns` must come before `# Errors`");
        let trailing = format!("{GOOD_FN}\n\n# Notes\n\nMore.");
        assert_one(&lint_fn(&trailing, SIGNATURE), "must be the last section");
        let custom = GOOD_FN.replace("Details about the step.", "Details.\n\n# Invariant\n\nHolds.");
        assert_eq!(lint_fn(&custom, SIGNATURE), Vec::<String>::new());
    }

    #[test]
    fn headings_inside_code_blocks_are_code() {
        let hidden = GOOD_FN.replace("assert_eq!(1 + 1, 2);", "# let hidden = 1;\nassert_eq!(hidden, 1);");
        assert_eq!(lint_fn(&hidden, SIGNATURE), Vec::<String>::new());
    }

    #[test]
    fn arguments_match_the_signature() {
        let missing = GOOD_FN.replace("* `delta` - The step δ, which\n  may span lines.\n", "");
        assert_one(&lint_fn(&missing, SIGNATURE), "parameter `delta` is missing");
        let stale = GOOD_FN.replace("`delta`", "`step`");
        let found = lint_fn(&stale, SIGNATURE);
        assert_one(&found, "parameter `delta` is missing");
        assert_one(&found, "documents `step`, which is not a parameter");
        let reordered = lint_fn(GOOD_FN, "pub fn step(delta: f64, state: f64) -> Result<f64, E>");
        assert_one(&reordered, "signature order");
        let dashed = GOOD_FN.replace("* `state`", "- `state`");
        assert_one(&lint_fn(&dashed, SIGNATURE), "entries must read");
        let colon = GOOD_FN.replace("`state` - ", "`state`: ");
        assert_one(&lint_fn(&colon, SIGNATURE), "entries must read");
        let none = lint_fn(GOOD_FN, "pub fn step(&self) -> Result<f64, E>");
        assert_one(&none, "remove `# Arguments`");
        let undocumented = GOOD_FN.replace("# Arguments", "# Inputs");
        assert_one(&lint_fn(&undocumented, SIGNATURE), "missing `# Arguments`");
    }

    #[test]
    fn returns_and_errors_match_the_signature() {
        let unit = lint_fn(GOOD_FN, "pub fn step(state: f64, delta: f64)");
        assert_one(&unit, "remove `# Returns`");
        assert_one(&unit, "remove `# Errors`");
        let infallible = lint_fn(GOOD_FN, "pub fn step(state: f64, delta: f64) -> f64");
        assert_one(&infallible, "remove `# Errors`");
        assert!(
            !infallible.iter().any(|message| message.contains("Returns")),
            "{infallible:?}"
        );
        let no_returns = GOOD_FN.replace("# Returns\n\nx ⊞ δ.\n\n", "");
        assert_one(&lint_fn(&no_returns, SIGNATURE), "no `# Returns`");
        let no_errors = GOOD_FN.replace("# Errors\n\n`E` if it fails.\n\n", "");
        assert_one(&lint_fn(&no_errors, SIGNATURE), "no `# Errors`");
    }

    #[test]
    fn examples_must_run() {
        let math = GOOD_FN.replace("```\nassert", "```math\nassert");
        assert_one(&lint_fn(&math, SIGNATURE), "needs a runnable code block");
        let ignored = GOOD_FN.replace("```\nassert", "```ignore\nassert");
        let found = lint_fn(&ignored, SIGNATURE);
        assert_one(&found, "```ignore blocks never run");
        assert_one(&found, "needs a runnable code block");
        let no_run = GOOD_FN.replace("```\nassert", "```rust,no_run\nassert");
        assert_eq!(lint_fn(&no_run, SIGNATURE), Vec::<String>::new());
        let open = GOOD_FN.trim_end_matches("```");
        assert_one(&lint_fn(open, SIGNATURE), "unclosed code block");
        assert_one(&lint(&lines("A type."), Kind::Type), "missing `# Examples`");
    }

    #[test]
    fn equations_use_katex() {
        let display = GOOD_FN.replace(
            "Details about the step.",
            "Details, $x \\boxplus \\delta$ inline:\n\n```math\nx \\boxplus \\delta \\\\ y\n```",
        );
        assert_eq!(lint_fn(&display, SIGNATURE), Vec::<String>::new());
        let text = GOOD_FN.replace("Details about the step.", "```text\nx ⊞ δ\n```");
        assert_one(&lint_fn(&text, SIGNATURE), "```text is not allowed");
        let dollars = GOOD_FN.replace("Details about the step.", "$$\nx = y\n$$");
        assert_one(&lint_fn(&dollars, SIGNATURE), "goes in a ```math block");
        let unbalanced = GOOD_FN.replace("Details about the step.", "Costs $5 and $x$.");
        assert_one(&lint_fn(&unbalanced, SIGNATURE), "unbalanced `$`");
        let code = GOOD_FN.replace("Details about the step.", "Use `$` or `$$` in code.");
        assert_eq!(lint_fn(&code, SIGNATURE), Vec::<String>::new());
        let escaped = GOOD_FN.replace("Details about the step.", "Spaced $a\\,b$ and $\\lVert x \\rVert$.");
        let found = lint_fn(&escaped, SIGNATURE);
        assert_eq!(found.len(), 1, "{found:?}");
        assert_one(&found, "backslash of `\\,`");
    }

    #[test]
    fn modules_need_examples_only_with_items() {
        assert_eq!(
            lint(&lines("A stub."), Kind::Module { has_items: false }),
            Vec::<String>::new()
        );
        assert_one(
            &lint(&lines("A module."), Kind::Module { has_items: true }),
            "missing `# Examples`",
        );
    }

    #[test]
    fn non_functions_have_no_function_sections() {
        let doc = "A type.\n\n# Returns\n\nNothing.\n\n# Examples\n\n```\n```";
        assert_one(&lint(&lines(doc), Kind::Type), "only functions have `# Returns`");
    }

    /// Walks a module tree: public items are checked, private, test and trait-impl items are not.
    #[test]
    fn walks_only_the_public_api() {
        let source = r"
            //! A module.
            /// Private, so unchecked
            fn private() {}
            /// A struct.
            pub struct Thing {
                /// no period
                pub field: f64,
                hidden: f64,
            }
            impl Thing {
                /// Gets it.
                pub fn get(&self) -> f64 { self.field }
            }
            impl core::fmt::Display for Thing {
                fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result { Ok(()) }
            }
            /// Tests.
            #[cfg(test)]
            pub mod tests { pub fn undocumented() {} }
        ";
        let file = syn::parse_file(source).unwrap();
        let mut checker = Checker::default();
        let scope = Scope {
            file: Path::new("lib.rs"),
            dir: PathBuf::new(),
            path: "krate".to_owned(),
        };
        checker.module(&scope, 1, &doc_lines(&file.attrs), &file.items).unwrap();
        let found: Vec<String> = checker
            .diagnostics
            .iter()
            .map(|d| d.relative_to(Path::new("")))
            .collect();
        assert_eq!(
            found,
            [
                "lib.rs:1: krate: missing `# Examples` with a runnable doctest",
                "lib.rs:6: krate::Thing: missing `# Examples` with a runnable doctest",
                "lib.rs:8: krate::Thing::field: summary must end with '.', '?' or '!'",
                "lib.rs:13: krate::Thing::get: returns a value but has no `# Returns`",
                "lib.rs:13: krate::Thing::get: missing `# Examples` with a runnable doctest",
            ]
        );
    }
}
