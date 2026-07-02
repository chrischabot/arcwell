use super::*;

#[test]
fn research_document_extracts_csv_table_with_cell_anchors() {
    let store = test_store("research-document-csv");
    let workflow = store
        .create_deep_research_run("startup funding tables")
        .unwrap();
    let path = store.paths().home.join("funding.csv");
    fs::write(
            &path,
            "Company,Funding,Note\nAlpha,123.5,\"=HYPERLINK(\"\"https://evil.example\"\")\"\nBeta,-42,\"wrapped\nnote\"\n",
        )
        .unwrap();

    let record = store
        .extract_research_document_file(ResearchDocumentInput {
            run_id: workflow.run.id.clone(),
            research_source_id: None,
            source_card_id: None,
            path,
            media_type: None,
        })
        .unwrap();
    assert_eq!(record.document.extraction_status, "extracted");
    assert_eq!(record.document.media_type, "text/csv");
    assert_eq!(record.tables.len(), 1);
    let table = &record.tables[0];
    assert_eq!(table.table.row_count, 3);
    assert_eq!(table.table.column_count, 3);
    assert_eq!(table.cells.len(), 9);
    let formula_cell = table
        .cells
        .iter()
        .find(|cell| cell.row_index == 1 && cell.column_index == 2)
        .unwrap();
    assert!(formula_cell.raw_text.starts_with("=HYPERLINK"));
    assert!(formula_cell.normalized_text.starts_with("'="));
    assert_eq!(formula_cell.column_header.as_deref(), Some("Note"));
    let negative_number = table
        .cells
        .iter()
        .find(|cell| cell.row_index == 2 && cell.column_index == 1)
        .unwrap();
    assert_eq!(negative_number.numeric_value, Some(-42.0));
    assert_eq!(negative_number.normalized_text, "-42");

    let read = store.read_research_run(&workflow.run.id).unwrap();
    assert_eq!(read.documents.len(), 1);
    assert_eq!(read.documents[0].tables[0].cells.len(), 9);
}

#[test]
fn research_document_extracts_xlsx_tables_with_formulas_as_untrusted_text() {
    let store = test_store("research-document-xlsx");
    let workflow = store
        .create_deep_research_run("startup funding workbook")
        .unwrap();
    let path = store.paths().home.join("funding.xlsx");
    let fixture = base64::engine::general_purpose::STANDARD
            .decode("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")
            .unwrap();
    fs::write(&path, fixture).unwrap();

    let record = store
        .extract_research_document_file(ResearchDocumentInput {
            run_id: workflow.run.id.clone(),
            research_source_id: None,
            source_card_id: None,
            path,
            media_type: None,
        })
        .unwrap();

    assert_eq!(record.document.extraction_status, "extracted");
    assert_eq!(record.tables.len(), 1);
    let table = &record.tables[0];
    assert_eq!(table.table.sheet_name.as_deref(), Some("Data"));
    assert_eq!(table.table.row_count, 3);
    assert_eq!(table.table.column_count, 3);
    assert_eq!(table.cells.len(), 9);
    let formula_cell = table
        .cells
        .iter()
        .find(|cell| cell.row_index == 2 && cell.column_index == 2)
        .unwrap();
    assert!(formula_cell.raw_text.starts_with("=SUM"));
    assert!(formula_cell.normalized_text.starts_with("'="));
    assert!(formula_cell.bbox_json.is_some());
}

