# `grafana-util workspace`

## Root

用途：針對本機 Grafana workspace 進行 scan、test、preview、package 與 apply。

適用時機：當你手上已經有本機 repo root 或 staged package，想先看懂它、驗證它、預覽影響、打包交接，或在審核後套用它時。

說明：`workspace` 是給使用者看的本機 package lane。先用 `scan` 找輸入，用 `test` 檢查結構是否安全，用 `preview` 看會改什麼，只有在審核完成後才用 `apply`。較低階的 contract 檢查與交接文件放在 `ci`。

Git Sync 與 file-provisioned dashboards 都屬於 source-owned。`workspace scan`、`test`、`preview` 可以檢視這些 tree，但 live dashboard 寫入應走 Git repository/PR 或 provisioning workflow，不應用 `workspace apply --execute-live` 直接覆蓋。

第一次使用流程：

1. `workspace scan`
2. `workspace test`
3. `workspace preview`
4. `workspace apply`

主要輸入：可選的 workspace 路徑、`--desired-file`、`--dashboard-export-dir`、`--dashboard-provisioning-dir`、`--alert-export-dir`、`workspace package` 的輸出檔、`--target-inventory`、`--availability-file`、`--mapping-file`、`--fetch-live`、`--live-file`、`--preview-file`、`--approve`、`--execute-live` 與 `--output-format`。

範例：

```bash
# 執行這個範例指令。
grafana-util workspace scan ./grafana-oac-repo
# 執行這個範例指令。
grafana-util workspace test ./grafana-oac-repo --fetch-live --output-format json
# 執行這個範例指令。
grafana-util workspace preview ./grafana-oac-repo --fetch-live --profile prod
# 執行這個範例指令。
grafana-util workspace package ./grafana-oac-repo --output-file ./workspace-package.json
# 執行這個範例指令。
grafana-util workspace apply --preview-file ./workspace-preview.json --approve --execute-live --profile prod
```

相關指令：`grafana-util status`、`grafana-util export`、`grafana-util config profile`。

## `scan`

用途：找出本機 workspace 或 staged package 裡有哪些內容。

## `test`

用途：確認本機 workspace 在結構上是否可以繼續往下走。

## `preview`

用途：顯示目前 workspace 輸入會造成哪些變動。

## `apply`

用途：把已審核的 preview 轉成 staged 或 live apply 結果。不要用 live apply 覆蓋 Git Sync-managed 或 file-provisioned dashboard；請更新它的來源。

## `package`

用途：把 dashboards、alerts、datasources 與 metadata 打包成一份可交接 artifact。

## `ci`

用途：提供給 CI 與自動化使用的低階 contract checks。

子命令：`summary`、`mark-reviewed`、`audit`、`input-test`、`alert-readiness`、`package-test`、`promote-test`。
