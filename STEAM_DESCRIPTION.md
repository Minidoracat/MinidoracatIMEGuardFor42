[h1]⌨️ Minidoracat IME Guard for B42[/h1]
[h3]By Minidoracat[/h3]

[hr][/hr]

[b]⚠️ 這是兩件式 MOD：只訂閱不會有效果，還要下載配套的 Windows 小工具（見下方）。[/b]

[b]🎮 單人／多人都可以用。[/b]單人：移動鍵不再被輸入法卡住，搜尋、命名等文字框會自動切回輸入法。[b]多人：伺服器必須把本 MOD 列進 mod 清單[/b]（PZ 連線時只載入伺服器清單上的 MOD），伺服器端本身不執行任何東西、不影響存檔。

[h2]✨ 這是什麼[/h2]
輸入法守衛。玩遊戲時自動把 PZ 切到英文鍵盤，打字（聊天、搜尋、命名）時自動切回你原本的輸入法。注音、拼音、日文、韓文……任何 Windows 輸入法都適用。

[h2]🧟 為什麼會被輸入法害死[/h2]
Windows 上只要中文／日文／韓文輸入法在組字模式，每個按鍵都先交給輸入法，PZ 會直接忽略——WASD、Shift 跑步、空白、E 全部沒反應。而 PZ 的操作鍵剛好是切輸入法的熱鍵（Shift＋Alt、Ctrl＋空白），玩到一半就被切回去。官方已明講不會在遊戲端處理，這個 MOD 就是為此而生。

[h2]🎬 35 秒示範影片[/h2]
[url=https://youtu.be/5q1bfbm3lgU]https://youtu.be/5q1bfbm3lgU[/url]（下載 → 執行 → 遊戲內打中文 → 關聊天直接走動）

[h2]🧰 怎麼用[/h2]
[olist]
[*] 訂閱並啟用本 MOD。
[*] 到 [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] 下載 [b]pz-ime-guard.exe[/b]，執行。第一次會跳 SmartScreen「Windows 已保護您的電腦」（未簽章的正常提示）：點「其他資訊 → 仍要執行」，或先對 exe 右鍵 → 內容 → 勾「解除封鎖」。它沒有視窗，只在系統匣放一個鍵帽圖示（通常收在「^」隱藏區），右下角小燈：綠＝英文鍵盤、橘＝打字中已切回你的輸入法、灰＝沒找到 PZ 或已暫停、紅＝系統沒裝英文（美國）鍵盤。
[*] 開遊戲。進遊戲時若工具沒在跑會提醒一次。
[/olist]
建議打開 Windows「讓我為每個應用程式視窗設定不同的輸入法」，切換就只影響 PZ。

[h2]🌐 支援的輸入法與語言[/h2]
[list]
[*] [b]會守護的輸入法：[/b]中文（注音、倉頡、無蝦米、微軟／搜狗拼音、RIME…）、日文（Microsoft IME、Google 日本語入力…）、韓文——中、日有玩家實證，韓文依微軟文件確認同樣會攔鍵。越南文 Telex／VNI、印度語系 Phonetic 等 Windows 內建輸入法機制相同，尚未實機驗證。
[*] [b]不介入：[/b]英文各國變體、俄、德、法、泰等純鍵盤配置——它們本來就沒這問題。
[*] [b]介面語言：[/b]工具提示、選單與 MOD 內提示有繁中、簡中、日文、韓文，其餘顯示英文；Steam 頁面有英、繁、簡、日、韓。
[/list]

[h2]🦀 為什麼用 Rust、為什麼開源[/h2]
[list]
[*] 單一 exe 約 360 KB，不用裝 .NET、Java 或任何執行環境，下載即用。
[*] Rust 是記憶體安全的語言，沒有緩衝區溢位那類漏洞；常駐時 CPU 約 0.3% 單核、記憶體不到 2 MB。
[*] 只呼叫 Windows 公開 API：列舉視窗、讀鍵盤配置、送一個切換訊息。不裝鍵盤 hook、不讀你的按鍵、不連網、不改登錄檔、不碰其他程式。MOD 本身是純 Lua，只寫一個「正在打字」的旗標到你的 Zomboid 資料夾。
[*] 原始碼公開在 [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url]，任何人都能檢視、自行編譯比對。每一版 exe 都由 GitHub Actions 在乾淨環境建置，並自動送 VirusTotal 掃描，SHA-256 與掃描連結都在 Release 頁。exe 未做程式碼簽章，SmartScreen 可能提示「未知發行者」——不放心就自己 cargo build。
[/list]

[h2]📋 MOD 資訊[/h2]
[list]
[*] [b]Mod ID:[/b] MinidoracatIMEGuardFor42
[*] [b]支援版本:[/b] Build 42.20.4+
[*] [b]單人 / 多人:[/b] 皆支援（多人需伺服器列入 mod 清單，見上方）
[*] [b]平台:[/b] 工具僅 Windows；Mac／Linux 沒有這個問題
[/list]

[h2]💬 意見回饋與交流[/h2]
[list]
[*] [url=https://discord.gg/Gur2V67]Discord 社群[/url]
[/list]

[h2]☕ 支持作者[/h2]
MOD 永遠免費，原始碼公開在 GitHub。喜歡的話可以請我喝杯咖啡，贊助會用在伺服器與 MOD 開發上。
[url=https://ko-fi.com/minidoracat][img]https://raw.githubusercontent.com/Minidoracat/workshop-resources/refs/heads/main/badges/badge_kofi.png[/img][/url] [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42][img]https://raw.githubusercontent.com/Minidoracat/workshop-resources/refs/heads/main/badges/badge_github.png[/img][/url]

[b]#Minidoracat[/b]

Workshop ID: 3802890539
Mod ID: MinidoracatIMEGuardFor42
