[h1]Minidoracat IME Guard for B42 42.22.2-0.3.0[/h1]
[i]2026-11-24[/i]
[b]English summary[/b]
[list]
[*] New (experimental): while you type in chat, search or naming boxes, your IME's own composition window now appears at the text cursor, so you can see what you are composing before you confirm it.
[*] Works with Traditional Chinese, Simplified Chinese and Japanese IMEs. Tested with Microsoft Bopomofo, Microsoft Pinyin and Microsoft Japanese IME; other IMEs of these languages are enabled but untested. Korean IMEs are not supported yet.
[*] You need both the updated MOD (Steam updates it) and the new companion tool 0.3.0 (the tool will tell you a new version is available, or get it from GitHub Releases).
[*] Password boxes never show the composition window. You can turn the feature off in the tool's tray menu.
[/list]

本版同時更新 MOD 與配套工具：組字顯示要新的 MOD 和新的工具一起才有。MOD 由 Steam 自動更新；工具會提示有新版，也可以到 GitHub Releases 下載 pz-ime-guard-0.3.0.exe。只更新其中一邊時沒有這項顯示，其他功能照舊。

[h3]✨ 新增[/h3]
[list]
[*] [b]打字時看得到組字中的字（實驗性）[/b]：以前在聊天、搜尋、命名欄打中文或日文，選字確認前什麼都看不到，只能盲打。現在文字游標的位置會出現輸入法自己的組字小視窗，注音符號、拼音、假名和還沒送出的字會跟著你按的鍵更新，候選字清單也跟在旁邊；送出、刪光或取消後自動消失。
[*] [b]支援的輸入法[/b]：繁體中文、簡體中文、日文輸入法。已實測的是微軟注音、微軟拼音、微軟日文輸入法；同語言的其他輸入法（倉頡、速成、搜狗拼音、Google 日本語入力等）同樣會開啟但還沒驗證。韓文輸入法實測沒有效果，不開。
[*] [b]可以關掉[/b]：在工具的系統匣右鍵選單取消「打字時顯示組字中的字（實驗性）」，預設開。
[/list]

[h3]📌 已知限制[/h3]
[list]
[*] 密碼框（遊戲把字遮成星號的輸入欄）不顯示組字，免得把你按的鍵顯示在旁邊，這些欄位照舊盲打。
[*] 組字到一半停手時，組字視窗大約每秒重畫一次，可能看到它閃一下（實測每次 0.05 秒以內）；連續打字時不會。
[*] 多行文字框裡，視窗出現在游標那一行的下一行、靠左；文字超過框寬後，視窗停在框的右緣。
[*] 視窗的字型和顏色由輸入法決定，不是遊戲的字型。
[*] 不是用遊戲原版文字框做的輸入欄抓不到游標，視窗會出現在畫面中間偏下。
[*] 打字途中工具被強制結束或當掉時，畫面上可能留下一塊不更新的組字視窗，重新開啟工具就會清掉。
[/list]
