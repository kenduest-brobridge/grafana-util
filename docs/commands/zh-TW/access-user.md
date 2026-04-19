# `grafana-util access user`

## 目的

列出或瀏覽 live 與本機的 Grafana 使用者，以及建立、修改、匯出、匯入、比對或刪除 Grafana 使用者。

## 使用時機

- 檢視目前 org 或全域管理範圍內的使用者。
- 從 live Grafana 或本機匯出套件中檢視使用者。
- 以登入名、電子郵件、角色與管理員設定建立或更新使用者。
- 匯出與匯入使用者清單套件。
- 從 org 成員關係或全域登錄中移除使用者。

## 採用前後對照

- **採用前**：使用者生命週期工作常常散在 org 設定、管理頁面與一次性的匯出或匯入腳本裡。
- **採用後**：同一個命名空間就能處理 inventory、建立／更新、匯出／匯入，以及定點移除，而且認證方式一致。

## 成功判準

- 建立或修改後的使用者會有預期的 login、email 與 role
- inventory 與套件可以在刪除或搬移前先比對
- 成員關係的範圍一直很清楚，不會不小心動到錯的 org 或全域登錄
- 給 automation / audit 用的 machine-readable `list` 與 export 輸出會包含結構化的 `origin` 與 `lastActive` 使用者中繼資料

## 失敗時先檢查

- 如果 list、add 或 delete 看起來是空的或不對，先確認選到的 profile 或 token 具有正確的 org 或 admin scope
- 如果建立或修改失敗，先核對 login / email 是否重複，以及目前範圍是 org 還是 global
- 如果匯入回報 blocked update，先確認目標 user 是否為 external、externally synced，或由 provisioned identity source 管理
- 如果匯入的行為不如預期，先確認套件來源與目標範圍，再重試

## 匯入注意事項

- User import 會把 `id`、`userId`、`uid`、`authLabels` 與 external/provisioned 旗標視為目標 evidence，不把它們當成跨環境 desired identity。
- `--dry-run --output-format table` 或 `--dry-run --json` 會在寫入前標出 Grafana 判定為外部管理的 profile、org role 或 Grafana admin blocked update。
- `modify` 也會在任何寫入前套用相同的 Grafana-source guardrails：external 或 provisioned user 會阻擋 profile/password 變更，externally synced user 會阻擋 org-role 變更，externally synced Grafana-admin 狀態會阻擋 admin 變更。

## 主要旗標

- `list`: `--input-dir`, `--scope`, `--all-orgs`, `--query`, `--login`, `--email`, `--org-role`, `--grafana-admin`, `--with-teams`, `--output-columns`, `--list-columns`, `--page`, `--per-page`, `--table`, `--csv`, `--json`, `--yaml`, `--output-format`
- `browse`: `--input-dir`, `--scope`, `--all-orgs`, `--current-org`, `--query`, `--login`, `--email`, `--org-role`, `--grafana-admin`, `--page`, `--per-page`
- `add`: `--login`, `--email`, `--name`, `--password` 或 `--password-file` 或 `--prompt-user-password`, `--org-role`, `--grafana-admin`, `--json`
- `modify`: `--user-id`, `--login`, `--email`, `--set-login`, `--set-email`, `--set-name`, `--set-password` 或 `--set-password-file` 或 `--prompt-set-password`, `--set-org-role`, `--set-grafana-admin`, `--json`
- `export` 與 `diff`: `--output-dir` 或 `--diff-dir`, `--run`, `--run-id`, `--overwrite`, `--dry-run`, `--scope`, `--with-teams`
- `import`: `--input-dir`, `--scope`, `--replace-existing`, `--dry-run`, `--table`, `--json`, `--output-format`, `--yes`
- `delete`: `--user-id`, `--login`, `--email`, 可選的 `--scope`, `--prompt`, `--yes`, `--json`

## 範例

如果沒有指定 `--output-dir`，`access user export` 會寫到 profile artifact workspace 的 `access/users/`。新匯出建議用 `--run timestamp`，需要固定 run 名稱時用 `--run-id <name>`。後續本機讀取可用 `access user list --local --run latest`。

```bash
# 在調整成員權限前，先看清楚單一 org 裡有哪些使用者。
grafana-util access user list --profile prod --scope org --output-format text
```

```bash
# 先看本機存好的使用者套件。
grafana-util access user list --input-dir ./access-users --output-format table
```

```bash
# 直接互動式瀏覽本機使用者套件，不碰 live Grafana。
grafana-util access user browse --input-dir ./access-users --login alice
```

```bash
# 使用時間戳 run id，將 users 匯出到 profile artifact workspace。
grafana-util access user export --profile prod --run timestamp --scope org --overwrite
```

```bash
# 用明確的認證與 org 範圍建立一個使用者。
grafana-util access user add --url http://localhost:3000 --basic-user admin --basic-password admin --login alice --email alice@example.com --name Alice --password secret
```

```bash
# 先看清楚目前 org 裡的使用者，再刪除這個帳號。
grafana-util access user list --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --scope org --json
```

```bash
# 在終端機中選一個使用者、確認目標，然後刪除。
grafana-util access user delete --url http://localhost:3000 --basic-user admin --basic-password admin --scope global --prompt
```

```bash
# 先在終端機中選刪除範圍，再選使用者並確認刪除。
grafana-util access user delete --url http://localhost:3000 --basic-user admin --basic-password admin --prompt
```

## 相關命令

- [access](./access.md)
- [access org](./access-org.md)
- [access team](./access-team.md)
- [access service-account](./access-service-account.md)
