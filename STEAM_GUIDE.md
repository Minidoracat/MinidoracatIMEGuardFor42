<!-- 指南網址：https://steamcommunity.com/sharedfiles/filedetails/?id=3817844233 -->
<!-- Steam 指南稿源（繁中）：MOD 的完整說明以本指南為準，簡介只放摘要。格式與同步方式見 pz-family-docs/release.md「完整說明：Steam 指南」 -->
<!-- 標題：📖 輸入法守衛 IME Guard MOD 完整說明：運作方式、常見問題與已知限制 -->
<!-- 摘要：Minidoracat 輸入法守衛 MOD 的圖文說明：配套工具的下載與執行、運作方式、支援的輸入法、系統匣圖示與小燈、設定、安全與隱私、已知限制與常見問題。 -->
=== 🚀 快速上手
[b]English version:[/b] [url=https://steamcommunity.com/sharedfiles/filedetails/?id=3817844283]IME Guard Mod Guide[/url]

MOD 本體（Steam Workshop）：
https://steamcommunity.com/sharedfiles/filedetails/?id=3802890539
[olist]
[*] [b]啟用 MOD[/b]：訂閱並啟用本 MOD。多人時伺服器必須把本 MOD 列進 mod 清單。
[*] [b]下載工具[/b]：到 [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] 下載 [b]pz-ime-guard-版本號.exe[/b]，執行。沒有安裝程式，放在哪個資料夾都可以。
[*] [b]SmartScreen 提示[/b]：第一次會跳「Windows 已保護您的電腦」（未簽章的正常提示）。點「其他資訊 → 仍要執行」，或先對 exe 右鍵 → 內容 → 勾「解除封鎖」。
[*] [b]開遊戲[/b]：確認系統匣出現鍵帽圖示後開遊戲。進入世界時若工具沒在跑，MOD 會提醒一次。
[/olist]
{{img:01-smartscreen-zh.jpg}}
[b]介紹影片[/b]：[url=https://youtu.be/RGuE1GUEs9I]https://youtu.be/RGuE1GUEs9I[/url]（輸入法吃掉移動鍵 → 開守衛就能跑 → 搜尋與聊天打中文 → 切出遊戲還原輸入法）
=== 🧰 為什麼需要它
[list]
[*] [b]按鍵被輸入法吃掉[/b]：Windows 上只要中文／日文／韓文輸入法在組字模式，每個按鍵都會先交給輸入法，PZ 會直接忽略。WASD、Shift 跑步、空白鍵、E 互動全部沒反應。
[*] [b]玩到一半被切回輸入法[/b]：PZ 的操作鍵剛好是 Windows 切輸入法的熱鍵：
[list]
[*] Shift 跑步＋Alt 衝刺＝Alt+Shift
[*] Ctrl 瞄準＋空白＝Ctrl+Space
[/list]
[*] [b]官方不在遊戲端處理[/b]：官方已明講不會在遊戲端處理；等遊戲底層的相關修正上線後，這個 MOD 就不再需要。
[/list]
=== 🧰 運作方式
[b]兩件缺一不可[/b]：
[list]
[*] [b]MOD[/b]：只回報「你是不是在打字」與「你是不是確認要關掉遊戲」，本身無法切換輸入法。
[*] [b]工具[/b]：依這些回報切換 PZ 視窗的鍵盤。
[/list]
[h3]遊玩固定英文[/h3]
{{img:02-play-eng-zh.jpg}}
[list]
[*] 沒在打字時，工具把 PZ 視窗切到英文（美國）等非輸入法鍵盤，WASD、Shift、空白、E 都能正常用。
[/list]
[h3]打字自動切回[/h3]
{{img:03-typing-zh.jpg}}
[list]
[*] [b]判斷依據[/b]：遊戲自己的「正在輸入文字」狀態。點進聊天、搜尋欄、命名等文字框就切回輸入法，離開就回英文。
[*] [b]切回哪一個[/b]：你在 PZ 視窗最後用過的輸入法，不是系統預設。
[*] [b]換另一種輸入法[/b]：裝了兩種想換另一種，在聊天框按 Win+Space 切一次就會記住。
[/list]
[h3]切出時還原[/h3]
[list]
[*] 離開遊戲時，嘗試還原進遊戲前觀察到的桌面輸入法（細節見常見問題「Alt+Tab」與「關遊戲」）。
[/list]
[h3]建議設定[/h3]
[list]
[*] 打開 Windows「設定 → 時間與語言 → 輸入 → 進階鍵盤設定 → 讓我為每個應用程式視窗設定不同的輸入法」，切換就只影響 PZ，其他程式保留各自的輸入法。
[/list]
=== 🧰 支援的輸入法與語言
工具看的是 Windows 鍵盤配置的語言，不認輸入法品牌。
[list]
[*] [b]會守護[/b]：
[list]
[*] 中文：注音、倉頡、無蝦米、微軟／搜狗拼音、RIME…
[*] 日文：Microsoft IME、Google 日本語入力…
[*] 韓文
[/list]
[*] [b]實證[/b]：中文、日文有玩家實證；韓文依微軟文件確認同樣會攔鍵。
[*] [b]機制相同但未實機驗證[/b]：越南文 Telex／VNI、印度語系 Phonetic 等 Windows 內建輸入法。
[*] [b]不介入[/b]：英文各國變體、俄、德、法、泰等純鍵盤配置，它們本來就沒這個問題。
[/list]
[h3]介面語言[/h3]
[table]
[tr][th]位置[/th][th]語言[/th][/tr]
[tr][td]MOD 內提示[/td][td]繁中、簡中、英文、日文、韓文、俄文、西班牙文、葡萄牙文、土耳其文、法文、波蘭文、德文（翻譯有問題請回報）[/td][/tr]
[tr][td]工具（pz-ime-guard.exe）的提示與選單[/td][td]繁中、簡中、日文、韓文，其餘顯示英文[/td][/tr]
[tr][td]Steam 頁面[/td][td]英、繁、簡、日、韓[/td][/tr]
[/table]
=== 🧰 系統匣圖示與小燈
{{img:04-tray-icon-zh.jpg}}
[list]
[*] [b]沒有視窗[/b]：工具只在系統匣放一個鍵帽圖示。
[*] [b]找不到圖示[/b]：Windows 11 通常收在「^」隱藏區，第一次啟動會跳一次說明。
[*] [b]看狀態[/b]：滑鼠停在圖示上會顯示目前狀態。
[/list]
[h3]右下角小燈[/h3]
[table]
[tr][th]顏色[/th][th]意思[/th][/tr]
[tr][td]綠[/td][td]英文鍵盤中，移動鍵安全[/td][/tr]
[tr][td]橘[/td][td]打字中，已切回你的輸入法[/td][/tr]
[tr][td]灰[/td][td]沒找到 PZ、PZ 不在前景、已暫停，或遊戲正在關閉（已停止守護）[/td][/tr]
[tr][td]紅[/td][td]系統沒有英文（美國）等非輸入法鍵盤可以切換（處理方式見常見問題）[/td][/tr]
[/table]
[h3]右鍵選單[/h3]
[list]
[*] [b]依序是[/b]：版本、Workshop 頁面、GitHub（原始碼／回報問題）、三個設定（見「設定」）、暫停、結束。
[*] [b]只會開一份[/b]：重複執行會提示後自動關閉。
[/list]
=== 🧰 安全與隱私
[list]
[*] [b]單一 exe[/b]：Rust 寫的，不用裝 .NET、Java 或任何執行環境。
[*] [b]不碰遊戲與按鍵[/b]：不注入遊戲程序、不讀取按鍵、不裝鍵盤 hook、不改登錄檔；只透過 Windows 公開 API 切換鍵盤配置。
[*] [b]MOD 是純 Lua[/b]：只把「打字中」與「正常退出」訊號寫到 Zomboid 資料夾。
[*] [b]連線[/b]：工具只在檢查更新時連 GitHub（可在右鍵關閉），不傳送任何使用者資料。
[*] [b]原始碼公開[/b]：在 [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url]，任何人都能檢視、自行編譯比對。
[*] [b]建置與掃描[/b]：每一版 exe 由 GitHub Actions 在乾淨環境建置並自動送 VirusTotal，SHA-256 與掃描連結在 Release 頁。
[*] [b]沒有程式碼簽章[/b]：SmartScreen 會提示「未知發行者」；不放心可以自己用 cargo build 編譯。
[/list]
=== ⚙️ 設定（右鍵選單）
[table]
[tr][th]選項[/th][th]預設[/th][/tr]
[tr][td]切出遊戲時還原桌面的輸入法[/td][td]開[/td][/tr]
[tr][td]自動檢查更新（啟動時與每日）[/td][td]開：有新版會問你要不要開下載頁[/td][/tr]
[tr][td]登入 Windows 時自動啟動[/td][td]關[/td][/tr]
[/table]
[h3]登入 Windows 時自動啟動[/h3]
[list]
[*] [b]怎麼做到的[/b]：只在目前使用者的「啟動」資料夾放一個捷徑，不寫登錄檔、不建服務；取消勾選即移除。
[*] [b]啟動時機[/b]：登入 Windows 後啟動，不是一開機就跑，Windows 也可能晚一點才叫起來。
[*] [b]在 Windows 設定停用[/b]：在「設定 → 應用程式 → 啟動」停用它不會刪掉捷徑，選單仍可能顯示已勾。
[*] [b]搬了 exe[/b]：把 exe 搬到別的資料夾後，手動開一次新位置的工具（選項保持勾選）就會改寫捷徑。
[/list]
=== ⚠️ 已知限制
[list]
[*] [b]只支援 Windows[/b]：Mac／Linux 沒有這個問題。
[*] [b]同時開兩個 PZ[/b]：同一台電腦同時開兩個 PZ 時，只守第一個視窗。
[*] [b]還原桌面輸入法屬於盡力而為[/b]：按 X 關窗或遊戲當掉時 MOD 來不及通知工具（見常見問題「Alt+Tab」與「關遊戲」）。
[/list]
=== ❓ 常見問題
[h3]搜尋框會切回輸入法嗎？能搜容器裡的物品嗎？[/h3]
[list]
[*] 會切回：點進遊戲的搜尋欄開始輸入時小燈變橘。
[*] IME Guard 只負責切換輸入法，[b]不會新增或擴大任何搜尋功能[/b]；搜尋能找到什麼由遊戲（或其他 MOD）決定。
[*] 如果某個文字框點進去小燈沒變橘，請回報是哪個畫面、哪個欄位。
[/list]
[h3]Alt+Tab 切出遊戲，偶爾還是停在英文？[/h3]
按住 Alt+Tab 慢慢選視窗、或在遊戲與其他程式間快速來回切換的情況已經改善：離開遊戲後會等真正的目標視窗出現（最多約 2 秒）再還原，英文狀態也不會再被誤記成你原本的輸入法。仍有這些限制：
[list]
[*] 工具必須確認自己曾把 PZ 切到英文鍵盤，而且知道桌面原本用的輸入法，才會還原；不知道原本是什麼時不猜。
[*] 還原有重試次數與時間上限，無法保證每個程式都接受切換。
[*] 暫停工具或取消勾選還原選項後，不會再補送先前的還原。
[/list]
最穩的做法是打開「運作方式」的「建議設定」：每個應用程式視窗不同輸入法。請先確認工具是最新版；還是發生的話，請回報當時怎麼切的（按住慢慢選、快速來回、切到哪個程式）。
[h3]關遊戲時，桌面被切回英文？[/h3]
從遊戲選單確認「離開到桌面」（主選單離開、遊戲內離開、死亡畫面、斷線畫面）時，MOD 會通知工具：遊戲還在收尾時工具就停止守護（小燈變灰），等你切到其他視窗時嘗試還原原本的輸入法。這需要 [b]MOD 與工具都是最新版[/b]，只更新其中一邊沒有效果。
[list]
[*] [b]按視窗右上角 X、遊戲當掉、用工作管理員強制結束[/b]：MOD 來不及通知，工具要等遊戲視窗消失才會停止守護。
[*] [b]回主選單不算關遊戲[/b]：視窗還在，工具會繼續守護。
[/list]
[h3]中國大陸打不開 GitHub，下載不了工具？[/h3]
[list]
[*] 先用 [url=https://steampp.net/]Watt Toolkit[/url]（免費、開源）：左側「网络加速」勾選「GitHub」，按「启动加速」，再打開 [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] 下載。
[*] 工具提示有新版時開的也是 GitHub 下載頁，一樣先開加速。
[/list]
[h3]怎麼更新工具？[/h3]
[list]
[*] 下載新版直接執行即可：舊版還開著也沒關係，新版會請舊版退出後接手，自動啟動也會改成啟動新版。
[*] 很舊的版本無法自動接手，會提示你先從系統匣結束舊版。
[*] 舊版 exe 可以自行刪除。
[/list]
[h3]SmartScreen 擋下來／會不會有毒？[/h3]
[list]
[*] 未簽章 exe 的正常提示，放行方式見「快速上手」第 3 步。
[*] 建置與掃描方式見「安全與隱私」。
[/list]
[h3]進遊戲時出現「未偵測到配套工具」？[/h3]
[list]
[*] 代表進入世界時 MOD 沒偵測到工具在跑。
[*] 確認系統匣有鍵帽圖示（可能在「^」隱藏區），沒有就執行 exe。
[/list]
[h3]圖示是紅燈？[/h3]
[list]
[*] 系統只裝了輸入法、沒有英文（美國）等一般鍵盤，工具沒有可以切過去的配置。
[*] 到 Windows 設定新增「英文（美國）」鍵盤（或在你目前語言的選項裡新增美式鍵盤）即可。
[*] 不用重開工具，它會自己偵測到。
[/list]
[h3]多人可以用嗎？[/h3]
[list]
[*] 可以，但伺服器必須把本 MOD 列進 mod 清單（PZ 連線時只載入伺服器清單上的 MOD）。
[*] 伺服器端本身不執行任何東西、不影響存檔；工具只在你自己的電腦上執行。
[/list]
[h3]怎麼移除？[/h3]
工具沒有安裝程式。
[olist]
[*] [b]自動啟動[/b]：若曾勾「登入 Windows 時自動啟動」，先取消勾選再結束工具；已經刪掉 exe 的話，在檔案總管網址列輸入 [b]shell:startup[/b]，刪掉裡面的 pz-ime-guard 捷徑。
[*] [b]刪檔[/b]：刪掉 exe。Zomboid\Lua 下的 MinidoracatIMEGuard 資料夾只放幾個小狀態檔，可一併刪除。
[*] [b]MOD[/b]：在 Workshop 取消訂閱即可。
[/olist]
=== 💬 回報方式
[list]
[*] [b][url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/issues]GitHub Issues[/url][/b]：回報 Bug 時請附上
[list]
[*] Windows 版本
[*] 使用的輸入法
[*] 當時的小燈顏色
[*] 重現步驟
[/list]
[*] [b]使用問題[/b]：直接在本指南留言
[*] [b][url=https://discord.gg/Gur2V67]Discord[/url][/b]：交流
[/list]
