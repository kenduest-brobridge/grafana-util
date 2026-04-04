# zh-TW Docs Style Guide

Use this guide when writing or reviewing Traditional Chinese docs for
`README.zh-TW.md`, `docs/user-guide/zh-TW/`, `docs/commands/zh-TW/`, and
`docs/landing/zh-TW.md`.

The goal is not "translate every English word". The goal is operator-facing
Traditional Chinese that matches Taiwan usage, stays readable, and does not
turn Grafana object names into awkward invented terms.

## Default Voice

Write like a technical teammate in Taiwan, not like a localization engine.

Prefer:

- direct, practical wording
- short sentences with a clear subject
- concrete usage advice
- "會怎樣" and "什麼時候用" explanations

Avoid:

- spec-language that sounds detached from real work
- AI-style emphasis such as "手術級", "精確形狀", "旨在帶領您"
- abstract labels that hide the action
- over-translating Grafana product objects

## Keep These Grafana Object Names In English

When these refer to Grafana product objects, CLI objects, or first-class repo
surface names, keep them in English inside zh-TW docs.

- dashboard
- alert
- data source
- service account
- team
- org
- profile
- secret
- lane
- bundle
- prompt
- raw
- live
- checkout

Use normal Chinese around them. Example:

- `新增 data source`
- `列出 team 成員`
- `這條 lane 適合日常維運`
- `把結果接到 CI/CD pipeline`

Do not force translations such as:

- `資料來源` when you clearly mean the Grafana `data source` object
- `服務帳號` when the page is about the Grafana `service account` object
- `團隊` when the page is about the Grafana `team` object
- `組織` when the page is specifically about the Grafana `org` object

Generic human-language usage is still fine. Example:

- `團隊平常怎麼維護這個流程`
- `組織內部的治理做法`

The key rule is: keep object names in English when they are product nouns, not
general concepts.

## Prefer These zh-TW Patterns

Use wording like this:

- `先從這裡開始`
- `適合什麼時候使用`
- `不適合的情況`
- `把輸出接到 CI/CD 或自動化流程`
- `用來檢查變更是否安全`
- `先確認目前狀態，再決定要不要套用`
- `看實際語法與常用旗標`
- `這頁會整理常見做法與限制`

Prefer these nouns:

- `手冊`
- `指令詳細說明`
- `常見做法`
- `狀態`
- `變更`
- `實戰範例`
- `疑難排解`
- `最佳實踐`
- `瀏覽文件`
- `查看說明`

## Avoid These Robot-Like Patterns

Rewrite phrases like these when you see them:

- `旨在帶領您`
- `精確形狀`
- `指令面`
- `治理關卡`
- `前置門禁`
- `需要機器可讀輸出的人`
- `原始訊號不夠直觀`
- `建議的驗證與秘密處理`

Better replacements:

- `會帶你一路看到`
- `實際語法`
- `指令頁`
- `最後一道檢查`
- `先做自動化檢查`
- `需要把輸出交給腳本或 pipeline 處理的人`
- `資訊不夠直接`
- `建議的連線與 secret 管理`

## Translation Review Checklist

Before merging zh-TW doc edits, check:

1. Would a Taiwan-based operator say this in a docs page?
2. Did we accidentally translate a Grafana object name that should stay in English?
3. Is the sentence describing a real action, not just a category label?
4. Does the page explain when to use the workflow, not only what it is called?
5. If the page includes English, is it product terminology rather than untranslated filler?

## Where To Apply This Guide

Use this guide for:

- `README.zh-TW.md`
- `docs/landing/zh-TW.md`
- `docs/user-guide/zh-TW/*.md`
- `docs/commands/zh-TW/*.md`

If a wording dispute comes up, prefer this file over ad hoc one-off choices in a
single page.