#[test]
fn severe_research_document_xlsx_marks_hidden_merged_and_date_cells() {
    // CLAIM: XLSX extraction does not silently turn hidden sheets, merged
    // ranges, or date-formatted cells into clean analyst evidence.
    // ORACLE: hidden/very-hidden sheets are skipped with document warnings,
    // merged cells carry merge metadata and downgraded confidence, and date
    // cells are normalized as date/time values rather than numeric measures.
    // SEVERITY: Severe because hidden sheets and merged/date formatting are
    // common in public spreadsheets and can otherwise poison table-backed
    // claims with invisible or mis-anchored evidence.
    let store = test_store("research-document-xlsx-hard");
    let workflow = store
        .create_deep_research_run("hard spreadsheet extraction")
        .unwrap();
    let path = store.paths().home.join("hard.xlsx");
    write_hard_xlsx_fixture(&path).unwrap();

    let record = store
        .extract_research_document_file(ResearchDocumentInput {
            run_id: workflow.run.id.clone(),
            research_source_id: None,
            source_card_id: None,
            path,
            media_type: None,
        })
        .unwrap();

    assert_eq!(record.document.extraction_status, "extracted");
    assert_eq!(record.document.sheet_count, 3);
    assert_eq!(record.tables.len(), 1);
    assert!(
        record
            .document
            .warning_flags
            .contains(&"xlsx_hidden_sheets_skipped".to_string())
    );
    assert!(
        record
            .document
            .warning_flags
            .contains(&"xlsx_very_hidden_sheets_skipped".to_string())
    );
    assert!(
        record
            .document
            .warning_flags
            .contains(&"xlsx_merged_cells_present".to_string())
    );
    assert!(
        record
            .document
            .warning_flags
            .contains(&"xlsx_datetime_cells_normalized".to_string())
    );

    let table = &record.tables[0];
    assert_eq!(table.table.sheet_name.as_deref(), Some("Visible"));
    assert!(table.table.confidence < 0.9);
    assert!(
        table
            .table
            .warning_flags
            .contains(&"xlsx_merged_cells_present".to_string())
    );
    assert!(
        table
            .table
            .warning_flags
            .contains(&"xlsx_datetime_cells_normalized".to_string())
    );
    assert!(
        table
            .cells
            .iter()
            .all(|cell| !cell.raw_text.contains("sk-hidden"))
    );

    let merged_header = table
        .cells
        .iter()
        .find(|cell| cell.row_index == 0 && cell.column_index == 0)
        .unwrap();
    assert_eq!(merged_header.raw_text, "Merged Header");
    assert!(merged_header.confidence < 0.8);
    assert_eq!(
        merged_header
            .bbox_json
            .as_ref()
            .and_then(|value| value.pointer("/merged_range/ref"))
            .and_then(Value::as_str),
        Some("A1:B1")
    );

    let date_cell = table
        .cells
        .iter()
        .find(|cell| cell.row_index == 1 && cell.column_index == 2)
        .unwrap();
    assert!(date_cell.raw_text.starts_with("2024-01-15T"));
    assert!(date_cell.normalized_text.starts_with("2024-01-15T"));
    assert_eq!(date_cell.numeric_value, None);
    assert_eq!(
        date_cell
            .bbox_json
            .as_ref()
            .and_then(|value| value.get("value_kind"))
            .and_then(Value::as_str),
        Some("date_time")
    );
}

