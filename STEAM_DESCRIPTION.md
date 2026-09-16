[h1]⌨️ Minidoracat IME Guard for B42[/h1]
[h3]By Minidoracat[/h3]

[hr][/hr]

[b]⚠️ 這是兩件式 MOD：只訂閱不會有效果，還要下載配套的 Windows 小工具（見下方）。[/b]

[h2]✨ 這是什麼[/h2]
輸入法守衛。玩遊戲時自動把 PZ 切到英文鍵盤，打字（聊天、搜尋、命名）時自動切回你原本的輸入法。注音、拼音、日文、韓文……任何 Windows 輸入法都適用。

[h2]🧟 為什麼會被輸入法害死[/h2]
Windows 上只要中文／日文／韓文輸入法在組字模式，每個按鍵都先交給輸入法，PZ 會直接忽略——WASD、Shift 跑步、空白、E 全部沒反應。而 PZ 的操作鍵剛好是切輸入法的熱鍵（Shift＋Alt、Ctrl＋空白），玩到一半就被切回去。官方已明講不會在遊戲端處理，這個 MOD 就是為此而生。

[h2]🧰 怎麼用[/h2]
[olist]
[*] 訂閱並啟用本 MOD。
[*] 到 [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] 下載 [b]pz-ime-guard.exe[/b]，執行。它沒有視窗，只在系統匣放一個鍵帽圖示（通常收在「^」隱藏區），右下角小燈：綠＝英文鍵盤、橘＝打字中已切回你的輸入法、灰＝沒找到 PZ 或已暫停、紅＝系統沒裝英文（美國）鍵盤。
[*] 開遊戲。進遊戲時若工具沒在跑會提醒一次。
[/olist]
建議打開 Windows「讓我為每個應用程式視窗設定不同的輸入法」，切換就只影響 PZ。

[h2]🔒 它做了什麼、沒做什麼[/h2]
[list]
[*] MOD 本身是純 Lua，只寫一個「正在打字」的旗標到你的 Zomboid 資料夾；不含任何 native 程式碼。
[*] 工具只在 PZ 在前景時，用 Windows 官方訊息切換 PZ 這個視窗的鍵盤配置。不裝鍵盤 hook、不代送按鍵、不改登錄檔、不碰其他程式。
[*] 原始碼公開：[url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url]
[/list]

[h2]📋 MOD 資訊[/h2]
[list]
[*] [b]Mod ID:[/b] MinidoracatIMEGuardFor42
[*] [b]支援版本:[/b] Build 42.20.4+
[*] [b]單人 / 多人:[/b] 皆支援（純 client 端，伺服器不用裝）
[*] [b]平台:[/b] 工具僅 Windows；Mac／Linux 沒有這個問題
[/list]

[h2]💬 意見回饋與交流[/h2]
[list]
[*] [url=https://discord.gg/Gur2V67]Discord 社群[/url]
[/list]


[b]#Minidoracat[/b]
