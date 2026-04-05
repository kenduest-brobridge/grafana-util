# dashboard

## 這一頁對應的工作流

| 工作流 | 常用子命令 |
| --- | --- |
| 盤點與瀏覽 live dashboard | `browse`、`list`、`inspect-live` |
| 匯出 / 匯入 / 比對 | `get`、`clone-live`、`export`、`import`、`diff`、`review`、`patch-file`、`publish`、`delete` |
| 變更前檢查 | `inspect-export`、`inspect-vars`、`governance-gate` |
| 拓樸與影響面 | `topology`、`impact` |
| 歷史與還原 | `history list`、`history restore`、`history export` |
| 截圖與素材 | `screenshot`、`raw-to-prompt` |

## 從這裡開始

- 先看現況：`dashboard browse` 或 `dashboard inspect-live`
- 先做草稿：`dashboard get`、`dashboard clone-live`、`dashboard export`
- 先做比對：`dashboard diff`、`dashboard review`
- 先做上線前檢查：`dashboard inspect-export`、`dashboard inspect-vars`、`dashboard governance-gate`
- 先看影響面：`dashboard topology`、`dashboard impact`
- 先處理歷史版本：`dashboard history list`、`dashboard history restore`、`dashboard history export`
- 先拿素材：`dashboard screenshot`、`dashboard raw-to-prompt`

## 說明

`grafana-util dashboard` 把 dashboard 相關工作收在同一個入口：從瀏覽、草稿、匯出、匯入、比對，到拓樸、影響面和截圖。它也可用 `grafana-util db` 呼叫。

如果是單一 dashboard 的 authoring 路徑，建議把它想成：
- `get` 或 `clone-live`：先做草稿
- `review`：先驗證草稿內容
- `patch-file`：改寫本地中繼資料
- `publish`：沿用 import pipeline 發回 Grafana

`review`、`patch-file`、`publish` 也都支援 `--input -`，可以直接吃標準輸入的一份 wrapped 或 bare dashboard JSON。這適合外部 generator 已經把 JSON 寫到 stdout 的情況。`patch-file --input -` 必須搭配 `--output`，若你是在本地反覆編修同一份檔案，則改用 `publish --watch`；它只支援本地檔案路徑，不支援 `--input -`。

## 歷史與還原工作流

如果你的問題不是「現在這份 dashboard 長什麼樣」，而是「哪個舊版本應該被救回來並變成新的最新版本」，就看這一組。

- [dashboard history](./dashboard-history.md)
- `dashboard history list`：列出單一 dashboard UID 的最近版本歷史。
- `dashboard history restore`：把某個歷史版本複製成新的最新 Grafana 版本。
- `dashboard history export`：把歷史版本匯出成可重用的 JSON 成品，方便審查或 CI。

這條路徑最適合要找回已知可用版本，但不想手動重建 dashboard 的情況。

## 採用前後對照

- **採用前**：dashboard 動作常分散在 UI、草稿 JSON 與臨時 shell 指令裡，要回頭重跑很麻煩。
- **採用後**：同一條命令群組就能把瀏覽、草稿、檢查、發佈與素材產生串起來。

## 成功判準

- 你能在開始前就判斷這次是要看 live、做草稿、跑檢查，還是直接發佈
- export / inspect / diff 的產物能互相對得起來，不會換個步驟就失去上下文
- 需要交給 review 或 CI 時，可以把 topology / impact / governance-gate 的結果拿去重跑

## 失敗時先檢查

- 如果 browse 或 inspect 結果比預期少，先核對 `--profile`、`--url`、org 與權限
- 如果 topology 或 impact 是空的，先確認你餵的是同一次 inspect 產物
- 如果治理檢查看起來怪怪的，先看 `governance` 和 `queries` 是否來自相同來源

## 重點旗標

- `--url`：Grafana 基底網址。
- `--token`、`--basic-user`、`--basic-password`：共用的線上 Grafana 憑證。
- `--profile`：從 `grafana-util.yaml` 載入 repo 本地預設值。
- `--color`：控制這個指令群組的 JSON 彩色輸出。

## 範例

```bash
# 先看 live dashboard 長什麼樣，再決定下一步要走哪條工作流。
grafana-util dashboard browse --profile prod
```

```bash
# 先盤點現況，再決定要走 browse 或 export。
grafana-util dashboard list --profile prod
```

```bash
# 先做 live 檢視，再決定要不要匯出或截圖。
grafana-util dashboard inspect-live --url http://localhost:3000 --basic-user admin --basic-password admin --interactive
```

```bash
# 先產生治理成品，留給 topology 或 governance-gate 接著用。
grafana-util dashboard inspect-live --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --output-format governance-json
```

```bash
# 先從標準輸入 review 一份生成儀表板，再決定要不要 publish。
jsonnet dashboards/cpu.jsonnet | grafana-util dashboard review --input - --output-format json
```

```bash
# 編修本地草稿時，每次儲存後自動重跑 publish dry-run。
grafana-util dashboard publish --url http://localhost:3000 --basic-user admin --basic-password admin --input ./drafts/cpu-main.json --dry-run --watch
```

## 相關指令

### 盤點

- [dashboard browse](./dashboard-browse.md)
- [dashboard get](./dashboard-get.md)
- [dashboard clone-live](./dashboard-clone-live.md)
- [dashboard list](./dashboard-list.md)

### 搬移

- [dashboard export](./dashboard-export.md)
- [dashboard import](./dashboard-import.md)
- [dashboard raw-to-prompt](./dashboard-raw-to-prompt.md)
- [dashboard patch-file](./dashboard-patch-file.md)

### 變更前檢查

- [dashboard inspect-export](./dashboard-inspect-export.md)
- [dashboard inspect-live](./dashboard-inspect-live.md)
- [dashboard inspect-vars](./dashboard-inspect-vars.md)
- [dashboard governance-gate](./dashboard-governance-gate.md)
- [dashboard topology](./dashboard-topology.md)

### 變更與套用

- [dashboard review](./dashboard-review.md)
- [dashboard publish](./dashboard-publish.md)
- [dashboard delete](./dashboard-delete.md)
- [dashboard diff](./dashboard-diff.md)

### 截圖與素材

- [dashboard screenshot](./dashboard-screenshot.md)
