# ai-status-archive-2026-04-15

## 2026-04-13 - Fix access user browse TUI layout
- State: Done
- Scope: access user browser detail navigation, shared TUI footer/dialog sizing/rendering, user browser footer control layout, and focused Rust regressions.
- Baseline: `access user browse` facts navigation counted fewer user fact rows than the right pane rendered, so Down/End could not reach the final rows. The user browser footer also allocated four terminal rows while rendering three control rows plus a status line inside a bordered block, causing clipping and visual misalignment. The edit/search overlays each owned their own centering and frame style instead of using a common TUI dialog surface.
- Current Update: corrected the user facts line count, added shared `tui_shell::footer_height`, `centered_fixed_rect`, `dialog_block`, and `render_dialog_shell` helpers, made footer controls clip instead of wrapping across rows, switched user browse footer controls to the shared grid alignment helper, and moved user edit/search overlays onto the shared dialog shell.
- Result: the facts pane can select the final user fact row, the footer has enough height for its controls/status without rows overwriting each other, and user browse overlays now share the same centered dialog frame and background treatment.

## 2026-04-13 - Add team browse membership actions
- State: Done
- Scope: access team browser member-row actions, shared team browse footer/dialog presentation, focused Rust regressions, and worker-assisted implementation review.
- Baseline: selecting a team member row in `access team browse` could show membership detail, but `e` only told users to select a team row and there was no direct way from the member row to remove that relationship or change team-admin state. Team browse also still owned local footer/control and dialog presentation code while user browse had moved to shared TUI shell helpers.
- Current Update: member rows now keep user-owned fields read-only and direct account edits to `access user browse`; `r` and member-row `d` open a confirmation dialog before removing the selected team membership through the existing team modify flow; `a` grants or revokes team-admin state through the existing membership update path. Team-row `d` opens the whole-team delete confirmation dialog. Team browse footer controls now use the shared control grid/height helpers, and team edit/search/delete overlays use the shared dialog shell.
- Result: team browse can manage team/member relationships without pretending to edit user profile fields, and the browser presentation is closer to the shared TUI treatment already used by user browse.
