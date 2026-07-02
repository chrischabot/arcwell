use super::*;

const RESEARCH_HOST_SEARCH_MAX_RESULTS: usize = 100;
const RESEARCH_HOST_SEARCH_MAX_DOMAINS: usize = 20;

pub(crate) fn normalize_research_host_search_input(
    input: ResearchHostSearchInput,
) -> Result<NormalizedResearchHostSearchInput> {
    validate_id(&input.run_id)?;
    let role_run_id = input
        .role_run_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| -> Result<String> {
            validate_id(value)?;
            Ok(value.to_string())
        })
        .transpose()?;
    let host = normalize_research_key(input.host, "research host")?;
    let tool_surface = normalize_research_key(input.tool_surface, "research search tool surface")?;
    validate_query(&input.query)?;
    let query = redact_secret_like_text(input.query.trim());
    let query_intent = input
        .query_intent
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| sanitize_work_text(value, 500))
        .transpose()?;
    if let Some(recency) = input.requested_recency
        && !(0..=3650).contains(&recency)
    {
        bail!("host search requested_recency must be between 0 and 3650 days");
    }
    if input.requested_domains.len() > RESEARCH_HOST_SEARCH_MAX_DOMAINS {
        bail!("too many host search requested domains");
    }
    let mut requested_domains = Vec::new();
    for domain in input.requested_domains {
        let domain = domain
            .trim()
            .trim_start_matches("site:")
            .to_ascii_lowercase();
        if domain.is_empty() {
            continue;
        }
        if domain.len() > 200 || domain.contains('/') || domain.contains('\\') {
            bail!("invalid host search requested domain");
        }
        if !requested_domains.contains(&domain) {
            requested_domains.push(domain);
        }
    }
    let cost_decision_id = input
        .cost_decision_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| -> Result<String> {
            validate_id(value)?;
            Ok(value.to_string())
        })
        .transpose()?;
    if input.results.is_empty() {
        bail!("host search proof must include at least one result");
    }
    if input.results.len() > RESEARCH_HOST_SEARCH_MAX_RESULTS {
        bail!("too many host search results");
    }
    let mut seen = BTreeSet::new();
    let mut normalized_results = Vec::new();
    for result in input.results {
        if result.rank == 0 || result.rank > RESEARCH_HOST_SEARCH_MAX_RESULTS {
            bail!("host search result rank is out of range");
        }
        let canonical_url = canonical_source_url(&result.url)?;
        if result.selected_for_ingest {
            validate_fetch_url(&canonical_url)?;
        }
        if !seen.insert((result.rank, canonical_url.clone())) {
            bail!("duplicate host search result rank/url");
        }
        let title = sanitize_work_text(&result.title, 500)?;
        if title.trim().is_empty() {
            bail!("host search result title cannot be empty");
        }
        let snippet = result
            .snippet
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| sanitize_work_text(value, 2_000))
            .transpose()?
            .context("host search result snippet cannot be empty")?;
        let published_at = result
            .published_at
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| sanitize_work_text(value, 100))
            .transpose()?;
        let source_family_guess = result
            .source_family_guess
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .map(|value| normalize_research_key(value.to_string(), "source family guess"))
            .transpose()?;
        let provider_metadata = sanitize_work_json(result.provider_metadata)?;
        normalized_results.push(NormalizedResearchHostSearchResult {
            rank: result.rank,
            title,
            url: result.url,
            canonical_url,
            snippet: Some(snippet),
            published_at,
            source_family_guess,
            provider_metadata,
            selected_for_ingest: result.selected_for_ingest,
        });
    }
    Ok(NormalizedResearchHostSearchInput {
        run_id: input.run_id,
        role_run_id,
        host,
        tool_surface,
        query,
        query_intent,
        requested_recency: input.requested_recency,
        requested_domains,
        cost_decision_id,
        results: normalized_results,
    })
}

pub(crate) const RESEARCH_DOCUMENT_MAX_BYTES: u64 = 10_000_000;
const RESEARCH_TABLE_MAX_ROWS: usize = 2_000;
const RESEARCH_TABLE_MAX_COLUMNS: usize = 200;
const RESEARCH_TABLE_MAX_CELLS: usize = 50_000;

pub(crate) fn normalize_research_document_input(
    mut input: ResearchDocumentInput,
) -> Result<ResearchDocumentInput> {
    validate_id(&input.run_id)?;
    input.research_source_id = input
        .research_source_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| -> Result<String> {
            validate_id(value)?;
            Ok(value.to_string())
        })
        .transpose()?;
    input.source_card_id = input
        .source_card_id
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| -> Result<String> {
            validate_id(value)?;
            Ok(value.to_string())
        })
        .transpose()?;
    if !input.path.exists() {
        bail!("research document path does not exist");
    }
    if !input.path.is_file() {
        bail!("research document path is not a file");
    }
    input.path = input
        .path
        .canonicalize()
        .with_context(|| format!("canonicalizing {}", input.path.display()))?;
    input.media_type = input
        .media_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| -> Result<String> {
            let value = value
                .split(';')
                .next()
                .unwrap_or(value)
                .trim()
                .to_ascii_lowercase();
            validate_key(&value)?;
            Ok(value)
        })
        .transpose()?;
    Ok(input)
}

