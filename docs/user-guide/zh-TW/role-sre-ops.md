# 🛠️ SRE / 維運角色導讀

這一頁是給值班 SRE、平台維運與 Grafana operator。目標不是背指令，而是先建立一套穩定的維運節奏：先確認 live 現況，再審查 staged 輸入，最後才決定要不要做匯入、套用或回放。

如果你平常會遇到這幾種情況，這頁就是你的起點：

- 維護前要確認現在的 Grafana 狀態到底健不健康
- 收到一包 staged 變更，但還不知道能不能安全套用
- 需要跨 org 盤點 dashboard、datasource、alert 或 access 資產
- 要做備份、漂移檢查、故障排除或 break-glass recovery

先抓住一個核心原則：**SRE 的工作不是「會不會下指令」，而是能不能在正確的 lane 上工作。**

- **live read lane**：先看 live 現況，不改任何東西
- **staged review lane**：先驗 staged input、preview、test
- **apply / replay lane**：確認輸入與 scope 都對了，才進真正的變更或回放

很多事故不是因為指令語法錯，而是因為直接跳過前兩條 lane，太早進第三條。

## 適用對象

- 值班 SRE、平台維運、Grafana operator
- 需要做健康檢查、盤點、備份、回放或漂移比對的人
- 需要在 maintenance window 前後做檢查把關的人
- 需要處理跨 org 可見性、管理員憑證與 break-glass 流程的人

## 主要目標

- 在碰任何變更前，先確認 live readiness、scope 與 auth 能不能覆蓋真實任務範圍
- 建立一條可重複的 operator path：`status` -> `workspace` -> domain command -> `--dry-run` / `apply`
- 讓 dashboard、datasource、alert、access 的維運流程共用同一套判斷模型，而不是每次重新猜
- 把 direct Basic auth、profile、token 的使用邏輯講清楚，避免憑證一開始就選錯

## 採用前後對照

- **以前**：SRE 常靠一串零散指令自行推測 readiness、scope 與 replay 風險；匯出、比對、review、apply 之間也沒有清楚的邊界。
- **現在**：先用可重複的 profile 或明確的 direct auth 做 live check，再走 staged review，最後才進 apply 或 replay。

差異不只是「指令比較整齊」，而是：

- live read 有清楚入口
- staged review 有清楚入口
- destructive / high-impact path 會自然被放到最後

## 成功判準

- 你能在開始前就說出這次工作屬於 live read、staged review，還是 apply / replay
- 你知道目前這張 credential 到底能不能看見你要處理的 org、folder 或 admin scope
- 你能穩定處理 dashboard、alert、access 其中任一條維運路線，而不需要每次重猜流程
- 重大變更前，你會先跑 test、preview、diff 或 `--dry-run`，而不是直接動 live

## 失敗時先檢查

- 如果 token scope 比任務還窄，先停下來換一張真的看得到目標範圍的憑證
- 如果 live check 成功、apply 失敗，先檢查寫入權限、org scope 與 staged input，而不是先怪 renderer 或 formatter
- 如果 export / diff / preview 的結果和預期差很多，先確認你是不是連到錯的 Grafana，或拿錯 bundle / staged root
- 如果你還說不出任務屬於哪條 lane，先回去看工作模型，不要直接進 live mutation

## 維運工作模型

把 `grafana-util` 當成三段式操作面會比較穩：

### 1. Live Read

這條 lane 的目標是回答：

- 現在 Grafana 到底長什麼樣
- 目前 credential 到底看得到什麼
- 這次工作是否真的需要跨 org 或 admin scope

常用入口：

- `status live`
- `status overview live`
- `dashboard browse` / `dashboard list`
- `datasource list`
- `alert list-rules`
- `access org list`

這一段不要急著改東西。先用它把範圍、權限與風險看清楚。

### 2. Staged Review

這條 lane 的目標是回答：

- staged inputs 本身是不是完整、可解析、可預覽
- 這包輸入會影響哪些資產
- 套用前能不能先在本地或 dry-run 裡看出問題

常用入口：

- `workspace scan`
- `workspace test`
- `workspace preview`
- `dashboard diff`
- `alert plan`
- `datasource diff`

這一段是防呆層。真正有經驗的 SRE，花時間最多的通常不是 apply，而是這段 review。

### 3. Apply / Replay

這條 lane 的目標是：

- 做受控的匯入、套用、回放或 restore
- 在明確 scope 下執行 high-impact 操作

常用入口：

- `workspace apply`
- `dashboard import` / `dashboard publish`
- `alert apply`
- `dashboard history restore`

這一段只有在前兩段都清楚時才該進。

## 典型維運任務

### 維護前健康檢查

情境：

- 進 maintenance window 前，先確認目前 host、org 與資產概況
- 不想等到 apply 才發現 token 根本看不到目標

常見做法：

1. `status live --profile prod`
2. `status overview live --profile prod`
3. 若要進 asset lane，再轉到 `dashboard` / `datasource` / `alert` / `access`

### Staged 變更審查

情境：

- 有一包 repo staged inputs，準備做 preview 或 apply
- 想先知道這包變更是否自洽、有沒有缺檔或 scope 錯置

常見做法：

1. `workspace scan .`
2. `workspace test . --fetch-live`
3. `workspace preview . --fetch-live`
4. 必要時再接 domain-specific diff / plan

### 備份與回放

情境：

- 做例行備份
- 事故後要回看當時的 dashboard / alert / access 狀態
- migration 前先把現在的 live content 存下來

常見做法：

- `export dashboard`
- `export datasource`
- `alert export`
- `access ... list --input-dir ...` 或對應 export bundle review

