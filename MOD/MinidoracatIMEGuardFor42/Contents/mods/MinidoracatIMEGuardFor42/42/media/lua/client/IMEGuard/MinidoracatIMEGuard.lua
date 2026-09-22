--[[
MinidoracatIMEGuardFor42 — 訊號端（純 Lua，零 native）。

【問題】Windows 上任何 IME（注音／拼音／日文／韓文…）在組字模式時，按鍵先被輸入法攔成
VK_PROCESSKEY，GLFW 的 win32 後端直接丟掉（win32_window.c），所以移動與所有操作鍵在進
Java 之前就消失了。Kahlua 碰不到 user32，MOD 本身無法切換輸入法。
【分工】本檔只回答兩個問題，其餘都由配套的 pz-ime-guard 常駐工具處理：
  1.「玩家現在是不是在打字？」——Core.isDoingTextEntry（Core.java:2036，GameKeyboard.isKeyDown:125
     用的同一個閘門）的變化寫進
       %USERPROFILE%\Zomboid\Lua\MinidoracatIMEGuard\state.txt   "1"＝打字中、"0"＝沒有
     工具讀它：0 → 把 PZ 視窗切到 en-US；1 → 切回玩家原本的輸入法。
  2.「玩家是不是按了關程序？」——見下方「正常退出」。
工具每 2 秒把 epoch 秒寫進同目錄 heartbeat.txt；進遊戲時讀不到或過期就提醒一次。
【邊界】getFileWriter 只能寫 Zomboid\Lua\ 下的 txt（LuaManager.java:6729-6733,1034）；
只在狀態改變時寫檔，OnTickEvenPaused／OnFETick 每 frame 只多一個 Java getter。
]]

local TAG = "[MinidoracatIMEGuardFor42]"
local DIR = "MinidoracatIMEGuard/"
local STATE_FILE = DIR .. "state.txt"
local EXIT_FILE = DIR .. "exiting.txt"
local HEARTBEAT_FILE = DIR .. "heartbeat.txt"
local HEARTBEAT_MAX_AGE = 10 -- 秒；工具每 2 秒寫一次

local lastTyping = nil
local warnedWriteFailed = false
local wrapped = {} -- 方法名 -> 我們裝上去的那一份；Lua 重載換新表時比對得出來要不要重裝

local function writeSignal(name, content)
    local writer = getFileWriter(name, true, false)
    if not writer then
        if not warnedWriteFailed then
            warnedWriteFailed = true
            print(TAG .. " cannot write Zomboid/Lua/" .. name .. "; companion tool will not see this signal")
        end
        return
    end
    writer:write(content)
    writer:close()
end

local function onTick()
    local typing = getCore():isDoingTextEntry()
    if typing == lastTyping then return end
    lastTyping = typing
    writeSignal(STATE_FILE, typing and "1" or "0")
end

-- OnTickEvenPaused 只在遊戲內觸發（GameWindow.java:366-367），主選單靠 OnFETick（MainScreenState.java:665）
Events.OnTickEvenPaused.Add(onTick)
Events.OnFETick.Add(onTick)

--[[ 正常退出（exiting.txt，"1"＝玩家已確認關程序）

為什麼需要：getCore():quitToDesktop() 只把 GameWindow.closeRequested 設 true（Core.java:2155），
之後 GameWindow.exit() 要跑完存檔、斷線、關 Steam、壓日誌、釋放 native 才結束（GameWindow.java:765-860），
這整段視窗還在、標題不變、多半還是前景。工具只看視窗，會一直以為還在遊戲裡繼續守 en-US，
玩家回到桌面時鍵盤就留在英文。所以由 Lua 在「已確認、且真的要關程序」的那一刻先講一聲。

只掛四條路徑，全部都在按下確認之後、exit() 阻塞之前，而且一定呼叫 getCore():quitToDesktop()：
  MainScreen:quitToDesktop            主選單 EXIT 與遊戲內 Quit 的共同終點（MainScreen.lua:1495-1509）
  ISPostDeathUI:onConfirmQuitToDesktop 死亡畫面（ISPostDeathUI.lua:132-140；只有 YES 那一支）
  ISServerDisconnectUI:onToDesktop     斷線畫面（ISServerDisconnectUI.lua:37-40）
  ISTermsOfServiceUI:onButtonQuit      服務條款畫面（ISTermsOfServiceUI.lua:118-120）
不掛 getCore():quit()／exitToMenu()：那是回主選單，視窗還在，工具應該繼續守。
沒有 OnQuit／OnExit 這種事件（LuaEventManager.AddEvents 清單裡沒有），不要假裝有。
蓋不到視窗 X／crash／taskkill——那些不跑 Lua，工具那邊靠「視窗消失」的既有 fallback。

殘留處理：關程序前寫的旗標若沒被工具消費掉（工具沒開、被殺掉），會留在磁碟上。OnGameBoot
（開機 GameWindow.java:669、回主選單 IngameState.java:1069、Lua reload Core.java:3962）一律先寫回 "0"，
工具那邊也只認自己啟動之後才寫入的版本，兩邊各自擋一次，才不會讓上一局的旗標把下一局停守。
]]
local function signalExit()
    writeSignal(EXIT_FILE, "1")
end

local function hookQuitToDesktop()
    -- 取代類別表上的方法；實例透過 metatable 找到的也是這一份
    local function wrap(owner, name, shouldSignal)
        if type(owner) ~= "table" then return end
        local original = owner[name]
        -- 已經是我們這一份就不要再包一層（OnGameBoot 開機／回主選單／Lua 重載都會觸發）
        if type(original) ~= "function" or original == wrapped[name] then return end
        local fn = function(self, button, ...)
            if shouldSignal == nil or shouldSignal(button) then
                signalExit()
            end
            return original(self, button, ...)
        end
        wrapped[name] = fn
        owner[name] = fn
    end
    -- ISModalDialog 的回呼對 YES／NO 都會叫同一個函式（ISModalDialog.lua:62），只有 YES 才是真的要關
    local function confirmed(button)
        return button ~= nil and button.internal == "YES"
    end
    wrap(MainScreen, "quitToDesktop", nil)
    wrap(ISPostDeathUI, "onConfirmQuitToDesktop", confirmed)
    wrap(ISServerDisconnectUI, "onToDesktop", nil)
    wrap(ISTermsOfServiceUI, "onButtonQuit", nil)
end

Events.OnGameBoot.Add(function()
    writeSignal(EXIT_FILE, "0")
    hookQuitToDesktop()
end)

local function toolAlive()
    local reader = getFileReader(HEARTBEAT_FILE, false)
    if not reader then return false end
    local line = reader:readLine()
    reader:close()
    local stamp = line and tonumber(line)
    return stamp ~= nil and math.abs(getTimestamp() - stamp) <= HEARTBEAT_MAX_AGE
end

Events.OnGameStart.Add(function()
    if toolAlive() then
        print(TAG .. " companion tool alive")
        return
    end
    print(TAG .. " companion tool not detected: Zomboid/Lua/" .. HEARTBEAT_FILE .. " missing or stale")
    local player = getPlayer()
    if player then
        player:setHaloNote(getText("UI_MinidoracatIMEGuard_ToolMissing"), 255, 200, 80, 400)
    end
end)
