# dashboard export

## 用途
將儀表板匯出成 `raw/`、`prompt/` 與 `provisioning/` 檔案，必要時也可附帶 `history/` 成品。

## 何時使用
當您需要一個本地匯出樹，供後續匯入、檢視、比對或檔案 provisioning 工作流程使用時，使用這個指令。若您也需要每個匯出 org 範圍各自的版本歷史成品，請加上 `--include-history`。`prompt/` 路徑是給 Grafana UI 匯入用，不是給 dashboard API 匯入用；如果您只有一般或 raw 的 dashboard JSON，需要先轉成 prompt JSON，請使用 `dashboard convert raw-to-prompt`。

## 採用前後對照
- **採用前**：匯出比較像一次性的備份動作，之後能不能 review、inspect 或 replay，通常要走到下一步才知道。
- **採用後**：匯出會變成整條工作流的第一份 artifact，後面可以接 inspect、diff、dry-run import 與 Git review。

## 重點旗標
- `--output-dir`：匯出樹的目標目錄。
- `--org-id`：匯出指定的 Grafana org。
- `--all-orgs`：把每個可見 org 匯出到各自的子目錄。建議使用 Basic auth。
- `--flat`：直接把檔案寫入各個匯出變體目錄。
- `--overwrite`：取代既有的匯出檔案。
- `--without-raw`、`--without-prompt`、`--without-provisioning`：略過某個變體。
- `--include-history`：把 dashboard 版本歷史成品寫到每個匯出 org 範圍下的 `history/` 子目錄。
- `--provider-name`、`--provider-org-id`、`--provider-path`：自訂產生的 provisioning provider 檔案。
- `--provider-disable-deletion`、`--provider-allow-ui-updates`、`--provider-update-interval-seconds`：調整 provisioning 行為。
- `--run`：未指定 `--output-dir` 時，寫入 artifact workspace run。`timestamp` 會建立新的時間戳 run，`latest` 會重用最新紀錄的 run。
- `--run-id`：未指定 `--output-dir` 時，寫入指定名稱的 artifact workspace run。
- `--dry-run`：預覽會寫出哪些內容。

## 說明
- 預設會寫出 `raw/`、`prompt/`、`provisioning/`。
- 搭配 `--all-orgs` 時，優先用 Basic auth。
- 非 flat 的 `raw/` 與 `prompt/` 匯出會對齊 Grafana folder path，例如 `Platform / Team / Infra` 會成為 `raw/Platform/Team/Infra/`。
- export index 會記錄每個 dashboard 的 `folderUid`、`folderTitle` 與完整 `folderPath`，讓後續 repair、review、conversion workflow 不必依賴 dashboard JSON 內一定有 `meta.folderUid`。
- 舊版匯出若仍是 leaf folder layout，可用 `dashboard convert export-layout` 修復。
- `--flat` 會把檔案直接寫在各變體目錄下。
- `--include-history` 會在每個匯出 org 範圍下加上 `history/`。
- provider 檔案會寫到 `provisioning/provisioning/dashboards.yaml`。
- `raw/` 給 API import 或 diff，`prompt/` 給 UI import，`provisioning/` 給 file provisioning。

## Artifact workspace 輸出

如果沒有指定 `--output-dir`，`dashboard export` 會寫到所選 profile config 的 artifact workspace。預設設定檔是目前目錄的 `grafana-util.yaml`，也可以用 root `--config <file>` 或 `GRAFANA_UTIL_CONFIG` 指定。

如果 config 沒有設定 `artifact_root`，預設目錄是設定檔旁邊的 `.grafana-util/artifacts`。相對路徑的 `artifact_root` 也會以設定檔所在目錄為基準解析。

run layout 如下：

```text
<artifact_root>/<profile-or-default>/runs/<run-id>/dashboards/
<artifact_root>/<profile-or-default>/latest-run.json
```

新匯出建議使用 `--run timestamp`，需要固定 run 名稱時使用 `--run-id <name>`；`--run latest` 較適合後續本機讀取命令用來讀取最新成功 run。

## 匯出變體差異

`dashboard export` 預設一次產生三種 dashboard 表示法。這不是因為工具想多寫幾份檔案，而是因為 Grafana 後面有三條完全不同的入口：CLI/API 回放、UI 匯入，以及檔案 provisioning。匯出時先把三種路徑分清楚，後面 review、交接、部署時才不需要猜「這份 JSON 到底能不能拿去 import」。

把 `raw/` 當成現場原始底片。它最接近 API 讀回來的狀態，適合拿來備份、比對、審查、dry-run，再由 `dashboard import` 推回 Grafana。如果這次匯出是為了留下可追溯、可重跑、可放進 Git 的紀錄，通常從 `raw/` 開始。