pub(crate) fn infer_research_document_media_type(path: &Path) -> String {
    match path
        .extension()
        .and_then(|value| value.to_str())
        .unwrap_or_default()
        .to_ascii_lowercase()
        .as_str()
    {
        "csv" => "text/csv".to_string(),
        "tsv" => "text/tab-separated-values".to_string(),
        "pdf" => "application/pdf".to_string(),
        "xlsx" | "xlsm" => {
            "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet".to_string()
        }
        _ => "application/octet-stream".to_string(),
    }
}

pub(crate) fn extract_research_document_content(
    document_id: &str,
    path: &Path,
    media_type: &str,
    bytes: &[u8],
) -> Result<ResearchDocumentExtraction> {
    match media_type {
        "text/csv" | "application/csv" => extract_delimited_document(document_id, bytes, ','),
        "text/tab-separated-values" | "text/tsv" => {
            extract_delimited_document(document_id, bytes, '\t')
        }
        "application/pdf" => extract_pdf_document(document_id, path),
        "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet"
        | "application/xlsx" => extract_xlsx_document(document_id, path),
        other => Ok(blocked_document_extraction(
            "unsupported_media_type",
            &format!("unsupported research document media type: {other}"),
            "unsupported",
        )),
    }
}

pub(crate) fn blocked_document_extraction(
    warning: &str,
    message: &str,
    extractor_name: &str,
) -> ResearchDocumentExtraction {
    ResearchDocumentExtraction {
        extractor_name: extractor_name.to_string(),
        extractor_version: "none".to_string(),
        status: format!("blocked_{warning}"),
        page_count: 0,
        sheet_count: 0,
        warning_flags: vec![warning.to_string()],
        error_message_redacted: Some(excerpt(message, 2_000)),
        spans: Vec::new(),
        tables: Vec::new(),
    }
}

pub(crate) fn extract_delimited_document(
    document_id: &str,
    bytes: &[u8],
    delimiter: char,
) -> Result<ResearchDocumentExtraction> {
    let text = String::from_utf8(bytes.to_vec()).context("research table file is not UTF-8")?;
    let rows = parse_delimited_rows(&text, delimiter)?;
    if rows.is_empty() {
        bail!("research table file is empty");
    }
    if rows.len() > RESEARCH_TABLE_MAX_ROWS {
        bail!("research table has too many rows");
    }
    let column_count = rows.iter().map(Vec::len).max().unwrap_or(0);
    if column_count > RESEARCH_TABLE_MAX_COLUMNS {
        bail!("research table has too many columns");
    }
    if rows.len().saturating_mul(column_count) > RESEARCH_TABLE_MAX_CELLS {
        bail!("research table has too many cells");
    }
    let table_id = "table-1".to_string();
    let table_db_id = research_table_db_id(document_id, &table_id);
    let headers = rows.first().cloned().unwrap_or_default();
    let mut cells = Vec::new();
    for (row_index, row) in rows.iter().enumerate() {
        for column_index in 0..column_count {
            let raw = row.get(column_index).cloned().unwrap_or_default();
            let normalized_text = normalize_table_cell_text(&raw);
            let row_header = if row_index > 0 {
                row.first()
                    .map(|value| sanitize_work_text(value, 500))
                    .transpose()?
                    .filter(|value| !value.trim().is_empty())
            } else {
                None
            };
            let column_header = headers
                .get(column_index)
                .map(|value| sanitize_work_text(value, 500))
                .transpose()?
                .filter(|value| !value.trim().is_empty());
            let raw_text = sanitize_work_text(&raw, 4_000)?;
            let (numeric_value, footnote_refs) = parse_table_numeric_and_footnote_refs(&raw);
            cells.push(ResearchTableCell {
                id: research_table_cell_id(&table_db_id, row_index, column_index),
                table_id: table_db_id.clone(),
                row_index,
                column_index,
                row_header,
                column_header,
                raw_text,
                normalized_text,
                numeric_value,
                unit: None,
                footnote_refs,
                bbox_json: None,
                confidence: 0.98,
            });
        }
    }
    let table = ResearchTable {
        id: table_db_id.clone(),
        document_id: document_id.to_string(),
        table_id,
        page_number: None,
        sheet_name: Some("csv".to_string()),
        caption: Some("Delimited table extraction".to_string()),
        bbox_json: None,
        row_count: rows.len(),
        column_count,
        extraction_method: if delimiter == '\t' {
            "tsv-parser"
        } else {
            "csv-parser"
        }
        .to_string(),
        confidence: 0.98,
        warning_flags: Vec::new(),
    };
    Ok(ResearchDocumentExtraction {
        extractor_name: table.extraction_method.clone(),
        extractor_version: "arcwell-delimited-v1".to_string(),
        status: "extracted".to_string(),
        page_count: 0,
        sheet_count: 1,
        warning_flags: Vec::new(),
        error_message_redacted: None,
        spans: Vec::new(),
        tables: vec![ResearchTableRecord { table, cells }],
    })
}

