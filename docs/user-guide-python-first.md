Grafana Utilities 使用指南（Python-first 完整版）
=============================================

本指南以 Python 入口為優先：`python3 -m grafana_utils`，逐命令完整說明參數用途、參數使用差異、與建議情境。

1) 全域前置
------------

先用 `-h` 驗證你現在看到的旗標版本，避免依賴舊文件誤差：

```bash
python3 -m grafana_utils -h
python3 -m grafana_utils dashboard -h
python3 -m grafana_utils alert -h
python3 -m grafana_utils datasource -h
python3 -m grafana_utils access -h
```

統一調用格式：

```text
python3 -m grafana_utils <domain> <command> [options]
```

如果工具已安裝，可改用：

```text
grafana-util <domain> <command> [options]
```

2) 全域共同參數（多數命令都可用）
-------------------------------

| 參數 | 用途 | 適用情境 |
| --- | --- | --- |
| `--url` | Grafana base URL。預設 `http://127.0.0.1:3000` | 所有命令都必需明確指定連線目標 |
| `--token`、`--api-token` | API token。優先使用 `--token` 名稱 | 腳本或無需跨 org 操作的單一環境 |
| `--prompt-token` | 互動式輸入 token，不會在 shell 顯示 | CI 或臨時作業避免 token 外洩 |
| `--basic-user` | Basic Auth 使用者 | 需要跨組織、`--org-id`、`--all-orgs` 等能力時多數需要 |
| `--basic-password` | Basic Auth 密碼 | 配合 `--basic-user` 使用 |
| `--prompt-password` | 互動式輸入密碼 | 避免密碼寫在命令列歷史 |
| `--timeout` | HTTP timeout 秒數，預設 30 | API 回應慢或網路不穩時可提高 |
| `--verify-ssl` | 開啟憑證驗證（預設關閉） | 內網自簽憑證以外的正式 TLS 環境 |

3) dashboard 命令
-----------------

### 3.1 `dashboard export`（legacy `export-dashboard`）

**用途**：匯出 dashboard 為 `raw/` 與 `prompt/`，後續可用於 API 匯入或 Web UI 匯入流程。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--export-dir` | 匯出根目錄 | 預設 `dashboards/`，會放 `raw/`、`prompt/` |
| `--page-size` | 分頁抓取大小 | 大 instance 可調高加速，但回應壓力也會上升 |
| `--org-id` | 固定匯出單一目標組織 | 僅在 Basic Auth 有效，切換來源 org |
| `--all-orgs` | 匯出目前憑證可見全部 org | Basic Auth 必要，用於集中備份 |
| `--flat` | 不保留資料夾階層，平鋪輸出 | 適合想直接用 git 差異掃描，不看資料夾結構 |
| `--overwrite` | 覆蓋既有檔案 | 用在 CI/重跑時 |
| `--without-dashboard-raw` | 不輸出 `raw/` | 僅需 Web UI 匯入檔 |
| `--without-dashboard-prompt` | 不輸出 `prompt/` | 僅需 API 匯入 |
| `--dry-run` | 不寫檔，顯示預期檔案 | 導入前驗證路徑與授權 |
| `--progress` | 匯出時輸出簡短進度 | 長時間批次建議先加 `--progress` |
| `-v`, `--verbose` | 詳細輸出，覆蓋 `--progress` | 追查個別 dashboard 問題 |
| `--help` | 顯示此命令完整 help | 可快速核對 parser 參數 |

**建議情境**
- 備份整庫：先用 `--all-orgs`，再結合 `diff`/`inspect-export`。
- 只準備 API 還原：加 `--without-dashboard-prompt`。
- 只準備 UI 匯入：加 `--without-dashboard-raw`。

示例：
```bash
python3 -m grafana_utils dashboard export --url http://localhost:3000 --basic-user admin --basic-password admin --export-dir ./dashboards --overwrite
```

### 3.2 `dashboard list`（legacy `list-dashboard`）

**用途**：列出 live dashboards 摘要，不寫檔。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--page-size` | 每頁 dashboard 數量 | 大庫調高可減少往返 |
| `--org-id` | 僅看指定 org | 需要 Basic Auth |
| `--all-orgs` | 聚合所有可見 org | 需要 Basic Auth，產出跨 org 報表 |
| `--with-sources` | table/csv 時補齊 datasource 名稱 | 會增加額外 API 呼叫 |
| `--table` | 表格輸出（預設） | `--json` 不要這樣看時 |
| `--csv` | CSV 輸出 | 給後續批次匯入或 Excel 分析 |
| `--json` | JSON 輸出 | API 串接腳本 |
| `--no-header` | 表格不顯示欄位標題 | 腳本化比較輸出時常用 |
| `--output-format` | 一次選定 `table/csv/json` | 不可與 `--table/--csv/--json` 混用 |

