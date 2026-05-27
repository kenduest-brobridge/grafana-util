# ai-changes-archive-2026-05-28

## 2026-05-25 - Shared datasource artifact detail projection
- Summary: Added a shared datasource browser detail projection for local artifact records and routed datasource local inspect plus snapshot datasource review rows through it before appending review evidence.
- Tests: cargo test --quiet datasource_browser_detail_lines_from_details_formats_local_artifact_identity; cargo test --quiet tail_inspect; cargo test --quiet snapshot_review_browser; cargo test --quiet snapshot; cargo test --quiet datasource (outside sandbox for local mock-server coverage after sandbox denied binding); RUSTFLAGS=-Dwarnings cargo check --quiet --no-default-features --all-targets; cargo fmt --check; python3 scripts/tui_inventory_report.py; make quality-ai-workflow; git diff --check
- Impact: Datasource local list and snapshot datasource review browser rows now share identity fact shaping for Name, UID, Type, Org, URL, Access, and Default while preserving existing shared review evidence projection and public CLI/doc surfaces.
- Rollback/Risk: Low. The change centralizes equivalent datasource fact strings and focused datasource/snapshot tests cover both local artifact browser paths.
- Follow-up: Continue migrating compatible local artifact browser detail sections onto shared projections where output contracts are already aligned.