fn write_hard_xlsx_fixture(path: &Path) -> Result<()> {
    let file = fs::File::create(path)?;
    let mut zip = zip::ZipWriter::new(file);
    let options =
        zip::write::SimpleFileOptions::default().compression_method(zip::CompressionMethod::Stored);
    for (name, body) in [
        (
            "[Content_Types].xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
  <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
  <Default Extension="xml" ContentType="application/xml"/>
  <Override PartName="/xl/workbook.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.sheet.main+xml"/>
  <Override PartName="/xl/worksheets/sheet1.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/worksheets/sheet2.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/worksheets/sheet3.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.worksheet+xml"/>
  <Override PartName="/xl/styles.xml" ContentType="application/vnd.openxmlformats-officedocument.spreadsheetml.styles+xml"/>
</Types>"#,
        ),
        (
            "_rels/.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="xl/workbook.xml"/>
</Relationships>"#,
        ),
        (
            "xl/workbook.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<workbook xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
  <sheets>
    <sheet name="Visible" sheetId="1" r:id="rId1"/>
    <sheet name="Hidden Secrets" sheetId="2" state="hidden" r:id="rId2"/>
    <sheet name="Very Hidden Secrets" sheetId="3" state="veryHidden" r:id="rId3"/>
  </sheets>
</workbook>"#,
        ),
        (
            "xl/_rels/workbook.xml.rels",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
  <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet1.xml"/>
  <Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet2.xml"/>
  <Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/worksheet" Target="worksheets/sheet3.xml"/>
  <Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/styles" Target="styles.xml"/>
</Relationships>"#,
        ),
        (
            "xl/styles.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<styleSheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <numFmts count="0"/>
  <fonts count="1"><font><sz val="11"/><name val="Calibri"/></font></fonts>
  <fills count="1"><fill><patternFill patternType="none"/></fill></fills>
  <borders count="1"><border/></borders>
  <cellStyleXfs count="1"><xf numFmtId="0" fontId="0" fillId="0" borderId="0"/></cellStyleXfs>
  <cellXfs count="2">
    <xf numFmtId="0" fontId="0" fillId="0" borderId="0" xfId="0"/>
    <xf numFmtId="14" fontId="0" fillId="0" borderId="0" xfId="0" applyNumberFormat="1"/>
  </cellXfs>
  <cellStyles count="1"><cellStyle name="Normal" xfId="0" builtinId="0"/></cellStyles>
</styleSheet>"#,
        ),
        (
            "xl/worksheets/sheet1.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <dimension ref="A1:C2"/>
  <sheetData>
    <row r="1">
      <c r="A1" t="inlineStr"><is><t>Merged Header</t></is></c>
      <c r="C1" t="inlineStr"><is><t>Reported At</t></is></c>
    </row>
    <row r="2">
      <c r="A2" t="inlineStr"><is><t>Alpha</t></is></c>
      <c r="B2"><v>123.5</v></c>
      <c r="C2" s="1"><v>45306</v></c>
    </row>
  </sheetData>
  <mergeCells count="1"><mergeCell ref="A1:B1"/></mergeCells>
</worksheet>"#,
        ),
        (
            "xl/worksheets/sheet2.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>sk-hidden-sheet-token</t></is></c></row></sheetData>
</worksheet>"#,
        ),
        (
            "xl/worksheets/sheet3.xml",
            r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<worksheet xmlns="http://schemas.openxmlformats.org/spreadsheetml/2006/main">
  <sheetData><row r="1"><c r="A1" t="inlineStr"><is><t>very hidden decision notes</t></is></c></row></sheetData>
</worksheet>"#,
        ),
    ] {
        zip.start_file(name, options)?;
        zip.write_all(body.as_bytes())?;
    }
    zip.finish()?;
    Ok(())
}

