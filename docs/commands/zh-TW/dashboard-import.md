# dashboard import

## 用途
透過 Grafana API 匯入儀表板 JSON 檔案。

## 何時使用
當您手上有屬於 Grafana API 狀態管理的本地匯出樹，想把儀表板推回 Grafana，無論是實際執行或 dry run，都可以使用這個指令。這個指令只吃 `raw/` 或 `provisioning/` 輸入，不吃 Grafana UI 的 `prompt/` 路徑。

不要用這個指令繞過來源管理。如果目標 dashboard 是 Grafana Git Sync 或 file provisioning 管理，應該修改 Git repository、PR 或 provisioning source，再由該流程部署，而不是強制走 dashboard API 匯入。

## 採用前後對照
- **採用前**：匯入比較像盲目 replay，folder、org 或 schema 問題往往要打到 live 後才知道。
- **採用後**：匯入會先變成可 preview 的回放步驟，先用 `--dry-run` 看清楚，再決定是否真的動 live。

## 重點旗標
- `--input-dir`：原始或合併匯出輸入的來源目錄。
- `--input-format`：選擇 `raw` 或 `provisioning`。
- `--local`、`--run`、`--run-id`：不傳 `--input-dir`，改從 artifact workspace 的 dashboard lane 讀取來源。
- `--org-id`、`--use-export-org`、`--only-org-id`、`--create-missing-orgs`：控制跨 org 路由。
- `--import-folder-uid`：強制指定目的資料夾 UID。
- `--ensure-folders`、`--replace-existing`、`--update-existing-only`：控制匯入行為。
- `--require-matching-folder-path`、`--require-matching-export-org`、`--strict-schema`、`--target-schema-version`：安全檢查。
- `--import-message`：儲存在 Grafana 的修訂訊息。
- `--interactive`、`--dry-run`、`--table`、`--json`、`--output-format`、`--output-columns`、`--list-columns`、`--no-header`、`--progress`、`--verbose`：預覽與回報控制。若想看完整 dry-run 表格欄位，可用 `--output-columns all`。

## 成功判準
- dry-run 先把 create/update 動作列清楚，再進入 live replay
- dry-run 也會顯示目標 UID 的 live dashboard ownership evidence，包含 provisioned 或 managed-state 警告
- 目的 org 與 folder 路由足夠明確，可以先 review
- 這次匯入使用的是正確的輸入 lane：`raw` 或 `provisioning`，不是 `prompt`
- Git Sync 或 file-provisioned targets 會被視為 source-owned，應回到 repository 或 provisioning workflow 處理

## 失敗時先檢查
- 如果 folder 或 org 落點不對，先檢查路由旗標，不要直接重跑 live import
- 如果看起來會刪或覆蓋太多，先停在 `--dry-run` 並回頭檢查匯出樹
- 如果 Grafana 回報目標 UID 是 provisioned 或 Git Sync-managed dashboard，不要改用直接 import 重試；請更新它的來源並透過原本 lane 重新部署
- 如果 schema 被擋下來，先確認來源資料是不是需要先正規化再匯入

## 範例
```bash
# 透過 Grafana API 匯入儀表板 JSON 檔案。
grafana-util dashboard import --profile prod --input-dir ./dashboards/raw --replace-existing
```

```bash
# 透過 Grafana API 匯入儀表板 JSON 檔案。
grafana-util dashboard import --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw --dry-run --table
```

```bash
# 透過 Grafana API 匯入儀表板 JSON 檔案。
grafana-util dashboard import --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --input-dir ./dashboards/raw --dry-run --table
```

```bash
# 從最新的 profile artifact workspace dashboard run 做 dry-run import。
grafana-util dashboard import --profile prod --local --dry-run --table
```

## 相關指令
- [dashboard export](./dashboard-export.md)
- [dashboard convert raw-to-prompt](./dashboard-convert-raw-to-prompt.md)
- [dashboard diff](./dashboard-diff.md)
- [dashboard publish](./dashboard-publish.md)
