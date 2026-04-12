# Data source 維運手冊

這一章整理 data source 的盤點、備份、回放與受控變更。重點是讓你知道哪些資料適合進 Git，哪些資料只適合在恢復時補回，也能先看 live Grafana 或本地 bundle 的 inventory。

Data source 看起來像設定表，實際上是 dashboard、alert 與查詢路徑共同依賴的入口。改一個 UID、type、URL 或 default datasource，影響可能會穿過很多 dashboard 與告警。更麻煩的是，真正能連線的 secret 通常不能被直接寫進 Git，所以備份、回放與 provisioning 不能被混成同一件事。

這章的讀法是：先把 data source 當成「可被引用的基礎資產」來盤點，再決定你要的是恢復包、provisioning 投影，還是 live mutation。先做這個判斷，後面的 export/import/diff 才不會變成只是在搬 JSON。

## 適用對象

- 負責 data source 盤點、同步與恢復的人
- 需要把 data source 資產接進 Git 或 provisioning 流程的人
- 需要先做 dry-run，再決定要不要套用的人

## 主要目標

- 先看懂 live data source 或本地 bundle 長什麼樣
- 再建立可回放的輸出樹
- 最後才進行匯入、修改或刪除

好的 data source 流程應該讓你在變更前就知道三件事：誰引用它、哪些欄位可以安全留存、哪些 secret 必須透過恢復機制補回。

## 採用前後對照

- 以前：data source 變更常被當成單次設定修改，回復方式與還原層級不夠清楚。
- 現在：先盤點、再做遮蔽式還原、再投影到 provisioning，最後才做 dry-run 或實際變更。

## 成功判準

- 你知道哪些欄位要放進恢復包，哪些欄位一定要遮蔽。
- 你能先確認 live inventory 或本地匯出 bundle，再決定要不要動它。
- 你能分清楚自己是在處理 recovery、provisioning，還是直接 live mutation。

## 失敗時先檢查

- 如果回放包裡出現明文 secret，先停下來修匯出路徑，不要先存檔。
- 如果匯入預覽跟你以為的 live data source 對不上，先檢查 UID 與 type mapping。
- 如果 provisioning 投影跟恢復包不一致，先確認你到底要的是哪一條 lane。

> **維運目標**：確保 data source 設定可以安全地備份、比對與回放，並透過 **Masked Recovery（遮蔽式還原）** 保護敏感憑證。

## Data source 工作流地圖

Data source 子命令的差異，主要在於它讀的是 live Grafana、本地 bundle，還是準備要寫回 live：

| 任務 | 起點 | 主要輸入 | 主要輸出 | 下一步 |
| --- | --- | --- | --- | --- |
| 確認支援型別 | `types` | CLI 內建型別表 | type 與必要欄位 | 決定 add / modify 欄位 |
| 盤點 live 或本地 bundle | `list`, `browse` | live Grafana 或 `--input-dir` | UID、type、URL、default 狀態 | export、diff 或 review |
| 備份與恢復 | `export`, `import` | live Grafana 或 `datasources.json` | masked recovery bundle / dry-run | dry-run 後 import |
| 比對漂移 | `diff` | 本地 bundle + live Grafana | create / update / delete 差異 | 修 bundle 或 import |
| 直接 live 變更 | `add`, `modify`, `delete` | flags + live Grafana | dry-run 或 mutation 結果 | list / diff 後驗證 |
| 產生 provisioning 投影 | `export` | live Grafana | `provisioning/datasources.yaml` | 交給 Grafana provisioning lane |

`datasources.json` 是恢復與 diff 的主資料；`provisioning/datasources.yaml` 是部署投影。前者用來說明「要恢復什麼」，後者用來配合 Grafana 的檔案式 provisioning。不要把 provisioning YAML 當成唯一真相來源。

## 核心工作流用途

data source 這組功能主要是為了這幾種場景設計：
- **資產盤點**：稽核現有的 data source、其類型以及後端 URL，來源可以是 live Grafana 或本地 bundle。
- **恢復與回放**：維護可供災難恢復的 data source 匯出紀錄。
- **Provisioning 投影**：產生 Grafana 檔案式配置系統所需的 YAML 檔案。
- **差異審查 (Drift Review)**：在套用變更前，比對本地暫存檔案與 live Grafana。
- **受控變更**：在 Dry-run 保護下新增、修改或刪除 live 的 data source。