#[test]
fn research_claim_document_anchors_round_trip_into_report_and_audit() {
    let store = test_store("research-claim-document-anchor");
    let workflow = store
        .create_deep_research_run("startup funding tables")
        .unwrap();
    let card = store
        .add_source_card(SourceCardInput {
            title: "Funding table".to_string(),
            url: "https://example.com/funding-table".to_string(),
            source_type: "table".to_string(),
            provider: "test".to_string(),
            summary: "The table reports Alpha funding of 123.5.".to_string(),
            claims: vec![SourceClaim {
                claim: "Alpha funding is 123.5.".to_string(),
                kind: "measurement".to_string(),
                confidence: 0.9,
            }],
            retrieved_at: None,
            metadata: json!({ "source_role": "primary", "trust_level": "high" }),
        })
        .unwrap();
    store
        .link_source_card_to_research_run(
            &workflow.run.id,
            &card.id,
            "tables",
            "full-text",
            "must-read-primary",
            None,
        )
        .unwrap();
    let path = store.paths().home.join("funding.csv");
    fs::write(&path, "Company,Funding\nAlpha,123.5\n").unwrap();
    let document = store
        .extract_research_document_file(ResearchDocumentInput {
            run_id: workflow.run.id.clone(),
            research_source_id: None,
            source_card_id: Some(card.id.clone()),
            path,
            media_type: None,
        })
        .unwrap();

    let records = store
        .ingest_research_claims_from_model_output(
            &workflow.run.id,
            &card.id,
            "test",
            "model",
            &format!(
                r#"{{
                        "claims": [{{
                            "text": "Alpha funding is 123.5.",
                            "kind": "measurement",
                            "subject": "Alpha",
                            "predicate": "funding",
                            "object": "123.5",
                            "confidence": 0.9,
                            "caveats": [],
                            "quote": "Alpha,123.5",
                            "source_anchor": "table row 2",
                            "document_anchors": [{{
                                "document_id": "{}",
                                "table_id": "table-1",
                                "row_index": 1,
                                "column_index": 1,
                                "quote": "123.5"
                            }}]
                        }}]
                    }}"#,
                document.document.id
            ),
        )
        .unwrap();
    assert_eq!(records.len(), 1);
    assert_eq!(records[0].document_anchors.len(), 1);
    assert_eq!(records[0].document_anchors[0].anchor_kind, "cell");
    assert!(
        records[0].document_anchors[0]
            .anchor_label
            .contains("[r1,c1]")
    );
    let mut input = research_convergence_test_input(&workflow.run.id);
    input.max_iterations = Some(3);
    input.no_progress_iteration_limit = Some(1);
    let step = store.run_research_convergence_to_stop(input).unwrap();
    assert!(step.status.settled);
    let convergence_report = store
        .compile_research_convergence_report(&workflow.run.id)
        .unwrap();
    assert_ne!(convergence_report.judgment.overall_decision, "reject");
    assert!(
        !convergence_report
            .judgment
            .blocking_findings
            .to_string()
            .contains("measurement_claims_without_document_anchor"),
        "cell-anchored measurements should satisfy the publication-grade anchor gate"
    );

    let report = store
        .compile_research_report(
            &workflow.run.id,
            "Test corpus saturated for document-anchor proof.",
            false,
        )
        .unwrap();
    assert!(report.markdown.contains("Document anchors:"));
    assert!(report.markdown.contains("\\[r1,c1\\]"));
    assert!(report.markdown.contains("### Document Artifacts"));

    let audit = store.audit_research_run(&workflow.run.id).unwrap();
    assert!(
        audit
            .audit
            .findings
            .iter()
            .all(|finding| finding.code != "measurement_claim_without_document_anchor")
    );
}

#[test]
fn pdf_layout_table_heuristic_extracts_cell_anchors() {
    let tables = extract_pdf_layout_tables(
            "rdoc-test",
            2,
            "Company      Funding      Notes\nAlpha        123.5        audited\nBeta         -42          restated\n",
        )
        .unwrap();
    assert_eq!(tables.len(), 1);
    assert_eq!(tables[0].table.table_id, "page-2-table-1");
    assert_eq!(tables[0].table.page_number, Some(2));
    assert_eq!(
        tables[0].table.warning_flags,
        vec!["pdf_layout_table_heuristic"]
    );
    let cell = tables[0]
        .cells
        .iter()
        .find(|cell| cell.row_index == 1 && cell.column_index == 1)
        .unwrap();
    assert_eq!(cell.raw_text, "123.5");
    assert_eq!(cell.numeric_value, Some(123.5));
    assert_eq!(cell.column_header.as_deref(), Some("Funding"));
    assert!(cell.bbox_json.is_some());
}

