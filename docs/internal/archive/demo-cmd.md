# Demo Commands

## 用途

這份清單是給現場 demo 用的，重點不是把所有細節都講完，而是快速展示 `grafana-util` 目前已經能怎麼處理 Grafana 維運常見痛點：

- dashboard 批次盤點、匯出、分析、匯入 dry-run
- datasource 盤點與互動式瀏覽
- alerting 資源匯出與遷移準備
- access inventory 盤點
- project / staged artifact 的整體彙總
- TUI / interactive 檢視能力，讓現場展示更直觀

以下命令預設 Grafana 在 `http://localhost:43000`，並以 `admin / admin` 的 Basic auth 作為現場展示預設。

## Demo 前準備

```bash
grafana-util --help
```

如果目前是在 repo 內直接跑、還沒安裝 binary，也可以把下面每條命令改成這個前綴：

```bash
cargo run --manifest-path rust/Cargo.toml --bin grafana-util -- ...
```

## Env 補充

可以用 env 簡化重複的登入資訊：

```bash
export GRAFANA_URL=http://localhost:43000
export GRAFANA_USERNAME=admin
export GRAFANA_PASSWORD=admin
```

## 認證與 All Orgs

- 主流程以 Basic auth 為主，範例使用 `admin / admin`。
- `--all-orgs` 用來展示跨 org 的盤點、瀏覽、匯出與檢視能力。
- API token 保留在補充區，主要對應單一 org 或自動化情境。
- 支援跨 org 的命令，範例分成 `single org` 與 `all-orgs` 兩種。

## 主命令清單

| 模組 | 主要用途 | 代表子命令 |
| --- | --- | --- |
| `dashboard` | dashboard 盤點、匯出、匯入、差異比對、分析、TUI 瀏覽 | `browse`, `list`, `export`, `import`, `diff`, `inspect-export`, `inspect-live`, `inspect-vars`, `governance-gate`, `topology`, `screenshot` |
| `datasource` | datasource 盤點、瀏覽、異動、匯出匯入、差異比對 | `types`, `list`, `browse`, `add`, `modify`, `delete`, `export`, `import`, `diff` |
| `alert` | alerting 資源匯出、匯入、差異比對與清單盤點 | `export`, `import`, `diff`, `list-rules`, `list-contact-points`, `list-mute-timings`, `list-templates` |
| `access` | user / org / team / service-account 盤點與管理 | `user`, `org`, `team`, `service-account` |
| `sync` | staged sync、plan、review-first 工作流 | `plan`, `apply` 等 |
| `overview` | 對 staged artifacts 做整體彙總 | `overview`, `overview live` |
| `project-status` | 專案級 staged / live 狀態檢視 | `staged`, `live` |

## 快速列出主命令

用途：先快速讓現場看到整個 CLI 的外層模組。

```bash
grafana-util --help
```

用途：如果要逐個模組快速看支援面。

```bash
grafana-util dashboard --help
grafana-util datasource --help
grafana-util alert --help
grafana-util access --help
grafana-util sync --help
grafana-util overview --help
grafana-util project-status --help
```

## 推薦 Demo Flow

### TUI 優先展示

如果現場希望先用比較好閱讀的方式帶過整體能力，建議先跑這兩個互動式命令：

dashboard browse 跨 org 展示版：

```bash
grafana-util dashboard browse \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs
```

datasource browse 單一 org 展示版：

```bash
grafana-util datasource browse \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin
```

這樣一開始就能先展示：

- 有 live tree / inventory 的可讀性
- 不只是匯出匯入，也有 terminal 內工作台式的操作體驗
- 跨 org 與 datasource 檢視不是只能看原始 JSON
- `dashboard inspect-export --interactive` / `dashboard inspect-live --interactive` 這類模式需要 TUI-capable build

### 1. 互動式瀏覽 dashboard tree

用途：直接展示不是只有匯出匯入，還能在 terminal 內做 live dashboard 導覽。

單一 org：

```bash
grafana-util dashboard browse \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin
```

跨 org：

```bash
grafana-util dashboard browse \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs
```

### 2. 跨 org 列出 dashboard 與 datasource 關聯

