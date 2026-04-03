# 專案進展歷史與工作流盤點

日期：`2026-03-30`
範圍：整理自專案建立以來到 `2026-03-30` 的 `git log`，並交叉比對目前 Rust 主線 CLI 與維護文件。
用途：作為個人開發盤點、每週進度整理，以及偏公司管理視角的專案進展說明。

## 1. 總結摘要

這個專案在 `2026-03-10` 起步時，還比較像是聚焦在 Grafana dashboard 與 alert export/import 的工具；到了 `2026-03-30`，它已經演變成以統一 `grafana-util` CLI 為核心的 Rust 主線維運工具組。

從 git 歷史可觀察到：

- 總檢視提交數：`411`
- 本次盤點涵蓋時間：`2026-03-10` 到 `2026-03-30`
- 目前維護主線：Rust `grafana-util`
- Python 角色：仍保留作為維護者參考與相容性脈絡，但已不是主要對外操作面

整體演進脈絡很清楚：

1. 先建立 dashboard 與 alert export/import 基礎能力
2. 再把命令介面統一成一個產品化 CLI
3. 接著擴大到 access、datasource、sync 等工作流
4. 再往治理、檢查、拓樸、稽核、review-first 安全流程深化
5. 最後補上跨模組的 overview / project-status 專案級可視化

換句話說，這個專案已經不只是備份或匯入工具，而是具備下列能力的 Grafana 維運工作台：

- dashboard 盤點、分析、匯出/匯入、比對、刪除、截圖與治理檢查
- datasource 盤點、匯出/匯入、比對，以及有限制的即時變更
- alerting 資產的盤點與 export/import/diff
- user、org、team、service account 的生命週期管理
- staged sync、review、apply、audit、bundle 與 promotion-preflight
- 專案層級的 overview 與 project-status 狀態彙整

## 2. 每週進展彙整

### 2026-W11（`2026-03-10` 到 `2026-03-15`，`200` commits）

這一週是專案基礎與產品定義期。

- 專案從 dashboard export/import 與 alert 工具起步。
- 統一命令介面的方向被建立，後續正式收斂成 `grafana-util`。
- Rust 不再只是附帶實驗，而開始成為正式實作主線。
- access 管理、datasource 流程、sync 規劃也都在這一週展開。
- 同時也把安裝、打包、建置、文件結構、認證模式、dry-run/diff、版本與 release 規則建起來。

主要成果：

- 專案從「幾支 Grafana 維運腳本」升級成「具有清楚命令模型的維運工具產品」

### 2026-W12（`2026-03-16` 到 `2026-03-22`，`119` commits）

這一週的重點是把契約、可用性與可信度做紮實。

- CLI help、別名與命令分組被整理得更像正式產品。
- sync artifact、schema 與 typed output 變得更正式。
- screenshot、變數檢查、governance gate、dependency inspection 開始成為可用功能，而不是概念。
- dashboard 與 alert 的 multi-org 能力進一步擴展。
- release 自動化與 GitHub 包裝流程被強化。
- Python / Rust parity 與輸出契約對齊被持續修正。

主要成果：

- 工具不只功能更多，也更像可以實際交付、發版、被信任使用的產品

### 2026-W13（`2026-03-23` 到 `2026-03-28`，`91` commits）

這一週是 Rust 深化與互動式操作面的進階期。

- governance gate 變得更完整，也更接近政策與規則導向。
- topology、dependency analysis、perf audit、Prometheus cost audit、schema review 等分析能力被補齊。
- sync audit 與 review diff 工作流成熟很多。
- dashboard 與 sync 的互動式 TUI 被建立起來。
- browse、delete、import-review 這類工作流變得更接近實務操作。
- promotion-preflight 與 datasource secret placeholder 檢查也開始出現。
- 大型 Rust 模組被持續拆分，以維持可維護性。

主要成果：

- 專案從「很多子命令的 CLI」進一步走向「具分析、審查與互動操作能力的維運工作台」

### 2026-W14（`2026-03-30`，`1` commit）

這一天的重點是專案層級可視化落地。

- staged 與 live 的 `overview` / `project-status` 正式落地

主要成果：

- 專案終於擁有跨模組的整體狀態視角，而不只是各功能各自輸出

## 3. 從歷史看得出的設計深度

這份 git 歷史真正值得強調的，不只是 commit 多，而是可以看出專案持續往更成熟的操作模型深化。

### 統一產品化設計

- 專案沒有停留在 dashboard、alert、access 各自獨立的工具。
- 它被有意識地整合成一個有命名空間的統一操作面：`grafana-util`。
- 這代表它有明確產品形狀，而不是一堆零散腳本。

### Rust 主線轉移

