# Dashboard 維運人員手冊

這一章是給要管 dashboard 生命週期的人。重點不是只會匯出或匯入，而是先看懂現況、知道變更會影響什麼，再決定要不要回放或套用。

## 適用對象

- 需要盤點、搬移、審查或截圖 dashboard 的 SRE / 平台工程師
- 想把 dashboard 放進 Git、review 或 CI 流程的人
- 需要先看 live 現況，再決定要不要匯出、匯入或刪除的人

## 主要目標

- 先看懂 live dashboard 長什麼樣
- 再確認匯出樹、依賴與變數是不是合理
- 最後才進入匯入、發佈或刪除

## 採用前後對照

- 以前：dashboard 工作常常是從零散 UI 點選、脆弱 JSON 處理、或看不清楚的依賴開始。
- 現在：先盤點、再 inspect、再 diff，最後才 replay 或發佈，順序更清楚。

## 成功判準

- 你知道目前這個任務應該走資產盤點、單一 dashboard authoring、export/import replay、summary、dependencies、policy 還是 screenshot。
- 你能在動 live state 前先說清楚自己走的是哪條 lane。
- 你能證明 dashboard 已經適合 replay 或 publish，而不是只是「看起來差不多」。

## 失敗時先檢查

- 如果匯出樹不完整，先修 source path，再談 replay。
- 如果 inspect 顯示查詢或變數缺失，先處理這些問題，再進入匯入流程。
- 如果你說不出 screenshot 或 dependencies 在證明什麼，可能是開錯工作流了。

## Dashboard 工作流地圖

Dashboard 子命令不是一串平行功能，而是圍繞幾種不同資料來源與結果來設計：

| 任務 | 起點 | 主要輸入 | 主要輸出 | 下一步 |
| --- | --- | --- | --- | --- |
| 找 dashboard | `browse`, `list`, `get` | live Grafana | UID、folder、title、JSON | clone、export 或 review |
| 建立本地草稿 | `clone`, `get`, `patch`, `serve` | live dashboard 或本地 JSON | 可 review 的草稿檔 | `review`, `publish` |
| 發佈草稿 | `review`, `publish`, `edit-live` | 本地 dashboard JSON | dry-run / publish 結果 | apply 後再 list / get |
| 備份與回放 | `export`, `import`, `diff` | live Grafana 或 export tree | `raw/`, `prompt/`, `provisioning/` | diff / dry-run / import |
| 分析依賴 | `summary`, `dependencies`, `variables`, `policy` | live 或本地 dashboard | query、data source、變數、policy 摘要 | 修 dashboard 或 data source |
| 留存證據 | `screenshot` | live dashboard URL / UID | dashboard 或 panel 截圖 | 放進 incident、PR 或文件 |
| 歷史恢復 | `history list`, `history export`, `history restore` | dashboard UID 與版本 | 歷史清單、版本成品或新 revision | review 後 restore |

如果你手上只有 dashboard UID，通常先 `get` 或 `clone`。如果你手上是一整包 export tree，先 `summary` / `diff`，不要直接 `import`。如果你要的是人可以看懂的交付物，`prompt/` 與 screenshot 比 `raw/` 更適合；如果你要的是可重播與可審計，`raw/` 才是主資料路徑。

## 草稿 authoring 工作流

當這次工作不是從整棵 export tree 開始，而是圍繞一份 dashboard 檔案反覆修改時，請直接走 authoring 這條路。

- 如果 Grafana 裡已經有最接近的來源，先用 `dashboard get` 或 `dashboard clone` 取回草稿。
- 如果你要一邊編修一邊在瀏覽器看本地草稿內容，先用 `dashboard serve` 開一個輕量 preview server。
- 在任何 mutation 前先跑 `dashboard review`，確認 title、UID、tags、folder UID 與阻擋性驗證問題。
- 要重寫本地草稿內容時，用 `dashboard patch` 原地修改或輸出成新檔。
- 如果你想從 live dashboard 開始直接編修，但又不想預設就回寫 Grafana，請用 `dashboard edit-live`；它會先給你 review 摘要和阻擋性驗證結果，再決定能不能回寫。
- 草稿準備好後，用 `dashboard publish` 走和正式 import 同一條 replay pipeline。

如果你的團隊是用 Jsonnet、grafanalib 或其他 generator 產 dashboard，不必每次都先落一個中繼暫存檔才能 review 或 publish。

