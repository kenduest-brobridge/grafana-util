# 📊 Grafana Utilities (維運治理工具集)

[English Version](README.md) | **繁體中文版**

`grafana-utils` 是一套專為 Grafana 管理者與 SRE 打造的維運治理工具集。

## 內容索引

- [這個工具是做什麼的](#這個工具是做什麼的)
- [功能支援總覽](#功能支援總覽)
- [下載入口](#下載入口)
- [快速開始](#快速開始)
- [常用命令地圖](#常用命令地圖)
- [文件導航](#文件導航)
- [相容性與目標](#相容性與目標)
- [專案狀態](#專案狀態)

## 這個工具是做什麼的

`grafana-util` 適合拿來做：
- Dashboard、Datasource、Alert、Org、User、Team、Service Account 盤點
- Grafana 狀態的 export、import、diff、dry-run
- Dashboard 治理分析、查詢盤點、datasource 依賴檢查
- Dashboard 與 panel 的截圖或 PDF 擷取

## 🏗️ 技術架構

目前維護中的 CLI 以 Rust `grafana-util` 二進位工具為主：
- 對外使用與 release 下載以 Rust binary 為主。
- Python 實作細節保留在 maintainer 文件中，供 parity 與驗證使用。

## 功能支援總覽

這裡用快速能力摘要呈現，比較適合 README 掃讀：

- `Dashboard`：支援 list、inspect、capture、export/import/diff。匯入流程支援 dry-run 與資料夾感知遷移。
- `Alerting`：支援 list，以及 rule 與相關 alerting 資源的 export/import/diff。
- `Datasource`：支援 list、export/import/diff，以及線上 add/modify/delete；也支援 dry-run 與多 org 回放。
- `Access User`：支援 list、add/modify/delete、export/import/diff，涵蓋全域與 org 範圍的使用者管理流程。
- `Access Org`：支援 list、add/modify/delete、export/import，處理組織管理與成員關係重建。
- `Access Team`：支援 list、add/modify/delete、export/import/diff，強調 membership-aware sync。
- `Access Service Account`：支援 list、add/delete、export/import/diff，以及 token 建立與刪除流程。

## 下載入口

下載入口：
- [最新版 release](https://github.com/kenduest-brobridge/grafana-utils/releases/latest)
- [所有 releases](https://github.com/kenduest-brobridge/grafana-utils/releases)

怎麼下載：
- 進入 release 頁面後展開 `Assets`
- 下載對應作業系統與 CPU 架構的 `grafana-util` 預編譯壓縮檔
- 如果目前沒有符合需求的 tagged release，就改成本地建置

本地建置：
```bash
cd rust && cargo build --release
```

## 🛠️ 快速開始

先看 CLI 入口：
```bash
grafana-util -h
grafana-util dashboard -h
grafana-util datasource -h
grafana-util alert -h
grafana-util access -h
```

## 常用情境範例

列出 dashboards：
```bash
grafana-util dashboard list \
  --url http://localhost:3000 \
  --basic-user admin \
  --basic-password admin \
  --with-sources \
  --table
```

檢查 live dashboards：
```bash
grafana-util dashboard inspect-live \
  --url http://localhost:3000 \
  --basic-user admin \
  --basic-password admin \
  --output-format governance-json
```

列出 datasources：
```bash
grafana-util datasource list \
  --url http://localhost:3000 \
  --basic-user admin \
  --basic-password admin \
  --table
```

列出 users：
```bash
grafana-util access user list \
  --url http://localhost:3000 \
  --basic-user admin \
  --basic-password admin \
  --scope global \
  --table
```

**批次匯出儀表板 (保留結構)：**
```bash
grafana-util dashboard export \
  --url http://localhost:3000 \
  --basic-user admin \
  --basic-password admin \
  --export-dir ./dashboards \
  --overwrite
```

**匯入前先執行模擬執行與比對：**
```bash
grafana-util dashboard import \
  --url http://localhost:3000 \
  --import-dir ./dashboards/raw \
  --replace-existing \
  --dry-run --table
```

## 常用命令地圖

想先找到入口時，可以先看這裡。

- `grafana-util dashboard ...`
  - 盤點、匯出/匯入/diff、inspect、截圖、PDF 擷取
- `grafana-util datasource ...`
  - 盤點、匯出/匯入/diff、線上 add/modify/delete
- `grafana-util alert ...`
  - alerting 資源的 list、匯出/匯入/diff
- `grafana-util access ...`
  - org、user、team、service-account 的盤點與變更流程
- `grafana-util sync ...`
  - 分階段 bundle、預檢、審查、套用

## 📄 文件導航

- **[繁體中文使用者指南](docs/user-guide-TW.md)**：包含全域參數、認證規則與各模組指令詳解。
- **[English User Guide](docs/user-guide.md)**：英文版操作說明。
- **[技術細節 (Rust)](docs/overview-rust.md)**
- **[開發者手冊](docs/DEVELOPER.md)**：維護與貢獻說明。

## 📈 相容性與目標
- 支援 Linux、macOS。
- 執行型態：Rust release binary。
- Grafana 版本：支援 8.x, 9.x, 10.x+。

## 專案狀態

本專案仍在持續開發中，歡迎透過 GitHub Issues 或 Pull Requests 回報問題與使用回饋。