- 這不是單純「加上一份 Rust 版本」。
- 歷史顯示它逐步把主要維護的 operator surface 移到 Rust，同時保留 Python 作為維護與相容性脈絡。
- 這表示作者在意的是長期維護、型別契約、打包品質與正式交付能力。

### Review-First 工作流設計

- 很多能力都不是直接對 Grafana 做變更，而是設計成 `export -> inspect/diff/preflight -> dry-run/review -> apply`。
- 這種模式在 dashboard、datasource、alert、sync 都看得到。
- 這反映出專案重視的是安全變更與可審查流程，而不是單純 API 封裝。

### Staged 與 Live 分層

- 專案反覆強化 staged artifact 與 live Grafana state 的區分。
- 這條設計線在 sync、promotion、overview、project-status 都很明顯。
- 這是很有份量的架構決策，因為它讓規劃、審查、執行與追蹤變得清楚。

### 治理與分析能力被當成一級功能

- 專案早就超過 CRUD、backup、import 這種基礎層次。
- 它已經加入 dependency inspection、governance policy、topology graph、blast-radius 報告、schema review，以及 cost/performance audit 訊號。
- 這讓它從「工具輔助」升級成「協助操作決策的系統」。

### Multi-Org 與 Replay 意識

- multi-org 的 inventory、export、import 與 org-routed replay 不是順手補上，而是反覆深化。
- 歷史中可以看到 org scope、permission metadata、export-org guard、replay 行為一直被處理。
- 這代表作者有把 Grafana 真實企業場景放在設計裡。

### 互動式維運體驗

- 專案沒有停留在純文字輸出。
- 它逐步建立 browse、review、TUI workbench 等互動式操作面。
- 這顯示作者不是只追求功能存在，而是也在設計實際操作體驗。

### 專案層級的可視化

- `overview` 與 `project-status` 的落地很重要，因為它把專案提升到跨模組整體觀。
- 這意味著系統可以從 dashboard / datasource / alert / access / sync 的上層去看整體狀態。
- 這是產品成熟度的躍升，不是表面上的附加功能。

## 4. 每日進展摘要

### 2026-03-10（`13` commits）

- 建立 dashboard export/import 與 alert rule utility 的起始能力。
- 形成第一版可操作的命令模型。
- 建立最早的文件與維護脈絡。

### 2026-03-11（`23` commits）

- 將關鍵 Grafana API 流程移植到 Rust。
- 加入 dry-run 與 diff 工作流。
- 擴展 dashboard list，並開始 access 管理，涵蓋 user、team、service account。
- 專案開始具備可安裝、可建置、可持續演進的基礎。

### 2026-03-12（`25` commits）

- 擴大 access CRUD 能力，覆蓋 user 與 team。
- 新增 dashboard datasource listing、multi-org listing、multi-org export。
- 將 Python 與 Rust 收斂到統一 `grafana-util` 方向。
- 拆分大型 Rust dashboard / access / alert 模組，避免後續失控。

### 2026-03-13（`38` commits）

- 明顯提升 dashboard import dry-run 輸出與 JSON 輸出。
- 加入 `inspect-export`、live inspection、query reporting 與更完整的報告格式。
- 開始建立 dashboard governance helper 與 datasource CLI 能力。
- 補上 Loki、Flux、SQL、Prometheus 等 query family analyzer。

### 2026-03-14（`25` commits）

- 強化 folder-path 與 org-scoped dashboard import 安全性。
- 增加 governance risk metadata 與 inspection output selector。
- 新增 datasource import 工作流並收緊 datasource 契約。
- 讓 dashboard 與 datasource 流程更接近可在真實環境安全重播的層次。

### 2026-03-15（`76` commits）

- 將統一 CLI 正式命名為 `grafana-util`。
- 擴大 access import/export/diff，覆蓋 users、teams 與 service-account snapshot。
- 新增 datasource live admin / modify、org-routed datasource export/import，以及 access org management。
- 新增 dashboard import routing preview / replay。
- 建立 sync 重要基礎：plan、preflight lineage、trace ID、continue-on-error、import extension。
- 這一天可以視為專案從多個子工具，正式跨進完整多模組 operator toolkit 的關鍵里程碑。

### 2026-03-16（`81` commits）

- 正式化 sync artifact schema 與 machine-readable contract。
- 改善 help 分組與 top-level alias。
- 新增 dashboard screenshot 與 variable inspection。
- 加入 dashboard governance CI gate 與 governance-json facts。
- 擴展 migration、sync、dependency inspection 契約。
- 專案在這一天同時強化了「操作能力」與「產品交付能力」。

### 2026-03-17（`17` commits）

- 擴大 Rust dashboard org-aware listing 與 inspection。
- 加入 multi-org alert inventory。
- 預設匯出 dashboard permission metadata。
- multi-org 與 permission-aware 工作流的完整性又提升了一層。