### 漂移與異常調查

情境：

- 現況跟 repo / staged inputs 對不起來
- 某些 dashboard 或 alert 看起來被手動改過

常見做法：

- `dashboard diff`
- `datasource diff`
- `alert diff`
- `dashboard summary` / `dashboard dependencies`

## 建議的連線與秘密資料處理方式

維運工作優先考慮「範圍正確、可重複、能安全交接」，不是只求當下能執行。

1. **日常維運優先用 `--profile`**
   把 URL、帳號與 secret source 收進 repo-local profile，讓值班交接與重跑都一致。

2. **secret 不要直接打在命令列**
   優先用 `password_env`、`token_env`、`os` 或 `encrypted-file`。
   本機引導或緊急操作可改用 `--prompt-password` / `--prompt-token`。

3. **Direct Basic auth 主要用在 break-glass 或 bootstrap**
   特別是：
   - 還沒有 profile
   - 需要管理員等級視野
   - 需要跨 org 或較廣權限的盤點

4. **Token auth 只用在 scope 非常清楚的工作**
   它很適合單一 org 的 read-only automation；不適合你還不知道 scope 的維運工作。

5. **跨 org 與管理層作業，不要先假設 token 夠用**
   `--all-orgs`、service account 管理、某些 access 操作，常會需要更完整的管理員憑證。

## 建議先跑的 6 個指令

下面這組不是「背起來就好」，而是對應一條自然的 operator path：

```bash
# 先看目前 live host 的基本狀態。
grafana-util status live --profile prod --output-format table
```

```bash
# 用人的角度看整體環境，而不是只看低階欄位。
grafana-util status overview live --profile prod --output-format interactive
```

```bash
# 掃描目前這包 staged inputs 到底有哪些 lane。
grafana-util workspace scan .
```

```bash
# 驗 staged inputs 與 live context 能不能對得起來。
grafana-util workspace test . --fetch-live --output-format json
```

```bash
# 在 apply 前先看 preview 結果。
grafana-util workspace preview . --fetch-live --output-format json
```

```bash
# 例行備份或變更前留底。
grafana-util export dashboard --output-dir ./backups --overwrite --progress
```

如果你這次任務主要在存取層，最後一條可換成：

```bash
# 先盤點 org，可快速知道目前 scope 與管理面。
grafana-util access org list --table
```

如果你正在做 break-glass 檢查，Basic auth 通常是比較安全的 fallback：

```bash
# 直接對 host 做廣範圍唯讀檢查。
grafana-util status live --url http://localhost:3000 --basic-user admin --prompt-password --all-orgs --output-format table
```

如果你手上的 token scope 很清楚，也可以只做較窄的唯讀檢查：

```bash
# 適合範圍明確的 read-only 檢查。
grafana-util status overview live --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --output-format json
```

## 什麼叫做處於良好的維運姿勢

你可以用下面幾點檢查自己目前是不是在穩定的維運姿勢裡：

- 你知道目前 credential 到底能不能看見要處理的 org、folder 或 admin scope
- 你能分清楚即時讀取、staged review 與真正 apply 是三種不同流程
- 高風險變更前，你會先跑 input test、preview、diff 或 `--dry-run`
- 問題一旦從 `status` 轉進 `dashboard`、`alert` 或 `access`，你知道要切到哪一頁
- 你留下來的輸出與備份，能讓下一位值班的人接得起來

## 接下來先讀哪些章節

- [Workspace 審查與狀態](status-workspace.md)
- [Dashboard 管理](dashboard.md)
- [Data source 管理](datasource.md)
- [告警治理](alert.md)
- [Access 管理](access.md)
- [疑難排解與名詞解釋](troubleshooting.md)

## 建議同時開著哪些指令頁

- [config](../../commands/zh-TW/config.md)
- [config profile](../../commands/zh-TW/profile.md)
- [status](../../commands/zh-TW/status.md)
- [workspace](../../commands/zh-TW/workspace.md)
- [dashboard](../../commands/zh-TW/dashboard.md)
- [alert](../../commands/zh-TW/alert.md)
- [access](../../commands/zh-TW/access.md)
- [完整指令索引](../../commands/zh-TW/index.md)

## 常見錯誤與限制

- 不要把 `status live` 當成變更前的唯一檢查；`workspace test`、`workspace preview` 與 domain-specific review 仍然要跑
- 不要在匯入或 apply 前略過 `--dry-run`、`plan` 或 diff，尤其是會覆寫既有資產時
- 不要假設 token 一定看得到所有 org；`--all-orgs` 與管理操作很容易因 scope 限制而出現部分結果
- 不要把 `tokens.json`、service-account token 輸出或明文 secret 當一般輸出檔處理
- 不要把「可以讀」誤判成「也可以寫」；讀權限和管理 / 寫入權限是兩回事

## 什麼時候切到更深的文件

- **inventory、export / import、inspect、screenshot** 類問題：切到 [Dashboard 管理](dashboard.md)
- **rule、route、contact point、plan / apply** 類問題：切到 [告警治理](alert.md)
- **org、user、team、service account** 類問題：切到 [Access 管理](access.md)
- **已經知道流程，只差精確旗標**：切到 [指令參考](../../commands/zh-TW/index.md)
- **問題卡在 staged vs live 邊界**：切到 [Workspace 審查與狀態](status-workspace.md)

## 下一步

- [回到手冊首頁](index.md)
- [先看 Workspace 審查與狀態](status-workspace.md)
- [再看 Dashboard 管理](dashboard.md)
- [需要精確旗標時查看指令參考](../../commands/zh-TW/index.md)