**參數差異提醒**：`--with-sources` 僅針對 list，`diff`/`import` 不會看它。

### 3.3 `dashboard list-data-sources`（legacy `list-data-sources`）

**用途**：列出 Grafana live datasource 清單。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--table` | 表格輸出 | 預設 |
| `--csv` | CSV 輸出 | 外部報表 |
| `--json` | JSON 輸出 | 供自動化流程 |
| `--no-header` | 表格無標題列 | 報告比對 |
| `--output-format` | `table/csv/json` 單旗標 | 互斥於 `--table/--csv/--json` |

### 3.4 `dashboard import`（legacy `import-dashboard`）

**用途**：將 `raw/` dashboard 匯入 Grafana，支援 dry-run、覆蓋策略、folder 控制。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--import-dir`（必填） | 指向 `raw/` 目錄 | 必須精準指定 raw，不是上層匯出根 |
| `--org-id` | 匯入到目標組織 | 需 Basic Auth，切換目標 org 時使用 |
| `--require-matching-export-org` | 匯入前檢查匯出 `orgId` | 跨環境移轉建議開啟 |
| `--replace-existing` | 存在則覆蓋更新 | 無此參數預設阻擋重複 |
| `--update-existing-only` | 僅更新已存在項目 | 用於「只補齊既有」場景 |
| `--import-folder-uid` | 強制寫入到指定目標 folder uid | 重建組織結構一致性時使用 |
| `--ensure-folders` | 先建缺少資料夾 | 大量匯入前配合 `--dry-run` 檢查更清楚 |
| `--import-message` | 版本紀錄 message | 供稽核 |
| `--require-matching-folder-path` | 僅當 folder path 相同才更新 | 保留 folder 對應規範時用 |
| `--dry-run` | 僅預覽 | 先跑一次確認 |
| `--table` | dry-run 時以表格顯示 | 觀察 UID/動作摘要 |
| `--json` | dry-run 時輸出 JSON | CI 輸出比對 |
| `--no-header` | dry-run table 無表頭 | 輸出對齊 |
| `--output-format` | `text/table/json` | 只能代替輸出旗標，不可與 `--table/--json` 同用 |
| `--output-columns` | dry-run table 欄位白名單 | 僅 `--dry-run --table` 可用 |
| `--progress` | 逐筆進度 | 批次 large 結構 |
| `--verbose` | 詳細日誌 | 除錯匯入邏輯 |

**差異情境**
- 有跨環境導入需求：`--org-id` + `--require-matching-export-org`。
- 確保不破壞既有資料：`--update-existing-only` 搭配 `--dry-run` 檢查。
- 想強制目標 folder：用 `--import-folder-uid` 或 `--require-matching-folder-path`。

### 3.5 `dashboard diff`

**用途**：比較 `raw/` 匯出與 live 狀態。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--import-dir`（必填） | 指向 raw 匯出目錄 | 僅比對，不改動 API |
| `--import-folder-uid` | 比對時覆寫目標 folder uid | 比對時假設目標組織 folder 對應改變 |
| `--context-lines` | diff 前後文行數，預設 3 | 大型 JSON 變更可調高觀察差異 |

### 3.6 `dashboard inspect-export`

**用途**：離線檢視原始匯出目錄，支援多種報表格式。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--import-dir`（必填） | 指向 raw 匯出目錄 | 不連線 Grafana |
| `--help-full` | 顯示延伸範例 | 快速查欄位與範例 |
| `--output-format` | `text/table/json/report-table/report-csv/report-json/report-tree/report-tree-table/governance/governance-json` | 覆蓋舊式旗標差異輸出 |
| `--report-columns` | report 欄位白名單 | 專注查詢層欄位 |
| `--report-filter-datasource` | 依 datasource 精準篩選 | 分析特定 datasource 來源 |
| `--report-filter-panel-id` | 依 panel id 篩選 | 單面板問題定位 |
| `--no-header` | 表格輸出不列標頭 | 報告比對 |
| `--json`（隱藏） | legacy compat（不推薦直接指定） | 由 `--output-format` 取代 |
| `--table`（隱藏） | legacy compat（不推薦直接指定） | 同上 |