pub(crate) fn parse_delimited_rows(input: &str, delimiter: char) -> Result<Vec<Vec<String>>> {
    let mut rows = Vec::new();
    let mut row = Vec::new();
    let mut cell = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes = false;
    while let Some(ch) = chars.next() {
        match ch {
            '"' => {
                if in_quotes && chars.peek() == Some(&'"') {
                    cell.push('"');
                    chars.next();
                } else {
                    in_quotes = !in_quotes;
                }
            }
            ch if ch == delimiter && !in_quotes => {
                row.push(cell);
                cell = String::new();
            }
            '\n' if !in_quotes => {
                row.push(cell);
                cell = String::new();
                if !(row.len() == 1 && row[0].is_empty() && rows.is_empty()) {
                    rows.push(row);
                }
                row = Vec::new();
            }
            '\r' if !in_quotes => {
                if chars.peek() == Some(&'\n') {
                    chars.next();
                }
                row.push(cell);
                cell = String::new();
                if !(row.len() == 1 && row[0].is_empty() && rows.is_empty()) {
                    rows.push(row);
                }
                row = Vec::new();
            }
            _ => cell.push(ch),
        }
    }
    if in_quotes {
        bail!("research table has an unterminated quoted cell");
    }
    if !cell.is_empty() || !row.is_empty() {
        row.push(cell);
        rows.push(row);
    }
    Ok(rows)
}

pub(crate) fn normalize_table_cell_text(raw: &str) -> String {
    let cleaned = sanitize_work_text(raw, 4_000).unwrap_or_else(|_| "[INVALID]".to_string());
    let trimmed = cleaned.trim_start();
    let formula_like = trimmed.starts_with('=')
        || trimmed.starts_with('@')
        || trimmed.starts_with('+')
        || (trimmed.starts_with('-') && parse_table_numeric_value(trimmed).is_none());
    if formula_like {
        format!("'{cleaned}")
    } else {
        cleaned
    }
}

pub(crate) fn parse_table_numeric_value(raw: &str) -> Option<f64> {
    let cleaned = raw.trim().replace(',', "");
    if cleaned.is_empty() || cleaned.starts_with('=') || cleaned.starts_with('@') {
        return None;
    }
    cleaned.parse::<f64>().ok()
}

pub(crate) fn parse_table_numeric_and_footnote_refs(raw: &str) -> (Option<f64>, Vec<String>) {
    let cleaned = raw.trim().replace(',', "");
    if cleaned.is_empty() || cleaned.starts_with('=') || cleaned.starts_with('@') {
        return (None, Vec::new());
    }
    if let Ok(value) = cleaned.parse::<f64>() {
        return (Some(value), Vec::new());
    }
    let mut candidate = cleaned.as_str();
    let mut refs = Vec::new();
    loop {
        let trimmed = candidate.trim_end();
        if let Some(stripped) = trimmed.strip_suffix('*') {
            refs.push("*".to_string());
            candidate = stripped;
            continue;
        }
        if let Some(close) = trimmed.strip_suffix(']')
            && let Some(open_index) = close.rfind('[')
        {
            let reference = close[open_index + 1..].trim();
            if !reference.is_empty()
                && reference.len() <= 20
                && reference
                    .chars()
                    .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '-' | '_' | '.'))
            {
                refs.push(reference.to_string());
                candidate = &close[..open_index];
                continue;
            }
        }
        break;
    }
    refs.sort();
    refs.dedup();
    (candidate.trim().parse::<f64>().ok(), refs)
}