用途：展示 dashboard inventory，不只是列 dashboard，還能看 datasource 使用脈絡。

單一 org：

```bash
grafana-util dashboard list \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --with-sources \
  --table
```

跨 org：

```bash
grafana-util dashboard list \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --with-sources \
  --table
```

### 3. 匯出 dashboard

用途：展示批次匯出，不需要進 Grafana UI 一個一個操作。

單一 org：

```bash
grafana-util dashboard export \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --export-dir ./dashboards \
  --overwrite \
  --progress
```

跨 org：

```bash
grafana-util dashboard export \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --export-dir ./dashboards \
  --overwrite \
  --progress
```

### 4. 先看 live dashboard inspect

用途：在真正 import 之前，先直接從 live Grafana 看 query / datasource / governance 面向，適合先做現場檢視。

單一 org：

```bash
grafana-util dashboard inspect-live \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --output-format report-table
```

單一 org 互動式：

```bash
grafana-util dashboard inspect-live \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --interactive
```

跨 org：

```bash
grafana-util dashboard inspect-live \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --output-format report-table
```

跨 org 互動式：

```bash
grafana-util dashboard inspect-live \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --interactive
```

### 5. 分析匯出的 dashboard

用途：展示匯出後不是只有檔案，還能做 query / datasource / governance 面向分析。

單一 org：

```bash
grafana-util dashboard inspect-export \
  --import-dir ./dashboards/raw \
  --output-format report-table
```

單一 org 互動式：

```bash
grafana-util dashboard inspect-export \
  --import-dir ./dashboards/raw \
  --interactive
```

跨 org：

```bash
grafana-util dashboard inspect-export \
  --import-dir ./dashboards \
  --output-format report-table
```

跨 org 互動式：

```bash
grafana-util dashboard inspect-export \
  --import-dir ./dashboards \
  --interactive
```

### 6. 匯入前先做 dry-run

用途：展示 import 是 review-first，不是直接盲改，現場可強調 create / update / skip 預覽。

單一 org：

```bash
grafana-util dashboard import \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --import-dir ./dashboards/raw \
  --replace-existing \
  --dry-run \
  --table
```

跨 org：

```bash
grafana-util dashboard import \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --import-dir ./dashboards \
  --use-export-org \
  --replace-existing \
  --dry-run \
  --table
```

### 7. 互動式瀏覽 datasource

用途：展示 datasource 不只是 list/export，也有 live browse 工作面。

這個展示建議先用單一 org；如果要做跨 org inventory，改用 `datasource list --all-orgs` 會更清楚。

```bash
grafana-util datasource browse \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin
```

跨 org inventory 補充範例：

```bash
grafana-util datasource list \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --table
```

### 8. 匯出 datasource

用途：補齊 staged datasource inventory，供後續 overview / project-level 彙總使用。

單一 org：

```bash
grafana-util datasource export \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --export-dir ./datasources \
  --overwrite
```

跨 org：

```bash
grafana-util datasource export \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --export-dir ./datasources \
  --overwrite
```

### 9. 匯出 alerting 資源

用途：展示 alerting 也能做 bundle 式匯出，補 Grafana UI 搬移不方便的缺口。

目前 alert 的 `--all-orgs` 主要支援在 `list-rules`、`list-contact-points`、`list-mute-timings`、`list-templates` 這類 inventory 命令；`export / import / diff` 目前沒有 `--all-orgs`。

```bash
grafana-util alert export \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --output-dir ./alerts \
  --overwrite
```

跨 org inventory 補充範例：

```bash
grafana-util alert list-rules \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --table
```

### 10. 盤點 access users

用途：展示這不只是一個 dashboard tool，也包含 access inventory。

單一 org：

```bash
grafana-util access user list \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --table
```

跨 org 如果想另外展示，可再補一個 access browse / org list 的命令；這裡先保留最穩定的單一 org inventory 範例。

### 11. 用 staged artifacts 做 overview

用途：展示專案不是只有單點命令，也有整體彙總視角。

目前這段是 staged artifact 彙總，不是 live 查詢命令，所以沒有 `--all-orgs` 旗標；dashboard 輸入目前直接指向一個 `raw/` 目錄。若前面是 `--all-orgs` 匯出，`overview` 目前仍是指向某一個 `org_*` 子目錄，而不是整個 combined root。