---

## 工作流程邊界

data source 匯出會產生兩個主要輸出物，各自負責不同的用途：

| 檔案 | 用途 | 最佳使用場景 |
| :--- | :--- | :--- |
| `datasources.json` | **Masked Recovery（遮蔽式還原）** | 標準回放合約。用於還原、回放與差異比對。 |
| `provisioning/datasources.yaml` | **Provisioning 投影** | 模擬 Grafana 檔案配置系統所需的磁碟結構。 |

**重要提示**：請始終把 `datasources.json` 視為真正的恢復來源。Provisioning YAML 只是從恢復包衍生出來的次要投影。

---

## 盤點：先確認 type、UID 與 default

Data source 變更要先從 inventory 開始。`datasource types` 讓你確認工具知道哪些 type 與必要欄位；`datasource list` 讓你看 live 或本地 bundle 的 UID、type、URL 與 default 狀態；`datasource browse` 適合用較接近瀏覽的方式看本地輸出樹。

盤點時最重要的是 UID 與 type。UID 是 dashboard、alert 與 provisioning 會引用的穩定身份；type 決定 Grafana 要用哪個 plugin 讀它。default datasource 看似只是 UI 預設值，但很多 dashboard 變數與 panel 查詢會受它影響。若 list 結果和預期不同，先查 org scope、profile、token 權限與本地 bundle 來源，不要直接 add 一個看起來缺少的 data source。

## 備份與回放：datasources.json 才是主契約

`datasource export` 的主產物是 `datasources.json`。它保留足夠的結構讓你 diff、dry-run import 或做災難恢復，同時避免把 secret 明文放進 Git。`provisioning/datasources.yaml` 則是部署投影，適合交給 Grafana provisioning lane，但不應該變成 review 的唯一來源。

匯入前一定先跑 dry-run。dry-run 裡的 create / update 才是這次回放真正會造成的變化；檔案存在不代表 live 一定會照你想像更新。如果 dry-run 顯示的 UID、name 或 type 對不上，先修 bundle 或 mapping，不要靠 import 去猜。

## Diff：先看漂移，再決定要修誰

`datasource diff` 是用來回答「本地 bundle 和 live Grafana 哪邊偏了」。如果本地是你認定的來源，diff 後通常接 import；如果 live 是被人工 hotfix 過的現況，diff 後應該先重新 export 或更新 review 文件。不要把 diff 當成 apply，它只是讓你知道兩邊差在哪裡。

Diff 特別適合在 dashboard 或 alert 變更前使用。當 dashboard 依賴的 UID 不存在，或 default data source 不一致，問題通常會在 dashboard apply 後才爆出來。先 diff data source，可以提早發現這類環境漂移。

## Live mutation：add / modify / delete 要當成例外

`datasource add`、`modify`、`delete` 是直接碰 live Grafana 的工具，適合小範圍修正、break-glass 或明確的維運變更。日常流程仍應優先走 export / diff / import，讓變更可以被 review。

如果必須 live mutation，先用 `--dry-run`，再用 `list` 或 `diff` 驗證結果。刪除 data source 前，先確認 dashboard、alert rule 與 provisioning 不再引用該 UID。工具可以幫你執行 mutation，但不會替你判斷上游依賴是否已經改完。

## 何時切到指令參考

這一章負責幫你決定 data source 工作流。當你已經知道要使用哪個 command，再切到指令參考確認 flags、輸出格式與完整範例：

- [datasource 指令總覽](../../commands/zh-TW/datasource.md)
- [datasource types](../../commands/zh-TW/datasource-types.md)
- [datasource browse](../../commands/zh-TW/datasource-browse.md)
- [datasource list](../../commands/zh-TW/datasource-list.md)
- [datasource export](../../commands/zh-TW/datasource-export.md)
- [datasource import](../../commands/zh-TW/datasource-import.md)
- [datasource diff](../../commands/zh-TW/datasource-diff.md)
- [datasource add](../../commands/zh-TW/datasource-add.md)
- [datasource modify](../../commands/zh-TW/datasource-modify.md)
- [datasource delete](../../commands/zh-TW/datasource-delete.md)
- [指令參考](../../commands/zh-TW/index.md)