pub(crate) fn extract_xlsx_document(
    document_id: &str,
    path: &Path,
) -> Result<ResearchDocumentExtraction> {
    let mut workbook = match open_workbook_auto(path) {
        Ok(workbook) => workbook,
        Err(error) => {
            return Ok(blocked_document_extraction(
                "xlsx_extraction_failed",
                &error.to_string(),
                "calamine",
            ));
        }
    };
    let sheet_names = workbook.sheet_names().to_vec();
    let sheet_metadata = workbook.sheets_metadata().to_vec();
    let mut tables = Vec::new();
    let mut warning_flags = Vec::new();
    let mut total_cells = 0usize;
    for (sheet_index, sheet_name) in sheet_names.iter().enumerate() {
        if sheet_index >= 50 {
            warning_flags.push("xlsx_sheet_count_capped".to_string());
            break;
        }
        if let Some(metadata) = sheet_metadata.get(sheet_index) {
            if metadata.typ != SheetType::WorkSheet {
                warning_flags.push("xlsx_non_worksheet_sheets_skipped".to_string());
                continue;
            }
            match metadata.visible {
                SheetVisible::Visible => {}
                SheetVisible::Hidden => {
                    warning_flags.push("xlsx_hidden_sheets_skipped".to_string());
                    continue;
                }
                SheetVisible::VeryHidden => {
                    warning_flags.push("xlsx_very_hidden_sheets_skipped".to_string());
                    continue;
                }
            }
        }
        let range = match workbook.worksheet_range(sheet_name) {
            Ok(range) => range,
            Err(error) => {
                warning_flags.push(format!(
                    "xlsx_sheet_extraction_failed:{}",
                    sanitize_key_fragment(sheet_name)
                ));
                warning_flags.push(excerpt(&error.to_string(), 120));
                continue;
            }
        };
        let row_count = range.height();
        let column_count = range.width();
        if row_count == 0 || column_count == 0 {
            continue;
        }
        if row_count > RESEARCH_TABLE_MAX_ROWS {
            bail!("xlsx sheet has too many rows");
        }
        if column_count > RESEARCH_TABLE_MAX_COLUMNS {
            bail!("xlsx sheet has too many columns");
        }
        total_cells = total_cells.saturating_add(row_count.saturating_mul(column_count));
        if total_cells > RESEARCH_TABLE_MAX_CELLS {
            bail!("xlsx workbook has too many cells");
        }
        let formula_range = workbook.worksheet_formula(sheet_name).ok();
        let data_start = range.start().unwrap_or((0, 0));
        let xml_formulas = xlsx_formula_map(path, sheet_index).unwrap_or_default();
        let merged_ranges = xlsx_merge_ranges(path, sheet_index).unwrap_or_default();
        let table_id = format!("sheet-{}-table-1", sheet_index + 1);
        let table_db_id = research_table_db_id(document_id, &table_id);
        let rows = range.rows().collect::<Vec<_>>();
        let headers = rows
            .first()
            .map(|row| row.iter().map(xlsx_cell_cached_text).collect::<Vec<_>>())
            .unwrap_or_default();
        let mut cells = Vec::new();
        let mut formula_count = 0usize;
        let mut merged_cell_count = 0usize;
        let mut date_cell_count = 0usize;
        for row_index in 0..row_count {
            let row = rows.get(row_index).copied().unwrap_or(&[]);
            for column_index in 0..column_count {
                let cell = row.get(column_index).unwrap_or(&Data::Empty);
                let cached_text = xlsx_cell_cached_text(cell);
                let absolute_row = data_start.0 as usize + row_index;
                let absolute_column = data_start.1 as usize + column_index;
                let date_time_iso = xlsx_cell_datetime_iso(cell);
                if date_time_iso.is_some() {
                    date_cell_count += 1;
                }
                let merged_range = merged_ranges
                    .iter()
                    .find(|range| range.contains(absolute_row, absolute_column));
                if merged_range.is_some() {
                    merged_cell_count += 1;
                }
                let formula = xml_formulas
                    .get(&(absolute_row, absolute_column))
                    .map(String::as_str)
                    .or_else(|| {
                        formula_range.as_ref().and_then(|formulas| {
                            xlsx_formula_at_absolute(formulas, absolute_row, absolute_column)
                        })
                    })
                    .map(|value| {
                        if value.starts_with('=') {
                            value.to_string()
                        } else {
                            format!("={value}")
                        }
                    });
                let raw = formula.clone().unwrap_or_else(|| cached_text.clone());
                if formula.is_some() {
                    formula_count += 1;
                }
                let normalized_text = normalize_table_cell_text(&raw);
                let row_header = if row_index > 0 {
                    row.first()
                        .map(xlsx_cell_cached_text)
                        .map(|value| sanitize_work_text(&value, 500))
                        .transpose()?
                        .filter(|value| !value.trim().is_empty())
                } else {
                    None
                };
                let column_header = headers
                    .get(column_index)
                    .map(|value| sanitize_work_text(value, 500))
                    .transpose()?
                    .filter(|value| !value.trim().is_empty());
                let bbox_json = xlsx_cell_metadata_json(
                    sheet_name,
                    row_index,
                    column_index,
                    absolute_row,
                    absolute_column,
                    formula.as_deref(),
                    &cached_text,
                    date_time_iso.as_deref(),
                    merged_range,
                );
                let (numeric_value, footnote_refs) = if formula.is_some() {
                    parse_table_numeric_and_footnote_refs(&cached_text)
                } else if date_time_iso.is_some() {
                    (None, Vec::new())
                } else {
                    parse_table_numeric_and_footnote_refs(&raw)
                };
                let mut confidence: f64 = 0.95;
                if formula.is_some() {
                    confidence = confidence.min(0.86);
                }
                if merged_range.is_some() {
                    confidence = confidence.min(0.78);
                }
                if date_time_iso.is_some() {
                    confidence = confidence.min(0.90);
                }
                cells.push(ResearchTableCell {
                    id: research_table_cell_id(&table_db_id, row_index, column_index),
                    table_id: table_db_id.clone(),
                    row_index,
                    column_index,
                    row_header,
                    column_header,
                    raw_text: sanitize_work_text(&raw, 4_000)?,
                    normalized_text,
                    numeric_value,
                    unit: None,
                    footnote_refs,
                    bbox_json,
                    confidence,
                });
            }
        }
        let mut table_warnings = Vec::new();
        if formula_count > 0 {
            warning_flags.push("xlsx_formulas_preserved_as_untrusted_text".to_string());
            table_warnings.push("xlsx_formulas_preserved_as_untrusted_text".to_string());
        }
        if merged_cell_count > 0 {
            warning_flags.push("xlsx_merged_cells_present".to_string());
            table_warnings.push("xlsx_merged_cells_present".to_string());
        }
        if date_cell_count > 0 {
            warning_flags.push("xlsx_datetime_cells_normalized".to_string());
            table_warnings.push("xlsx_datetime_cells_normalized".to_string());
        }
        let mut table_confidence: f64 = 0.95;
        if formula_count > 0 {
            table_confidence = table_confidence.min(0.90);
        }
        if merged_cell_count > 0 {
            table_confidence = table_confidence.min(0.86);
        }
        if date_cell_count > 0 {
            table_confidence = table_confidence.min(0.92);
        }
        table_warnings.sort();
        table_warnings.dedup();
        tables.push(ResearchTableRecord {
            table: ResearchTable {
                id: table_db_id,
                document_id: document_id.to_string(),
                table_id,
                page_number: None,
                sheet_name: Some(sanitize_work_text(sheet_name, 500)?),
                caption: Some(format!(
                    "Worksheet `{}`",
                    sanitize_work_text(sheet_name, 200)?
                )),
                bbox_json: Some(json!({
                    "sheet_index": sheet_index,
                    "sheet_name": sheet_name,
                    "used_range": {
                        "rows": row_count,
                        "columns": column_count
                    },
                    "formula_cells": formula_count,
                    "merged_cells": merged_cell_count,
                    "datetime_cells": date_cell_count
                })),
                row_count,
                column_count,
                extraction_method: "calamine-xlsx".to_string(),
                confidence: table_confidence,
                warning_flags: table_warnings,
            },
            cells,
        });
    }
    warning_flags.sort();
    warning_flags.dedup();
    if tables.is_empty() {
        return Ok(blocked_document_extraction(
            "xlsx_no_extractable_sheets",
            "XLSX workbook contained no extractable non-empty worksheets",
            "calamine",
        ));
    }
    Ok(ResearchDocumentExtraction {
        extractor_name: "calamine".to_string(),
        extractor_version: "calamine-0.35".to_string(),
        status: "extracted".to_string(),
        page_count: 0,
        sheet_count: sheet_names.len(),
        warning_flags,
        error_message_redacted: None,
        spans: Vec::new(),
        tables,
    })
}

