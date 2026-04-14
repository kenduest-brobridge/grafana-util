# Workspace 審查與狀態

這一章聚焦在 workspace apply 前後的最後檢查，幫你確認目前狀態、差異與套用前的準備是否到位。

在實際維運裡，最危險的時刻通常不是按下 apply，而是 apply 之前那段模糊地帶：你知道手上有一包檔案，也知道要往某個 Grafana 環境推，但還不能確定這包東西是不是完整、是不是過期、是不是會改到超出預期的範圍。這一章就是為了處理那段模糊地帶。

讀這章時，不要急著找哪個指令可以「完成部署」。先把問題拆成三個判斷：live Grafana 現況能不能信任、staged workspace 本身能不能解析、preview 出來的變更是不是你願意承擔的結果。三個答案都清楚後，apply 才只是最後一步。

## 適用對象

- 要先看 live / staged 狀態再決定下一步的人
- 負責 workspace 審查、test 或 apply gate 的人
- 需要把 status / workspace / config profile 串成固定流程的人

## 主要目標

- 先區分 live 與 staged
- 再確認 workspace package、輸入結構與差異摘要
- 最後才進入 apply

換句話說，這章不是部署教學，而是部署前的判斷框架。它幫你把「我覺得應該可以」換成「我已經看過 live、staged 與 preview，而且知道下一步會改到哪些內容」。

## 採用前後對照

- 以前：status、snapshot 與 workspace review 常常像是三套名稱接近但分工不清的工具。
- 現在：即時檢查、staged 審查與快照式總覽被放進同一條導引路線裡。

## 成功判準

- 你能在 workspace apply 前，先判斷這章是在處理整備度、快照還是審查。
- 你知道流程從 status 進到 mutation 時，應該切到哪一個 command。
- 你能說清楚 workspace apply 前應該先做哪些檢查。

## 失敗時先檢查

- 如果 staged 與 live 看的不是同一個面，先停下來確認哪一條 lane 過期。
- 如果 snapshot 或 summary 跟預期不符，先把它當成流程警訊，不要只當成排版問題。
- 如果你說不出為什麼需要看這章，可能代表你走錯 lane 了。

## Status / Workspace 工作流地圖

Status 與 workspace 子命令的重點是「先證明狀態可以信任，再決定要不要 apply」：

| 任務 | 起點 | 主要輸入 | 主要輸出 | 下一步 |
| --- | --- | --- | --- | --- |
| 檢查 live Grafana | `status live`, `status overview live` | Grafana 連線、profile、可選 staged 摘要 | ready / warning / blocking 狀態 | export、review 或停下來修 |
| 檢查 staged package | `status staged`, `workspace scan`, `workspace test` | 本地 workspace | schema / package / policy 結果 | preview |
| 預覽即將套用內容 | `workspace preview` | staged workspace + target profile | apply 前摘要 | 人工 review |
| 套用 workspace | `workspace apply` | 已審查 workspace | live mutation 結果 | apply 後 status live |
| 打包與交接 | `workspace package`, `workspace ci` | 本地 workspace | bundle / CI artifact | 交給 CI 或下一環境 |
| 產生快照證據 | `status snapshot`, `snapshot export`, `snapshot review` | live 或 staged state | snapshot artifact / review output | incident、PR 或 audit |

如果你的問題是「目前 Grafana 能不能信任」，先看 `status live`。如果問題是「這包檔案能不能套用」，先看 `workspace scan/test/preview`。如果你要留下審查證據，才切到 snapshot。

## 狀態操作面

這裡會區分 **Live**（目前 Grafana 上真的在跑的內容）和 **Staged**（你準備要部署的內容）。

### 1. 即時整備度檢查 (Live Check)
```bash
# 用表格快速看 live Grafana 目前是否 ready。
grafana-util status live --output-format table
```

```bash
# 帶入 staged 摘要檔，讓 live 檢查同時提供更多對照資訊。
grafana-util status live --profile prod --sync-summary-file ./sync-summary.json --package-test-file ./workspace-package-test.json --output-format json
```
**預期輸出：**
```text
OVERALL: status=ready

COMPONENT    HEALTH   REASON
Dashboards   ok       32/32 可存取
Datasources  ok       秘密資訊恢復驗證通過
Alerts       ok       無孤立規則
```
`status live` 走的是共用的 live 狀態檢查流程。若同時帶入 staged sync 檔案，就能在不改變指令用法的前提下，讓 live 視圖多出更多對照資訊。

