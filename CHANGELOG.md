# Changelog

<!-- 撰寫規則摘要（完整版見 AGENTS.md「CHANGELOG 撰寫規則」）：
  - bullet 寫給玩家，會整段照貼 Workshop 更新說明：症狀先行、遊戲內名詞、
    禁檔名/函式名/行號/引擎術語；影響版本誠實寫清楚
  - 「> 技術要點：」（選用）只放管理員/modder 需要的行為事實；單次變更的完整
    技術原因寫在 commit message 本文（綁 diff、可 git log 搜尋、永不公開）
  - 紅線：不寫攻擊配方（安全修正只寫「強化了驗證」）、不寫玩家名/座標/Steam ID、
    不寫主機名/路徑/IP -->

所有重要的變更都會記錄在此檔案中。

格式基於 [Keep a Changelog](https://keepachangelog.com/zh-TW/1.1.0/)，版本號遵循 `{PZ版本}-{主版本}.{次版本}.{修訂}` 格式。

## [42.20.4-0.1.0] - 2026-09-17

### 新增

- **初始版本**：玩遊戲時自動把 PZ 切到英文鍵盤，打字（聊天、搜尋、命名）時自動切回你原本的輸入法，注音、拼音、日文、韓文輸入法不再卡住移動與操作鍵。需搭配 Windows 配套工具 `pz-ime-guard.exe`（GitHub Releases 下載）；進遊戲時工具沒在跑會提醒一次。純 client 端，單人／多人皆可用。

> 技術要點：MOD 只寫「正在打字」旗標到 `Zomboid/Lua/MinidoracatIMEGuard/state.txt`，切換由工具以 `WM_INPUTLANGCHANGEREQUEST` 對 PZ 視窗執行並回讀確認；工具每 2 秒寫 `heartbeat.txt`。