```bash
# 從標準輸入檢視一份生成儀表板。
jsonnet dashboards/cpu.jsonnet | grafana-util dashboard review --input - --output-format json
```

```bash
# 從標準輸入發佈一份生成儀表板。
jsonnet dashboards/cpu.jsonnet | grafana-util dashboard publish --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --input - --replace-existing
```

如果你是在本地反覆編修同一份草稿，請改用檔案路徑搭配 `dashboard publish --watch`，不要用 `--input -`。watch 模式會在每次儲存穩定後重跑 publish 或 dry-run，而且就算其中一次驗證或 API 呼叫失敗，也會繼續監看後續變更。

```bash
# 編修本地草稿時，每次儲存後自動重跑 dry-run publish。
grafana-util dashboard publish --url http://localhost:3000 --basic-user admin --basic-password admin --input ./drafts/cpu-main.json --dry-run --watch
```

```bash
# 在本地瀏覽器裡持續檢視一份 dashboard 草稿。
grafana-util dashboard serve --input ./drafts/cpu-main.json --port 18080 --open-browser
```

```bash
# 從 live dashboard 拉進 editor，但預設仍先落成新的本地草稿。
grafana-util dashboard edit-live --profile prod --dashboard-uid cpu-main --output ./drafts/cpu-main.edited.json
```

`dashboard patch --input -` 必須搭配 `--output`，因為標準輸入不能原地覆寫。
如果目標是 Grafana 內建的 General folder，`dashboard publish` 會把它正規化回預設 root publish 路徑，不會硬送出字面上的 `general` folder UID。
`dashboard serve` 的定位是輕量 preview 與文件檢視，不是把完整 Grafana renderer 內嵌到本地 server。

## 歷史與還原工作流

當您要找回一個已知可用的 dashboard 版本時，請直接走 history 這條路，不要手動重建 JSON。

- [dashboard history](../../commands/zh-TW/dashboard-history.md)
- `dashboard history list`：列出單一 dashboard UID 的最近版本歷史。
- `dashboard history restore`：把某個歷史版本複製成新的最新 Grafana revision。
- `dashboard history export`：把版本歷史匯出成可重用的成品，方便審查或 CI。

還原不是破壞性覆蓋。您選到的舊版本會繼續保留在 history 裡，而還原出來的副本會變成新的最新版本。

> **維運優先設計**：本工具把 Dashboard 視為版本控制資產。目標是讓搬移、比對與審查流程更安全，並在變更碰到即時環境前先看清楚差異與影響。

## 核心工作流用途

Dashboard 相關功能主要是為了處理大規模維運而設計：
- **資產盤點**：了解跨一個或多個組織的 Dashboard 現況。
- **結構化匯出**：用分開的資料路徑在環境間遷移 Dashboard。
- **深度檢視**：離線分析查詢 (Queries) 與 data source 依賴。
- **截圖與視覺檢查**：產出可重現的 dashboard 或 panel 截圖，用於文件、事件處理紀錄與除錯。
- **差異審查 (Drift Review)**：在套用變更前，比對本地暫存檔案與 live Grafana。
- **受控變更**：透過強制性的 Dry-run 執行匯入或刪除。

---

## 🔎 檢視與截圖工作流

如果你的目標不是匯入或匯出，而是先看清楚 dashboard 目前長什麼樣、依賴哪些 data source、變數怎麼帶入，這一組指令應該先看。

- `dashboard summary（即時）`：直接分析 live dashboard 的結構、查詢與依賴。
- `dashboard summary（本地）`：離線分析已匯出的 dashboard 樹。
- `dashboard variables`：確認變數、data source 選項與 URL 帶入值。
- `dashboard screenshot`：用 headless browser 產生可重現的 dashboard 或 panel 截圖。
- `dashboard dependencies`：快速掌握 dashboard 與上游依賴之間的關係。

常見情境：

- 事件處理後需要補一張當下畫面的截圖
- 想先確認某個 panel 是不是吃到正確的變數與 data source
- 要整理文件或 review 附圖，但不想手動截圖
- 想在變更前先看 dashboard 依賴與查詢結構

---

## 工作流程邊界（三條資料路徑）