`--datasource-export-dir ./datasources` 對應前面的第 8 步 `grafana-util datasource export ... --export-dir ./datasources`。`--alert-export-dir ./alerts` 對應前面的第 9 步 `grafana-util alert export ... --output-dir ./alerts`。如果前面沒有先匯出 datasource 或 alert，也可以先只看 dashboard overview。

前置輸出：

- dashboard single org: `./dashboards/raw`
- dashboard all-orgs: `./dashboards/org_1_Main_Org/raw`
- datasource single org: `./datasources`
- datasource all-orgs: `./datasources/org_1_Main_Org`
- alert export: `./alerts`

單一 org 範例：

```bash
grafana-util overview \
  --dashboard-export-dir ./dashboards/raw \
  --datasource-export-dir ./datasources \
  --alert-export-dir ./alerts \
  --output text
```

單一 org 最小範例：

```bash
grafana-util overview \
  --dashboard-export-dir ./dashboards/raw \
  --output text
```

單一 org 互動式：

```bash
grafana-util overview \
  --dashboard-export-dir ./dashboards/raw \
  --datasource-export-dir ./datasources \
  --alert-export-dir ./alerts \
  --output interactive
```

如果你的 dashboard 匯出是從 `--all-orgs` 產生的多 org 目錄，這一段要改成指向其中一個 org 的 `raw/` 子目錄，例如：

```bash
grafana-util overview \
  --dashboard-export-dir ./dashboards/org_1_Main_Org/raw \
  --datasource-export-dir ./datasources/org_1_Main_Org \
  --alert-export-dir ./alerts \
  --output text
```

multi-org 子目錄互動式：

```bash
grafana-util overview \
  --dashboard-export-dir ./dashboards/org_1_Main_Org/raw \
  --datasource-export-dir ./datasources/org_1_Main_Org \
  --alert-export-dir ./alerts \
  --output interactive
```

### 12. diff / sync / project-level 補充範例

用途：補上差異比對、staged sync、以及專案層互動式檢視。

dashboard diff：

- 先有 single-org dashboard export 輸出，也就是：
  `grafana-util dashboard export --url http://localhost:43000 --basic-user admin --basic-password admin --export-dir ./dashboards --overwrite`
- 這條命令預期路徑是 `./dashboards/raw`

```bash
grafana-util dashboard diff \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --import-dir ./dashboards/raw
```

dashboard diff all-orgs 處理：

- 目前 `dashboard diff` 沒有 `--all-orgs`，也沒有可用的 `--org-id` 切換旗標。
- 目前以單一 org `raw/` 目錄對目前登入 org 做 diff 為主。
- all-orgs workflow 目前建議改用 `dashboard inspect-export --import-dir ./dashboards` 做跨 org 檢視，或用 `dashboard import --use-export-org --dry-run` 做跨 org review。

datasource diff：

- 先有 datasource export 輸出：
  `grafana-util datasource export --url http://localhost:43000 --basic-user admin --basic-password admin --export-dir ./datasources --overwrite`

```bash
grafana-util datasource diff \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --diff-dir ./datasources
```

datasource diff all-orgs 處理：

- 目前沒有直接的 `--all-orgs` diff 旗標。
- 若 datasource export 是 `--all-orgs` 產生的多 org 目錄，diff 目前要逐個 org 目錄處理。

alert diff：

- 先有 alert export 輸出：
  `grafana-util alert export --url http://localhost:43000 --basic-user admin --basic-password admin --output-dir ./alerts --overwrite`
- 這條命令預期路徑是 `./alerts/raw`

```bash
grafana-util alert diff \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --diff-dir ./alerts/raw \
  --json
```

alert diff all-orgs 處理：

- 目前沒有 `--all-orgs`。
- 跨 org 盤點請改用 `alert list-rules --all-orgs` 或其他 `alert list-* --all-orgs` 命令。

sync bundle：

- 先有 staged 輸出：
  - single-org dashboard export: `./dashboards/raw`
  - single-org datasource export: `./datasources/datasources.json`
  - alert export: `./alerts/raw`