### 2. 暫存整備度檢查 (Staged Check)
在執行 `apply` 之前，這一步很適合拿來當 CI/CD 的強制檢查。
```bash
# 在 CI 裡檢查 desired file 是否能繼續往 apply 走。
grafana-util status staged --desired-file ./desired.json --output-format json
```

```bash
# 同時檢查 dashboard、alert 與 desired input 的 staged readiness。
grafana-util status staged --dashboard-export-dir ./dashboards/raw --alert-export-dir ./alerts --desired-file ./desired.json --output-format table
```
**預期輸出：**
```json
{
  "status": "ready",
  "blockers": [],
  "warnings": ["1 個儀表板缺少唯一的目錄分配"]
}
```
`status staged` 比較偏向給腳本或 CI 判讀的驗證結果。`blockers` 代表一定得先處理，`warnings` 則表示需要人工再多看一眼。

---

## Workspace 審查生命週期

管理從 Git 到正式 Grafana 環境的過渡。

Workspace 這組命令處理的是「一包 staged files 能不能進下一個環境」。它不應該一開始就被看成 apply 工具。`scan` 先回答這包裡有什麼，`test` 回答結構能不能被工具理解，`preview` 回答即將改到什麼，`apply` 才是最後動作。

如果 preview 的變更數量、operation order 或 blocked reason 和你預期不同，先回頭修 workspace，不要用 apply 去驗證猜測。Workspace review 的產物應該能被 PR、incident 或變更單引用，而不是只留在終端畫面。

### 第一次使用，先走這條最短路徑

如果你還不確定要從哪裡開始，先照這個順序走：

1. `workspace scan .`
2. `workspace test .`
3. `workspace preview . --fetch-live --profile <profile>`
4. `workspace apply --preview-file ./workspace-preview.json --approve --execute-live --profile <profile>`

workspace 路徑是最短路徑，因為 `workspace` 會先嘗試在目前 repo 或工作目錄裡找常見 staged inputs，包含同一個 mixed repo root 裡的 Git Sync dashboards、`alerts/raw`、`datasources/provisioning`。若這不符合你的目錄布局，再改用 `--desired-file`、`--dashboard-export-dir`、`--alert-export-dir`、`workspace package`、`--target-inventory` 這些明確旗標。

混合 workspace tree 範例：

```text
./grafana-oac-repo/
  dashboards/git-sync/raw/
  dashboards/git-sync/provisioning/
  alerts/raw/
  datasources/provisioning/datasources.yaml
```

### 1. Workspace 掃描
先看目前 workspace package 的高階摘要與輸入形狀。
```bash
# 先從同一個 mixed repo root 自動發現常見 staged inputs。
grafana-util workspace scan ./grafana-oac-repo
```

同一個 workspace root 可以同時包含 `dashboards/git-sync/raw`、`dashboards/git-sync/provisioning`、`alerts/raw` 與 `datasources/provisioning/datasources.yaml`。

```bash
# 用明確 staged 匯出目錄建立 inspection 輸出。
grafana-util workspace scan --dashboard-export-dir ./dashboards/raw --alert-export-dir ./alerts/raw --output-format json
```
**預期輸出：**
```text
WORKSPACE PACKAGE SUMMARY:
- dashboards: 5 modified, 2 added
- alerts: 3 modified
- access: 1 added
- total impact: 11 operations
```
先用 scan 看整個 workspace package 的規模與輸入形狀，再往下看 preview。若總數異常偏大，應先停下來檢查 staged 輸入。

### 2. Workspace 測試
驗證匯出 / 匯入目錄結構與 staged readiness。
```bash
# 先檢查目前 mixed workspace 自動發現到的 staged package。
grafana-util workspace test ./grafana-oac-repo --availability-file ./availability.json
```

```bash
# 把 live availability hints 併進 staged 檢查。
grafana-util workspace test ./grafana-oac-repo --fetch-live --output-format json
```
**預期輸出：**
```text
PREFLIGHT CHECK:
- dashboards: valid (7 files)
- datasources: valid (1 inventory found)
- result: 0 errors, 0 blockers
```
test 適合放在 preview 或 apply 前，做 staged readiness 與結構層檢查。通過只代表輸入形狀合理，不代表 live 狀態已經完全吻合。