Dashboard 匯出會刻意產生三種不同的資料路徑。這不是單純的目錄分類，而是把三種工作流在一開始就切開：一條給自動化回放，一條給人工 UI 匯入，一條給 Grafana 從磁碟讀取。當事故後要留證、變更前要 review、或要把 dashboard 交給另一個環境時，先選對路徑，比事後修一份錯用的 JSON 便宜很多。

`raw/` 是最值得保留的原始紀錄。它接近 API 回傳的樣子，也最適合成為 Git 裡可追溯的來源。要做備份、災難恢復、diff、audit、dry-run import，或準備之後用 CLI 推回 Grafana，從這裡開始。

`prompt/` 是給人操作 UI 的交接稿。它服務的是 Grafana UI 的 "Upload JSON" 流程，而不是 `grafana-util dashboard import`。當你要把 dashboard 交給另一個團隊、另一個 org，或只是希望對方用瀏覽器匯入，這條路徑才是比較乾淨的交付物。

`provisioning/` 是部署投影。它讓 Grafana 透過 provisioning 設定從檔案系統讀 dashboard，適合被放進容器映像、ConfigMap、volume mount 或 GitOps-style 部署流程。這條路徑可以部署，但不適合作為日常 review 與回放的唯一真相來源。

| 路徑 (Lane) | 用途 | 最佳使用場景 |
| :--- | :--- | :--- |
| `raw/` | **標準回放 (Replay)** | `grafana-util dashboard import` 的主要來源。可還原且 API 友善。 |
| `prompt/` | **UI 匯入** | 與 Grafana UI 內建的 "Upload JSON" 功能相容。若您手上只有一般或 raw 的 dashboard JSON，請先用 `grafana-util dashboard convert raw-to-prompt` 轉換。 |
| `provisioning/` | **檔案配置** | 供 Grafana 透過其內建配置系統從磁碟讀取 Dashboard。 |

如果您對 `dashboard export` 加上 `--include-history`，匯出樹就會在每個 org 範圍下多出一個 `history/` 子目錄。當您使用 `--all-orgs` 時，每個匯出的 org root 都會各自有一份 history 樹。

當您需要的是單一 dashboard UID 的獨立 JSON 成品時，請直接用 `dashboard history export`。當您想讓版本歷史跟著 export tree 一起輸出時，請用 `dashboard export --include-history`。

---

## Prompt 與變數說明

- `$datasource` 是 dashboard variable 參照。
- `${DS_*}` 是由 `__inputs` 產生的 external import placeholder。
- 一份 prompt dashboard 同時出現這兩種寫法是合理的。
- 這通常代表 dashboard 一邊保留 Grafana datasource variable 的工作流，一邊也需要 external import input。
- 不要把 `$datasource` 直接解讀成 mixed datasource family；很多情況只是 panel 仍透過同一個 datasource variable 做選擇。

---

## 暫存 vs 即時：維運邏輯

- **暫存工作 (Staged)**：本地匯出樹、驗證、離線檢視與 Dry-run 審查。
- **即時工作 (Live)**：直接對接 Grafana 的盤點、即時 Diff、匯入與刪除。

**黃金守則**：先用 `list` 或 `browse` 發現資產，`export` 到暫存目錄，透過 `dashboard dependencies` 與 `diff` 驗證，最後在 Dry-run 符合預期後才執行 `import` 或 `delete`。

---

## 閱讀即時資產盤點

使用 `dashboard list` 快速取得資產全貌。

```bash
# 使用 dashboard list 快速取得資產全貌。
grafana-util dashboard list \
  --url http://localhost:3000 \
  --basic-user admin \
  --basic-password admin \
  --table
```

**範例輸出：**
```text
UID                      NAME                                      FOLDER  FOLDER_UID      FOLDER_PATH  ORG        ORG_ID
-----------------------  ----------------------------------------  ------  --------------  -----------  ---------  ------
rYdddlPWl                Node Exporter Full for Host               Demo    ffhrmit0usjk0b  Demo         Main Org.  1
spring-jmx-node-unified  Spring JMX + Node Unified Dashboard (VM)  Demo    ffhrmit0usjk0b  Demo         Main Org.  1
```

**如何解讀：**
- **UID**：用於自動化與刪除的穩定身份識別。
- **FOLDER_PATH**：Dashboard 所屬的目錄路徑。
- **ORG/ORG_ID**：確認該物件隸屬於哪個組織。

---

