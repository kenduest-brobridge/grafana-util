# dashboard clone-folder

## 用途
把某個 live Grafana folder 內的 dashboards 複製到另一個 live folder。

## 使用時機
當你需要為一整個 folder 建立 staging、review 或 promotion 副本，但不想改動來源 dashboards 時使用。

## 主要參數
- `--source-folder-uid`：來源 Grafana folder UID。
- `--source-path`：來源 Grafana folder path，例如 `Platform / Infra`。
- `--target-folder-uid`：目的 Grafana folder UID。
- `--target-folder-title`：建立目的 folder 時使用的標題。
- `--create-target-folder`：允許建立缺少的目的 folders。
- `--recursive`：包含子 folders 與其中的 dashboards。
- `--uid-prefix` / `--uid-suffix`：複製 dashboard UID 的 deterministic 規則。
- `--replace-existing`：更新已存在的目標 dashboard UID，而不是阻擋。
- `--dry-run`：只預覽，不修改 Grafana。
- `--yes`：確認 live 寫入。

## 範例
```bash
# 預覽複製直屬 dashboards 到既有目的 folder。
grafana-util dashboard clone-folder --url http://localhost:3000 --token "$GRAFANA_API_TOKEN" --source-folder-uid infra --target-folder-uid staging-infra --dry-run --table
```

```bash
# 建立目的 folder，並複製整個 folder subtree。
grafana-util dashboard clone-folder --url http://localhost:3000 --basic-user admin --basic-password admin --source-path 'Platform / Infra' --target-folder-uid staging-infra --target-folder-title 'Staging Infra' --create-target-folder --recursive --uid-suffix '-staging' --yes
```

## 相關命令
- [dashboard clone](./dashboard-clone.md)
- [dashboard publish](./dashboard-publish.md)
- [dashboard delete](./dashboard-delete.md)
