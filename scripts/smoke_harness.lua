--[[
煙霧測試：用假的 PZ 全域載入**真正的** MOD Lua，跑行為情境並斷言結果。

    lua scripts/smoke_harness.lua        （repo 根目錄執行；標準 Lua 5.x 即可）

限制（必須誠實面對）：這是標準 Lua，不是遊戲的 Kahlua。
- 標準 Lua 有 next/assert/xpcall，Kahlua 沒有——本 harness **測不出**誤用，
  那由 scripts/verify_mod.py 的靜態掃描負責（發版前兩者都要跑）
- 輸入法切換本身在外部工具（tools/pz-ime-guard），這裡只驗訊號端：
  「打字狀態變了才寫檔、寫的內容對、工具不在線時提醒一次」。實機證據見 .omc/artifacts/e2e-*。
]]

local MEDIA = "MOD/MinidoracatIMEGuardFor42/Contents/mods/MinidoracatIMEGuardFor42/42/media/lua"

-- ===== 假的 PZ 全域 =====
local now = 1789590000            -- getTimestamp 是 epoch 秒
local typing = false
local files = {}                  -- Zomboid/Lua/<name> -> content
local writerAvailable = true
local prints, halo = {}, {}

function getTimestamp() return now end
function getText(key) return key end
function print(...) prints[#prints + 1] = table.concat({ ... }, " ") end
function getCore() return { isDoingTextEntry = function() return typing end } end
function getPlayer()
    return { setHaloNote = function(_, text) halo[#halo + 1] = text end }
end
function getFileWriter(name, _, _)
    if not writerAvailable then return nil end
    files[name] = ""
    return { write = function(_, s) files[name] = files[name] .. s end, close = function() end }
end
function getFileReader(name, _)
    local content = files[name]
    if content == nil then return nil end
    local done = false
    return {
        readLine = function() if done or content == "" then return nil end done = true return content end,
        close = function() end,
    }
end

local handlers = {}
Events = setmetatable({}, {
    __index = function(_, name)
        return { Add = function(fn) handlers[name] = handlers[name] or {}; table.insert(handlers[name], fn) end }
    end,
})
local function fire(name) for _, fn in ipairs(handlers[name] or {}) do fn() end end

-- ===== 載入受測程式碼 =====
local chunk = assert(loadfile(MEDIA .. "/client/IMEGuard/MinidoracatIMEGuard.lua"))
chunk()

-- ===== 測試工具 =====
local failures = 0
local function check(ok, label)
    if ok then io.write("  PASS  ", label, "\n")
    else failures = failures + 1; io.write("  FAIL  ", label, "\n") end
end
local STATE, HB = "MinidoracatIMEGuard/state.txt", "MinidoracatIMEGuard/heartbeat.txt"

-- ===== 情境一：打字狀態變化才寫檔，內容 1/0 =====
io.write("情境一：訊號寫檔\n")
check(#handlers.OnTickEvenPaused == 1 and #handlers.OnFETick == 1, "遊戲內與主選單各掛一個 tick handler")
local writes = 0
local rawWriter = getFileWriter
getFileWriter = function(...) writes = writes + 1; return rawWriter(...) end
fire("OnTickEvenPaused")
check(files[STATE] == "0" and writes == 1, "第一個 tick 寫出 0")
fire("OnTickEvenPaused"); fire("OnFETick")
check(writes == 1, "狀態沒變就不再寫檔")
typing = true; fire("OnFETick")
check(files[STATE] == "1" and writes == 2, "文字框取得焦點 -> 寫 1")
fire("OnTickEvenPaused")
check(writes == 2, "打字中連續 tick 不重寫")
typing = false; fire("OnTickEvenPaused")
check(files[STATE] == "0" and writes == 3, "失焦 -> 寫 0")

-- ===== 情境二：寫檔失敗只警告一次、不炸 =====
io.write("情境二：寫檔失敗\n")
writerAvailable = false
local before = #prints
typing = true; fire("OnTickEvenPaused")
typing = false; fire("OnTickEvenPaused")
check(#prints == before + 1 and prints[#prints]:find("cannot write", 1, true), "getFileWriter 回 nil：一次警告，後續靜默")
writerAvailable = true

-- ===== 情境三：進遊戲時的工具在線判定 =====
io.write("情境三：heartbeat\n")
local function gameStart() halo = {}; fire("OnGameStart"); return #halo end
files[HB] = nil
check(gameStart() == 1 and halo[1] == "UI_MinidoracatIMEGuard_ToolMissing", "沒有 heartbeat -> 提醒一次")
files[HB] = tostring(now - 3)
check(gameStart() == 0, "3 秒前的 heartbeat -> 不提醒")
files[HB] = tostring(now - 11)
check(gameStart() == 1, "11 秒前的 heartbeat -> 過期，提醒")
files[HB] = ""
check(gameStart() == 1, "空的 heartbeat 檔（工具正在改寫）-> 視為不在線，不炸")
files[HB] = "garbage"
check(gameStart() == 1, "非數字內容 -> 視為不在線，不炸")
files[HB] = tostring(now + 5)
check(gameStart() == 0, "時鐘稍微超前的 heartbeat 也算在線")

io.write("\n")
if failures > 0 then
    io.write(failures, " 項失敗\n")
    os.exit(1)
end
io.write("全部通過\n")
