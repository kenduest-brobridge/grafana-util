# Alert 維運人員手冊

這一章不是 `grafana-util alert --help` 的重寫版。它要先回答一個比較實際的問題：當你要改 Grafana Alerting 時，哪些東西會一起被影響，應該先看 live 現況、先整理 desired state，還是先產生 plan 給人 review。

告警的困難不在於多打一條 rule，而在於一條 rule 很少單獨存在。它會連到 folder、rule group、label、contact point、notification policy、template 與 mute timing。改錯其中一段，可能不是「規則沒生效」，而是半夜真的沒有人收到通知。

所以這一章會用工作流來讀 `alert` 子命令：先分清楚盤點、搬移、編寫、路由與套用，再進入單一命令的精確參數。指令參考仍然重要，但它應該是你確認旗標時打開的頁面，不應該取代這裡的操作判斷。

## 適用對象

- 負責 Grafana Alerting 規則、contact point、route、template 或 mute timing 的人。
- 需要先審查告警變更，再套用到 live Grafana 的維運人員。
- 要把 alert 變更接到 Git、review 或 CI 的團隊。

## 先看懂 Alert 物件

`grafana-util alert` 不是只處理 rule。它會把告警拆成幾種互相連動的資產：

| 資產 | 你要先問的問題 | 常用命令 |
| --- | --- | --- |
| Rule | 哪個 folder / rule group 會多一條或改一條規則？條件、labels、annotations 是否完整？ | `list-rules`, `new-rule`, `add-rule`, `clone-rule` |
| Contact point | 通知會送到哪裡？receiver 名稱是否和路由一致？ | `list-contact-points`, `new-contact-point`, `add-contact-point` |
| Route / policy | 哪些 labels 會匹配到哪個 receiver？會不會吃到太寬的 matcher？ | `set-route`, `preview-route` |
| Template | 通知內容是否要一起搬移或建立？ | `list-templates`, `new-template` |
| Mute timing | 什麼時間不通知？搬移時是否也要帶走？ | `list-mute-timings` |
| Desired state | 本地檔案代表你希望 Grafana 變成的狀態。 | `init`, `add-*`, `set-route`, `plan` |
| Raw bundle | 從 live Grafana 匯出的快照，適合備份、搬移與 diff。 | `export`, `import`, `diff` |
| Plan | 將 desired state 與 live Grafana 對照後得到的變更計畫。 | `plan`, `apply` |

這些物件的界線很重要。`raw/` 匯出包適合搬移現況；`desired/` 適合整理你要推進 live 的目標狀態；`plan` 則是 apply 前的審查證據。把三者混在一起，最容易造成「看起來只是改一條 rule，實際上動到 route 或 delete 候選」的問題。

## 主要目標

- 在動 live Grafana 前先知道現況。
- 把要變更的 alert state 變成本地可讀、可 review 的檔案。
- 用 plan 說清楚 create / update / delete / noop，而不是直接 apply。
- 在搬移或恢復時，分清楚 raw bundle 與 desired-state workflow。

## 採用前後對照

| 採用前 | 採用後 |
| :--- | :--- |
| 告警變更只看單一 rule，contact point、route、template 與 mute timing 之後才補查。 | 把 rule、receiver、route、template、mute timing、desired state、raw bundle 與 plan 當成同一組可審查輸入。 |
| 套用靠記憶或剛產生的命令直接執行。 | 先產生 inventory 或 plan，確認審查成品後才 apply。 |

## 成功判準

- 你能說清楚這次工作屬於 inventory、backup、authoring、routing 還是 review/apply。
- 你知道每個 subcommand 吃的是 live Grafana、raw bundle、desired dir，還是 reviewed plan。
- 在 `apply` 前，你已經看過 plan，且知道 `--prune` 是否會產生 delete 候選。

## Alert 工作流地圖

| 你現在要做的事 | 起點 | 主要輸入 | 主要輸出 | 下一步 |
| --- | --- | --- | --- | --- |
| 盤點 live alert 現況 | `list-rules`, `list-contact-points`, `list-mute-timings`, `list-templates` | Grafana 連線與權限 | 表格、YAML 或 JSON inventory | 決定是否 export 或 authoring |
| 備份或搬移現況 | `export` | live Grafana | `raw/` bundle | `diff`, `import`, review |
| 比對匯出包與 live | `diff` | `raw/` bundle + live Grafana | 差異摘要或 JSON | 決定是否 import 或重新 export |
| 重建匯出包 | `import` | `raw/` bundle | live Grafana mutation 或 dry-run | dry-run 後才實際匯入 |
| 建立 desired state | `init`, `new-*`, `add-*`, `clone-rule`, `set-route` | 本地 desired dir | 可 review 的 staged files | `preview-route`, `plan` |
| 預覽 routing | `preview-route` | desired dir + labels | matcher / receiver 預覽 | 修正 route 或進入 plan |
| 產生變更計畫 | `plan` | desired dir + live Grafana | plan JSON / text | 人工 review |
| 套用已審查計畫 | `apply` | reviewed plan file | live Grafana mutation 結果 | apply 後再 inventory |

