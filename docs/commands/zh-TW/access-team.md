# `grafana-util access team`

## 目的

列出或瀏覽 live 與本機的 Grafana team，以及建立、修改、匯出、匯入、比對或刪除 Grafana team。

## 使用時機

- 檢視 team 清單與 team 成員關係。
- 從 live Grafana 或本機匯出套件中檢視 team。
- 建立或更新 team 成員與管理員指派。
- 匯出或匯入 team 套件。
- 以 id 或精確名稱刪除 team。

## 採用前後對照

- **採用前**：team 成員關係常常散在 UI 側邊選單或零碎腳本裡。
- **採用後**：同一個命名空間就能處理 inventory、成員更新、匯出／匯入與刪除，而且認證方式一致。

## 成功判準

- team 成員變更都綁定到精確的 team id 或名稱
- 在新增或移除成員前，可以先看出管理員指派
- 匯出的套件可以在另一個環境重複使用，不必手動重建 team

## 失敗時先檢查

- 如果 list、add、modify 或 delete 失敗，先確認這個 team 在選到的 org 裡存在，而且認證範圍正確
- 如果成員看起來不完整，先核對精確的 member 名稱，以及是否有加上 `--with-members`
- 如果匯入回報 blocked membership update，先確認目標 team 是否為 provisioned，以及每個 member identity 是否能解析成有 email 的 live org user
- 如果匯入結果不如預期，先確認來源套件與目標環境，再重試

## 匯入注意事項

- Team import 可以接受匯出檔裡的 member identity，但 Grafana bulk membership API 實際上用 email 套用成員；importer 會先把 login、email 或 user id 解析成 live org user email。
- `--dry-run --output-format table` 或 `--dry-run --json` 會在寫入前標出 provisioned team 的 blocked update。

## 主要旗標

- `list`: `--input-dir`, `--query`, `--name`, `--with-members`, `--output-columns`, `--list-columns`, `--page`, `--per-page`, `--table`, `--csv`, `--json`, `--yaml`, `--output-format`
- `browse`: `--input-dir`, `--query`, `--name`, `--with-members`, `--page`, `--per-page`
- `add`: `--name`, `--email`, `--member`, `--admin`, `--json`
- `modify`: `--team-id`, `--name`, `--add-member`, `--remove-member`, `--add-admin`, `--remove-admin`, `--json`
- `export` 與 `diff`: `--output-dir` 或 `--diff-dir`, `--run`, `--run-id`, `--overwrite`, `--dry-run`, `--with-members`
- `import`: `--input-dir`, `--replace-existing`, `--dry-run`, `--table`, `--json`, `--output-format`, `--yes`
- `delete`: `--team-id`, `--name`, `--prompt`, `--yes`, `--json`

## 範例

如果沒有指定 `--output-dir`，`access team export` 會寫到 profile artifact workspace 的 `access/teams/`。新匯出建議用 `--run timestamp`，需要固定 run 名稱時用 `--run-id <name>`。後續本機讀取可用 `access team list --local --run latest`。

```bash
# 在新增或移除成員前，先確認 team membership。
grafana-util access team list --profile prod --output-format text
```

```bash
# 先看本機存好的 team 套件。
grafana-util access team list --input-dir ./access-teams --output-format table
```

```bash
# 直接互動式瀏覽本機 team 套件，不碰 live Grafana。
grafana-util access team browse --input-dir ./access-teams --name platform-team
```

```bash
# 使用時間戳 run id，將 teams 匯出到 profile artifact workspace。
grafana-util access team export --profile prod --run timestamp --with-members --overwrite
```

```bash
# 建立一個有明確成員與管理員指派的 team。
grafana-util access team add --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --name platform-team --email platform@example.com --member alice --admin alice --json
```

```bash
# 在終端機中選一個 team、確認目標，然後刪除。
grafana-util access team delete --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --prompt
```

## 相關命令

- [access](./access.md)
- [access user](./access-user.md)
- [access org](./access-org.md)
- [access service-account](./access-service-account.md)
