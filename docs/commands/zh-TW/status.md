# `grafana-util status`

## Root

用途：透過單一 status surface 讀取 live 與 staged 的 Grafana 狀態。

適用時機：當你想看 readiness、overview、snapshot，或直接讀 live 資料，但還不想進 mutation 流程時。

說明：`status` 是給使用者看的唯讀入口。`live` 用來做即時 gate，`staged` 用來看本機 artifact，`overview` 用來看全域摘要，`snapshot` 用來看 bundle 風格的 review，`resource` 則用來直接讀 live resource。

範例：

```bash
# 執行這個範例指令。
grafana-util status live --profile prod --output-format yaml
# 執行這個範例指令。
grafana-util status staged --desired-file ./desired.json --output-format json
# 執行這個範例指令。
grafana-util status overview --dashboard-export-dir ./dashboards/raw --alert-export-dir ./alerts --output-format table
# 執行這個範例指令。
grafana-util status overview live --url http://localhost:3000 --basic-user admin --basic-password admin --output-format interactive
```

相關指令：`grafana-util export`、`grafana-util workspace`、`grafana-util config profile`。

## `live`

用途：從 Grafana live read surface 產生 readiness 視圖。

JSON/YAML 輸出會包含共用 project status contract：`kind`、`schemaVersion`、
`toolVersion`、`scope`、`overall`、`domains`、`topBlockers`、`nextActions`。
live 輸出也會從 Grafana `GET /api/health` 補上 `discovery.instance`。
成功時會有 `source: api-health`、`status: available`，以及 `database`、
`version`、`commit` 等 health 欄位。若 health 讀取失敗，
`discovery.instance.status` 會是 `unavailable` 並帶 `error`；這不會單獨把
domain readiness 判成 blocked。

成功時的 instance metadata：

```json
{
  "discovery": {
    "instance": {
      "source": "api-health",
      "status": "available",
      "health": {
        "database": "ok",
        "version": "12.4.0",
        "commit": "abc123"
      }
    }
  }
}
```

health 讀取失敗時：

```json
{
  "discovery": {
    "instance": {
      "source": "api-health",
      "status": "unavailable",
      "error": "..."
    }
  }
}
```

automation 如果需要 Grafana build 資訊，讀
`discovery.instance.health.version` 與 `discovery.instance.health.commit`。
readiness gate 仍應讀 `overall` 與 `domains`。