### 3.7 `dashboard inspect-live`

**用途**：直接抓 live dashboards 做同一套報表分析，不落地固定檔案。

參數同 `inspect-export`，外加：

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--page-size` | 抓 dashboard 分頁 | live 呼叫頁數控制 |

4) alert 命令
-------------

### 4.1 `alert export`（legacy `export-alert`）

**用途**：匯出 alerting resource 到 `raw/`。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--output-dir`（預設 `alerts`） | 匯出根目錄 | 與 dashboard 分開目錄管理 |
| `--flat` | 以平鋪方式輸出 alert 資源 | 不想保留分群層時 |
| `--overwrite` | 覆蓋既有匯出 | CI 重跑時必加 |

### 4.2 `alert import`（legacy `import-alert`）

**用途**：讀 `raw/` 載入 alerting 資源，含映射修復選項。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--import-dir`（必填） | 匯入目錄 raw | 直指 `raw/` |
| `--replace-existing` | 已存在則更新 | 遷移舊環境到新環境時常用 |
| `--dry-run` | 僅預覽 | 上線前確認風險 |
| `--dashboard-uid-map` | dashboard UID 對照檔 | linked rule 來源 dashboard 改名/重建時必需 |
| `--panel-id-map` | panel id 對照檔 | linked rule panel 變更時修補 |

### 4.3 `alert diff`（legacy `diff-alert`）

**用途**：比較 local export 與 live alerting 狀態。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--diff-dir`（必填） | 比對來源 raw 目錄 | 只比較，不改動 |
| `--dashboard-uid-map` | 同 import 的 UID 對照 | 對 linked alert 的比對一致性 |
| `--panel-id-map` | 同 import 的 panel 對照 | 同上 |

### 4.4 `alert list-rules`（legacy `list-alert-rules`）
### 4.5 `alert list-contact-points`（legacy `list-alert-contact-points`）
### 4.6 `alert list-mute-timings`（legacy `list-alert-mute-timings`）
### 4.7 `alert list-templates`（legacy `list-alert-templates`）

**用途**（共同）：列出 live alerting 資源。  
這四個子命令共用輸出欄位控制：

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--table` | 表格輸出（預設） | 與機器人腳本相比較 |
| `--csv` | CSV | 匯出給外部處理 |
| `--json` | JSON | 交由腳本處理 |
| `--no-header` | 表格不列標頭 | diff/比對時 |
| `--output-format` | `table/csv/json`，互斥於三旗標 | 簡化輸出指定 |

5) datasource 命令
------------------

### 5.1 `datasource list`

**用途**：列出 live datasource inventory。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--table` | 表格輸出（預設） | 人工巡檢 |
| `--csv` | CSV | 批次匯出 |
| `--json` | JSON | API 自動化 |
| `--no-header` | 表格不列標頭 | 比較輸出 |
| `--output-format` | `table/csv/json` | 與對應旗標互斥 |

### 5.2 `datasource export`