pub(crate) fn xlsx_formula_at_absolute(
    formulas: &Range<String>,
    absolute_row: usize,
    absolute_column: usize,
) -> Option<&str> {
    let (start_row, start_col) = formulas.start()?;
    let row = u32::try_from(absolute_row).ok()?;
    let col = u32::try_from(absolute_column).ok()?;
    if row < start_row || col < start_col {
        return None;
    }
    formulas
        .get(((row - start_row) as usize, (col - start_col) as usize))
        .map(|value| value.trim())
        .filter(|value| !value.is_empty())
}

pub(crate) fn xlsx_formula_map(
    path: &Path,
    sheet_index: usize,
) -> Result<BTreeMap<(usize, usize), String>> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let sheet_path = format!("xl/worksheets/sheet{}.xml", sheet_index + 1);
    let mut file = archive.by_name(&sheet_path)?;
    let mut xml = String::new();
    file.read_to_string(&mut xml)?;
    let doc = roxmltree::Document::parse(&xml)?;
    let mut formulas = BTreeMap::new();
    for cell in doc
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "c")
    {
        let Some(cell_ref) = cell.attribute("r") else {
            continue;
        };
        let Some(position) = xlsx_cell_ref_to_zero_based(cell_ref) else {
            continue;
        };
        let formula = cell
            .children()
            .find(|node| node.is_element() && node.tag_name().name() == "f")
            .and_then(|node| node.text())
            .map(str::trim)
            .filter(|value| !value.is_empty());
        if let Some(formula) = formula {
            formulas.insert(position, formula.to_string());
        }
    }
    Ok(formulas)
}

#[derive(Debug, Clone)]
pub(crate) struct XlsxMergedRange {
    pub(crate) start_row: usize,
    pub(crate) start_column: usize,
    pub(crate) end_row: usize,
    pub(crate) end_column: usize,
    pub(crate) reference: String,
}

impl XlsxMergedRange {
    fn contains(&self, row: usize, column: usize) -> bool {
        row >= self.start_row
            && row <= self.end_row
            && column >= self.start_column
            && column <= self.end_column
    }
}

pub(crate) fn xlsx_merge_ranges(path: &Path, sheet_index: usize) -> Result<Vec<XlsxMergedRange>> {
    let file = fs::File::open(path)?;
    let mut archive = zip::ZipArchive::new(file)?;
    let sheet_path = format!("xl/worksheets/sheet{}.xml", sheet_index + 1);
    let mut file = archive.by_name(&sheet_path)?;
    let mut xml = String::new();
    file.read_to_string(&mut xml)?;
    let doc = roxmltree::Document::parse(&xml)?;
    let mut ranges = Vec::new();
    for merge_cell in doc
        .descendants()
        .filter(|node| node.is_element() && node.tag_name().name() == "mergeCell")
    {
        let Some(reference) = merge_cell.attribute("ref") else {
            continue;
        };
        let Some((start, end)) = xlsx_cell_range_ref_to_zero_based(reference) else {
            continue;
        };
        ranges.push(XlsxMergedRange {
            start_row: start.0.min(end.0),
            start_column: start.1.min(end.1),
            end_row: start.0.max(end.0),
            end_column: start.1.max(end.1),
            reference: reference.to_string(),
        });
    }
    Ok(ranges)
}