#[test]
fn severe_pdf_layout_table_marks_wrapped_headers_and_footnotes_as_low_confidence() {
    // CLAIM: PDF layout tables expose precise cell anchors only with explicit
    // caveats when the layout has wrapped headers, irregular rows, or
    // footnoted numeric cells.
    // ORACLE: the extracted table and affected cells carry low confidence,
    // warning flags, parsed numeric values, and footnote refs instead of
    // looking like clean CSV/XLSX evidence.
    // SEVERITY: Severe because these are common government/benchmark PDF
    // table shapes that can make analyst-grade reports cite the wrong cell.
    let tables = extract_pdf_layout_tables(
            "rdoc-test",
            4,
            "Company      2025 revenue\n             USD m      Notes\nAlpha        123.5*     audited\nBeta         98.0 [a]   restated\n",
        )
        .unwrap();

    assert_eq!(tables.len(), 1);
    let table = &tables[0];
    assert!(table.table.confidence < 0.75);
    assert!(
        table
            .table
            .warning_flags
            .contains(&"pdf_table_irregular_columns".to_string())
    );
    assert!(
        table
            .table
            .warning_flags
            .contains(&"pdf_table_possible_wrapped_header".to_string())
    );
    assert!(
        table
            .table
            .warning_flags
            .contains(&"pdf_table_footnote_markers".to_string())
    );

    let alpha_revenue = table
        .cells
        .iter()
        .find(|cell| cell.row_index == 2 && cell.column_index == 1)
        .unwrap();
    assert_eq!(alpha_revenue.raw_text, "123.5*");
    assert_eq!(alpha_revenue.numeric_value, Some(123.5));
    assert_eq!(alpha_revenue.footnote_refs, vec!["*"]);
    assert!(alpha_revenue.confidence < 0.85);
    assert!(
        alpha_revenue
            .bbox_json
            .as_ref()
            .unwrap()
            .get("footnote_refs")
            .unwrap()
            .as_array()
            .unwrap()
            .iter()
            .any(|value| value.as_str() == Some("*"))
    );

    let beta_revenue = table
        .cells
        .iter()
        .find(|cell| cell.row_index == 3 && cell.column_index == 1)
        .unwrap();
    assert_eq!(beta_revenue.raw_text, "98.0 [a]");
    assert_eq!(beta_revenue.numeric_value, Some(98.0));
    assert_eq!(beta_revenue.footnote_refs, vec!["a"]);
    assert!(beta_revenue.confidence < 0.85);
}

#[test]
fn severe_research_claim_document_anchors_reject_cross_run_and_missing_cells_atomically() {
    // CLAIM: document anchors are resolved against real same-run document artifacts
    // before a claim is durably accepted.
    // ORACLE: cross-run and missing-cell anchors error, and the target run keeps zero claims.
    // SEVERITY: Severe because forged anchors would make polished reports look grounded.
    let store = test_store("research-claim-document-anchor-severe");
    let left = store.create_deep_research_run("left run").unwrap();
    let right = store.create_deep_research_run("right run").unwrap();
    let card = store
        .add_source_card(SourceCardInput {
            title: "Left funding source".to_string(),
            url: "https://example.com/left-funding".to_string(),
            source_type: "table".to_string(),
            provider: "test".to_string(),
            summary: "Alpha funding is 123.5.".to_string(),
            claims: vec![SourceClaim {
                claim: "Alpha funding is 123.5.".to_string(),
                kind: "measurement".to_string(),
                confidence: 0.9,
            }],
            retrieved_at: None,
            metadata: json!({ "source_role": "primary", "trust_level": "high" }),
        })
        .unwrap();
    store
        .link_source_card_to_research_run(
            &left.run.id,
            &card.id,
            "tables",
            "full-text",
            "must-read-primary",
            None,
        )
        .unwrap();
    let right_path = store.paths().home.join("right.csv");
    fs::write(&right_path, "Company,Funding\nAlpha,123.5\n").unwrap();
    let right_document = store
        .extract_research_document_file(ResearchDocumentInput {
            run_id: right.run.id.clone(),
            research_source_id: None,
            source_card_id: None,
            path: right_path,
            media_type: None,
        })
        .unwrap();

    let cross_run = format!(
        r#"{{
                "claims": [{{
                    "text": "Alpha funding is 123.5.",
                    "kind": "measurement",
                    "confidence": 0.9,
                    "document_anchors": [{{
                        "document_id": "{}",
                        "table_id": "table-1",
                        "row_index": 1,
                        "column_index": 1
                    }}]
                }}]
            }}"#,
        right_document.document.id
    );
    assert!(
        store
            .ingest_research_claims_from_model_output(
                &left.run.id,
                &card.id,
                "test",
                "model",
                &cross_run,
            )
            .is_err()
    );
    assert!(store.list_research_claims(&left.run.id).unwrap().is_empty());

    let left_path = store.paths().home.join("left.csv");
    fs::write(&left_path, "Company,Funding\nAlpha,123.5\n").unwrap();
    let left_document = store
        .extract_research_document_file(ResearchDocumentInput {
            run_id: left.run.id.clone(),
            research_source_id: None,
            source_card_id: Some(card.id.clone()),
            path: left_path,
            media_type: None,
        })
        .unwrap();
    let missing_cell = format!(
        r#"{{
                "claims": [{{
                    "text": "Alpha funding is 123.5.",
                    "kind": "measurement",
                    "confidence": 0.9,
                    "document_anchors": [{{
                        "document_id": "{}",
                        "table_id": "table-1",
                        "row_index": 10,
                        "column_index": 1
                    }}]
                }}]
            }}"#,
        left_document.document.id
    );
    assert!(
        store
            .ingest_research_claims_from_model_output(
                &left.run.id,
                &card.id,
                "test",
                "model",
                &missing_cell,
            )
            .is_err()
    );
    assert!(store.list_research_claims(&left.run.id).unwrap().is_empty());
}