**用途**：匯出 datasource inventory 為標準化 JSON 盤點文件。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--export-dir`（預設 `datasources`） | 匯出根目錄 | 建議每環境獨立目錄 |
| `--overwrite` | 覆蓋既有輸出 | 定期重建時 |
| `--dry-run` | 僅預覽 | 先驗證權限與路徑 |

### 5.3 `datasource import`

**用途**：將匯入檔寫回 Grafana，支援 dry-run、欄位顯示定制。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--import-dir`（必填） | 匯入目錄，需包含 `datasources.json` 與 `export-metadata.json` | 目標目錄非 `raw/` |
| `--org-id` | 匯入目標 org | 跨 org 遷移前常見 |
| `--require-matching-export-org` | 匯入前比對 orgId | 避免對錯誤 org 進行更新 |
| `--replace-existing` | 更新既有相同 resource | 同步兩環境時建議 |
| `--update-existing-only` | 僅更新既有，缺者跳過 | 僅補齊已存在資源 |
| `--dry-run` | 僅預覽行為 | 變更前核對清單 |
| `--table` | dry-run 表格輸出 | 讀取匯入摘要 |
| `--json` | dry-run JSON 摘要 | 供機器人比較 |
| `--no-header` | 表格不顯示標頭 | 報表化 |
| `--output-format` | `text/table/json`，互斥於 `--table` 與 `--json` | 僅 table/json 常見 |
| `--output-columns` | dry-run table 欄位白名單 | 與 `--table` 搭配才生效 |
| `--progress` | 進度 `current/total` | 批次大時觀察 |
| `--verbose` | 詳細日誌，覆蓋 `--progress` | 權限/單筆失敗時 |

### 5.4 `datasource diff`

**用途**：比較 export 與 live datasource。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--diff-dir`（必填） | 比對來源目錄 | 只做比較不改動 |

6) access 命令
--------------

`group` 在 parser 中等價於 `team`，即可用 `access group ...` 作為別名。

### 6.1 `access user list`

**用途**：列出使用者（org 或 global）。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--scope` | `org` 或 `global` | 選取列舉來源 |
| `--query` | 不分大小寫模糊比對 login/email/name | 搜尋大量帳號 |
| `--login` | 精準查 login | 快速定位單人 |
| `--email` | 精準查 email | 精準定位 |
| `--org-role` | 依角色篩選 | 權限盤點 |
| `--grafana-admin` | `true/false` | 管理員帳號審核 |
| `--with-teams` | 載入 team 成員關聯 | 只在需要時 |
| `--page` | 頁碼，預設 1 | 分頁遍歷 |
| `--per-page` | 每頁筆數，預設 30 | 大量輸出時 |
| `--table` | 表格輸出 | default |
| `--csv` | CSV | 外部分析 |
| `--json` | JSON | 流程自動化 |
| `--output-format` | `text/table/csv/json` | 與 `--table/--csv/--json` 互斥 |

### 6.2 `access user add`

**用途**：建立使用者。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--login` | 登入名（必填） | 建檔 |
| `--email` | 電子郵件（必填） | 通知與搜尋 |
| `--name` | 顯示名稱（必填） | 人員識別 |
| `--password` | 初始密碼（必填） | 本地帳號建立 |
| `--org-role` | 建立後 org 角色 | 權限預設 |
| `--grafana-admin` | true/false，設為伺服器管理員 | 僅做明確需求 |
| `--json` | 以 JSON 輸出 | 自動化處理結果 |

### 6.3 `access user modify`

**用途**：修改既有使用者欄位。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--user-id` / `--login` / `--email` | 三擇一定位使用者 | 避免歧義 |
| `--set-login` | 更新 login | 帳號更名 |
| `--set-email` | 更新 email | 通訊/查找更新 |
| `--set-name` | 更新顯示名稱 | 組織治理 |
| `--set-password` | 重設密碼 | 帳號回收/輪替 |
| `--set-org-role` | 更新 org 角色 | 權限調整 |
| `--set-grafana-admin` | 更新 grafana-admin | 權限調整 |
| `--json` | JSON 輸出 | 審計比對 |

### 6.4 `access user delete`

**用途**：刪除/移除使用者。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--user-id` / `--login` / `--email` | 三擇一定位使用者 | 必填其一 |
| `--scope` | `org` 或 `global`，預設 `global` | 刪除範圍 |
| `--yes` | 必要確認旗標 | 防呆 |
| `--json` | JSON 輸出 | 流程紀錄 |

### 6.5 `access team list`

**用途**：列出 teams。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--query` | fuzzy 搜尋 team 名稱/email | 批量找人群 |
| `--name` | 精準 team 名稱 | 快速定位 |
| `--with-members` | 同時顯示 members | 团队盤點 |
| `--page` | 頁碼 | |
| `--per-page` | 每頁筆數 | |
| `--table` / `--csv` / `--json` | 輸出格式 | |
| `--output-format` | `text/table/csv/json`，互斥於上述三旗標 | 一致化 CLI 體驗 |

