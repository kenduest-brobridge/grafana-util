# grafana-util Docs

先用搜尋或常用任務進入文件，不要一開始就被 handbook、command reference 這些架構選項打斷。

## 先從任務開始

優先搜尋指令、手冊頁或常用工作流程。只有在你想瀏覽整體架構時，才往下看區塊索引。

## 快速開始

先從多數使用者第一天就會用到的工作流程開始。

### 匯出儀表板

把 Grafana 儀表板 JSON 匯出來，方便備份、檢查或做搬遷前準備。

- [打開文件](../commands/zh-TW/dashboard-export.md)

### 匯入儀表板

把儀表板載入目標環境，直接跟著既有流程完成匯入。

- [打開文件](../commands/zh-TW/dashboard-import.md)

### 新增資料來源

先把 datasource 設定好，再接儀表板與告警規則。

- [打開文件](../commands/zh-TW/datasource-add.md)

## 常用任務

先看任務導向文件，再決定要不要進完整手冊或整份指令樹。

### 檢視告警規則

先列出並檢查告警規則，再調整路由、標籤或通知點。

- [打開文件](../commands/zh-TW/alert-list-rules.md)

### 權限與團隊設定

管理 org、team 與 service account 權限，不用猜指令藏在哪裡。

- [打開文件](../commands/zh-TW/access.md)

### 疑難排解流程

當指令流程不順時，直接跳到手冊的 troubleshooting 章節。

- [打開文件](../user-guide/zh-TW/troubleshooting.md)

## 完整參考

已經知道要去哪一塊時，再進完整手冊或整份指令索引。

- [開始使用](../user-guide/zh-TW/getting-started.md)
- [手冊總覽](../user-guide/zh-TW/index.md)
- [指令索引](../commands/zh-TW/index.md)

### 完整手冊

敘述型文件、角色導向指南、架構說明與 troubleshooting 章節都在這裡。

- [瀏覽手冊](../user-guide/zh-TW/index.md)

### 指令參考

需要完整命名空間與子指令樹時，直接進指令參考。

- [瀏覽指令](../commands/zh-TW/index.md)

## 維護者

維護者文件保持獨立，首頁就能維持任務導向，不必把內部資訊攤在入口頁。

- [開發者指南](../../DEVELOPER.md)
- [Manpages](../man/index.html)
