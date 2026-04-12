# 身分與存取管理 (Identity & Access)

這一章整理 Grafana 的身分與存取資產：org、使用者、team 與 service account。重點是先把人、組織與自動化憑證的生命週期講清楚，再談匯出、同步與回放。
盤點可以直接來自 live Grafana，也可以來自本機匯出的套件；流程本身是一樣的。

Access 相關工作最容易被低估，因為它常被看成「新增一個 user」或「刪一個 token」。實際上，每個 user、team、org 與 service account 都在決定誰看得到資料、誰能修改資產、哪個自動化流程可以在半夜替你動 Grafana。這章的目標是讓你在動手前先看懂範圍。

讀這章時，先問任務影響的是人、團隊、組織邊界，還是機器憑證。接著再決定要做 live 盤點、本機快照、diff，或真的變更。不要從「我有一個 token」開始推導流程；應該從「這次權限變更會影響誰」開始。

## 適用對象

- 負責 org、使用者、team 或 service account 管理的人
- 需要做權限盤點、同步、匯出或回放的人
- 需要把身分資產接進 Git 或 CI 流程的人

## 主要目標

- 先理解 org / user / team / service account 的關係
- 再把盤點、匯出、匯入與 diff 做成可重複流程，而且盤點可以來自 live 或本機
- 需要時才對 token 做輪替或刪除

好的 access 流程不是讓修改變快，而是讓修改前的 scope 變清楚。能先說出「誰會被影響」比能背出子命令更重要。

## 採用前後對照

- 以前：身分與存取的工作分散在不同 command 群組，命名風格也不一致。
- 現在：org、user、team 與 service account 放在同一份導引，入口與術語都更清楚。

## 成功判準

- 你一眼就能分辨任務是 org 管理、user 管理、team 管理，還是 service account 處理。
- 你在動手前就知道自己要進哪一組 command。
- 你知道什麼時候需要先看審查步驟，再做變更。

## 失敗時先檢查

- 如果這次身分變更會影響到比預期更多的 org，先停下來確認範圍。
- 如果不確定 token 或 service account 能不能做這件事，先看指令頁，不要直接 mutate。
- 如果你還覺得 team / org / user 的說法不一致，先回 glossary 和 command reference。

## Access 工作流地圖

Access 子命令要先按「影響誰」分類，而不是按你手上有哪個 token 分類：

| 任務 | 起點 | 主要輸入 | 主要輸出 | 下一步 |
| --- | --- | --- | --- | --- |
| 管理 org | `access org list/export/import/diff` | live Grafana 或 org bundle | org inventory / dry-run / diff | import 前 review |
| 管理 user | `access user list/add/modify/delete/export/import/diff` | login、org role、bundle | 使用者 inventory 或 mutation 結果 | list / diff 驗證 |
| 管理 team | `access team list/export/import/diff` | org scope、team bundle | team 與 members inventory | dry-run 後 import |
| 管理 service account | `access service-account list/add/modify/delete/export/import/diff` | org scope、service account bundle | automation identity inventory | token 管理或 diff |
| 管理 token | `access service-account token add/delete/list` | service account id/name | 一次性 token secret 或刪除結果 | 立即保存 secret / 輪替舊 token |
| 比對漂移 | `diff` 子命令 | 本地 bundle + live Grafana | 變更差異 | 修 bundle 或 import |

最重要的判斷是 scope：全域 user、某個 org 裡的 role、team membership、service account 權限與 token lifecycle 是不同層次。不要用能執行 API 的 token 反推它應該能做所有 access 工作。

## org 管理

需要用 Basic auth 盤點、匯出或回放 org 時，請使用 `access org`。它的 `list` 也可以直接讀本機 bundle。

### 1. 列出、匯出與回放 org
```bash
# 先列出 live Grafana 裡目前可見的 org。
grafana-util access org list --table
```