#[test]
fn severe_research_report_judgment_blocks_measurements_without_document_anchors() {
    // CLAIM: convergence report acceptance is publication-grade enough to
    // reject numeric/measurement claims that lack document/table/span/cell anchors.
    // ORACLE: a settled convergence run over a bare measurement compiles a
    // rejected judgment with an explicit citation-quality blocker.
    // SEVERITY: Severe because numeric prose without precise anchors is a
    // high-confidence-looking mirage in analyst reports.
    let store = test_store("research-report-judgment-measurement-anchor");
    let workflow = store
        .create_deep_research_run("unanchored benchmark measurement")
        .unwrap();
    let card = store
        .add_source_card(SourceCardInput {
            title: "Benchmark summary".to_string(),
            url: "https://example.com/benchmark-summary".to_string(),
            source_type: "paper".to_string(),
            provider: "test".to_string(),
            summary: "The summary says Codec Z improves compression by 12.4 percent.".to_string(),
            claims: vec![SourceClaim {
                claim: "Codec Z improves compression by 12.4 percent.".to_string(),
                kind: "measurement".to_string(),
                confidence: 0.88,
            }],
            retrieved_at: None,
            metadata: json!({ "source_role": "primary", "trust_level": "high" }),
        })
        .unwrap();
    store
        .link_source_card_to_research_run(
            &workflow.run.id,
            &card.id,
            "papers",
            "full-text",
            "must-read-primary",
            None,
        )
        .unwrap();
    store
        .ingest_research_claims_from_model_output(
            &workflow.run.id,
            &card.id,
            "test",
            "model",
            r#"{
                    "claims": [{
                        "text": "Codec Z improves compression by 12.4 percent.",
                        "kind": "measurement",
                        "subject": "Codec Z",
                        "predicate": "improves compression by",
                        "object": "12.4 percent",
                        "confidence": 0.88,
                        "caveats": ["Fixture summary only."],
                        "quote": "12.4 percent"
                    }]
                }"#,
        )
        .unwrap();
    let mut input = research_convergence_test_input(&workflow.run.id);
    input.max_iterations = Some(3);
    input.no_progress_iteration_limit = Some(1);
    let step = store.run_research_convergence_to_stop(input).unwrap();
    assert!(
        step.status.settled,
        "the fixture must settle except for publication-grade citation quality"
    );

    let report = store
        .compile_research_convergence_report(&workflow.run.id)
        .unwrap();
    assert_eq!(report.judgment.overall_decision, "reject");
    assert_eq!(
        report.judgment.scores["measurement_claims_without_document_anchor"].as_u64(),
        Some(1)
    );
    assert!(
        report
            .judgment
            .blocking_findings
            .to_string()
            .contains("measurement_claims_without_document_anchor")
    );
}

