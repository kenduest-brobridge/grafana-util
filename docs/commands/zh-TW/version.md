# `grafana-util version`

## 用途
印出目前 `grafana-util` 版本。

## 何時使用
當你需要確認目前安裝的 binary，或讓自動化流程取得可解析的版本資訊時，使用這個命令。

## 主要旗標
- `--json`：以 JSON 輸出版本資訊，方便外部工具讀取

## 範例
```bash
# 印出人類可讀的版本。
grafana-util version
```

```bash
# 印出機器可讀的版本資訊。
grafana-util version --json
```

## 相關指令
- [config](./config.md)
- [status](./status.md)