## 盤點與取得：先確認你要改哪一份

Dashboard 工作常常從一個模糊線索開始：某個畫面、某個 folder、某個 team 名稱，或 incident 裡留下的一張截圖。這時候不要先 export 整包，也不要直接 publish。先把目標 dashboard 的 UID、folder、org 與 title 找出來。

`dashboard browse` 適合用 folder 視角找資產；`dashboard list` 適合拿到可排序、可過濾的 inventory；`dashboard get` 適合確認單一 UID 的完整 JSON。找到目標後，下一步才決定要 `clone` 成草稿、`export` 進回放流程，或用 `summary` / `dependencies` 做影響分析。

如果 `list` 或 `browse` 少了你預期的 dashboard，先查 profile、org scope、folder 權限與 token 能見度。空結果不等於 live Grafana 沒有該 dashboard。

## 匯出模式怎麼選

`dashboard export` 不是只產生「一份 JSON」。它同時服務三種工作：CLI 回放、UI 匯入、Grafana provisioning。選錯 lane 會讓後續 review 變得很痛苦。

- `raw/`：給 `dashboard import`、`dashboard diff`、audit 與災難恢復。這是最適合放進 Git 的主資料。
- `prompt/`：給人拿去 Grafana UI Upload JSON。這是交接稿，不是 CLI 回放主資料。
- `provisioning/`：給 Grafana 從磁碟讀取。這是部署投影，不應該取代 raw review。

如果你只是要把一個 dashboard 交給另一個人手動匯入，`prompt/` 會比 `raw/` 好懂。如果你要讓 CI 比對、dry-run 或回放，請回到 `raw/`。如果你要放到 GitOps 或容器 volume，才使用 `provisioning/`。

## Review / Publish：草稿要先能被解釋

`dashboard review` 和 `dashboard publish` 的關係，是「先確認草稿是否值得送出，再決定是否寫入 live」。Review 時至少要看 title、UID、folder UID、tags、data source 參照、變數與 blocking issue。Publish 前如果你說不出這份 JSON 從哪裡來、要覆蓋哪個 UID、會進哪個 folder，就不應該 apply。

`dashboard publish --dry-run` 適合放在 PR 或人工變更前；`--watch` 適合本地反覆改同一份草稿。真正 publish 後，再回到 `dashboard get` 或 `dashboard list` 驗證 live state，而不是只相信 publish 的終端輸出。

## 分析與證據：不只是看起來有沒有變

`dashboard summary`、`dependencies`、`variables`、`policy` 與 `screenshot` 不是附屬功能。它們回答的是 review 裡常見的問題：這個 dashboard 查哪些 data source、變數是否會改變查詢結果、policy 是否允許進 production、以及畫面證據能不能放進 incident 或 PR。

當 review 的目標是「證明這次變更不會破壞查詢或變數」，先用 `summary`、`variables`、`dependencies`。當目標是「留下一張人能看懂的證據」，才用 `screenshot`。截圖不能取代結構檢查；結構檢查也不能取代需要交給人的視覺證據。

## 何時切到指令參考

這一章負責幫你決定工作流。當你已經知道要使用哪個 command，再切到指令參考確認 flags、輸出格式與完整範例：

- [dashboard 指令總覽](../../commands/zh-TW/dashboard.md)
- [dashboard browse](../../commands/zh-TW/dashboard-browse.md)
- [dashboard get](../../commands/zh-TW/dashboard-get.md)
- [dashboard clone](../../commands/zh-TW/dashboard-clone.md)
- [dashboard list](../../commands/zh-TW/dashboard-list.md)
- [dashboard export](../../commands/zh-TW/dashboard-export.md)
- [dashboard import](../../commands/zh-TW/dashboard-import.md)
- [dashboard convert raw-to-prompt](../../commands/zh-TW/dashboard-convert-raw-to-prompt.md)
- [dashboard patch](../../commands/zh-TW/dashboard-patch.md)
- [dashboard serve](../../commands/zh-TW/dashboard-serve.md)
- [dashboard edit-live](../../commands/zh-TW/dashboard-edit-live.md)
- [dashboard review](../../commands/zh-TW/dashboard-review.md)
- [dashboard publish](../../commands/zh-TW/dashboard-publish.md)
- [dashboard delete](../../commands/zh-TW/dashboard-delete.md)
- [dashboard diff](../../commands/zh-TW/dashboard-diff.md)
- [dashboard summary](../../commands/zh-TW/dashboard-summary.md)
- [dashboard variables](../../commands/zh-TW/dashboard-variables.md)
- [dashboard history](../../commands/zh-TW/dashboard-history.md)
- [dashboard policy](../../commands/zh-TW/dashboard-policy.md)
- [dashboard dependencies](../../commands/zh-TW/dashboard-dependencies.md)
- [dashboard screenshot](../../commands/zh-TW/dashboard-screenshot.md)
- [指令參考](../../commands/zh-TW/index.md)

