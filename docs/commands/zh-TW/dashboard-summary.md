# dashboard summary

## 用途
直接分析 live Grafana，整理出 dashboard、變數、data source 與查詢概況，讓你在不先匯出的情況下就能先看清楚現在的環境。

## 何時使用
當你的來源是 live Grafana，而不是本地匯出樹時，就先開這頁。它適合拿來做盤點、治理輸出、相依性前置分析，或在要跑 `policy`、`impact`、`variables` 之前先把 live 狀態整理清楚。

## 最短成功路徑

1. 先決定你是要看人看的結果，還是給下一步工具吃的結果。
2. 指向 live Grafana：`--profile ...` 或 `--url ...` + 憑證。
3. 先跑一次 `--output-format governance` 或 `--output-format table`。
4. 若結果有用，再決定是否改成 `governance-json`、`queries-json` 或 `--interactive`。

## 你應該選 `summary` 還是 `dependencies`

- 來源是 **live Grafana**：用 `dashboard summary`
- 來源是 **本地匯出樹**：用 [dashboard dependencies](./dashboard-dependencies.md)
- 你只是想快速看有哪些 dashboard：先看 `dashboard browse` 或 `dashboard list`
- 你是想追某個 datasource 的受影響範圍：改看 [dashboard impact](./dashboard-impact.md)

## 重點旗標
- `--page-size`：儀表板搜尋的每頁筆數。
- `--concurrency`：最大平行抓取工作數。
- `--org-id`：分析指定的 Grafana org。
- `--all-orgs`：跨所有可見 org 分析。
- `--output-format`、`--output-file`、`--interactive`、`--no-header`：輸出控制。
- `--report-columns`：把 table、csv 或 tree-table 的 query 輸出裁成指定欄位。可用 `all` 展開完整 query 欄位集合。
- `--list-columns`：列出支援的 `--report-columns` 值後直接結束。
- `--progress`：顯示抓取進度。

## 先決定輸出長什麼樣

- `table` / `text`：給人直接看，適合手動盤點與 review 前確認
- `governance` / `governance-json`：給 `policy`、`impact` 或後續治理流程重用
- `queries-json`：給查詢層分析或外部工具接手
- `interactive`：來源已經確認正確，想現場往下鑽時再開

## 範例（由淺到深）

```bash
# 先用 repo 既有 profile 盤點 live Grafana。
grafana-util dashboard summary --profile prod --output-format governance
```

```bash
# 直接連到 live Grafana，先用 interactive 模式現場看內容。
grafana-util dashboard summary --url http://localhost:3000 --basic-user admin --basic-password admin --interactive
```

```bash
# 產生可重用的治理輸出，留給後續 policy / impact / CI。
grafana-util dashboard summary --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --output-format governance
```

## 什麼叫做這次跑成功

- 你能在不先匯出的情況下，看出 live 環境有哪些 dashboard 與查詢結構
- 產物格式足以直接接到下一步，而不是還要重新抓一次資料
- 同一份輸出可以支撐 review、治理或影響分析，而不是每一步都重跑不同命令

## 失敗時先檢查

- 如果結果看起來比 UI 少，先檢查 org、權限與憑證，不要先懷疑 renderer
- 如果要給下一步工具吃，先確認你選的是 `governance-json` 或 `queries-json`，而不是只有人看得懂的格式
- 如果 interactive 能看到、JSON 卻不對，先比對 `--output-format` 與 top-level shape

## 相關指令
- [dashboard dependencies](./dashboard-dependencies.md)
- [dashboard variables](./dashboard-variables.md)
- [dashboard impact](./dashboard-impact.md)
- [dashboard policy](./dashboard-policy.md)
