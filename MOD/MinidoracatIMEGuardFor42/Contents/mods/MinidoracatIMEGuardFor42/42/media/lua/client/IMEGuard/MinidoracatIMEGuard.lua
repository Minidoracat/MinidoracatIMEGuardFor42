--[[
MinidoracatIMEGuardFor42 — 訊號端（純 Lua，零 native）。

【問題】Windows 上任何 IME（注音／拼音／日文／韓文…）在組字模式時，按鍵先被輸入法攔成
VK_PROCESSKEY，GLFW 的 win32 後端直接丟掉（win32_window.c），所以移動與所有操作鍵在進
Java 之前就消失了。Kahlua 碰不到 user32，MOD 本身無法切換輸入法。
【分工】本檔只回答一個問題：「玩家現在是不是在打字？」——把 Core.isDoingTextEntry
（Core.java:2036，GameKeyboard.isKeyDown:125 用的同一個閘門）的變化寫進
  %USERPROFILE%\Zomboid\Lua\MinidoracatIMEGuard\state.txt   內容 "1"＝打字中、"0"＝沒有
配套的 pz-ime-guard 常駐工具讀它：0 → 把 PZ 視窗切到 en-US；1 → 切回玩家原本的輸入法。
工具每 2 秒把 epoch 秒寫進同目錄 heartbeat.txt；進遊戲時讀不到或過期就提醒一次。
【邊界】getFileWriter 只能寫 Zomboid\Lua\ 下的 txt（LuaManager.java:6729-6733,1034）；
只在狀態改變時寫檔，OnTickEvenPaused／OnFETick 每 frame 只多一個 Java getter。
]]

local TAG = "[MinidoracatIMEGuardFor42]"
local DIR = "MinidoracatIMEGuard/"
local STATE_FILE = DIR .. "state.txt"
local HEARTBEAT_FILE = DIR .. "heartbeat.txt"
local HEARTBEAT_MAX_AGE = 10 -- 秒；工具每 2 秒寫一次

local lastTyping = nil
local warnedWriteFailed = false

local function writeState(typing)
    local writer = getFileWriter(STATE_FILE, true, false)
    if not writer then
        if not warnedWriteFailed then
            warnedWriteFailed = true
            print(TAG .. " cannot write Zomboid/Lua/" .. STATE_FILE .. "; companion tool will not see typing state")
        end
        return
    end
    writer:write(typing and "1" or "0")
    writer:close()
end

local function onTick()
    local typing = getCore():isDoingTextEntry()
    if typing == lastTyping then return end
    lastTyping = typing
    writeState(typing)
end

-- OnTickEvenPaused 只在遊戲內觸發（GameWindow.java:366-367），主選單靠 OnFETick（MainScreenState.java:665）
Events.OnTickEvenPaused.Add(onTick)
Events.OnFETick.Add(onTick)

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
