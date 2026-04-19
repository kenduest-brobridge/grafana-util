# dashboard diff

## 用途
比較本地儀表板檔案與線上 Grafana 儀表板的差異。

## 何時使用
當您想在匯入或發佈儀表板 bundle 之前先看出會變更哪些內容時，使用這個指令。

## 重點旗標
- `--input-dir`：拿這個匯出目錄與 Grafana 比對。
- `--input-format`：選擇 `raw` 或 `provisioning`。
- `--local`、`--run`、`--run-id`：不傳 `--input-dir`，改拿 artifact workspace 的 dashboard lane 與 Grafana 比對。
- `--import-folder-uid`：覆寫比對時的目的資料夾 UID。
- `--context-lines`：統一 diff 的上下文行數。
- `--output-format`：選擇 `text` 或 `json`。

## 範例
```bash
# 比較本地儀表板檔案與線上 Grafana 儀表板的差異。
grafana-util dashboard diff --url http://localhost:3000 --basic-user admin --basic-password admin --input-dir ./dashboards/raw
```

```bash
# 比較本地儀表板檔案與線上 Grafana 儀表板的差異。
grafana-util dashboard diff --url http://localhost:3000 --basic-user admin --basic-password admin --org-id 2 --input-dir ./dashboards/raw --output-format json
```

```bash
# 拿最新的 profile artifact workspace dashboard run 與 live Grafana 比對。
grafana-util dashboard diff --profile prod --local --output-format json
```

## 相關指令
- [dashboard export](./dashboard-export.md)
- [dashboard import](./dashboard-import.md)
- [dashboard dependencies](./dashboard-dependencies.md)
- [共用 diff JSON contract](../../user-guide/zh-TW/diff-json-contract.md)

CLI schema 快速查詢：

- `grafana-util dashboard diff --help-schema`
- [共用 diff JSON contract](../../user-guide/zh-TW/diff-json-contract.md)