#[test]
fn severe_research_report_judgment_blocks_untrusted_only_source_evidence() {
    // CLAIM: final convergence judgments require publication-grade source-card
    // evidence, not merely any claim-shaped source.
    // ORACLE: a settled run whose statement is backed only by an untrusted
    // source gets a rejected judgment with a primary-source blocker.
    // SEVERITY: Severe because generated evidence recursion is one of the
    // easiest ways for a polished research report to become hollow.
    let store = test_store("research-report-judgment-untrusted-source");
    let workflow = store
        .create_deep_research_run("untrusted-only source evidence")
        .unwrap();
    let card = store
        .add_source_card(SourceCardInput {
            title: "Unverified forum mirror about Codec Z".to_string(),
            url: "https://example.com/untrusted-codec-z".to_string(),
            source_type: "forum_mirror".to_string(),
            provider: "test".to_string(),
            summary: "An untrusted mirror claims Codec Z is production ready.".to_string(),
            claims: vec![SourceClaim {
                claim: "Codec Z is production ready.".to_string(),
                kind: "fact".to_string(),
                confidence: 0.9,
            }],
            retrieved_at: None,
            metadata: json!({
                "source_role": "primary",
                "trust_level": "untrusted",
                "reliability_score": 0.1
            }),
        })
        .unwrap();
    store
        .link_source_card_to_research_run(
            &workflow.run.id,
            &card.id,
            "forums",
            "snippet-only",
            "untrusted-fixture",
            None,
        )
        .unwrap();
    store
        .ingest_research_claims_from_model_output(
            &workflow.run.id,
            &card.id,
            "test",
            "model",
            r#"{
                    "claims": [{
                        "text": "Codec Z is production ready.",
                        "kind": "fact",
                        "subject": "Codec Z",
                        "predicate": "is",
                        "object": "production ready",
                        "confidence": 0.9,
                        "caveats": ["Generated answer only."],
                        "quote": "production ready"
                    }]
                }"#,
        )
        .unwrap();
    let mut input = research_convergence_test_input(&workflow.run.id);
    input.max_iterations = Some(3);
    input.no_progress_iteration_limit = Some(1);
    let step = store.run_research_convergence_to_stop(input).unwrap();
    assert!(
        step.status.settled,
        "source-quality gate should reject at judgment time, not by faking convergence blockers"
    );

    let report = store
        .compile_research_convergence_report(&workflow.run.id)
        .unwrap();
    assert_eq!(report.judgment.overall_decision, "reject");
    assert_eq!(
        report.judgment.scores["claims_without_primary_source_evidence"].as_u64(),
        Some(1)
    );
    assert!(
        report
            .judgment
            .blocking_findings
            .to_string()
            .contains("claims_without_primary_source_evidence")
    );
}

#[test]
fn severe_research_report_judgment_blocks_stale_current_evidence() {
    // CLAIM: stale source-card evidence cannot support a publication-grade
    // current-position judgment without fresh verification.
    // ORACLE: a stale source-backed statement records an explicit stale
    // judgment blocker.
    // SEVERITY: Severe because stale sources can make old claims look newly
    // verified in fast-changing research domains.
    let store = test_store("research-report-judgment-stale-source");
    let workflow = store
        .create_deep_research_run("stale source evidence")
        .unwrap();
    let card = store
        .add_source_card(SourceCardInput {
            title: "Old codec deployment note".to_string(),
            url: "https://example.com/old-codec-note".to_string(),
            source_type: "official_doc".to_string(),
            provider: "test".to_string(),
            summary: "An old note says Codec Z is disabled by default.".to_string(),
            claims: vec![SourceClaim {
                claim: "Codec Z is disabled by default.".to_string(),
                kind: "fact".to_string(),
                confidence: 0.86,
            }],
            retrieved_at: Some("2020-01-01T00:00:00Z".to_string()),
            metadata: json!({ "source_role": "primary", "trust_level": "high" }),
        })
        .unwrap();
    assert!(
        source_card_metadata_strings(&card.metadata, "quality_flags")
            .iter()
            .any(|flag| flag == "stale_source"),
        "fixture source must be normalized as stale"
    );
    store
        .link_source_card_to_research_run(
            &workflow.run.id,
            &card.id,
            "official-docs",
            "full-text",
            "stale-source-fixture",
            None,
        )
        .unwrap();
    store
        .ingest_research_claims_from_model_output(
            &workflow.run.id,
            &card.id,
            "test",
            "model",
            r#"{
                    "claims": [{
                        "text": "Codec Z is disabled by default.",
                        "kind": "fact",
                        "subject": "Codec Z",
                        "predicate": "is",
                        "object": "disabled by default",
                        "confidence": 0.86,
                        "caveats": ["Old source."],
                        "quote": "disabled by default"
                    }]
                }"#,
        )
        .unwrap();
    let mut input = research_convergence_test_input(&workflow.run.id);
    input.max_iterations = Some(1);
    input.no_progress_iteration_limit = Some(1);
    store.run_research_convergence_to_stop(input).unwrap();

    let report = store
        .compile_research_convergence_report(&workflow.run.id)
        .unwrap();
    assert_eq!(report.judgment.overall_decision, "reject");
    assert_eq!(
        report.judgment.scores["stale_current_statement_evidence"].as_u64(),
        Some(1)
    );
    assert!(
        report
            .judgment
            .blocking_findings
            .to_string()
            .contains("stale_current_statement_evidence")
    );
}