---

## 閱讀即時資產盤點

使用 `datasource list` 驗證目前 Grafana 的外掛與目標狀態。

```bash
# 使用 datasource list 驗證目前 Grafana 的外掛與目標狀態。
grafana-util datasource list \
  --url http://localhost:3000 \
  --basic-user admin \
  --basic-password admin \
  --table
```

**範例輸出：**
```text
UID             NAME        TYPE        URL                     IS_DEFAULT  ORG  ORG_ID
--------------  ----------  ----------  ----------------------  ----------  ---  ------
dehk4kxat5la8b  Prometheus  prometheus  http://prometheus:9090  true             1
```

**如何解讀：**
- **UID**：用於自動化的穩定身份識別。
- **TYPE**：識別外掛實作 (例如 prometheus, loki)。
- **IS_DEFAULT**：標示這是否為該 org 的預設 data source。
- **URL**：該紀錄關聯的後端目標位址。

---

## 常用指令

| 指令 | 帶有參數的完整範例 |
| :--- | :--- |
| **盤點 (List)** | `grafana-util datasource list --all-orgs --table` 或 `grafana-util datasource list --input-dir ./datasources --table` |
| **匯出 (Export)** | `grafana-util datasource export --output-dir ./datasources --overwrite` |
| **匯入 (Import)** | `grafana-util datasource import --input-dir ./datasources --replace-existing --dry-run --table` |
| **比對 (Diff)** | `grafana-util datasource diff --input-dir ./datasources` |
| **新增 (Add)** | `grafana-util datasource add --uid <UID> --name <NAME> --type prometheus --datasource-url <URL> --dry-run --table` |

---

## 操作範例

### 1. 匯出盤點資產
```bash
# 匯出 data source inventory 與 provisioning 投影。
grafana-util datasource export --output-dir ./datasources --overwrite
```
**範例輸出：**
```text
Exported datasource inventory -> datasources/datasources.json
Exported metadata            -> datasources/export-metadata.json
Datasource export completed: 3 item(s)
```

### 2. Dry-Run 匯入預覽
```bash
# 匯入前先預覽會 create 還是 update。
grafana-util datasource import --input-dir ./datasources --replace-existing --dry-run --table
```
**範例輸出：**
```text
UID         NAME               TYPE         ACTION   DESTINATION
prom-main   prometheus-main    prometheus   update   existing
loki-prod   loki-prod          loki         create   missing
```
- **ACTION=create**：將建立新的 data source 紀錄。
- **ACTION=update**：將取代現有的紀錄。
- **DESTINATION=missing**：Grafana 目前沒有這個 UID，因此匯入時會建立新紀錄。
- **DESTINATION=existing**：Grafana 目前已經有這個 UID，因此匯入時會覆蓋既有 data source 紀錄。

### 3. 直接即時新增 (Dry-Run)
```bash
# 先 dry-run 一個 live add，不立即寫入 Grafana。
grafana-util datasource add \
  --uid prom-main --name prom-new --type prometheus \
  --datasource-url http://prometheus:9090 --dry-run --table
```
**範例輸出：**
```text
INDEX  NAME       TYPE         ACTION  DETAIL
1      prom-new   prometheus   create  would create datasource uid=prom-main
```

### 4. 本地盤點
```bash
# 從剛匯出的本地套件讀取 data source inventory。
grafana-util datasource list --input-dir ./datasources --table
```
**範例輸出：**
```text
UID             NAME        TYPE        URL                     IS_DEFAULT  ORG  ORG_ID
--------------  ----------  ----------  ----------------------  ----------  ---  ------
dehk4kxat5la8b  Prometheus  prometheus  http://prometheus:9090  true             1
```
- **UID**：用於自動化的穩定身份識別碼。
- **TYPE**：識別外掛實作 (例如 prometheus, loki)。
- **IS_DEFAULT**：標示這是否為該 org 的預設 data source。
- **URL**：該紀錄關聯的後端目標位址。

---
[⬅️ 上一章：Dashboard 管理](dashboard.md) | [🏠 回首頁](index.md) | [➡️ 下一章：告警治理](alert.md)