pub(crate) fn xlsx_cell_range_ref_to_zero_based(
    range_ref: &str,
) -> Option<((usize, usize), (usize, usize))> {
    let mut parts = range_ref.split(':');
    let start = xlsx_cell_ref_to_zero_based(parts.next()?.trim())?;
    let end = match parts.next() {
        Some(value) => xlsx_cell_ref_to_zero_based(value.trim())?,
        None => start,
    };
    if parts.next().is_some() {
        return None;
    }
    Some((start, end))
}

pub(crate) fn xlsx_cell_ref_to_zero_based(cell_ref: &str) -> Option<(usize, usize)> {
    let mut column = 0usize;
    let mut row = 0usize;
    let mut saw_column = false;
    let mut saw_row = false;
    for ch in cell_ref.chars() {
        if ch.is_ascii_alphabetic() {
            if saw_row {
                return None;
            }
            saw_column = true;
            column = column
                .saturating_mul(26)
                .saturating_add((ch.to_ascii_uppercase() as u8 - b'A' + 1) as usize);
        } else if ch.is_ascii_digit() {
            saw_row = true;
            row = row
                .saturating_mul(10)
                .saturating_add((ch as u8 - b'0') as usize);
        } else {
            return None;
        }
    }
    if !saw_column || !saw_row || column == 0 || row == 0 {
        return None;
    }
    Some((row - 1, column - 1))
}

// allow: refactoring this N-arg signature is out of scope for the lint-cleanup pass.
#[allow(clippy::too_many_arguments)]
pub(crate) fn xlsx_cell_metadata_json(
    sheet_name: &str,
    row_index: usize,
    column_index: usize,
    absolute_row: usize,
    absolute_column: usize,
    formula: Option<&str>,
    cached_text: &str,
    date_time_iso: Option<&str>,
    merged_range: Option<&XlsxMergedRange>,
) -> Option<Value> {
    if formula.is_none() && date_time_iso.is_none() && merged_range.is_none() {
        return None;
    }
    let mut metadata = Map::new();
    metadata.insert("sheet_name".to_string(), json!(sheet_name));
    metadata.insert("row_index".to_string(), json!(row_index));
    metadata.insert("column_index".to_string(), json!(column_index));
    metadata.insert("absolute_row_index".to_string(), json!(absolute_row));
    metadata.insert("absolute_column_index".to_string(), json!(absolute_column));
    if let Some(formula) = formula {
        metadata.insert("formula".to_string(), json!(formula));
        metadata.insert("cached_value".to_string(), json!(cached_text));
        metadata.insert("formula_evaluation".to_string(), json!("not_performed"));
    }
    if let Some(date_time_iso) = date_time_iso {
        metadata.insert("value_kind".to_string(), json!("date_time"));
        metadata.insert("date_time_iso".to_string(), json!(date_time_iso));
    }
    if let Some(range) = merged_range {
        metadata.insert(
            "merged_range".to_string(),
            json!({
                "ref": range.reference,
                "start_row_index": range.start_row,
                "start_column_index": range.start_column,
                "end_row_index": range.end_row,
                "end_column_index": range.end_column
            }),
        );
    }
    Some(Value::Object(metadata))
}

pub(crate) fn xlsx_cell_cached_text(cell: &Data) -> String {
    if let Some(date_time_iso) = xlsx_cell_datetime_iso(cell) {
        return date_time_iso;
    }
    match cell {
        Data::Empty => String::new(),
        Data::String(value) => value.clone(),
        Data::Float(value) => value.to_string(),
        Data::Int(value) => value.to_string(),
        Data::Bool(value) => value.to_string(),
        Data::DateTime(value) => value.to_string(),
        Data::DateTimeIso(value) => value.clone(),
        Data::DurationIso(value) => value.clone(),
        Data::Error(value) => format!("{value:?}"),
    }
}

pub(crate) fn xlsx_cell_datetime_iso(cell: &Data) -> Option<String> {
    match cell {
        Data::DateTime(value) => {
            let (year, month, day, hour, minute, second, millis) = value.to_ymd_hms_milli();
            if millis > 0 {
                Some(format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}.{millis:03}"
                ))
            } else {
                Some(format!(
                    "{year:04}-{month:02}-{day:02}T{hour:02}:{minute:02}:{second:02}"
                ))
            }
        }
        Data::DateTimeIso(value) => Some(value.clone()),
        _ => None,
    }
}

pub(crate) fn sanitize_key_fragment(input: &str) -> String {
    input
        .chars()
        .filter(|ch| ch.is_ascii_alphanumeric() || *ch == '-' || *ch == '_')
        .take(40)
        .collect::<String>()
}

