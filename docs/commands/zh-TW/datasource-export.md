# datasource export

## 用途
將線上 Grafana datasource inventory 匯出成標準化 JSON 與衍生的
provisioning projection。

## 何時使用
當您需要一個本地 dataworkspace package，供後續檢查、比對或匯入時，
使用這個指令。

## 重點旗標
- `--output-dir`：匯出樹的目標目錄。
- `--org-id`：匯出指定的 Grafana org。
- `--all-orgs`：把每個可見 org 匯出到各自的子目錄。需要 Basic auth。
- `--overwrite`：取代既有檔案。
- `--without-datasource-provisioning`：略過 provisioning 變體。
- `--run`：未指定 `--output-dir` 時，寫入 artifact workspace run。`timestamp` 會建立新的時間戳 run，`latest` 會重用最新紀錄的 run。
- `--run-id`：未指定 `--output-dir` 時，寫入指定名稱的 artifact workspace run。
- `--dry-run`：預覽會寫出哪些內容。

## Artifact workspace 輸出

如果沒有指定 `--output-dir`，`datasource export` 會寫到：

```text
<artifact_root>/<profile-or-default>/runs/<run-id>/datasources/
```

artifact root 來自 `grafana-util.yaml` 裡的 `artifact_root`。如果沒有設定，預設是設定檔旁邊的 `.grafana-util/artifacts`。設定檔不在目前目錄時，可用 root `--config <file>` 或 `GRAFANA_UTIL_CONFIG` 指定。

新匯出建議用 `--run timestamp`，需要固定名稱時用 `--run-id <name>`；後續可用 `datasource list --local --run latest` 讀取最新紀錄的 datasource lane。

## 範例
```bash
# 將線上 Grafana datasource inventory 匯出成標準化 JSON 與 provisioning 檔案。
grafana-util datasource export --url http://localhost:3000 --basic-user admin --basic-password admin --output-dir ./datasources --overwrite
```

```bash
# 將線上 Grafana datasource inventory 匯出成標準化 JSON 與 provisioning 檔案。
grafana-util datasource export --url http://localhost:3000 --basic-user admin --basic-password admin --all-orgs --output-dir ./datasources --overwrite
```

```bash
# 使用時間戳 run id，將 datasource inventory 匯出到 profile artifact workspace。
grafana-util datasource export --profile prod --run timestamp --overwrite
```

## 採用前後對照

- **採用前**：live datasource 狀態很容易散掉，因為匯出後的結構不夠標準，也不容易再利用。
- **採用後**：一個匯出就能得到本地 bundle，後續檢視、比對或匯入都能直接沿用。

## 成功判準

- 匯出樹完整到可以日後不連 Grafana 也能檢查
- 標準化 JSON 與 provisioning projection 都能和來源 inventory 對得上
- provisioning projection 仍是衍生輸出，只有在秘密 placeholder 被補齊後，
  才能視為可直接套用的 Grafana provisioning 檔案
- 這個 bundle 可以直接拿去做 diff 或 import，不需要再手動清理

## 失敗時先檢查

- 如果匯出樹少了 org 資料，先確認 org 範圍與驗證資訊是否真的看得到它
- 如果 `--all-orgs` 失敗，先改用 Basic auth，並確認帳號是否能看見每個目標 org
- 如果 bundle 看起來像舊資料，先確認匯出目錄與 `--overwrite` 是否有刻意使用
- 如果你原本期待匯出裡直接有 secret 明文，先記得 Grafana live datasource API 不會回這些值；export 只會保留 config 與 `secureJsonDataPlaceholders`
- 如果 provisioning 輸出還有 placeholder，先把它當成中介產物，補完
  secrets 後再拿去當 Grafana provisioning 檔案

## 相關指令
- [datasource list](./datasource-list.md)
- [datasource import](./datasource-import.md)
- [datasource diff](./datasource-diff.md)