- 若前面跑的是 all-orgs dashboard/datasource export，這裡不要直接沿用 single-org 路徑。

```bash
grafana-util sync bundle \
  --dashboard-export-dir ./dashboards/raw \
  --alert-export-dir ./alerts/raw \
  --datasource-export-file ./datasources/datasources.json \
  --output-file ./sync-source-bundle.json \
  --output json
```

sync bundle all-orgs 處理：

```bash
grafana-util sync bundle \
  --dashboard-export-dir ./dashboards/org_1_Main_Org/raw \
  --alert-export-dir ./alerts/raw \
  --datasource-export-file ./datasources/org_1_Main_Org/datasources.json \
  --output-file ./sync-source-bundle-org1.json \
  --output json
```

sync bundle-preflight：

- 先有 `sync bundle` 產生的 `./sync-source-bundle.json`
- 另外還要先準備 `./target-inventory.json`
- 這條屬於進階 staged review 範例，不是前面 export 完就能直接接上的第一條命令

```bash
grafana-util sync bundle-preflight \
  --source-bundle ./sync-source-bundle.json \
  --target-inventory ./target-inventory.json \
  --output json
```

sync plan：

```bash
grafana-util sync plan \
  --desired-file ./desired-plan.json \
  --live-file ./live.json \
  --output json
```

sync review：

```bash
grafana-util sync review \
  --plan-file ./sync-plan.json \
  --review-note "docs-reviewed" \
  --reviewed-by docs-user \
  --output json
```

sync apply：

```bash
grafana-util sync apply \
  --plan-file ./sync-plan-reviewed.json \
  --approve \
  --output json
```

project-status staged 互動式：

```bash
grafana-util project-status staged \
  --dashboard-export-dir ./dashboards/raw \
  --datasource-export-dir ./datasources \
  --alert-export-dir ./alerts \
  --output interactive
```

project-status live all-orgs 互動式：

```bash
grafana-util project-status live \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --output interactive
```

overview live 互動式：

```bash
grafana-util overview live \
  --url http://localhost:43000 \
  --basic-user admin \
  --basic-password admin \
  --all-orgs \
  --output interactive
```

## Token 補充

如果你想額外示範 token 模式，可以用這種寫法，但建議放在補充，不要取代現場主流程：

```bash
export GRAFANA_API_TOKEN=your-token

grafana-util datasource list \
  --url http://localhost:43000 \
  --token "$GRAFANA_API_TOKEN" \
  --json
```

現場主流程仍建議維持 `admin / admin` + `--all-orgs`，比較能把工具價值講清楚。

## 最短版現場展示順序

Single org：

```bash
grafana-util --help
grafana-util dashboard browse --url http://localhost:43000 --basic-user admin --basic-password admin
grafana-util dashboard list --url http://localhost:43000 --basic-user admin --basic-password admin --with-sources --table
grafana-util dashboard inspect-live --url http://localhost:43000 --basic-user admin --basic-password admin --interactive
grafana-util dashboard export --url http://localhost:43000 --basic-user admin --basic-password admin --export-dir ./dashboards --overwrite --progress
grafana-util dashboard inspect-export --import-dir ./dashboards/raw --interactive
grafana-util dashboard import --url http://localhost:43000 --basic-user admin --basic-password admin --import-dir ./dashboards/raw --replace-existing --dry-run --table
```

All orgs：

```bash
grafana-util --help
grafana-util dashboard browse --url http://localhost:43000 --basic-user admin --basic-password admin --all-orgs
grafana-util dashboard list --url http://localhost:43000 --basic-user admin --basic-password admin --all-orgs --with-sources --table
grafana-util dashboard inspect-live --url http://localhost:43000 --basic-user admin --basic-password admin --all-orgs --interactive
grafana-util dashboard export --url http://localhost:43000 --basic-user admin --basic-password admin --all-orgs --export-dir ./dashboards --overwrite --progress
grafana-util dashboard inspect-export --import-dir ./dashboards --interactive
grafana-util dashboard import --url http://localhost:43000 --basic-user admin --basic-password admin --import-dir ./dashboards --use-export-org --replace-existing --dry-run --table
```
