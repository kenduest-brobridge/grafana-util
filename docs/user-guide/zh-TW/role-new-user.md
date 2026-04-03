# 👤 新手快速入門

這一頁為第一次接觸 `grafana-util` 的讀者所準備。目標不是一次學完所有指令，而是先把連線、設定檔 (Profile) 與唯讀檢查流程跑順。

## 適用對象

- 第一次使用此 CLI 工具的工程師或維運人員。
- 需要先確認連線、版本與 Profile 是否正常的人員。
- 還不需要執行匯入、套用或跨組織作業的使用者。

## 主要目標

- 建立可重複使用的連線設定檔 (Profile)。
- 熟悉 `status live` 與 `overview live` 的使用。
- 釐清何時使用 Profile，以及何時需使用直接基本驗證 (Direct Basic Auth)。
- 瞭解 API Token 適用於範圍明確的單一組織自動化作業。

## 典型新手任務

- 確認執行檔已加入 `PATH` 環境變數。
- 為本地實驗環境或開發用 Grafana 建立 Profile。
- 執行一次安全的即時讀取 (Live Read)，並區分 `status live` 與 `overview live` 的差異。
- 瞭解後續進行儀表板、告警或存取權限管理時應參考的說明文件。

## 身分驗證與機密資料管理建議

1. **優先使用 `--profile`**：這是日常操作最穩定的路徑，也能避免在命令列中重複貼上機密資訊。
2. **手動引導時使用 Basic Auth**：若尚未建立 Profile，可使用直接基本驗證進行初始化或臨時檢查，並建議搭配 `--prompt-password` 手動輸入密碼。
3. **特定自動化場景使用 Token**：僅在您清楚 Token 權限僅需涵蓋單一組織或特定範圍時使用。
4. **安全儲存機密資訊**：建議將密碼與 Token 存放於 `password_env`、`token_env` 或系統秘密儲存庫 (Secret Store)，避免以明文形式出現在命令列中。

## 建議先執行的 5 個指令

```bash
grafana-util profile init --overwrite
grafana-util profile add dev --url http://127.0.0.1:3000 --basic-user admin --prompt-password
grafana-util profile list
grafana-util profile show --profile dev --output-format yaml
grafana-util status live --profile dev --output yaml
```

如果您暫時還沒有 Profile，可改用此指令作為測試入口：

```bash
grafana-util status live --url http://localhost:3000 --basic-user admin --prompt-password --output yaml
```

如果您手邊已有範圍明確的 Token，也可以執行同效的唯讀檢查：

```bash
grafana-util overview live --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --output json
```

## 學習進度檢核

當您符合以下幾點時，即可進階到後續章節：

- 可在常用的終端機環境中正常執行 `grafana-util --version`。
- `profile show --profile dev` 解析出的欄位符合預期。
- `status live --profile dev` 能穩定回傳可讀的結果。
- 已清楚後續要進行儀表板 (Dashboards)、告警 (Alerts) 或存取權限 (Access) 管理的操作流程。

## 後續閱讀建議

- [開始使用](getting-started.md)
- [技術參考手冊](reference.md)
- [疑難排解與名詞解釋](troubleshooting.md)

## 推薦搭配參考的指令頁面

- [profile](../../commands/zh-TW/profile.md)
- [status](../../commands/zh-TW/status.md)
- [overview](../../commands/zh-TW/overview.md)
- [指令詳細總索引](../../commands/zh-TW/index.md)

## 常見錯誤與限制

- **旗標誤用**：請勿混用 `--output-format` 與 `--output`，這兩個旗標位於不同的輸出控制層級。
- **設定檔安全性**：請勿在 `grafana-util.yaml` 中寫入明文密碼，除非僅用於一次性的實驗或展示。
- **Token 權限限制**：窄權限 Token 無法執行所有操作，特別是跨組織盤點或管理類任務。
- **循序漸進**：在熟悉 Profile、Status 與 Overview 的讀取流程前，建議先不要執行匯入或套用變更的作業。

## 何時切換至深度文件

- **流程理解**：需要理解完整操作脈絡時，請參閱維運指南 (Handbook) 章節。
- **精確查閱**：已知操作流程僅需確認特定旗標時，請參閱指令詳細說明頁。
- **故障排除**：語法正確但結果不符預期時，請參閱疑難排解章節。

## 下一步

- [回到手冊首頁](index.md)
- [開始使用](getting-started.md)
- [技術參考手冊](reference.md)
- [查看指令詳細總索引](../../commands/zh-TW/index.md)
