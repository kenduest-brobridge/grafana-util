# alert

## 先判斷你現在要做哪一種事

| 你現在想做的事 | 先開哪個命令頁 | 這頁會幫你回答什麼 |
| --- | --- | --- |
| 想先看目前有哪些規則、聯絡點、靜音時段 | [alert list-rules](./alert-list-rules.md)、[alert list-contact-points](./alert-list-contact-points.md) | 先盤點現況與範圍 |
| 想先把現況匯出、比對或搬移 | [alert export](./alert-export.md)、[alert diff](./alert-diff.md)、[alert import](./alert-import.md) | 先決定搬移或 review 路徑 |
| 想先規劃變更，再決定要不要套用 | [alert plan](./alert-plan.md)、[alert apply](./alert-apply.md) | 先看 planned change，而不是直接改 live |
| 想建立規則、contact point 或通知模板草稿 | [alert new-rule](./alert-new-rule.md)、[alert new-contact-point](./alert-new-contact-point.md)、[alert new-template](./alert-new-template.md) | 先做 authoring，不要直接猜 API payload |
| 想調通知路由 | [alert set-route](./alert-set-route.md)、[alert preview-route](./alert-preview-route.md) | 先看 routing 結果，再決定是否落地 |

## 先選哪一條資料路徑

- **live Grafana**：先用 `list-*` 盤點，必要時再 `export`
- **desired-state / 本地草稿樹**：先用 `plan` 看變更，再決定 `apply`
- **單一規則或 contact point 草稿**：先用 `new-*`、`add-*` 或 `clone-rule`
- **通知路由**：先走 `preview-route`，確認結果後再 `set-route`

## 這個入口是做什麼的

`grafana-util alert` 把告警工作流收在同一個較淺的入口，並依任務把 help 分成盤點、搬移、撰寫與審查幾個區塊：從盤點、匯出、比對，到路由設計、草稿撰寫，再到 plan / apply。這頁適合先搞懂規則、通知路由與 contact point 的關係，再決定要往哪個子命令深入。

目前公開命令不使用舊的 `alert live ...`、`alert migrate ...`、`alert author ...` 分層；請直接使用 `grafana-util alert list-rules`、`grafana-util alert export`、`grafana-util alert init`、`grafana-util alert preview-route` 這類 current path。

## 這一組頁面怎麼讀比較不會亂

1. 先看這頁，判斷你是在做 inventory、authoring、routing，還是 apply。
2. 進到子命令頁後，先看「何時使用」與「最短成功路徑」。
3. 再確認輸入來源是 live、本地 desired-state，還是單一草稿檔。
4. 最後才看完整 flags 與輸出格式。

## 採用前後對照

- **採用前**：告警工作常分散在 UI、臨時 export，或不容易重跑的 shell 指令裡。
- **採用後**：同一個命令群組就能把 inventory、撰寫、diff、規劃與套用放在一起。

## 成功判準

- 你在開始前就能判斷這次 alert 變更屬於盤點、撰寫、路由設計，還是 review / apply
- plan 或 export 可以一路走到 review，而不會把 policy 或 routing context 弄丟
- 同一條流程也能在 CI 或事故回顧時重跑

## 失敗時先檢查

- 如果 inventory 指令抓到的東西比預期少，先確認 auth scope 是否涵蓋需要的 org 或 folder
- 如果 review 或 apply 步驟怪怪的，先看 alert plan JSON，再決定是不是 CLI 真有問題
- 如果結果要交給自動化，請把輸出格式寫清楚，讓下游步驟知道 contract

## 主要旗標

- `--profile`、`--url`、`--token`、`--basic-user`、`--basic-password`
- `--prompt-password`、`--prompt-token`、`--timeout`、`--verify-ssl`
- root `alert` 是 namespace；操作旗標都在 leaf subcommand，例如 `list-rules`、`export`、`preview-route`、`plan`、`apply`

## 範例

```bash
# 先盤點目前有哪些 alert 規則。
grafana-util alert list-rules --profile prod --json
```

```bash
# 先建立一個暫存中的 alert desired-state 樹。
grafana-util alert init --desired-dir ./alerts/desired
```

```bash
# 先把現況匯出，再拿去做 diff 或 review。
grafana-util alert export --url http://localhost:3000 --basic-user admin --basic-password admin --output-dir ./alerts --overwrite
```

```bash
# 先試跑規劃，不要直接套用。
grafana-util alert plan --desired-dir ./alerts/desired --output-format json
```

## 各工作流入口

| 工作流 | 入口頁 | 常見延伸頁 |
| --- | --- | --- |
| 盤點 | [alert list-rules](./alert-list-rules.md) | [alert list-contact-points](./alert-list-contact-points.md)、[alert list-mute-timings](./alert-list-mute-timings.md)、[alert list-templates](./alert-list-templates.md) |
| 搬移 | [alert export](./alert-export.md) | [alert import](./alert-import.md)、[alert diff](./alert-diff.md) |
| 變更前檢查 | [alert plan](./alert-plan.md) | [alert apply](./alert-apply.md) |
| 規則與路由撰寫 | [alert new-rule](./alert-new-rule.md) | [alert add-rule](./alert-add-rule.md)、[alert clone-rule](./alert-clone-rule.md)、[alert preview-route](./alert-preview-route.md)、[alert set-route](./alert-set-route.md) |