pub(crate) fn extract_pdf_document(
    document_id: &str,
    path: &Path,
) -> Result<ResearchDocumentExtraction> {
    let output = Command::new("pdftotext")
        .arg("-layout")
        .arg("-enc")
        .arg("UTF-8")
        .arg(path)
        .arg("-")
        .output();
    let output = match output {
        Ok(output) => output,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => {
            return Ok(blocked_document_extraction(
                "pdf_text_extractor_unavailable",
                "pdftotext is not available in this environment",
                "pdftotext",
            ));
        }
        Err(error) => {
            return Ok(blocked_document_extraction(
                "pdf_text_extraction_failed",
                &error.to_string(),
                "pdftotext",
            ));
        }
    };
    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();
        let lower = stderr.to_ascii_lowercase();
        let warning = if lower.contains("password") || lower.contains("encrypt") {
            "encrypted_pdf"
        } else {
            "pdf_text_extraction_failed"
        };
        return Ok(blocked_document_extraction(warning, &stderr, "pdftotext"));
    }
    let text = String::from_utf8_lossy(&output.stdout).to_string();
    let pages: Vec<String> = text
        .split('\u{0c}')
        .map(str::trim)
        .filter(|page| !page.is_empty())
        .map(ToOwned::to_owned)
        .collect();
    if pages.is_empty() {
        return Ok(blocked_document_extraction(
            "scanned_or_empty_pdf",
            "PDF text extraction produced no text; OCR is required before this can be evidence",
            "pdftotext",
        ));
    }
    let mut spans = Vec::new();
    let mut tables = Vec::new();
    let mut offset = 0usize;
    for (index, page) in pages.iter().enumerate() {
        let page_number = index + 1;
        let span_id = format!("page-{page_number}");
        let text_excerpt = sanitize_work_text(page, 8_000)?;
        let char_start = offset;
        let char_end = offset + page.len();
        offset = char_end + 1;
        spans.push(ResearchDocumentSpan {
            id: research_document_span_db_id(document_id, &span_id),
            document_id: document_id.to_string(),
            span_id,
            page_number: Some(page_number),
            section_label: Some(format!("page {page_number}")),
            char_start,
            char_end,
            text_sha256: sha256(page.as_bytes()),
            text_excerpt,
            bbox_json: None,
            confidence: 0.9,
            warning_flags: Vec::new(),
        });
        tables.extend(extract_pdf_layout_tables(document_id, page_number, page)?);
    }
    let mut warning_flags = if tables.is_empty() {
        vec!["pdf_tables_not_precise".to_string()]
    } else {
        vec!["pdf_layout_table_heuristic".to_string()]
    };
    warning_flags.sort();
    warning_flags.dedup();
    Ok(ResearchDocumentExtraction {
        extractor_name: "pdftotext".to_string(),
        extractor_version: "external".to_string(),
        status: if tables.is_empty() {
            "extracted_text".to_string()
        } else {
            "extracted_text_and_tables".to_string()
        },
        page_count: pages.len(),
        sheet_count: 0,
        warning_flags,
        error_message_redacted: None,
        spans,
        tables,
    })
}

#[derive(Debug, Clone)]
pub(crate) struct PdfLayoutCell {
    pub(crate) text: String,
    pub(crate) char_start: usize,
    pub(crate) char_end: usize,
    pub(crate) line_number: usize,
}