### 6.6 `access team add`

**用途**：建立 team，可同時加人員。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--name` | team 名稱（必填） | 建立基本識別 |
| `--email` | team 聯絡信箱 | 文件化 |
| `--member` | 指定初始 member，可重複 | 一次掛上常用成員 |
| `--admin` | 指定初始 admin，可重複 | 權限初始化 |
| `--json` | JSON 輸出 | 自動化 |

### 6.7 `access team modify`

**用途**：調整 team 成員及 admin。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--team-id` / `--name` | 三擇一定位 team | |
| `--add-member` | 新增 member（可重複） | 批量授權 |
| `--remove-member` | 移除 member（可重複） | 解除權限 |
| `--add-admin` | 設為 admin（可重複） | 權限提升 |
| `--remove-admin` | 解除 admin（可重複） | 權限降級 |
| `--json` | JSON 輸出 | |

### 6.8 `access team delete`

**用途**：刪除 team。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--team-id` / `--name` | 三擇一定位 team | 必填其一 |
| `--yes` | 刪除確認 | 防呆 |
| `--json` | JSON 輸出 | |

### 6.9 `access service-account list`

**用途**：列出服務帳號。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--query` | fuzzy 搜尋名稱/login | |
| `--page` | 頁碼 | |
| `--per-page` | 每頁筆數 | |
| `--table` / `--csv` / `--json` | 輸出格式 | |
| `--output-format` | `text/table/csv/json` | 與輸出旗標互斥 |

### 6.10 `access service-account add`

**用途**：新增服務帳號。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--name` | 服務帳號名稱（必填） | 自動化專用帳號 |
| `--role` | `Viewer|Editor|Admin|None`，預設 `Viewer` | 最小權限原則 |
| `--disabled` | `true/false` | 控制啟用狀態 |
| `--json` | JSON 輸出 | |

### 6.11 `access service-account delete`

**用途**：刪除服務帳號。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--service-account-id` / `--name` | 定位目標資源 | 其一必填 |
| `--yes` | 刪除確認 | |
| `--json` | JSON 輸出 | |

### 6.12 `access service-account token add`