## 盤點：先知道 live 裡有什麼

如果你還不確定 Grafana 目前有哪些告警資產，先不要 export、import 或 apply。從 inventory 開始：

```bash
# 先看規則，確認 folder、rule group 與 labels 是否符合預期。
grafana-util alert list-rules --profile prod --output-format table
```

```bash
# 看 contact point，確認 receiver 名稱是否能對上 route。
grafana-util alert list-contact-points --profile prod --output-format yaml
```

```bash
# 看通知模板與 mute timing，避免搬移時漏掉通知內容或靜音設定。
grafana-util alert list-templates --profile prod --output-format table
grafana-util alert list-mute-timings --profile prod --output-format table
```

如果 inventory 少了你預期的資源，先查權限與 org scope。不要急著用 `import` 或 `apply` 補東西，因為你可能只是沒有看見完整範圍。

`list-rules` 的目的不是單純把規則印出來。它是 alert 變更前後的第一個 sanity check。看輸出時，先確認 rule name、folder、rule group、org scope、labels 與 receiver 相關欄位；如果你準備搬移規則，也要留意 dashboard UID / panel id 的關聯是否還能在目標環境成立。人工 review 優先用 `--output-format table`，要交給 CI 或保存成 artifact 時改用 JSON / YAML。

盤點完的下一步取決於你看到什麼：如果只是要保留現況，接 `alert export`；如果要從既有 rule 衍生新規則，接 `clone-rule`；如果 receiver 或 route 名稱對不上，接 `list-contact-points` 與 `preview-route`；如果盤點是 apply 後驗證，應該再跑一次 `plan` 或至少保存 apply 前後的 inventory。

## 搬移：raw bundle 是現況快照

`alert export` 會把 live Grafana 的 alert 資源匯出成 `raw/` JSON。這條路徑適合備份、搬移、比對與災難恢復，不等同於 desired-state authoring。

```bash
# 匯出 live alert state，建立可保存的 raw bundle。
grafana-util alert export --profile prod --output-dir ./alerts --overwrite
```

```bash
# 匯入前先比對 raw bundle 與目標 Grafana。
grafana-util alert diff --profile prod --diff-dir ./alerts/raw --output-format json
```

```bash
# 先 dry-run，確認會匯入或更新哪些資源。
grafana-util alert import \
  --profile prod \
  --input-dir ./alerts/raw \
  --replace-existing \
  --dry-run --json
```

如果 alert rule 有 dashboard / panel 關聯，而且來源與目標環境的 UID 或 panel id 不同，先準備 `--dashboard-uid-map` 與 `--panel-id-map`。這類 mapping 應該在 diff / import / plan 前就決定，不要等 apply 後才補救。

## 編寫：desired state 是變更意圖

Authoring 路徑的重點是：你先在本地建立想要的 alert state，等 review 完再讓 live Grafana 改變。

```bash
# 建立 desired-state 目錄。
grafana-util alert init --desired-dir ./alerts/desired
```

如果你只是要一個低階骨架，先用 `new-*`：

```bash
# 建立一個 rule 骨架，之後再補細節。
grafana-util alert new-rule --desired-dir ./alerts/desired --name cpu-main
```

如果你要一次建立較完整的 rule，使用 `add-rule`。這條路徑比較適合日常維運，因為可以同時帶上 folder、rule group、receiver、severity、threshold、labels 與 annotations。

```bash
# 建立完整一點的 staged rule，仍然不碰 live Grafana。
grafana-util alert add-rule \
  --desired-dir ./alerts/desired \
  --name cpu-high \
  --folder platform-alerts \
  --rule-group cpu \
  --receiver pagerduty-primary \
  --severity critical \
  --expr 'A' \
  --threshold 80 \
  --above \
  --for 5m \
  --label team=platform \
  --annotation summary='CPU high'
```

如果你要從既有規則衍生一條新規則，用 `clone-rule`；如果要建立 contact point 或 template 草稿，使用 `new-contact-point` / `new-template`；如果要較高階地建立 contact point，使用 `add-contact-point`。