把 `prompt/` 當成要交給人進 Grafana UI 的版本。它整理成 UI Import dashboard 流程比較能接受的形狀，目的是讓接手的人可以用瀏覽器匯入，而不是讓自動化流程直接 API replay。若手上只有一般 dashboard JSON 或 `raw/` 檔案，先用 `dashboard convert raw-to-prompt` 轉成這條路徑。

把 `provisioning/` 當成 Grafana 從磁碟讀取的部署投影。它包含 dashboard JSON 與 provider YAML，適合被掛進 Grafana provisioning path，讓 Grafana 啟動或 reload provisioning 時接管。它不是互動式匯入，也不應該取代 `raw/` 成為審查與回放的主要來源。

| 變體 | 內容 | 適合下一步 | 注意事項 |
| :--- | :--- | :--- | :--- |
| `raw/` | 從 Grafana API 讀回並保留 API 匯入所需上下文的 dashboard JSON。 | `dashboard import`、`dashboard diff`、`dashboard review`、`dashboard dependencies`、`workspace scan/test/preview`。 | 這是最適合拿來做 CLI replay、比對、審查與 Git 版本控管的格式。若不確定要留哪一種，先保留 `raw/`。 |
| `prompt/` | 轉成 Grafana UI 匯入提示可接受的 dashboard JSON。 | 人工交接、貼到 Grafana UI 的 Import dashboard 流程、把一般 dashboard JSON 整理成 UI 匯入格式。 | 不是給 `dashboard import` API replay 用。若手上只有一般或 raw JSON，使用 `dashboard convert raw-to-prompt` 轉成這個格式。 |
| `provisioning/` | 檔案 provisioning 目錄，含 dashboard JSON 與 provider YAML。 | 掛載到 Grafana provisioning path、建立 GitOps-style 檔案供應流程。 | provider YAML 預設位於 `provisioning/provisioning/dashboards.yaml`。這條路徑適合由 Grafana 啟動或 reload provisioning 時讀取，不是互動式 API 匯入流程。 |

常見選擇：

- 要做備份、review、diff、dry-run import，從 `raw/` 開始。
- 要把 dashboard 交給人從 Grafana UI 匯入，交付 `prompt/`。
- 要放進 Grafana file provisioning，使用 `provisioning/`，並確認 provider 參數是否符合部署路徑。
- 若只需要其中一種輸出，可用 `--without-raw`、`--without-prompt` 或 `--without-provisioning` 減少噪音。

## 成功判準
- 產生出可供 API replay 與進一步 inspect 的 `raw/` 樹
- 如果需要較乾淨的 handoff，也有對應的 `prompt/` 樹
- 如果有加 `--include-history`，每個匯出 org 範圍下都會有對應的 `history/` 樹
- 匯出結果足夠穩定，可直接拿去比對、審查或納入版本控制

## 失敗時先檢查
- 如果 dashboard 數量不對，先檢查 org 範圍，不要先懷疑 exporter
- 如果 `--all-orgs` 的輸出看起來不完整，先確認憑證是否真的看得到所有 org
- 如果預期中的 history 成品沒出現，先確認是否有加上 `--include-history`，也要確認是不是看錯了 org 範圍
- 如果下一步是匯入，先確認這次該沿用 `raw/` 還是 `prompt/`

## 範例
```bash
# 將儀表板匯出成 `raw/`、`prompt/` 與 `provisioning/` 檔案。
grafana-util dashboard export --profile prod --output-dir ./dashboards --overwrite
```

```bash
# 將儀表板匯出成 `raw/`、`prompt/` 與 `provisioning/` 檔案。
grafana-util dashboard export --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --output-dir ./dashboards --overwrite
```

```bash
# 將儀表板匯出成 `raw/`、`prompt/` 與 `provisioning/` 檔案。
grafana-util dashboard export --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --output-dir ./dashboards --overwrite
```

```bash
# 匯出 dashboard，並把每個 org 的版本歷史成品一併寫入可重用的目錄樹。
grafana-util dashboard export --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --include-history --output-dir ./dashboards --overwrite
```

```bash
# 使用時間戳 run id，將 dashboard 匯出到 profile artifact workspace。
grafana-util dashboard export --profile prod --run timestamp --overwrite
```

## 相關指令
- [dashboard dependencies](./dashboard-dependencies.md)
- [dashboard import](./dashboard-import.md)
- [dashboard diff](./dashboard-diff.md)
- [dashboard convert raw-to-prompt](./dashboard-convert-raw-to-prompt.md)
- [dashboard convert export-layout](./dashboard-convert-export-layout.md)
- [dashboard history](./dashboard-history.md)
