#![allow(clippy::print_stdout, reason = "консольный инструмент печатает отчёт")]

use std::collections::HashSet;
use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::Command;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use walkdir::WalkDir;
use xtask::report_rules::{self, LINE_PITCH_POINTS, LINE_PITCH_TOLERANCE};
use xtask::source_rules::{self, LARGE_FILE_LINES};

const MIN_REPORT_PAGES: usize = 25;
const MAX_REPORT_PAGES: usize = 35;
const MIN_REVIEW_PAGES: usize = 15;
const MAX_REVIEW_PAGES: usize = 20;
const MIN_SOURCES: usize = 10;
const MIN_FOREIGN_PRINT_SOURCES: usize = 3;
const MIN_BODY_FONT_SHARE: f64 = 0.85;
const BODY_FONT_SIZE: u32 = 14;

fn main() -> Result<()> {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let words: Vec<&str> = arguments.iter().map(String::as_str).collect();
    match words.as_slice() {
        ["lint-source"] => lint_source(Path::new("crates")),
        ["report-format-check", input] => report_format_check(Path::new(input)),
        ["report-check", input] => report_check(Path::new(input)),
        _ => bail!("команды: lint-source | report-format-check <typ> | report-check <typ>"),
    }
}

fn rust_files(root: &Path) -> Vec<PathBuf> {
    WalkDir::new(root)
        .into_iter()
        .filter_map(Result::ok)
        .map(walkdir::DirEntry::into_path)
        .filter(|path| path.extension().is_some_and(|extension| extension == "rs"))
        .collect()
}

fn lint_source(root: &Path) -> Result<()> {
    let files = rust_files(root);
    let mut crate_functions = HashSet::new();
    let mut sources = Vec::new();
    for path in files {
        let source = std::fs::read_to_string(&path)?;
        crate_functions.extend(
            source_rules::function_names(&source)
                .with_context(|| format!("разбор {}", path.display()))?,
        );
        sources.push((path, source));
    }

    let mut violations = Vec::new();
    for (path, source) in &sources {
        let shown = path.display();
        let stem = path
            .file_stem()
            .and_then(|stem| stem.to_str())
            .unwrap_or("");
        if source_rules::is_forbidden_module_name(stem) {
            violations.push(format!("{shown}: запрещённое имя модуля"));
        }
        for line in source_rules::forbidden_comment_lines(source) {
            violations.push(format!("{shown}:{line}: комментарий в коде"));
        }
        for test in source_rules::test_names(source)? {
            if source_rules::is_badly_named_test(&test.name, &crate_functions) {
                violations.push(format!(
                    "{shown}:{}: тест `{}` назван не по поведению",
                    test.line, test.name
                ));
            }
        }
        let lines = source.lines().count();
        if lines > LARGE_FILE_LINES {
            println!("сигнал ревью: {shown} содержит {lines} строк");
        }
    }

    if violations.is_empty() {
        println!("lint-source: нарушений нет");
        return Ok(());
    }
    bail!("lint-source:\n{}", violations.join("\n"));
}

fn tool_output(program: &str, arguments: &[&str]) -> Result<String> {
    let output = Command::new(program)
        .args(arguments)
        .output()
        .with_context(|| format!("запуск {program}"))?;
    if !output.status.success() {
        bail!("{program}: {}", String::from_utf8_lossy(&output.stderr));
    }
    Ok(String::from_utf8_lossy(&output.stdout).into_owned())
}

fn compile_report(input: &Path) -> Result<PathBuf> {
    let output_dir = input
        .parent()
        .context("у входного файла нет каталога")?
        .join("out");
    std::fs::create_dir_all(&output_dir)?;
    let stem = input.file_stem().context("у файла нет имени")?;
    let pdf = output_dir.join(stem).with_extension("pdf");
    let output = Command::new("typst")
        .args(["compile", "--root", "."])
        .arg(input)
        .arg(&pdf)
        .output()
        .context("запуск typst")?;
    let diagnostics = String::from_utf8_lossy(&output.stderr);
    if !output.status.success() {
        bail!("typst:\n{diagnostics}");
    }
    if diagnostics.contains("warning:") {
        bail!("typst выдал предупреждения:\n{diagnostics}");
    }
    Ok(pdf)
}

#[derive(Default)]
struct Checklist {
    table: String,
    failed: usize,
}