```bash
# 不碰 live Grafana，直接檢視本機 org bundle。
grafana-util access org list --input-dir ./access-orgs --table
```

```bash
# 把 org inventory 匯出成可重播的本機套件。
grafana-util access org export --output-dir ./access-orgs
```

```bash
# 匯入前先 dry-run，確認會建立或更新哪些 org。
grafana-util access org import --input-dir ./access-orgs --dry-run
```
**預期輸出：**
```text
ID   NAME        IS_MAIN   QUOTA
1    Main Org    true      -
5    SRE Team    false     10

Exported org inventory -> access-orgs/orgs.json
Exported org metadata   -> access-orgs/export-metadata.json

PREFLIGHT IMPORT:
  - would create 0 org(s)
  - would update 1 org(s)
```
先用 list 確認主 org，再用 export/import 建立可重播的 org 快照。

---

## 使用者與 team 管理

需要調整成員、管理快照或檢查漂移時，請使用 `access user` 與 `access team`。它們的 `list` 與 `browse` 都可以讀本機 bundle。

先分清楚 user 與 team 的責任。User 決定某個登入身份存在不存在、是否停用、以及它在 org 裡的 role；team 則是協作與權限管理的集合。當你要處理「這個人能不能登入」時，先看 `access user`。當你要處理「這群人能不能一起看到某批資產」時，先看 `access team`。

`list` 適合盤點 live 或本地 bundle；`export` 適合留下可 review 快照；`diff` 用來判斷 live 是否偏離 bundle；`import` 則應該在 dry-run 後才執行。直接 `add`、`modify`、`delete` user 時，要先確認 scope 是 global、org role，還是 Grafana admin，不要只看 login 名稱。

### 1. 新增、修改與比對使用者
```bash
# 新增一個具備全域 Admin 角色的使用者
grafana-util access user add --login dev-user --role Admin --prompt-password

# 修改現有使用者在特定 org 中的角色
grafana-util access user modify --login dev-user --org-id 5 --role Editor

# 將儲存的使用者快照與即時 Grafana 比對
grafana-util access user diff --diff-dir ./access-users --scope global
```

如果要看同一份本機套件，可以改用：

```bash
# 從本機套件檢視同一份使用者 inventory
grafana-util access user list --input-dir ./access-users
```
**預期輸出：**
```text
Created user dev-user -> id=12 orgRole=Editor grafanaAdmin=true

No user differences across 12 user(s).
```
如果不想把密碼留在 shell history 裡，請改用 `--prompt-password`。`--scope global` 需要 Basic auth。

### 2. team 盤點與同步
```bash
# 先看指定 org 裡目前有哪些 team。
grafana-util access team list --org-id 1 --table
```

```bash
# 從本機 team bundle 檢查同一份 inventory。
grafana-util access team list --input-dir ./access-teams --table
```

```bash
# 匯出 team 時連成員狀態一起保留。
grafana-util access team export --output-dir ./access-teams --with-members
```

```bash
# 匯入 team bundle 前先用表格預覽差異。
grafana-util access team import --input-dir ./access-teams --replace-existing --dry-run --table
```
**預期輸出：**
```text
ID   NAME           MEMBERS   EMAIL
10   Platform SRE   5         sre@company.com

Exported team inventory -> access-teams/teams.json
Exported team metadata   -> access-teams/export-metadata.json

LOGIN       ROLE    ACTION   STATUS
dev-admin   Admin   update   existing
ops-user    Viewer  create   missing
```
匯出時加上 `--with-members` 才會保留成員狀態；要做可能覆寫的匯入前，先用 `--dry-run --table` 看一次。

---

## service account 管理

service account 是自動化流程常見的基礎元件。它的 inventory 也可以先從本機套件看，不必先碰 live Grafana。

Service account 是機器身份，不是比較安全的 user shortcut。它應該對應到一個明確的自動化用途，例如 CI deployment、nightly audit 或 incident bot。`access service-account` 管身份本身；`access service-account token` 管可用來呼叫 API 的 token。這兩層要分開 review。