#[test]
fn severe_research_document_extraction_fails_closed_for_malformed_or_unsupported_inputs() {
    // CLAIM: document extraction records only bounded, inspectable artifacts and does not
    // pretend malformed PDFs, unsupported XLSX, missing files, or malformed CSV are evidence.
    // ORACLE: missing/malformed inputs error or record explicit blocked status with warnings,
    // and no table/span evidence is manufactured for unsupported documents.
    // SEVERITY: Severe because document/table artifacts can back numeric research claims.
    let store = test_store("research-document-severe");
    let workflow = store.create_deep_research_run("benchmark tables").unwrap();

    assert!(
        store
            .extract_research_document_file(ResearchDocumentInput {
                run_id: workflow.run.id.clone(),
                research_source_id: None,
                source_card_id: None,
                path: store.paths().home.join("missing.csv"),
                media_type: None,
            })
            .is_err()
    );

    let malformed_csv = store.paths().home.join("malformed.csv");
    fs::write(&malformed_csv, "a,b\n\"unterminated,b\n").unwrap();
    assert!(
        store
            .extract_research_document_file(ResearchDocumentInput {
                run_id: workflow.run.id.clone(),
                research_source_id: None,
                source_card_id: None,
                path: malformed_csv,
                media_type: None,
            })
            .is_err()
    );

    let xlsx = store.paths().home.join("book.xlsx");
    fs::write(&xlsx, b"not actually an xlsx").unwrap();
    let xlsx_record = store
        .extract_research_document_file(ResearchDocumentInput {
            run_id: workflow.run.id.clone(),
            research_source_id: None,
            source_card_id: None,
            path: xlsx,
            media_type: None,
        })
        .unwrap();
    assert!(
        xlsx_record
            .document
            .extraction_status
            .starts_with("blocked_")
    );
    assert!(
        xlsx_record
            .document
            .warning_flags
            .contains(&"xlsx_extraction_failed".to_string())
    );
    assert!(xlsx_record.tables.is_empty());
    assert!(xlsx_record.spans.is_empty());

    let pdf = store.paths().home.join("bad.pdf");
    fs::write(&pdf, b"%PDF-1.7\nnot a valid pdf").unwrap();
    let pdf_record = store
        .extract_research_document_file(ResearchDocumentInput {
            run_id: workflow.run.id.clone(),
            research_source_id: None,
            source_card_id: None,
            path: pdf,
            media_type: None,
        })
        .unwrap();
    assert!(
        pdf_record
            .document
            .extraction_status
            .starts_with("blocked_")
    );
    assert!(pdf_record.tables.is_empty());
    assert!(pdf_record.document.error_message_redacted.is_some());
}