### 2026-03-18（`6` commits）

- 擴充 all-org dashboard export metadata。
- 持續強化產品化路徑，而不是隨意再開一條新功能線。

### 2026-03-19（`3` commits）

- 精修 dashboard inspect 的 datasource / query reporting 語意。
- 強化 inspection output 本身的可信度與解釋力。

### 2026-03-21（`9` commits）

- 新增 datasource preset contract 與 live gate。
- 再次收緊 dashboard import / inspection 契約。
- 擴展 access replay、alert replay 與 sync bundle live tooling。
- 提升 blast-radius 與 dependency contract 的覆蓋度。

### 2026-03-22（`3` commits）

- 透過略過不必要 preflight 改善 dashboard import 效能。
- 讓 Python sync workflow 更靠近 Rust 行為。
- 重點放在讓新工作流更有效率且內部一致。

### 2026-03-23（`28` commits）

- 大幅深化 Rust governance、dependency、topology、audit、policy check。
- 加入 dashboard governance gate、datasource/query/complexity policy、schema review、concurrent scan。
- 新增 perf audit、graph/topology output、Prometheus cost audit。
- 擴展 sync audit、review diff、non-rule alert ownership、staged alert visibility。
- 建立 dashboard 與 sync 的互動式 TUI。

### 2026-03-24（`22` commits）

- 將文件重心轉向 Rust-first operator surface。
- 拆分 Rust sync orchestration internals 與 dashboard 維護熱點模組。
- 保留 Python 參考脈絡，而不是粗暴移除。
- 這一天應該被理解為邊界整理與產品定位收斂的重要節點。

### 2026-03-25（`4` commits）

- 拆分 Rust 測試熱點，恢復可維護性與品質閘道穩定性。
- 顯示作者把 maintainability 與 validation 視為產品品質的一部分，而不是邊角工作。

### 2026-03-26（`1` commit）

- 新增 Rust dashboard browser 與 delete workflow。

### 2026-03-27（`17` commits）

- 擴展 dashboard browse 與 governance workflow。
- 新增 shared inspect workbench，並拆分過大的 Rust CLI / support 模組。
- 加入 sync promotion-preflight skeleton 與正式化 promotion mapping。
- 加入 datasource secret placeholder preflight。
- 這表示專案開始往更安全的環境交接與 promotion 設計深化。

### 2026-03-28（`19` commits）

- 收緊 dashboard、datasource、sync、promotion 的邊界。
- 改善 Rust TUI shell grammar、overlay 行為與互動文字一致性。
- 加入互動式 dashboard import review 與 import dry-run mode。
- 持續拆分 interactive import state / loader / review 以維持可維護性。

### 2026-03-30（`1` commit）

- 正式落地 staged / live `overview` 與 `project-status`，提供跨模組可視化。

## 5. 目前的產品形狀

目前主要維護的 root commands 為：

- `grafana-util dashboard`
- `grafana-util datasource`
- `grafana-util alert`
- `grafana-util access`
- `grafana-util sync`
- `grafana-util overview`
- `grafana-util project-status`

部分 top-level alias 也已存在：

- `grafana-util db`
- `grafana-util ds`
- `grafana-util sy`

從策略面來看，目前專案結構很清楚：

- `dashboard`、`datasource`、`alert`、`access`、`sync` 是功能模組主軸
- `overview` 與 `project-status` 是專案層級可視化主軸
- Rust 是正式對外的操作面
- Python 則保留作為維護參考與相容性脈絡

## 6. 目前工作流地圖

### Dashboard 工作流

主要操作故事：

1. 盤點 live dashboards
2. 分析 dependencies、queries、governance、topology
3. 匯出 artifacts
4. 審查 staged import 或 delete 影響
5. 執行 import / delete，或用 screenshot 協助分析

代表性命令：

- `grafana-util dashboard browse`
- `grafana-util dashboard list`
- `grafana-util dashboard export`
- `grafana-util dashboard inspect-export`
- `grafana-util dashboard inspect-live`
- `grafana-util dashboard inspect-vars`
- `grafana-util dashboard governance-gate`
- `grafana-util dashboard topology`
- `grafana-util dashboard import`
- `grafana-util dashboard diff`
- `grafana-util dashboard delete`
- `grafana-util dashboard screenshot`

### Datasource 工作流

主要操作故事：

1. 盤點 live datasource inventory
2. 匯出標準化 datasource 狀態
3. 比對 drift
4. 預覽 org-aware import / replay
5. 需要時執行有限制的 live mutation

代表性命令：

- `grafana-util datasource list`
- `grafana-util datasource export`
- `grafana-util datasource import`
- `grafana-util datasource diff`
- `grafana-util datasource add`
- `grafana-util datasource modify`
- `grafana-util datasource delete`

