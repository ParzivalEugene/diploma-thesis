use xtask::report_rules::{
    Page, TextLine, disallowed_fonts, margin_violations, median_line_pitch, page_count,
    parse_bbox_layout, share_of_characters_at_size,
};

const A4_WIDTH: f64 = 595.28;
const A4_HEIGHT: f64 = 841.89;

fn line(x_min: f64, y_min: f64, x_max: f64, y_max: f64, text: &str) -> TextLine {
    TextLine {
        x_min,
        y_min,
        x_max,
        y_max,
        text: text.to_owned(),
    }
}

fn a4_page(lines: Vec<TextLine>) -> Page {
    Page {
        width: A4_WIDTH,
        height: A4_HEIGHT,
        blocks: vec![lines],
    }
}

#[test]
fn page_count_is_read_from_pdfinfo() {
    assert_eq!(
        page_count("Title: x\nPages:           34\nEncrypted: no\n"),
        Some(34)
    );
}

#[test]
fn foreign_font_is_reported_and_subset_prefix_is_tolerated() {
    let output = "name type\n---- ----\nABCDEF+TimesNewRomanPSMT CID\nABCDEF+Arial CID\nSTIXTwoMath-Regular CID\n";
    assert_eq!(disallowed_fonts(output), ["ABCDEF+Arial"]);
}

#[test]
fn bbox_layout_keeps_lines_inside_blocks() {
    let xhtml = r#"
<page width="595.0" height="842.0">
<block xMin="1" yMin="1" xMax="2" yMax="2">
<line xMin="85.0" yMin="60.0" xMax="500.0" yMax="75.0">
<word xMin="85" yMin="60" xMax="90" yMax="75">Граф</word>
<word xMin="95" yMin="60" xMax="99" yMax="75">дорог</word>
</line>
</block>
</page>
"#;
    let pages = parse_bbox_layout(xhtml);
    assert_eq!(pages[0].blocks[0][0].text, "Граф дорог");
    assert!((pages[0].blocks[0][0].x_min - 85.0).abs() < f64::EPSILON);
}

#[test]
fn line_pitch_ignores_outliers_by_taking_the_median() {
    let page = a4_page(vec![
        line(90.0, 100.0, 500.0, 115.0, "a"),
        line(90.0, 124.15, 500.0, 139.0, "b"),
        line(90.0, 148.3, 500.0, 163.0, "c"),
        line(90.0, 190.0, 500.0, 205.0, "d"),
    ]);
    let pitch = median_line_pitch(&[page]).unwrap();
    assert!((pitch - 24.15).abs() < 0.01);
}

#[test]
fn narrow_table_rows_do_not_affect_the_line_pitch() {
    let page = a4_page(vec![
        line(90.0, 100.0, 500.0, 115.0, "abzac"),
        line(90.0, 121.0, 500.0, 136.0, "abzac"),
        line(90.0, 200.0, 200.0, 215.0, "row"),
        line(90.0, 224.0, 200.0, 239.0, "row"),
        line(90.0, 248.0, 200.0, 263.0, "row"),
    ]);
    let pitch = median_line_pitch(&[page]);
    assert!(pitch.is_some_and(|value| (value - 21.0).abs() < 0.01));
}

#[test]
fn line_past_the_left_margin_is_reported() {
    let page = a4_page(vec![line(70.0, 100.0, 500.0, 115.0, "сбежала")]);
    assert_eq!(margin_violations(3, &page).len(), 1);
}

#[test]
fn line_inside_all_margins_is_accepted() {
    let page = a4_page(vec![line(85.1, 100.0, 552.0, 115.0, "норма")]);
    assert_eq!(margin_violations(3, &page), Vec::<String>::new());
}

#[test]
fn page_number_in_the_bottom_margin_is_not_a_violation() {
    let page = a4_page(vec![line(300.0, 800.0, 310.0, 815.0, "12")]);
    assert_eq!(margin_violations(12, &page), Vec::<String>::new());
}

#[test]
fn text_in_the_bottom_margin_is_a_violation() {
    let page = a4_page(vec![line(90.0, 800.0, 310.0, 815.0, "абзац")]);
    assert_eq!(margin_violations(12, &page).len(), 1);
}

#[test]
fn character_share_counts_only_the_requested_size() {
    let xml = r##"
<fontspec id="0" size="14" family="Times" color="#000000"/>
<fontspec id="1" size="10" family="Times" color="#000000"/>
<text top="1" left="1" width="1" height="1" font="0">abc</text>
<text top="1" left="1" width="1" height="1" font="1">d</text>
"##;
    let share = share_of_characters_at_size(xml, 14).unwrap();
    assert!((share - 0.75).abs() < f64::EPSILON);
}