## 路由：先 preview，再 set-route

Routing 最容易出現「語法正確但通知送錯地方」的問題。先用 `preview-route` 看 matcher 輸入，再用 `set-route` 寫入受工具管理的 route。

```bash
# 先預覽 labels 會如何被路由輸入解讀。
grafana-util alert preview-route \
  --desired-dir ./alerts/desired \
  --label team=platform \
  --severity critical
```

```bash
# 確認後再寫入受管理 route。
grafana-util alert set-route \
  --desired-dir ./alerts/desired \
  --receiver pagerduty-primary \
  --label team=platform \
  --severity critical
```

`set-route` 是替換受管理路由，不是任意 merge 手工 route tree。若你要保留複雜既有 policy，先 export / diff，看清楚 live policy，再決定是否改成 desired-state 管理。

## Review / Apply：plan 是最後關卡

`plan` 是 desired state 與 live Grafana 之間的審查文件。它應該在 review 裡被保存、討論，然後才交給 `apply`。

```bash
# 產生 plan，不直接碰 live mutation。
grafana-util alert plan \
  --profile prod \
  --desired-dir ./alerts/desired \
  --prune \
  --output-format json
```

讀 plan 時至少看這幾件事：

- `create`：desired state 有，但 live Grafana 沒有。
- `update`：兩邊都有，但內容不同。
- `delete`：使用 `--prune` 時，live 有而 desired state 沒有。
- `noop`：兩邊一致。
- `blocked`：工具判斷不應直接套用的項目。

`--prune` 很有用，但也最危險。它代表「desired state 沒有列到的 live 資源可以被視為刪除候選」。第一次導入時，建議先不帶 `--prune` 跑一次，再帶 `--prune` 對照差異。

```bash
# 只套用已審查、已保存的 plan。
grafana-util alert apply \
  --profile prod \
  --plan-file ./alert-plan-reviewed.json \
  --approve \
  --output-format json
```

`apply` 不應該直接吃剛產生但沒 review 的臨時輸出。實務上，plan file 應該進 PR、變更單或至少被保存成一個可追溯 artifact。

## 什麼時候切到指令參考

這一章負責流程判斷。當你已經知道要用哪個 subcommand，再打開指令參考確認 flags：

- [alert 指令總覽](../../commands/zh-TW/alert.md)
- [alert list-rules](../../commands/zh-TW/alert-list-rules.md)
- [alert list-contact-points](../../commands/zh-TW/alert-list-contact-points.md)
- [alert list-mute-timings](../../commands/zh-TW/alert-list-mute-timings.md)
- [alert list-templates](../../commands/zh-TW/alert-list-templates.md)
- [alert export](../../commands/zh-TW/alert-export.md)
- [alert import](../../commands/zh-TW/alert-import.md)
- [alert diff](../../commands/zh-TW/alert-diff.md)
- [alert new-rule](../../commands/zh-TW/alert-new-rule.md)
- [alert add-rule](../../commands/zh-TW/alert-add-rule.md)
- [alert clone-rule](../../commands/zh-TW/alert-clone-rule.md)
- [alert new-contact-point](../../commands/zh-TW/alert-new-contact-point.md)
- [alert add-contact-point](../../commands/zh-TW/alert-add-contact-point.md)
- [alert new-template](../../commands/zh-TW/alert-new-template.md)
- [alert preview-route](../../commands/zh-TW/alert-preview-route.md)
- [alert set-route](../../commands/zh-TW/alert-set-route.md)
- [alert plan](../../commands/zh-TW/alert-plan.md)
- [alert apply](../../commands/zh-TW/alert-apply.md)
- [alert delete](../../commands/zh-TW/alert-delete.md)

## 失敗時先檢查

- 如果 inventory 看不到預期資源，先確認 profile、token 權限、org scope 與 folder 權限。
- 如果 export / import 少了 dashboard-linked rule，先確認 dashboard UID 與 panel id mapping。
- 如果 plan 比預期多出 delete，先移除 `--prune` 對照一次。
- 如果 preview-route 結果空白，先確認 label key/value 與 receiver 名稱，不要直接 apply。
- 如果 apply 結果和 plan 不一致，先確認 reviewed plan 是否仍對應目前 live Grafana；中間若有人改過 UI，應重新 plan。

---
[⬅️ 上一章：Data source 管理](datasource.md) | [🏠 回首頁](index.md) | [➡️ 下一章：Access 管理](access.md)