---

## 常用指令

| 指令 | 帶有參數的完整範例 |
| :--- | :--- |
| **盤點 (List)** | `grafana-util dashboard list --all-orgs --with-sources --table` |
| **匯出 (Export)** | `grafana-util dashboard export --output-dir ./dashboards --overwrite --progress` |
| **匯出 + 歷史** | `grafana-util dashboard export --output-dir ./dashboards --include-history --overwrite --progress` |
| **Raw 轉 Prompt** | `grafana-util dashboard convert raw-to-prompt --input-dir ./dashboards/raw --output-dir ./dashboards/prompt --overwrite --progress` |
| **匯入 (Import)** | `grafana-util dashboard import --input-dir ./dashboards/raw --replace-existing --dry-run --table` |
| **比對 (Diff)** | `grafana-util dashboard diff --input-dir ./dashboards/raw --input-format raw` |
| **相依性 (Dependencies)** | `grafana-util dashboard dependencies --input-dir ./dashboards/raw --input-format raw --output-format text` |
| **刪除 (Delete)** | `grafana-util dashboard delete --uid <UID> --url <URL> --basic-user admin --basic-password admin` |
| **變數檢視 (Variables)** | `grafana-util dashboard variables --uid <UID> --url <URL> --table` |
| **檔案修正 (Patch)** | `grafana-util dashboard patch --input <FILE> --name "New Title" --output <FILE>` |
| **發佈 (Publish)** | `grafana-util dashboard publish --input <FILE> --url <URL> --basic-user admin --basic-password admin` |
| **複製 (Clone)** | `grafana-util dashboard clone --source-uid <UID> --output <FILE> --url <URL>` |

---

## 操作範例

### 1. 匯出進度 (Export Progress)
在大規模匯出時使用 `--progress` 以取得簡潔的日誌。
```bash
# 在大規模匯出時使用 --progress 以取得簡潔的日誌。
grafana-util dashboard export --output-dir ./dashboards --overwrite --progress
```
**範例輸出：**
```text
Exporting dashboard 1/7: mixed-query-smoke
Exporting dashboard 2/7: smoke-prom-only
...
Exporting dashboard 7/7: two-prom-query-smoke
```

### 2. Dry-Run 匯入預覽
在變更前務必確認目標動作。
```bash
# 在變更前務必確認目標動作。
grafana-util dashboard import --input-dir ./dashboards/raw --dry-run --table
```
**範例輸出：**
```text
UID                    DESTINATION  ACTION  FOLDER_PATH                    FILE
---------------------  -----------  ------  -----------------------------  --------------------------------------
mixed-query-smoke      exists       update  General                        ./dashboards/raw/Mixed_Query_Dashboard.json
subfolder-chain-smoke  missing      create  Platform / Team / Apps / Prod  ./dashboards/raw/Subfolder_Chain.json
```
- **ACTION=create**：將新增 Dashboard。
- **ACTION=update**：將取代現有的 live Dashboard。
- **DESTINATION=missing**：Grafana 目前沒有這個 UID，因此匯入時會建立新紀錄。
- **DESTINATION=exists**：Grafana 目前已經有這個 UID，因此匯入時會對應到既有 Dashboard。

### 3. Provisioning 比對
比對本地配置檔案與實例現況。
```bash
# 比對本地配置檔案與實例現況。
grafana-util dashboard diff --input-dir ./dashboards/provisioning --input-format provisioning
```
**範例輸出：**
```text
--- live/cpu-main
+++ export/cpu-main
-  "title": "CPU Overview"
+  "title": "CPU Overview v2"
```

---
[⬅️ 上一章：系統架構與設計原則](architecture.md) | [🏠 回首頁](index.md) | [➡️ 下一章：Data source 管理](datasource.md)
