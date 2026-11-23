--[[
煙霧測試：用假的 PZ 全域載入**真正的** MOD Lua，跑行為情境並斷言結果。

    lua scripts/smoke_harness.lua        （repo 根目錄執行；標準 Lua 5.x 即可）

限制（必須誠實面對）：這是標準 Lua，不是遊戲的 Kahlua。
- 標準 Lua 有 next/xpcall，Kahlua 沒有——本 harness **測不出**誤用，
  那由 scripts/verify_mod.py 的靜態掃描負責（發版前兩者都要跑）
- 輸入法切換本身在外部工具（tools/pz-ime-guard），這裡只驗訊號端：「打字狀態變了才寫檔、寫的內容對、
  工具不在線時提醒一次、關程序的四條路徑會寫退出旗標而取消不會、打字中把游標位置與文字變動寫進 caret.txt」。
  實機證據見 .omc/artifacts/e2e-*。
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
function getCore()
    return {
        isDoingTextEntry = function() return typing end,
        getScreenWidth = function() return 1600 end,
        getScreenHeight = function() return 900 end,
    }
end
function getPlayer()
    return { setHaloNote = function(_, text) halo[#halo + 1] = text end }
end

-- 假的 UI 樹：Java 的 ArrayList（size／get 從 0 起算）、面板、UITextBox2。字寬一律 7、行高 20、內距 2。
local ui = {}
local function jlist(t) return { size = function() return #t end, get = function(_, i) return t[i + 1] end } end
local function panel(children, visible)
    return {
        class = "UIElement",
        isVisible = function() return visible ~= false end,
        getControls = function() return jlist(children) end,
    }
end
local function textbox(o)
    return {
        class = "UITextBox2", o = o,
        isVisible = function() return true end,
        isFocused = function() return o.focused end,
        getInset = function() return 2 end,
        getAbsoluteX = function() return o.x end,
        getAbsoluteY = function() return o.y end,
        getWidth = function() return o.w end,
        getTable = function() return nil end,
        isMultipleLine = function() return o.multi or false end,
        getCursorLine = function() return o.line or 0 end,
        getCursorPos = function() return o.cursor end,
        getText = function() return o.text end,
        getYScroll = function() return o.scroll or 0 end,
    }
end
function instanceof(e, name) return e.class == name or (name == "UIElement" and e.class == "UITextBox2") end
UIManager = { getUI = function() return jlist(ui) end }
UIFont = { Small = "Small" }
function getTextManager()
    return { MeasureStringX = function(_, _, s) return 7 * #s end, getFontHeight = function() return 20 end }
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
local CARET = "MinidoracatIMEGuard/caret.txt"

-- ===== 情境一：打字狀態變化才寫檔，內容 1/0 =====
io.write("情境一：訊號寫檔\n")
check(#handlers.OnTickEvenPaused == 1 and #handlers.OnFETick == 1, "遊戲內與主選單各掛一個 tick handler")
local writes, order = {}, {} -- 每個檔寫了幾次、寫的先後
local rawWriter = getFileWriter
getFileWriter = function(name, ...)
    writes[name] = (writes[name] or 0) + 1
    order[#order + 1] = name
    return rawWriter(name, ...)
end
local function n(name) return writes[name] or 0 end
fire("OnTickEvenPaused")
check(files[STATE] == "0" and n(STATE) == 1, "第一個 tick 寫出 0")
fire("OnTickEvenPaused"); fire("OnFETick")
check(n(STATE) == 1, "狀態沒變就不再寫檔")
check(n(CARET) == 0, "沒在打字不寫 caret.txt")
typing = true; fire("OnFETick")
check(files[STATE] == "1" and n(STATE) == 2, "文字框取得焦點 -> 寫 1")
check(files[CARET] == "1 800 600", "找不到聚焦的 UITextBox2 -> 位置給畫面中下方的固定點")
fire("OnTickEvenPaused")
check(n(STATE) == 2 and n(CARET) == 1, "打字中連續 tick 不重寫")
typing = false; fire("OnTickEvenPaused")
check(files[STATE] == "0" and n(STATE) == 3 and n(CARET) == 1, "失焦 -> 寫 0，不動 caret.txt")

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

-- ===== 情境四：正常退出訊號（只有真的要關程序才寫） =====
io.write("情境四：退出訊號\n")
local EXIT = "MinidoracatIMEGuard/exiting.txt"
local called = {}
MainScreen = { quitToDesktop = function() called[#called + 1] = "main" end }
ISPostDeathUI = { onConfirmQuitToDesktop = function(_, button) called[#called + 1] = "death:" .. button.internal end }
ISServerDisconnectUI = { onToDesktop = function() called[#called + 1] = "disconnect" end }
ISTermsOfServiceUI = { onButtonQuit = function() called[#called + 1] = "terms" end }
files[EXIT] = "1" -- 上一局關程序前寫的，工具沒開所以沒被消費掉
fire("OnGameBoot")
check(files[EXIT] == "0", "OnGameBoot 清掉殘留旗標，不讓上一局停守下一局")
MainScreen:quitToDesktop()
check(files[EXIT] == "1" and called[1] == "main", "主選單／遊戲內 Quit 確認後寫 1，原函式照常執行")

fire("OnGameBoot")
ISPostDeathUI:onConfirmQuitToDesktop({ internal = "NO" })
check(files[EXIT] == "0" and called[2] == "death:NO", "死亡畫面按 NO：原函式照跑，但不寫旗標")
ISPostDeathUI:onConfirmQuitToDesktop({ internal = "YES" })
check(files[EXIT] == "1" and called[3] == "death:YES", "死亡畫面按 YES：寫 1")

fire("OnGameBoot")
ISServerDisconnectUI:onToDesktop({})
check(files[EXIT] == "1" and called[4] == "disconnect", "斷線畫面 Quit：寫 1")
fire("OnGameBoot")
ISTermsOfServiceUI:onButtonQuit({})
check(files[EXIT] == "1" and called[5] == "terms", "服務條款 Quit：寫 1")

fire("OnGameBoot") -- 回主選單也會再觸發一次 OnGameBoot（IngameState.java:1069）
local beforeWrites = n(EXIT)
MainScreen:quitToDesktop()
check(n(EXIT) == beforeWrites + 1, "重複 OnGameBoot 不重複包裝（一次退出只寫一次）")

-- ===== 情境五：組字顯示的位置訊號（caret.txt，一行 "<seq> <x> <y>"） =====
io.write("情境五：游標位置與文字變動\n")
local function caret()
    local s, x, y = files[CARET]:match("^(%d+) (%-?%d+) (%-?%d+)$")
    return tonumber(s), tonumber(x), tonumber(y)
end
local function tick(times) for _ = 1, times or 1 do fire("OnTickEvenPaused") end end
local a = textbox({ x = 300, y = 300, w = 400, text = "abc", cursor = 3, focused = true })
local hidden = textbox({ x = 1, y = 1, w = 50, text = "", cursor = 0, focused = true })
ui = { panel({ panel({ a }) }), panel({ hidden }, false) }
order = {}
typing = true; tick()
local seq, x, y = caret()
check(x == 300 + 2 + 7 * 3 and y == 300 + 2, "單行框：游標 = 框左上 + 內距 + 游標前的字寬（跳過看不見的面板）")
check(order[1] == CARET and order[2] == STATE and files[STATE] == "1", "先寫位置再寫 state：工具被叫醒時已經讀得到")
local wrote = n(CARET)
tick(3)
check(n(CARET) == wrote, "文字與位置都沒變就不寫")
a.o.text, a.o.cursor = "abcd", 4; tick()
local seq2, x2 = caret()
check(seq2 == seq + 1 and x2 == 300 + 2 + 7 * 4, "文字變了（送出一個字）-> 立刻寫，seq 加一")
a.o.text = "abcx"; tick()
local seq3, x3 = caret()
check(seq3 == seq2 + 1 and x3 == x2, "文字變了但座標沒變 -> seq 仍加一，整行不同")
wrote = n(CARET)
a.o.x = 310; tick()
check(n(CARET) == wrote, "只有位置變（拖面板）-> 不立刻寫")
tick(14)
seq, x = caret()
check(n(CARET) == wrote + 1 and seq == seq3 and x == 310 + 2 + 7 * 4, "位置變動最多每 15 tick 寫一次，seq 不變")

local b = textbox({ x = 50, y = 700, w = 200, text = "", cursor = 0, focused = true })
a.o.focused = false; ui[#ui + 1] = panel({ b }); tick()
seq, x, y = caret()
check(seq == seq3 + 1 and x == 52 and y == 702, "焦點換到另一個框 -> 重掃找到新的")
local m = textbox({ x = 100, y = 100, w = 300, text = "l1\nl2", cursor = 5, focused = true, multi = true, line = 1, scroll = -4 })
b.o.focused = false; ui[#ui + 1] = panel({ m }); tick()
seq, x, y = caret()
check(x == 102 and y == 100 + 2 + 2 * 20 - 4, "多行框：放在游標行的下一行靠左（含捲動量）")
local wide = textbox({ x = 10, y = 10, w = 60, text = string.rep("w", 50), cursor = 50, focused = true })
m.o.focused = false; ui[#ui + 1] = panel({ wide }); tick()
seq, x = caret()
check(x == 10 + 2 + (60 - 2 * 2), "文字比框寬：游標最遠算到框右緣")

local scans, rawGetUI = 0, UIManager.getUI
UIManager.getUI = function() scans = scans + 1; return rawGetUI() end
wide.o.focused = false; tick()
seq, x, y = caret()
check(scans == 1 and x == 800 and y == 600, "聚焦的不是 UITextBox2 -> 固定點")
tick(29)
check(scans == 1, "找不到框時不每 tick 掃 UI 樹")
a.o.focused = true; tick()
seq, x = caret()
check(scans == 2 and x == 310 + 2 + 7 * 4, "每 30 tick 重掃一次，找到就改用它")
wrote = n(CARET)
typing = false; tick()
check(files[STATE] == "0" and n(CARET) == wrote, "結束打字 -> 寫 0，不動 caret.txt")

io.write("\n")
if failures > 0 then
    io.write(failures, " 項失敗\n")
    os.exit(1)
end
io.write("全部通過\n")
