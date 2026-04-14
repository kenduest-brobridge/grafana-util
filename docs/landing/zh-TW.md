# 套用 Grafana 變更前，先完成審查

線上盤點、匯出匯入、差異比對、變更預覽與安全套用，整合在同一套流程裡。首頁只放最常用入口；完整章節與所有指令可以用右上方快速跳轉進入。

- [從手冊開始](../user-guide/zh-TW/index.md)
- [直接查指令](../commands/zh-TW/index.md)
- [疑難排解](../user-guide/zh-TW/troubleshooting.md)

## 建議起點

如果你還不確定要看哪一份文件，先從這三個入口選一個。

### 第一次使用

先確認 `grafana-util --version`，再用 `grafana-util status live` 做第一個唯讀檢查。連線成功後，再建立可重用的 connection profile。

- [開始使用](../user-guide/zh-TW/getting-started.md)
- [新手快速入門](../user-guide/zh-TW/role-new-user.md)
- [Profile 指令參考](../commands/zh-TW/profile.md)

### 日常維運

先看 live 或 staged 狀態，再決定要匯出、比對、審查或套用變更。

- [SRE / 維運角色導讀](../user-guide/zh-TW/role-sre-ops.md)
- [Workspace 審查與狀態](../user-guide/zh-TW/status-workspace.md)
- [Status 指令參考](../commands/zh-TW/status.md)

### 自動化與 CI

用在 pipeline、release automation 或重複驗證流程。這裡會先說清楚輸入、輸出、失敗處理與穩定語法。

- [自動化 / CI 角色導讀](../user-guide/zh-TW/role-automation-ci.md)
- [技術參考](../user-guide/zh-TW/reference.md)
- [指令參考](../commands/zh-TW/index.md)

## 常見工作

已經知道工作目標時，直接從這裡進入對應章節。

### Dashboard

處理 browse、export、summary、review、patch、publish 與 screenshot。

- [Dashboard 管理](../user-guide/zh-TW/dashboard.md)
- [Dashboard 指令參考](../commands/zh-TW/dashboard.md)

### Data source 與 alert

處理 Grafana 整合、告警規則、contact point 與治理檢查。

- [Data source 管理](../user-guide/zh-TW/datasource.md)
- [告警治理](../user-guide/zh-TW/alert.md)
- [Alert 指令參考](../commands/zh-TW/alert.md)

### Access 與 credentials

處理 org、team、service account、token 與權限導向的操作變更。

- [Access 管理](../user-guide/zh-TW/access.md)
- [Access 指令參考](../commands/zh-TW/access.md)

## 完整參考

需要完整覆蓋時，改從完整面向進入。

### 完整手冊

手冊適合閱讀工作脈絡、操作順序與建議路線。

- [維運導引手冊](../user-guide/zh-TW/index.md)

### 完整指令參考

指令參考適合查每個 command、subcommand、參數與範例。

- [指令參考](../commands/zh-TW/index.md)
- [grafana-util(1)](../man/grafana-util.html)

### Source 與 release

需要變更歷史、release notes 或 issue 狀態時，改看 repository。

- [GitHub repository](https://github.com/kenduest-brobridge/grafana-util)
- [GitHub releases](https://github.com/kenduest-brobridge/grafana-util/releases)
- [Issue tracker](https://github.com/kenduest-brobridge/grafana-util/issues)

## 維護者

維護者文件留在 repo 內部文件，不跟公開手冊混在一起。

- [開發者指南](../DEVELOPER.md)