**用途**：建立服務帳號 token。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--service-account-id` / `--name` | 定位 token 擁有者 | `--service-account-id` 或 `--name` 二擇一 |
| `--token-name` | token 名稱（必填） | 管理與輪替 |
| `--seconds-to-live` | TTL 秒數 | 有效期控制 |
| `--json` | JSON 輸出 | |

### 6.13 `access service-account token delete`

**用途**：刪除服務帳號 token。

| 參數 | 用途 | 差異 / 情境 |
| --- | --- | --- |
| `--service-account-id` / `--name` | 定位擁有者 | 擁有者定位 |
| `--token-id` / `--token-name` | 定位 token | 二擇一 |
| `--yes` | 刪除確認 | |
| `--json` | JSON 輸出 | |

7) 共通輸出與互斥規則摘要
-------------------------

| 規則 | 說明 |
| --- | --- |
| 輸出格式互斥 | 多數 list/import 使用了 `Mutually exclusive` 群組，`--table`、`--csv`、`--json`、`--output-format` 多數不能同時使用 |
| legacy 命令 | 某些命令可用舊名稱（如 dashboard export-dashboard、alert list-alert-rules）但建議用現代 `dashboard ...` 或 `alert ...` |
| dry-run 優先 | 含 `--dry-run` 的命令，先預覽再實際寫入 |
| 認證策略差異 | dashboard 跨 org 相關（`org-id`、`all-orgs`）多數需 Basic；token 常用於目前上下文 |
| group alias | `access group` 等同於 `access team` |

8) 常見情境快速對照
------------------

### 8.1 跨環境 dashboard 遷移
1. `dashboard export --all-orgs` 或固定 `--org-id`  
2. `dashboard import --dry-run --replace-existing --table`  
3. 確認無誤後再移除 `--dry-run`

### 8.2 只做稽核，不改動
1. 全部 `diff` 或 `inspect-export` / `inspect-live`  
2. list 使用 `--json` 匯出做差異比對  
3. 僅加 `--dry-run` 或 `--help` 確認流程

### 8.3 使用者權限整理
1. `access user list --scope global --table` 做現況盤點  
2. `access user modify` 調整 role / admin  
3. `access team modify` 調整成員  
4. `access service-account` 管理機器人 token

### 8.4 參數變體選擇原則
1. 需要穩定機器人輸出 -> `--json`  
2. 需人工閱讀 -> `--table` + 適當 `--no-header`  
3. 需匯入前檢查 -> 對 import/diff 類命令加 `--dry-run`  
4. 擔心跨 org 混淆 -> `--org-id` + `--require-matching-export-org`

你直接可以使用以下兩段：

9) 每命令 SOP 模板（最短可跑版本）
------------------------------------

每行可直接貼到腳本，僅替換參數值即可：

python3 -m grafana_utils dashboard export --url <URL> --basic-user <USER> --basic-password <PASS> --export-dir <DIR> [--overwrite] [--all-orgs]
python3 -m grafana_utils dashboard export --url <URL> --token <TOKEN> --org-id <ORG_ID> --export-dir <DIR> [--overwrite]
python3 -m grafana_utils dashboard list --url <URL> --basic-user <USER> --basic-password <PASS> [--org-id <ORG_ID>|--all-orgs] [--table|--csv|--json|--output-format table|csv|json] [--with-sources]
python3 -m grafana_utils dashboard list-data-sources --url <URL> --basic-user <USER> --basic-password <PASS> [--table|--csv|--json|--output-format table|csv|json]
python3 -m grafana_utils dashboard import --url <URL> --basic-user <USER> --basic-password <PASS> --import-dir <DIR>/raw --replace-existing [--dry-run] [--table|--json|--output-format text|table|json] [--output-columns uid,destination,action,folder_path,destination_folder_path,file]
python3 -m grafana_utils dashboard diff --url <URL> --basic-user <USER> --basic-password <PASS> --import-dir <DIR>/raw [--import-folder-uid <UID>] [--context-lines 3]
python3 -m grafana_utils dashboard inspect-export --import-dir <DIR>/raw --output-format report-tree --report-filter-panel-id <PANEL_ID>
python3 -m grafana_utils dashboard inspect-live --url <URL> --basic-user <USER> --basic-password <PASS> --output-format report-json

python3 -m grafana_utils alert export --url <URL> --basic-user <USER> --basic-password <PASS> --output-dir <DIR> [--flat] [--overwrite]
python3 -m grafana_utils alert import --url <URL> --basic-user <USER> --basic-password <PASS> --import-dir <DIR>/raw --replace-existing [--dry-run] [--dashboard-uid-map <FILE>] [--panel-id-map <FILE>]
python3 -m grafana_utils alert diff --url <URL> --basic-user <USER> --basic-password <PASS> --diff-dir <DIR>/raw [--dashboard-uid-map <FILE>] [--panel-id-map <FILE>]
python3 -m grafana_utils alert list-rules --url <URL> --basic-user <USER> --basic-password <PASS> [--table|--csv|--json|--output-format table|csv|json]
python3 -m grafana_utils alert list-contact-points --url <URL> --basic-user <USER> --basic-password <PASS> [--table|--csv|--json]
python3 -m grafana_utils alert list-mute-timings --url <URL> --basic-user <USER> --basic-password <PASS> [--table|--csv|--json]
python3 -m grafana_utils alert list-templates --url <URL> --basic-user <USER> --basic-password <PASS> [--table|--csv|--json]

python3 -m grafana_utils datasource list --url <URL> --token <TOKEN> [--table|--csv|--json|--output-format table|csv|json]
python3 -m grafana_utils datasource export --url <URL> --basic-user <USER> --basic-password <PASS> --export-dir <DIR> [--overwrite] [--dry-run]
python3 -m grafana_utils datasource import --url <URL> --basic-user <USER> --basic-password <PASS> --import-dir <DIR> --replace-existing [--dry-run] [--output-format table|text|json] [--output-columns uid,name,type,destination,action,org_id,file]
python3 -m grafana_utils datasource diff --url <URL> --basic-user <USER> --basic-password <PASS> --diff-dir <DIR>

python3 -m grafana_utils access user list --url <URL> --token <TOKEN> --scope org [--table|--csv|--json|--output-format text|table|csv|json]
python3 -m grafana_utils access user add --url <URL> --basic-user <USER> --basic-password <PASS> --login <LOGIN> --email <EMAIL> --name <NAME> --password <PWD> [--org-role Editor] [--grafana-admin true|false] [--json]
python3 -m grafana_utils access user modify --url <URL> --basic-user <USER> --basic-password <PASS> --login <LOGIN> --set-email <EMAIL> [--set-name <NAME>] [--set-org-role Viewer|Editor|Admin|None] [--set-grafana-admin true|false] [--json]
python3 -m grafana_utils access user delete --url <URL> --basic-user <USER> --basic-password <PASS> --login <LOGIN> --scope global --yes [--json]
python3 -m grafana_utils access team list --url <URL> --token <TOKEN> [--query <QUERY>|--name <NAME>] [--with-members] [--table|--csv|--json]
python3 -m grafana_utils access team add --url <URL> --token <TOKEN> --name <NAME> [--email <EMAIL>] [--member <LOGIN_OR_EMAIL>] [--admin <LOGIN_OR_EMAIL>]
python3 -m grafana_utils access team modify --url <URL> --token <TOKEN> --name <NAME> [--add-member <LOGIN_OR_EMAIL>] [--remove-member <LOGIN_OR_EMAIL>] [--add-admin <LOGIN_OR_EMAIL>] [--remove-admin <LOGIN_OR_EMAIL>] [--json]
python3 -m grafana_utils access team delete --url <URL> --token <TOKEN> --name <NAME> --yes [--json]
python3 -m grafana_utils access service-account list --url <URL> --token <TOKEN> [--query <QUERY>] [--table|--csv|--json]
python3 -m grafana_utils access service-account add --url <URL> --token <TOKEN> --name <NAME> [--role Viewer|Editor|Admin|None] [--disabled true|false] [--json]
python3 -m grafana_utils access service-account delete --url <URL> --token <TOKEN> --name <NAME> --yes [--json]
python3 -m grafana_utils access service-account token add --url <URL> --token <TOKEN> --name <SA_NAME> --token-name <TOKEN_NAME> [--seconds-to-live <SECONDS>] [--json]
python3 -m grafana_utils access service-account token delete --url <URL> --token <TOKEN> --name <SA_NAME> --token-name <TOKEN_NAME> --yes [--json]

10) 參數互斥與差異矩陣
----------------------

`OUTPUT` 類（`--output-format` 與 `--table/--csv/--json` 的互斥關係）：

| 命令 | `--output-format` 允許值 | `--table/--csv/--json` 同時可用 | 備註 |
| --- | --- | --- | --- |
| dashboard list | table/csv/json | 不可 | output-format 取代三旗標 |
| dashboard list-data-sources | table/csv/json | 不可 | 同上 |
| dashboard import | text/table/json | 僅 `--table`/`--json` 與輸出式互斥 | text 僅 dry-run 表示摘要 |
| alert list-* | table/csv/json | 不可 | list 共通 |
| datasource list | table/csv/json | 不可 | 同上 |
| datasource import | text/table/json | 僅 `--table`/`--json` 與輸出式互斥 | text 僅 dry-run |
| access user list | text/table/csv/json | 不可 | access list 共通 |
| access team list | text/table/csv/json | 不可 | access list 共通 |
| access service-account list | text/table/csv/json | 不可 | access list 共通 |

`DRY-RUN` 類：

| 命令 | `--dry-run` 影響 |
| --- | --- |
| dashboard import | 僅預覽 create/update/skip，不寫入 |
| datasource import | 僅預覽 create/update/skip，不寫入 |
| alert import | 僅預覽，不寫入 |

`ORG` 控制：

| 命令 | `--org-id` | `--all-orgs` |
| --- | --- | --- |
| dashboard list | 可用（需 Basic） | 可用（需 Basic） |
| dashboard export | 可用（需 Basic） | 可用（需 Basic） |
| dashboard import | 可用（需 Basic） | 不可 |
| datasource import | 可用（需 Basic） | 不可 |
| datasource list/export | 不在 parser 暴露 | 不在 parser 暴露 |
| alert 全部 | 無 | 無 |
| access 全部 | 依 `--scope`/context | 無 |
