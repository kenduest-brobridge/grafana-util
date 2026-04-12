# config

## Root

用途：說明 `grafana-util config` 這個 namespace 是做什麼的、什麼時候停在 root 看整體設定模型，以及什麼時候該進 `config profile`。

適用時機：
- 當你要先理解設定模型，再決定要進哪個 profile 子命令
- 當你正在判斷 direct flags 與 repo-local `--profile` 的分工
- 當你想先理解 URL、auth 預設值與 secret 應該放在哪裡

主要旗標：
- root 頁是 namespace 入口說明；實際操作旗標都在 `config profile`

範例：

```bash
# 先看 config namespace 的整體入口。
grafana-util config --help
```

```bash
# 在目前 checkout 初始化一份 starter config。
grafana-util config profile init --overwrite
```

## 先判斷你現在要做哪一種事

| 你現在想做的事 | 先開哪個命令頁 | 這頁會幫你回答什麼 |
| --- | --- | --- |
| 想建立 repo-local 連線設定，之後少打一堆參數 | [config profile](./profile.md) | 怎麼建立、驗證與重複使用 profile |
| 想確認目前 checkout 內的 profile 長怎樣 | [config profile](./profile.md#show) | 先看解析結果，不要直接猜設定值 |
| 想確認目前會選到哪個 profile | [config profile](./profile.md#current) | 先知道 default / current 選擇規則 |
| 想驗證 profile 與 live Grafana 是否可連通 | [config profile](./profile.md#validate) | 先驗證 auth 形狀與 health check |
| 想初始化一份新的 `grafana-util.yaml` | [config profile](./profile.md#init) | 先有一份最小可用設定檔，再慢慢補 |

## 這個入口是做什麼的

`grafana-util config` 是 repo-local 設定入口。公開 surface 目前不大，但語意很重要：它不是某一個 leaf command，而是整個「連線預設、secret source、profile 重複使用」的 namespace。

如果你的問題是：

- 這個 repo 應該怎麼保存 Grafana URL 與 auth 預設值
- 什麼時候應該直接帶旗標，什麼時候應該改用 `--profile`
- secret 應該放 env、OS store，還是 `encrypted-file`

就先看這頁，再進 `config profile` 的子命令頁。

## 先選哪一條設定路徑

- **第一次連線驗證**：先用 direct flags 跑 `status live`，證明真的連得上
- **日常維運 / repo-local workflow**：進 `config profile add` 建立具名 profile
- **CI / automation**：優先用 `token_env`、`password_env` 或 secret store
- **break-glass / bootstrap**：先用 direct Basic auth，再決定要不要整理成 profile

## `config` 與 `config profile` 的關係

- `config`：namespace / 入口頁，回答「這個 repo 該怎麼存放與重用連線設定」
- `config profile`：真正執行 list / show / current / validate / add / example / init 的命令群組

也就是說：

- 如果你在找「我該從哪裡開始整理設定」，先看 `config`
- 如果你已經知道自己要 list / show / add / validate，直接進 [config profile](./profile.md)

## 採用前後對照

- **採用前**：live 指令常反覆帶 `--url`、Basic auth 或 token，shell 歷史與 onboarding 筆記都容易越來越亂
- **採用後**：把重複的連線預設收進 repo-local profile，讓日常維運、CI 與交接都維持在同一套設定邏輯下

## 成功判準

- 同一個 checkout 有清楚的 repo-local 連線預設
- 常用 live 指令能改成 `--profile`，不必每次重打 auth
- secret storage mode 符合目前機器與工作模式
- 團隊能分清楚 `config` 是入口，`config profile` 才是實際操作群組

## 失敗時先檢查

- 如果 saved profile 行為和 direct flags 不一致，先看 [config profile show](./profile.md#show)
- 如果 validation 失敗，先確認 secret mode 是否適用於目前機器
- 如果 repo 還是得反覆帶一長串 auth flags，先檢查 intended profile 是否真的建立並選中
- 如果你連 `config` 和 `config profile` 的層級都還沒分清楚，先停下來看這頁，不要直接跳 leaf

## 各工作流入口

| 工作流 | 入口頁 | 常見延伸頁 |
| --- | --- | --- |
| 盤點與檢視 | [config profile](./profile.md#list) | [show](./profile.md#show)、[current](./profile.md#current) |
| 驗證 | [config profile](./profile.md#validate) | [show](./profile.md#show)、[current](./profile.md#current) |
| 建立與初始化 | [config profile](./profile.md#add) | [example](./profile.md#example)、[init](./profile.md#init) |

## 範例

```bash
# 先看 config namespace 的整體入口。
grafana-util config --help
```

```bash
# 在目前 checkout 初始化一份 starter config。
grafana-util config profile init --overwrite
```

```bash
# 建立可重複使用的 production profile。
grafana-util config profile add prod --url https://grafana.example.com --basic-user admin --prompt-password --store-secret encrypted-file
```

## 相關指令

- [config profile](./profile.md)
- [status](./status.md)
- [workspace](./workspace.md)
- [version](./version.md)
