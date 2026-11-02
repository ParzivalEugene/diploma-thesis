pub const POINTS_PER_MM: f64 = 72.0 / 25.4;
pub const MARGIN_LEFT_MM: f64 = 30.0;
pub const MARGIN_RIGHT_MM: f64 = 15.0;
pub const MARGIN_VERTICAL_MM: f64 = 20.0;
pub const LINE_PITCH_POINTS: f64 = 24.15;
pub const LINE_PITCH_TOLERANCE: f64 = 0.2;
pub const MARGIN_TOLERANCE_POINTS: f64 = 2.0;

const BODY_LINE_MIN_WIDTH_SHARE: f64 = 0.8;

const ALLOWED_FONT_MARKERS: [&str; 2] = ["TimesNewRoman", "STIXTwoMath"];

#[derive(Debug, Clone, PartialEq)]
pub struct TextLine {
    pub x_min: f64,
    pub y_min: f64,
    pub x_max: f64,
    pub y_max: f64,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Page {
    pub width: f64,
    pub height: f64,
    pub blocks: Vec<Vec<TextLine>>,
}

pub fn page_count(pdfinfo_output: &str) -> Option<usize> {
    pdfinfo_output
        .lines()
        .find_map(|line| line.strip_prefix("Pages:"))
        .and_then(|value| value.trim().parse().ok())
}

pub fn disallowed_fonts(pdffonts_output: &str) -> Vec<String> {
    pdffonts_output
        .lines()
        .skip(2)
        .filter_map(|line| line.split_whitespace().next())
        .filter(|name| {
            !ALLOWED_FONT_MARKERS
                .iter()
                .any(|marker| name.contains(marker))
        })
        .map(str::to_owned)
        .collect()
}

pub fn parse_bbox_layout(xhtml: &str) -> Vec<Page> {
    let mut pages = Vec::new();
    let mut current_line: Option<TextLine> = None;
    for raw in xhtml.lines() {
        let line = raw.trim();
        if line.starts_with("<page ") {
            pages.push(Page {
                width: attribute(line, "width").unwrap_or_default(),
                height: attribute(line, "height").unwrap_or_default(),
                blocks: Vec::new(),
            });
        } else if line.starts_with("<block ") {
            if let Some(page) = pages.last_mut() {
                page.blocks.push(Vec::new());
            }
        } else if line.starts_with("<line ") {
            current_line = Some(TextLine {
                x_min: attribute(line, "xMin").unwrap_or_default(),
                y_min: attribute(line, "yMin").unwrap_or_default(),
                x_max: attribute(line, "xMax").unwrap_or_default(),
                y_max: attribute(line, "yMax").unwrap_or_default(),
                text: String::new(),
            });
        } else if line.starts_with("<word ") {
            if let Some(current) = current_line.as_mut() {
                let word = line
                    .split_once('>')
                    .and_then(|(_, rest)| rest.split_once("</word>"))
                    .map_or("", |(word, _)| word);
                if !current.text.is_empty() {
                    current.text.push(' ');
                }
                current.text.push_str(word);
            }
        } else if line.starts_with("</line>")
            && let (Some(finished), Some(block)) = (
                current_line.take(),
                pages.last_mut().and_then(|page| page.blocks.last_mut()),
            )
        {
            block.push(finished);
        }
    }
    pages
}

fn attribute(tag: &str, name: &str) -> Option<f64> {
    let marker = format!("{name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let rest = &tag[start..];
    rest[..rest.find('"')?].parse().ok()
}

pub fn median_line_pitch(pages: &[Page]) -> Option<f64> {
    let mut pitches: Vec<f64> = pages.iter().flat_map(page_pitches).collect();
    if pitches.is_empty() {
        return None;
    }
    pitches.sort_by(f64::total_cmp);
    Some(pitches[pitches.len() / 2])
}

fn page_pitches(page: &Page) -> Vec<f64> {
    let text_width = page.width - (MARGIN_LEFT_MM + MARGIN_RIGHT_MM) * POINTS_PER_MM;
    let mut tops: Vec<f64> = page
        .blocks
        .iter()
        .flatten()
        .filter(|line| line.x_max - line.x_min >= BODY_LINE_MIN_WIDTH_SHARE * text_width)
        .map(|line| line.y_min)
        .collect();
    tops.sort_by(f64::total_cmp);
    tops.windows(2)
        .map(|pair| pair[1] - pair[0])
        .filter(|pitch| *pitch > 1.0)
        .collect()
}

pub fn margin_violations(page_number: usize, page: &Page) -> Vec<String> {
    let left = MARGIN_LEFT_MM * POINTS_PER_MM - MARGIN_TOLERANCE_POINTS;
    let right = page.width - MARGIN_RIGHT_MM * POINTS_PER_MM + MARGIN_TOLERANCE_POINTS;
    let top = MARGIN_VERTICAL_MM * POINTS_PER_MM - MARGIN_TOLERANCE_POINTS;
    let bottom = page.height - MARGIN_VERTICAL_MM * POINTS_PER_MM + MARGIN_TOLERANCE_POINTS;
    page.blocks
        .iter()
        .flatten()
        .filter(|line| !is_page_number(line, bottom))
        .filter(|line| {
            line.x_min < left || line.x_max > right || line.y_min < top || line.y_max > bottom
        })
        .map(|line| format!("стр. {page_number}: «{}» вне полей", line.text))
        .collect()
}

fn is_page_number(line: &TextLine, bottom: f64) -> bool {
    line.y_min >= bottom - MARGIN_TOLERANCE_POINTS
        && line.text.chars().all(|symbol| symbol.is_ascii_digit())
}

pub fn share_of_characters_at_size(pdftohtml_xml: &str, size: u32) -> Option<f64> {
    let mut fonts = std::collections::BTreeMap::new();
    let mut at_size = 0usize;
    let mut total = 0usize;
    for raw in pdftohtml_xml.lines() {
        let line = raw.trim();
        if line.starts_with("<fontspec ") {
            if let (Some(id), Some(font_size)) =
                (text_attribute(line, "id"), text_attribute(line, "size"))
            {
                fonts.insert(id.to_owned(), font_size.to_owned());
            }
        } else if line.starts_with("<text ")
            && let Some(font) = text_attribute(line, "font")
        {
            let characters = tag_free_length(line);
            total += characters;
            if fonts
                .get(font)
                .is_some_and(|value| value == &size.to_string())
            {
                at_size += characters;
            }
        }
    }
    (total > 0).then(|| ratio(at_size, total))
}

#[expect(
    clippy::cast_precision_loss,
    reason = "счётчики символов в отчёте далеко меньше 2^52"
)]
fn ratio(part: usize, whole: usize) -> f64 {
    part as f64 / whole as f64
}

fn text_attribute<'a>(tag: &'a str, name: &str) -> Option<&'a str> {
    let marker = format!(" {name}=\"");
    let start = tag.find(&marker)? + marker.len();
    let rest = &tag[start..];
    Some(&rest[..rest.find('"')?])
}

fn tag_free_length(line: &str) -> usize {
    let mut inside_tag = false;
    let mut count = 0;
    for symbol in line.chars() {
        match symbol {
            '<' => inside_tag = true,
            '>' => inside_tag = false,
            _ if !inside_tag => count += 1,
            _ => {}
        }
    }
    count
}
