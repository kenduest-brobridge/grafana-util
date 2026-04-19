# `grafana-util access org`

## 目的

列出 live 或本機的 Grafana org、建立、修改、匯出、匯入、比對或刪除 Grafana org。

## 使用時機

- 檢視 org 清單與 org 使用者。
- 從 live Grafana 或本機匯出套件中檢視 org。
- 建立新 org 或重新命名既有 org。
- 在環境之間匯出或匯入 org 套件。
- 以 id 或精確名稱刪除 org。

## 採用前後對照

- **採用前**：org 管理常常先從一次性的管理員點擊流程或只適用單一環境的腳本開始。
- **採用後**：同一個命名空間就能處理 inventory、重新命名、匯出／匯入與刪除，而且可以重複使用管理員認證。

## 成功判準

- org 名稱與 id 在 inventory 與變更流程中都保持精確
- 匯出與匯入套件可在環境搬移時重複使用
- 高權限操作都綁定到明確的 admin-backed profile，而不是臨時 token

## 失敗時先檢查

- 如果建立、重新命名、匯出、匯入或刪除失敗，先確認選到的 profile 具備必要的管理員權限
- 如果以名稱查詢卻對到錯的 org，先核對精確的 org id 或精確名稱，再重試
- 如果套件匯出或匯入看起來不完整，先確認目標環境是否選對

## 主要旗標

- `list`: `--input-dir`, `--org-id`, `--name`, `--query`, `--with-users`, `--table`, `--csv`, `--json`, `--yaml`, `--output-format`
- `add`: `--name`, `--json`
- `modify`: `--org-id`, `--name`, `--set-name`, `--json`
- `export` 與 `diff`: `--org-id`, `--name`, `--output-dir` 或 `--diff-dir`, `--run`, `--run-id`, `--overwrite`, `--dry-run`, `--with-users`
- `import`: `--input-dir`, `--replace-existing`, `--dry-run`, `--yes`
- `delete`: `--org-id`, `--name`, `--prompt`, `--yes`, `--json`

## 說明

- 只要 profile 具備必要管理員權限，就可優先用 `--profile` 做可重複的 org inventory。
- org 管理面通常比窄權限 API token 更廣。建立、重新命名、匯出、匯入與刪除流程，較建議使用 Basic auth 或管理員憑證支援的 profile。
- 當 org bundle 包含 users 時，import dry-run 會查詢 live org users，讓 role plan 反映目標環境。Externally synced user 的 role 變更會在 apply 前標示為 blocked。
- 如果沒有指定 `--output-dir`，`access org export` 會寫到 profile artifact workspace 的 `access/orgs/`。新匯出建議用 `--run timestamp`，需要固定 run 名稱時用 `--run-id <name>`。

## 範例

```bash
# 在重新命名或搬移前，先確認 org inventory。
grafana-util access org list --profile prod --output-format text
```

```bash
# 先看本機存好的 org 套件。
grafana-util access org list --input-dir ./access-orgs --output-format table
```

```bash
# 使用時間戳 run id，將 org inventory 匯出到 profile artifact workspace。
grafana-util access org export --profile prod --run timestamp --with-users --overwrite
```

```bash
# 確認目前 org 名稱後，重新命名這個 org。
grafana-util access org modify --url http://localhost:3000 --basic-user admin --basic-password admin --name platform --set-name platform-core --json
```

```bash
# 在正式刪除前，先看清楚這個 org 的資訊。
grafana-util access org list --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --json
```

```bash
# 只在確認精確名稱後，才刪除這個 org。
grafana-util access org delete --url http://localhost:3000 --basic-user admin --basic-password admin --name platform --yes
```

```bash
# 在終端機中選一個 org、確認目標，然後刪除。
grafana-util access org delete --url http://localhost:3000 --basic-user admin --basic-password admin --prompt
```

## 相關命令

- [access](./access.md)
- [access user](./access-user.md)
- [access team](./access-team.md)