pub(crate) fn extract_pdf_layout_tables(
    document_id: &str,
    page_number: usize,
    page_text: &str,
) -> Result<Vec<ResearchTableRecord>> {
    let mut groups: Vec<Vec<Vec<PdfLayoutCell>>> = Vec::new();
    let mut current: Vec<Vec<PdfLayoutCell>> = Vec::new();
    for (line_index, line) in page_text.lines().enumerate() {
        let cells = parse_pdf_layout_table_line(line, line_index + 1);
        if cells.len() >= 2 {
            current.push(cells);
        } else if !current.is_empty() {
            if current.len() >= 2 {
                groups.push(std::mem::take(&mut current));
            } else {
                current.clear();
            }
        }
    }
    if current.len() >= 2 {
        groups.push(current);
    }
    let mut records = Vec::new();
    for (group_index, group) in groups.into_iter().enumerate() {
        let column_count = group.iter().map(Vec::len).max().unwrap_or(0);
        if column_count < 2 {
            continue;
        }
        if group.len() > RESEARCH_TABLE_MAX_ROWS || column_count > RESEARCH_TABLE_MAX_COLUMNS {
            bail!("pdf heuristic table exceeds table caps");
        }
        if group.len().saturating_mul(column_count) > RESEARCH_TABLE_MAX_CELLS {
            bail!("pdf heuristic table exceeds cell cap");
        }
        let has_irregular_columns = group.iter().any(|row| row.len() != column_count);
        let has_possible_wrapped_header = group.first().is_some_and(|row| row.len() < column_count)
            || group
                .iter()
                .take(2)
                .any(|row| row.first().is_some_and(|cell| cell.char_start > 4));
        let mut has_footnote_markers = false;
        let table_id = format!("page-{page_number}-table-{}", group_index + 1);
        let table_db_id = research_table_db_id(document_id, &table_id);
        let headers = group
            .first()
            .map(|row| row.iter().map(|cell| cell.text.clone()).collect::<Vec<_>>())
            .unwrap_or_default();
        let mut cells = Vec::new();
        for (row_index, row) in group.iter().enumerate() {
            for column_index in 0..column_count {
                let parsed = row.get(column_index);
                let raw = parsed.map(|cell| cell.text.as_str()).unwrap_or("");
                let normalized_text = normalize_table_cell_text(raw);
                let row_header = if row_index > 0 {
                    row.first()
                        .map(|cell| sanitize_work_text(&cell.text, 500))
                        .transpose()?
                        .filter(|value| !value.trim().is_empty())
                } else {
                    None
                };
                let column_header = headers
                    .get(column_index)
                    .map(|value| sanitize_work_text(value, 500))
                    .transpose()?
                    .filter(|value| !value.trim().is_empty());
                let (numeric_value, footnote_refs) = parse_table_numeric_and_footnote_refs(raw);
                if !footnote_refs.is_empty() {
                    has_footnote_markers = true;
                }
                let mut confidence: f64 = 0.76;
                if row.len() != column_count || parsed.is_none() {
                    confidence = confidence.min(0.70);
                }
                if has_possible_wrapped_header && row_index <= 1 {
                    confidence = confidence.min(0.68);
                }
                if !footnote_refs.is_empty() {
                    confidence = confidence.min(0.82);
                }
                let bbox_json = parsed.map(|cell| {
                    json!({
                        "page_number": page_number,
                        "line_number": cell.line_number,
                        "char_start": cell.char_start,
                        "char_end": cell.char_end,
                        "row_column_count": row.len(),
                        "expected_column_count": column_count,
                        "footnote_refs": footnote_refs,
                        "unit": "character_offset"
                    })
                });
                cells.push(ResearchTableCell {
                    id: research_table_cell_id(&table_db_id, row_index, column_index),
                    table_id: table_db_id.clone(),
                    row_index,
                    column_index,
                    row_header,
                    column_header,
                    raw_text: sanitize_work_text(raw, 4_000)?,
                    normalized_text,
                    numeric_value,
                    unit: None,
                    footnote_refs,
                    bbox_json,
                    confidence,
                });
            }
        }
        let mut warning_flags = vec!["pdf_layout_table_heuristic".to_string()];
        let mut table_confidence: f64 = 0.76;
        if has_irregular_columns {
            warning_flags.push("pdf_table_irregular_columns".to_string());
            table_confidence = table_confidence.min(0.70);
        }
        if has_possible_wrapped_header {
            warning_flags.push("pdf_table_possible_wrapped_header".to_string());
            table_confidence = table_confidence.min(0.68);
        }
        if has_footnote_markers {
            warning_flags.push("pdf_table_footnote_markers".to_string());
            table_confidence = table_confidence.min(0.74);
        }
        warning_flags.sort();
        warning_flags.dedup();
        records.push(ResearchTableRecord {
            table: ResearchTable {
                id: table_db_id,
                document_id: document_id.to_string(),
                table_id,
                page_number: Some(page_number),
                sheet_name: None,
                caption: Some(format!("PDF layout table on page {page_number}")),
                bbox_json: Some(json!({
                    "page_number": page_number,
                    "line_start": group
                        .first()
                        .and_then(|row| row.first())
                        .map(|cell| cell.line_number),
                    "line_end": group
                        .last()
                        .and_then(|row| row.first())
                        .map(|cell| cell.line_number),
                    "unit": "character_offset"
                })),
                row_count: group.len(),
                column_count,
                extraction_method: "pdftotext-layout-heuristic".to_string(),
                confidence: table_confidence,
                warning_flags,
            },
            cells,
        });
    }
    Ok(records)
}

pub(crate) fn parse_pdf_layout_table_line(line: &str, line_number: usize) -> Vec<PdfLayoutCell> {
    let mut cells = Vec::new();
    let mut start: Option<usize> = None;
    let mut whitespace_run = 0usize;
    for (index, ch) in line.char_indices() {
        if ch.is_whitespace() {
            whitespace_run += 1;
            if whitespace_run >= 2
                && let Some(cell_start) = start.take()
            {
                let cell_end = index + ch.len_utf8() - whitespace_run;
                let text = line[cell_start..=cell_end].trim().to_string();
                if !text.is_empty() {
                    cells.push(PdfLayoutCell {
                        text,
                        char_start: cell_start,
                        char_end: cell_end + 1,
                        line_number,
                    });
                }
            }
        } else {
            if start.is_none() {
                start = Some(index);
            }
            whitespace_run = 0;
        }
    }
    if let Some(cell_start) = start {
        let text = line[cell_start..].trim().to_string();
        if !text.is_empty() {
            cells.push(PdfLayoutCell {
                text,
                char_start: cell_start,
                char_end: line.len(),
                line_number,
            });
        }
    }
    cells
}