impl Checklist {
    fn record(&mut self, name: &str, passed: bool, details: &str) {
        let mark = if passed { "ok " } else { "НЕТ" };
        let _ = writeln!(self.table, "[{mark}] {name}: {details}");
        if !passed {
            self.failed += 1;
        }
    }

    fn finish(self) -> Result<()> {
        print!("{}", self.table);
        if self.failed > 0 {
            bail!("не пройдено проверок: {}", self.failed);
        }
        Ok(())
    }
}

fn format_checks(pdf: &Path, list: &mut Checklist) -> Result<()> {
    let path = pdf.to_str().context("путь не UTF-8")?;

    let fonts = report_rules::disallowed_fonts(&tool_output("pdffonts", &[path])?);
    list.record(
        "шрифты",
        fonts.is_empty(),
        &format!("посторонние: {fonts:?}"),
    );

    let layout = tool_output("pdftotext", &["-bbox-layout", path, "-"])?;
    let pages = report_rules::parse_bbox_layout(&layout);
    let body = pages.get(1..).unwrap_or_default();

    let pitch = report_rules::median_line_pitch(body);
    let pitch_ok =
        pitch.is_some_and(|value| (value - LINE_PITCH_POINTS).abs() <= LINE_PITCH_TOLERANCE);
    list.record(
        "интервал 1,5",
        pitch_ok,
        &format!("медиана шага {pitch:?} pt, нужно {LINE_PITCH_POINTS}"),
    );

    let margins: Vec<String> = body
        .iter()
        .enumerate()
        .flat_map(|(index, page)| report_rules::margin_violations(index + 2, page))
        .collect();
    list.record(
        "поля 30/15/20/20 мм",
        margins.is_empty(),
        &margins
            .first()
            .cloned()
            .unwrap_or_else(|| "в пределах".to_owned()),
    );

    let html = tool_output("pdftohtml", &["-xml", "-stdout", "-i", "-zoom", "1", path])?;
    let share = report_rules::share_of_characters_at_size(&html, BODY_FONT_SIZE);
    list.record(
        "кегль 14",
        share.is_some_and(|value| value >= MIN_BODY_FONT_SHARE),
        &format!("доля символов {share:?}, нужно от {MIN_BODY_FONT_SHARE}"),
    );
    Ok(())
}

fn report_format_check(input: &Path) -> Result<()> {
    let pdf = compile_report(input)?;
    let mut list = Checklist::default();
    format_checks(&pdf, &mut list)?;
    list.finish()
}

#[derive(Deserialize)]
struct Facts {
    review_start: usize,
    review_end: usize,
    report_end: usize,
    cited: usize,
    foreign_print: usize,
    unreferenced_figures: Vec<String>,
}

fn report_check(input: &Path) -> Result<()> {
    let pdf = compile_report(input)?;
    let mut list = Checklist::default();
    format_checks(&pdf, &mut list)?;

    let source = input.to_str().context("путь не UTF-8")?;
    let facts: Facts = serde_json::from_str(&tool_output(
        "typst",
        &[
            "eval",
            "--root",
            ".",
            "--in",
            source,
            "query(<report-facts>).first().value",
        ],
    )?)
    .context("разбор report-facts")?;

    let body_pages = facts.report_end.saturating_sub(1);
    list.record(
        "объём 25–35 стр. без титульного и приложений",
        (MIN_REPORT_PAGES..=MAX_REPORT_PAGES).contains(&body_pages),
        &format!("{body_pages} стр."),
    );
    let review_pages = (facts.review_end + 1).saturating_sub(facts.review_start);
    list.record(
        "обзор 15–20 стр.",
        (MIN_REVIEW_PAGES..=MAX_REVIEW_PAGES).contains(&review_pages),
        &format!("{review_pages} стр."),
    );
    list.record(
        "источников от 10",
        facts.cited >= MIN_SOURCES,
        &format!("{}", facts.cited),
    );
    list.record(
        "иностранных печатных от 3",
        facts.foreign_print >= MIN_FOREIGN_PRINT_SOURCES,
        &format!("{}", facts.foreign_print),
    );
    list.record(
        "ссылки на рисунки и таблицы",
        facts.unreferenced_figures.is_empty(),
        &format!("без ссылок: {:?}", facts.unreferenced_figures),
    );
    list.finish()
}