Token secret 通常只會出現一次。建立 token 後，先把 secret 存到正確的 secret store，再輪替舊 token。不要用終端 log 當成 token 保存方式，也不要因為 list 看得到 service account，就假設手上的 token 仍然有效。

### 1. 列出與匯出 service account
```bash
# 用 JSON 盤點 live service account，方便交給腳本處理。
grafana-util access service-account list --json
```

```bash
# 從本機套件用文字格式檢視 service account inventory。
grafana-util access service-account list --input-dir ./access-sa --output-format text
```

```bash
# 匯出 service account inventory，供 review 或回放使用。
grafana-util access service-account export --output-dir ./access-sa
```
**預期輸出：**
```text
[
  {
    "id": "15",
    "name": "deploy-bot",
    "role": "Editor",
    "disabled": false,
    "tokens": "1",
    "orgId": "1"
  }
]

Listed 1 service account(s) at http://127.0.0.1:3000

Exported service account inventory -> access-sa/service-accounts.json
Exported service account tokens    -> access-sa/tokens.json
```
`access service-account export` 會寫出盤點結果與 token bundle。`tokens.json` 包含敏感資訊，請妥善保管。

### 2. 建立與刪除 token
```bash
# 以名稱新增一個 token
grafana-util access service-account token add --name deploy-bot --token-name nightly --seconds-to-live 3600

# 以數字 ID 新增 token，並保留一次性的 secret
grafana-util access service-account token add --service-account-id 15 --token-name ci-deployment-token --json

# 驗證後刪除舊 token
grafana-util access service-account token delete --service-account-id 15 --token-name nightly --yes --json
```
**預期輸出：**
```text
Created service-account token nightly -> serviceAccountId=15

{
  "serviceAccountId": "15",
  "name": "ci-deployment-token",
  "secondsToLive": "3600",
  "key": "eyJ..."
}

{
  "serviceAccountId": "15",
  "tokenId": "42",
  "name": "nightly",
  "message": "Service-account token deleted."
}
```
如果需要一次性的 `key`，請加上 `--json`。純文字輸出適合寫入日誌，不適合拿來擷取憑證。

---

## 漂移檢查 (Diff)

比較本機快照與 live Grafana 之間的差異。

```bash
# 比較本機快照與 live Grafana 之間的差異。
grafana-util access user diff --input-dir ./access-users
```

```bash
# 比較本機快照與 live Grafana 之間的差異。
grafana-util access team diff --diff-dir ./access-teams
```

```bash
# 比較本機快照與 live Grafana 之間的差異。
grafana-util access service-account diff --diff-dir ./access-sa
```
**預期輸出：**
```text
--- Live Users
+++ Snapshot Users
-  "login": "old-user"
+  "login": "new-user"

No team differences across 4 team(s).
No service account differences across 2 service account(s).
```
可以用 diff 輸出判斷快照是否適合匯入，也能先看出 live 環境是否已經發生漂移。

## 何時切到指令參考

這一章負責幫你判斷 access scope。當你已經知道要處理 org、user、team、service account 還是 token，再切到指令參考確認 flags、輸出格式與完整範例：

- [access 指令總覽](../../commands/zh-TW/access.md)
- [access user](../../commands/zh-TW/access-user.md)
- [access org](../../commands/zh-TW/access-org.md)
- [access team](../../commands/zh-TW/access-team.md)
- [access service-account](../../commands/zh-TW/access-service-account.md)
- [access service-account token](../../commands/zh-TW/access-service-account-token.md)
- [指令參考](../../commands/zh-TW/index.md)

---
[⬅️ 上一章：告警治理](alert.md) | [🏠 回首頁](index.md) | [➡️ 下一章：Workspace 審查與狀態](status-workspace.md)