### 3. Workspace 預覽
先建立可操作的 preview，確認這次真的會改到哪些東西。
```bash
# 預覽目前 mixed workspace 對 live Grafana 的影響。
grafana-util workspace preview ./grafana-oac-repo --fetch-live --profile prod
```

```bash
# 用明確 desired/live 輸入產出 JSON preview。
grafana-util workspace preview --desired-file ./desired.json --live-file ./live.json --output-format json
```

preview 對應底層 plan contract。對使用者來說，先想「這次會改到什麼」比先想「我要 build 哪種 plan 文件」更自然。
這份 preview contract 也是排序契約的公開面：`ordering.mode`、每筆 operation 的 `orderIndex` / `orderGroup` / `kindOrder`，以及 `summary.blocked_reasons` 會讓審查者看出 plan 的執行順序與尚未解除的受阻工作。

如果同一個 mixed workspace root 最後要交接成 bundle，直接跑 `workspace package ./grafana-oac-repo --output-file ./workspace-package.json`，保留產生的 `workspace-package.json` 作為可攜式的 review artifact。

---

## 互動模式 (TUI) 語意

`status overview live --output-format interactive` 會透過共用的 status live 路徑顯示 live project overview。

```bash
# status overview live --output-format interactive 會透過共用的 status live 路徑顯示 live project overview。
grafana-util status overview live --url http://localhost:3000 --basic-user admin --basic-password admin --output-format interactive
```

TUI 使用以下視覺語言：
- **🟢 綠色**：組件健康且完全可達。
- **🟡 黃色**：組件可用，但有警告，例如缺少中繼資料。
- **🔴 紅色**：組件受阻，在進行任何部署前都需要處理。

如果要看 staged 產物的人工審查畫面，用不帶 `live` 的 `status overview`；如果要拿結構化輸出做 live 檢查，改用 `status live`。

## Snapshot：用來留下證據，不是取代 preview

Snapshot 相關命令適合在 incident、PR 或 audit 需要證據時使用。它可以保存目前 live 或 staged state 的摘要，讓後續 review 有一份可以引用的 artifact。它不是 workspace preview 的替代品，因為 preview 仍然是回答「apply 會做什麼」的主要入口。

如果你要判斷能不能 apply，先用 `workspace scan/test/preview`。如果你要把當下狀態交給別人 review，或要把變更前後狀態放進紀錄，才使用 `status snapshot`、`snapshot export` 或 `snapshot review`。

## 何時切到指令參考

這一章負責幫你判斷 live、staged、preview、apply 與 snapshot 的位置。當你已經知道要使用哪條 lane，再切到指令參考確認 flags、輸出格式與完整範例：

Primary lane：

- [workspace](../../commands/zh-TW/workspace.md)
- [workspace scan](../../commands/zh-TW/workspace-scan.md)
- [workspace test](../../commands/zh-TW/workspace-test.md)
- [workspace preview](../../commands/zh-TW/workspace-preview.md)
- [workspace apply](../../commands/zh-TW/workspace-apply.md)
- [status staged](../../commands/zh-TW/status.md#staged)
- [status live](../../commands/zh-TW/status.md#live)
- [status overview live](../../commands/zh-TW/status.md#overview)

Advanced workflows：

- 如果你需要較低階 staged contract，或要看 bundle / promotion handoff 文件，從 [workspace ci](../../commands/zh-TW/workspace.md#ci) 或 [指令詳細總索引](../../commands/zh-TW/index.md) 開始。
- [snapshot](../../commands/zh-TW/snapshot.md)
- [snapshot export](../../commands/zh-TW/snapshot.md#export)
- [snapshot review](../../commands/zh-TW/snapshot.md#review)
- [config profile](../../commands/zh-TW/profile.md)
- [config profile list](../../commands/zh-TW/profile.md#list)
- [config profile show](../../commands/zh-TW/profile.md#show)
- [config profile add](../../commands/zh-TW/profile.md#add)
- [config profile example](../../commands/zh-TW/profile.md#example)
- [config profile init](../../commands/zh-TW/profile.md#init)

---
[⬅️ 上一章：Access 管理](access.md) | [🏠 回首頁](index.md) | [➡️ 下一章：維運情境手冊](scenarios.md)
