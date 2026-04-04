# `grafana-util snapshot`

## Root

用途：匯出並檢視 Grafana snapshot inventory bundles。

適用時機：當你想建立一個本機 snapshot root，收錄 dashboard 與 datasource inventory，供後續檢視時。

說明：如果你需要一份離線 snapshot，之後不用重新連到 Grafana 也能繼續檢視，先看這一頁最合適。`snapshot` 指令群組適合交接、備份、事件回顧，或任何想先留下本機 artifact 再往下分析的工作流。

主要旗標：root 指令本身只是指令群組；操作旗標都在 `export` 和 `review`。共用的 root 旗標是 `--color`。

範例：

```bash
# 用途：Root。
grafana-util snapshot export --profile prod --export-dir ./snapshot
grafana-util snapshot review --input-dir ./snapshot --output json
grafana-util snapshot export --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --export-dir ./snapshot
```

相關指令：`grafana-util overview`、`grafana-util status staged`、`grafana-util change bundle`。

## `export`

用途：將 dashboard 與 datasource inventory 匯出到本機 snapshot bundle。

適用時機：當你需要一個不必連到 Grafana 也能檢視的本機 snapshot root 時。

主要旗標：`--export-dir`、`--overwrite`，以及共用的 Grafana 連線與驗證旗標。

範例：

```bash
# 用途：export。
grafana-util snapshot export --profile prod --export-dir ./snapshot
grafana-util snapshot export --url http://localhost:3000 --basic-user admin --basic-password admin --export-dir ./snapshot --overwrite
grafana-util snapshot export --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --export-dir ./snapshot
```

相關指令：`snapshot review`、`change bundle`、`overview`。

## `review`

用途：在不接觸 Grafana 的情況下檢視本機 snapshot inventory。

適用時機：當你想把匯出的 snapshot root 以 table、csv、text、json、yaml 或 interactive 格式查看時。

主要旗標：`--input-dir`、`--interactive`、`--output`。

範例：

```bash
# 用途：review。
grafana-util snapshot review --input-dir ./snapshot --output table
grafana-util snapshot review --input-dir ./snapshot --interactive
```

相關指令：`snapshot export`、`overview`、`status staged`。