### Alert 工作流

主要操作故事：

1. 盤點 live alert inventory
2. 匯出 alert rules 與相關 alerting resources
3. 比對 staged 與 live 狀態
4. 預覽或執行 import 工作流

代表性命令：

- `grafana-util alert list-rules`
- `grafana-util alert export`
- `grafana-util alert import`
- `grafana-util alert diff`

### Access 工作流

主要操作故事：

1. 盤點身分與組織狀態
2. 管理 users、orgs、teams、service accounts
3. 匯出 / 匯入受控身份資產
4. 比對 staged 與 live membership / account 狀態

代表性命令：

- `grafana-util access user ...`
- `grafana-util access org ...`
- `grafana-util access team ...`
- `grafana-util access service-account ...`

目前已涵蓋的行為：

- list
- add
- modify
- delete
- export
- import
- diff
- service-account token add/delete

### Sync 與 Promotion 工作流

主要操作故事：

1. 彙整 desired state
2. 建立 plan 與 preflight 文件
3. review 後再 apply
4. 稽核 lock / live state
5. 打包 staged assets 成為一個 bundle
6. 為環境 promotion 準備交接與檢查資料

代表性命令：

- `grafana-util sync summary`
- `grafana-util sync plan`
- `grafana-util sync review`
- `grafana-util sync apply`
- `grafana-util sync audit`
- `grafana-util sync preflight`
- `grafana-util sync assess-alerts`
- `grafana-util sync bundle-preflight`
- `grafana-util sync bundle`
- `grafana-util sync promotion-preflight`

### 專案層級可視化工作流

主要操作故事：

1. 聚合多個模組的 staged artifacts
2. 輸出一份專案級狀態畫面
3. 區分 staged readiness 與 live Grafana 狀態
4. 讓操作者可以從整體專案視角進行判讀，而不是只看單一模組

代表性命令：

- `grafana-util overview`
- `grafana-util overview live`
- `grafana-util project-status`
- `grafana-util project-status live`

## 7. 從需求面來看目前功能覆蓋度

如果不按模組名稱，而是按實際維運需求來看，這個專案目前已經有下列覆蓋度：

- dashboard 遷移與分析：強
- datasource 遷移與 live 管理：強
- alert 資產 export/import/diff：穩定
- 身分與組織生命週期管理：穩定
- review-first 的 sync 與 promotion 安全流程：強，而且是專案的重要差異化
- 專案層級 staged/live 可視化：剛落地，但戰略價值很高

## 8. 進展判讀

從 repository 歷史與目前維護文件來看，專案大致處於以下階段：

- 基礎建設階段：完成
- 統一 CLI 階段：完成
- 多模組工作流擴張階段：已達目前階段所需
- governance / review / audit 階段：大致已落地
- 專案級可視化階段：已於 `2026-03-30` 落地

目前專案的姿態，已經不再是「盡量快速多做幾個功能」。
比較準確的描述是：

- 穩定既有模組契約
- 保持 docs / help / output 與真實行為一致
- 只有在使用者或真實場景證明缺少關鍵訊號時，才重開某一條功能線
- 把 `overview` 與 `project-status` 控制在薄層消費者角色，而不是再養出新的龐大中心模組

## 9. 實務結論

回頭看這段歷史，這個專案已經完成的事情其實很有份量：

- 它建立了 `grafana-util` 這個明確的產品形狀
- 它把主要維護操作面移到 Rust
- 它為 dashboard、datasource、alert、access、sync 建立了有實質價值的工作流
- 它透過 review-first、dry-run、preflight、governance、audit 等能力，形成自己的差異化
- 它最近又補上了 project-level overview / status，讓專案進展與狀態可以從更高層次被理解

如果這份內容要拿給外部 reviewer、主管或團隊成員看，最應該被看見的是：

- 這不是淺層的功能累加
- 歷史裡可以明確看見 workflow safety、typed output、架構邊界、org-aware replay、operator review surface 被反覆深化
- 這個專案展現的是有意識的系統設計與產品化開發，而不是隨手疊命令

如果要用類似公司週報的方式濃縮，可這樣表達：

- Week 1：建立產品方向與統一命令模型
- Week 2：讓契約、審查面與操作工作流變得可信
- Week 3：深化 Rust 分析、TUI、browse、promotion 與 review-first 流程
- Week 4：補上專案級可視化並開始轉向穩定化

## 10. 來源依據

本文件整理依據包括：

- `git log`（自專案建立至 `2026-03-30`）
- `README.md`
- `docs/user-guide.md`
- `docs/internal/current-capability-inventory-2026-03-30.md`
- `docs/internal/overview-architecture.md`
- `docs/internal/project-status-architecture.md`
- `rust/src/cli.rs`
- `python/grafana_utils/unified_cli.py`
