# -*- coding: utf-8 -*-
"""發版前驗證閘門：一次跑完全部靜態檢查，任一失敗以非零碼結束。

用法（repo 根目錄或任意位置）：
    python scripts/verify_mod.py

零設定：自動偵測 MOD/<folder>/Contents/mods/<folder>/42/。
涵蓋的檢查與其對應的實際事故（皆有反編譯出處，詳見 AGENTS.md 踩坑錄）：

  1. luac -p 語法        — 需要 PATH 有 luac；沒有則列為 SKIP 而非 PASS
 1b. 每個函式的累計 local — Debug 用固定 200 格記錄宣告；含離開作用域的變數，預算 190
  2. BOM / CRLF          — 有 BOM 或 CRLF 的翻譯檔會被引擎「靜默忽略」
  3. 翻譯鍵集一致          — 缺鍵的語系會顯示原始 key
  4. 裸 % 檢查           — 42.20.1 起 formatted() 遇裸 % 崩潰；只允許 %1-%9 與 %%
  5. Kahlua 禁用全域       — next/assert/xpcall 不存在（BaseLib 未註冊），呼叫→
                           「Object tried to call nil」。luac 與標準 Lua 測試都攔不住
                           （語法合法、標準 Lua 有這些函式），只能靜態掃描
  6. table.sort 禁用      — Kahlua 的 sort 是遞迴 quicksort（coroutine 堆疊上限 3000），
                           已排序輸入退化 O(n) 深度、數百筆即溢位；一律用迭代 merge sort
  7. MOD/ 樹雜物          — .omc/.claude/.gitnexus 目錄與 .gitkeep 檔；Workshop 整包上傳不看 .gitignore
 7b. mod.info 多值欄位語法 — require/incompatible/load order 只接受逗號且 key 緊貼 =
  8. 佔位符殘留            — {{TOKEN}} 漏替換
  9. Steam 描述位元組      — 各語言 ≤8000 UTF-8 bytes（中日文 3 bytes/字，容易低估）
 10. 沙盒選項翻譯配對       — 每個 option 要有 Sandbox_<translation> 標題＋ _tooltip＋分頁名
 11. CHANGELOG 洩漏掃描     — bullet 會被整段貼到公開的 Workshop 更新說明；掃基礎設施
                           樣式（/home/ 路徑、IP、SteamID64、ssh、主機名）當最後防線。
                           攻擊配方與玩家識別資訊機器認不出來，靠撰寫規則（AGENTS.md）

新增檢查時：同步把對應的坑記進 AGENTS.md 踩坑錄，並依「踩坑進化協議」回流到
pz-mod-template（見 AGENTS.md）。
"""
import json
import os
import re
import shutil
import subprocess
import sys
import tempfile

REPO = os.path.dirname(os.path.dirname(os.path.abspath(__file__)))

passed, failed, skipped = [], [], []

# 豁免清單（選用）：scripts/verify_ignore.txt，每行一個子字串樣式（# 開頭為註解）。
# 命中樣式的 finding 會列出但不計 FAIL——用於「已逐一查證屬合理例外」的殘留
# （例：翻譯包鏡像了來源 MOD 原文的裸 %）。每個樣式旁必須有註解說明查證依據。
IGNORE_PATTERNS = []
_ign = os.path.join(os.path.dirname(os.path.abspath(__file__)), "verify_ignore.txt")
if os.path.isfile(_ign):
    with open(_ign, encoding="utf-8") as _fh:
        for _line in _fh:
            _line = _line.strip()
            if _line and not _line.startswith("#"):
                IGNORE_PATTERNS.append(_line)


def ok(label):
    passed.append(label)
    print(f"  PASS  {label}")


def fail(label, details=None):
    details = details or []
    kept = [d for d in details if not any(p in d for p in IGNORE_PATTERNS)]
    waived = [d for d in details if any(p in d for p in IGNORE_PATTERNS)]
    for d in waived:
        print(f"  WAIVE {label}: {d}（verify_ignore.txt 豁免）")
    if not kept:
        if waived:
            ok(f"{label}（{len(waived)} 筆豁免）")
        else:
            ok(label)
        return
    failed.append(label)
    print(f"  FAIL  {label}")
    for d in kept:
        print(f"        {d}")


def skip(label, why):
    skipped.append(label)
    print(f"  SKIP  {label} — {why}")


def find_media():
    hits = []
    mod_root = os.path.join(REPO, "MOD")
    if os.path.isdir(mod_root):
        for folder in os.listdir(mod_root):
            p = os.path.join(mod_root, folder, "Contents", "mods")
            if not os.path.isdir(p):
                continue
            for inner in os.listdir(p):
                media = os.path.join(p, inner, "42", "media")
                if os.path.isdir(media):
                    hits.append(media)
    return hits


def iter_files(root, exts):
    for base, dirs, files in os.walk(root):
        dirs[:] = [d for d in dirs if d not in (".git",)]
        for name in files:
            if os.path.splitext(name)[1] in exts:
                yield os.path.join(base, name)


def lua_local_issues(listing):
    # 家族 190 預算；不能只看 main，也不能以同時活躍的 slots 取代累計 locals。
    summaries = re.findall(
        r"^(?:main|function) <([^\n]+)>[^\n]*\n[^\n]*?(\d+) locals?\b",
        listing, re.MULTILINE)
    if not summaries:
        return ["luac 未提供可辨識的函式摘要"]
    return [f"{source}: {count} locals（>190，Kahlua Debug 上限 200）"
            for source, count in summaries if int(count) > 190]


def self_test_lua_limits():
    compiler = shutil.which("luac")
    if not compiler:
        raise RuntimeError("local 邊界測試需要 luac")
    with tempfile.TemporaryDirectory() as directory:
        path = os.path.join(directory, "local limits.lua")
        for name, count, nested, reject in (
                ("at-budget", 190, False, False),
                ("over-budget", 191, False, True),
                ("nested-expired-locals", 201, True, True)):
            source = "do local value = 1 end\n" * count
            if nested:
                source = "local function nested()\n" + source + "end\n"
            with open(path, "w", encoding="utf-8", newline="\n") as stream:
                stream.write(source)
            result = subprocess.run([compiler, "-p", "-l", path], capture_output=True,
                                    text=True, encoding="utf-8", errors="replace", check=True)
            if bool(lua_local_issues(result.stdout)) != reject:
                raise AssertionError(name)
    if not lua_local_issues(""):
        raise AssertionError("missing compiler summary must fail closed")
    print("PASS Lua local 邊界：190／191、內層函式的失效作用域、缺少編譯摘要")


if __name__ == "__main__" and "--self-test-lua-limits" in sys.argv:
    self_test_lua_limits()
    sys.exit(0)


MEDIA_DIRS = find_media()
if not MEDIA_DIRS:
    print("找不到 MOD/*/Contents/mods/*/42/media，中止")
    sys.exit(2)

LUA_FILES = [f for m in MEDIA_DIRS for f in iter_files(os.path.join(m, "lua"), {".lua"})
             if os.path.isdir(os.path.join(m, "lua"))]

# ---- 1. luac 語法 ----
luac = shutil.which("luac")
if not luac:
    skip("Lua 語法（luac -p）", "PATH 沒有 luac")
    skip("Kahlua local 預算（每個函式 ≤190）", "PATH 沒有 luac")
else:
    bad, bad_limits = [], []
    for f in LUA_FILES:
        r = subprocess.run([luac, "-p", "-l", f], capture_output=True,
                           text=True, encoding="utf-8", errors="replace")
        if r.returncode != 0:
            bad.append(r.stderr.strip().splitlines()[-1] if r.stderr else f)
            bad_limits.append(f"{os.path.relpath(f, REPO)}: 語法失敗，無法檢查 local 預算")
        else:
            bad_limits.extend(lua_local_issues(r.stdout))
    fail("Lua 語法（luac -p）", bad) if bad else ok(f"Lua 語法（luac -p，{len(LUA_FILES)} 檔）")
    fail("Kahlua local 預算（每個函式 ≤190）", bad_limits) if bad_limits \
        else ok("Kahlua local 預算（每個函式 ≤190）")

# ---- 2. BOM / CRLF ----
bad = []
for m in MEDIA_DIRS:
    for f in iter_files(m, {".lua", ".json", ".txt"}):
        with open(f, "rb") as fh:
            data = fh.read()
        rel = os.path.relpath(f, REPO)
        if data.startswith(b"\xef\xbb\xbf"):
            bad.append(f"BOM: {rel}")
        if b"\r" in data:
            bad.append(f"CRLF: {rel}")
fail("BOM / CRLF（42/media 下）", bad) if bad else ok("BOM / CRLF（42/media 下）")

# ---- 3+4. 翻譯鍵集一致 / 裸 % ----
# 裸 % 的判定分兩種模式：
#   嚴格（家族自製 MOD，語系含 EN 等四語）：只認引擎 Translator.formatted() 的 %1-%9 與 %%
#   寬容（翻譯包，語系 ⊆ {CH,CN}）：另接受 printf 指令（%s/%d/%.1f…）——第三方 MOD 常用
#     string.format(getText(...)) 消費譯文，這時保留 %d 才是對的，逸出反而弄壞
# 刻意不含 printf 旗標字元（-+空白#0）：含空白旗標會讓「50% done」的「% d」被解析成
# 合法指令而漏抓——翻譯實務上只會出現簡單的 %s/%d/%.1f，罕見旗標用法交給豁免清單
PRINTF_RE = re.compile(r"%\d*(?:\.\d+)?[sdifuxXcqgGeE]")


def find_bare_pct(value, tolerant):
    s = str(value)
    i = 0
    while i < len(s):
        if s[i] != "%":
            i += 1
            continue
        if i + 1 < len(s) and s[i + 1] in "123456789%":
            i += 2          # 消耗合法配對——lookahead 不消耗會把 "40%%" 誤報（踩過）
            continue
        if tolerant:
            mm = PRINTF_RE.match(s, i)
            if mm:
                i = mm.end()
                continue
        return True
    return False


for m in MEDIA_DIRS:
    troot = os.path.join(m, "lua", "shared", "Translate")
    if not os.path.isdir(troot):
        continue
    langs = sorted(d for d in os.listdir(troot) if os.path.isdir(os.path.join(troot, d)))
    tolerant = set(langs) <= {"CH", "CN"}   # 翻譯包偵測
    names = sorted({n for l in langs for n in os.listdir(os.path.join(troot, l)) if n.endswith(".json")})
    mismatch, badpct, broken = [], [], []
    for n in names:
        keysets = {}
        for l in langs:
            p = os.path.join(troot, l, n)
            if not os.path.isfile(p):
                mismatch.append(f"{n}: {l} 缺檔")
                continue
            try:
                with open(p, encoding="utf-8") as fh:
                    data = json.load(fh)
            except Exception as e:
                broken.append(f"{l}/{n}: {e}")
                continue
            keysets[l] = set(data)
            for k, v in data.items():
                if find_bare_pct(v, tolerant):
                    badpct.append(f"{l}/{n} 的 {k}")
        if len(keysets) > 1:
            base = next(iter(keysets.values()))
            for l, ks in keysets.items():
                if ks != base:
                    mismatch.append(f"{n}: {l} 鍵集不一致（差 {len(ks ^ base)} 鍵）")
    if broken:
        fail("翻譯 JSON 可解析", broken)
    else:
        ok("翻譯 JSON 可解析")
    fail("翻譯鍵集一致", mismatch) if mismatch else ok(f"翻譯鍵集一致（{'/'.join(langs)}）")
    pct_label = "翻譯值無裸 %（翻譯包模式：另接受 printf 指令）" if tolerant else "翻譯值無裸 %（僅 %1-%9 與 %%）"
    fail(pct_label, sorted(set(badpct))) if badpct else ok(pct_label)

# ---- 5+6. Kahlua 禁用全域 / table.sort ----
FORBIDDEN = ("next", "assert", "xpcall")
hits_forbidden, hits_sort = [], []
for f in LUA_FILES:
    rel = os.path.relpath(f, REPO)
    with open(f, encoding="utf-8") as fh:
        for lineno, line in enumerate(fh, 1):
            code = line.split("--", 1)[0]
            for name in FORBIDDEN:
                for mm in re.finditer(rf"(?<![\w_:.]){name}\s*\(", code):
                    hits_forbidden.append(f"{rel}:{lineno} 用了 {name}()")
            if re.search(r"(?<![\w_])table\.sort\s*\(", code):
                hits_sort.append(f"{rel}:{lineno}")
fail("Kahlua 禁用全域（next/assert/xpcall）", hits_forbidden) if hits_forbidden \
    else ok("Kahlua 禁用全域（next/assert/xpcall）")
fail("無 table.sort（用迭代 sortSafe，見 AGENTS.md）", hits_sort) if hits_sort \
    else ok("無 table.sort")

# ---- 7. MOD/ 樹雜物 ----
# .gitkeep 也算雜物：引擎會把 MOD 樹內任何檔案列舉成 mod 資源（console 出現
# "overrides media/lua/client/.gitkeep"），且 Workshop 上傳整包不看 .gitignore。
# MOD/ 樹內空目錄不撐 .gitkeep，靠首個實檔建立（引擎對不存在的 lua 子目錄不報錯）。
junk = []
for base, dirs, files in os.walk(os.path.join(REPO, "MOD")):
    for d in list(dirs):
        if d in (".omc", ".claude", ".gitnexus"):
            junk.append(os.path.relpath(os.path.join(base, d), REPO))
            dirs.remove(d)
    for name in files:
        if name == ".gitkeep":
            junk.append(os.path.relpath(os.path.join(base, name), REPO))
fail("MOD/ 樹無雜物（AI 狀態目錄／.gitkeep）", junk) if junk \
    else ok("MOD/ 樹無雜物（AI 狀態目錄／.gitkeep）")

# ---- 7b. mod.info 多值欄位語法 ----
# ChooseGameInfo.java:224/226/228/230 用 contains("key=") 後直接 split(",")。
manifest_bad = []
multi_keys = ("require", "incompatible", "loadModAfter", "loadModBefore")
canonical_re = re.compile(
    r"^\s*(require|incompatible|loadModAfter|loadModBefore)=(.*)$")
for base, _, files in os.walk(os.path.join(REPO, "MOD")):
    if "mod.info" not in files:
        continue
    info = os.path.join(base, "mod.info")
    rel_info = os.path.relpath(info, REPO)
    with open(info, encoding="utf-8") as fh:
        for lineno, line in enumerate(fh, 1):
            for key in multi_keys:
                marker = key + "="
                if marker in line:
                    match = canonical_re.match(line.rstrip("\r\n"))
                    if not match or match.group(1) != key:
                        manifest_bad.append(
                            f"{rel_info}:{lineno}: {key}= 前不得有註解或其他文字")
                        continue
                    value = match.group(2)
                    if "#" in value:
                        manifest_bad.append(
                            f"{rel_info}:{lineno}: mod.info 不支援 {key} 行尾註解")
                    if ";" in value:
                        manifest_bad.append(
                            f"{rel_info}:{lineno}: {key} 多值必須用逗號，不是分號")
                elif re.search(rf"{key}\s+=", line):
                    manifest_bad.append(
                        f"{rel_info}:{lineno}: {key}= 鍵名與等號間不得有空格")
fail("mod.info 多值欄位語法", manifest_bad) if manifest_bad \
    else ok("mod.info 多值欄位語法")

# ---- 8. 佔位符殘留 ----
tokens = []
SELF = os.path.abspath(__file__)   # 本檔 docstring 有 {{TOKEN}} 範例字樣，排除自己
for base, dirs, files in os.walk(REPO):
    dirs[:] = [d for d in dirs if d not in (".git", ".omc", ".claude", ".gitnexus", "__pycache__")]
    for name in files:
        p = os.path.join(base, name)
        if os.path.abspath(p) == SELF:
            continue
        try:
            with open(p, encoding="utf-8") as fh:
                text = fh.read()
        except (UnicodeDecodeError, OSError):
            continue
        for mm in re.finditer(r"\{\{[A-Z_]+\}\}", text):
            tokens.append(f"{os.path.relpath(p, REPO)}: {mm.group()}")
fail("無 {{TOKEN}} 佔位符殘留", tokens) if tokens else ok("無 {{TOKEN}} 佔位符殘留")

# ---- 9. Steam 描述位元組 ----
descs = [f for f in os.listdir(REPO) if f.startswith("STEAM_DESCRIPTION") and f.endswith(".md")]
over = []
for f in descs:
    size = os.path.getsize(os.path.join(REPO, f))
    if size > 8000:
        over.append(f"{f}: {size} bytes（上限 8000）")
if descs:
    fail("Steam 描述 ≤8000 bytes", over) if over else ok(f"Steam 描述 ≤8000 bytes（{len(descs)} 檔）")

# ---- 10. 沙盒選項翻譯配對 ----
for m in MEDIA_DIRS:
    sb = os.path.join(m, "sandbox-options.txt")
    if not os.path.isfile(sb):
        continue
    with open(sb, encoding="utf-8") as fh:
        txt = fh.read()
    opts = set(re.findall(r"translation\s*=\s*(\S+?)\s*,", txt))
    pages = set(re.findall(r"page\s*=\s*(\S+?)\s*,", txt))
    ch = os.path.join(m, "lua", "shared", "Translate", "CH", "Sandbox.json")
    if not os.path.isfile(ch):
        fail("沙盒選項翻譯配對", ["有 sandbox-options.txt 但無 CH/Sandbox.json"])
        continue
    with open(ch, encoding="utf-8") as fh:
        keys = set(json.load(fh))
    miss = [f"缺標題: Sandbox_{o}" for o in opts if f"Sandbox_{o}" not in keys]
    miss += [f"缺 tooltip: Sandbox_{o}_tooltip" for o in opts if f"Sandbox_{o}_tooltip" not in keys]
    miss += [f"缺分頁名: Sandbox_{p}" for p in pages if f"Sandbox_{p}" not in keys]
    fail("沙盒選項翻譯配對", miss) if miss else ok(f"沙盒選項翻譯配對（{len(opts)} 選項）")

# ---- 11. CHANGELOG 洩漏掃描 ----
LEAK_PATTERNS = [
    (re.compile(r"/home/\w+"), "Linux 家目錄路徑"),
    (re.compile(r"[A-Z]:\\Users\\"), "Windows 使用者路徑"),
    (re.compile(r"\b(?:\d{1,3}\.){3}\d{1,3}\b"), "IPv4 位址"),
    (re.compile(r"\b7656\d{13}\b"), "SteamID64"),
    (re.compile(r"\bssh\b", re.IGNORECASE), "ssh 字樣"),
    (re.compile(r"pz-?server", re.IGNORECASE), "伺服器主機名"),
]
_cl = os.path.join(REPO, "CHANGELOG.md")
if os.path.isfile(_cl):
    leaks = []
    with open(_cl, encoding="utf-8") as fh:
        for lineno, line in enumerate(fh, 1):
            for pat, desc in LEAK_PATTERNS:
                mm = pat.search(line)
                if mm:
                    leaks.append(f"CHANGELOG.md:{lineno} {desc}（{mm.group()[:40]}）")
    fail("CHANGELOG 無基礎設施洩漏樣式", leaks) if leaks else ok("CHANGELOG 無基礎設施洩漏樣式")

# ---- 12. drainable 輸入消耗語意（本 MOD drainable）----
# CraftRecipeManager.java:633/652：item 輸入沒有 flags[ItemCount] 時計量按
# getCurrentUses() 而非件數——空電 drainable 每件貢獻 0 卻仍被收進消耗集合
# （:621-630），湊足需求那一刻整批連坐銷毀（processDestroyAndUsedItems，
# CraftRecipeData.java:544-564）。AutoDrive 正式服實爆：39 個空電 GPS 一次製作全滅。
# 兩種合法寫法：flags[ItemCount]（按件計，Base.Battery 型，recipes_electrical.txt:30）
# 或 flags[IsFull]（滿件才可入料、天然無空件連坐，Base.Claybag 型，
# recipes_sacks.txt:27）。非 destroy mode 也不安全：空件同樣被連坐收集，且 UseItem
# 對 uses<=0 且無 KeepOnDeplete=true 的物品一樣 RemoveItem（ItemUser.java:69-72）。
# 引擎 parser 逐項對齊：mode 的 key 是字面 `mode:`（大小寫敏感，InputScript.java:666
# startsWith），值才 equalsIgnoreCase（:670-674），缺省 Normal（:68），同行多個
# mode: token 取最後；flags split(";") 後直接 InputFlag.valueOf（:744-748，無 trim、
# 大小寫敏感）——閘門比引擎寬（re.I、strip、substring search）就會放行載入期會炸
# 或行為相反的寫法。輕量掃描：只認本 MOD scripts 內宣告的 drainable 短名與其 Tags
# （原版 drainable 的輸入這裡看不到；同名跨 module 極罕見，寧可誤報）。
drain_bad = []


def _input_mode(raw):
    # mode:use 是引擎 no-op（InputScript.java:670 守衛外不賦值），本閘門對它偏嚴；
    # 適用前提：輸入行無 +/- 續行（續行在 OnPostWorldDictionaryInit 覆寫 mode 成
    # Keep，InputScript.java:817、:834-835）。
    mode = "normal"
    for tok in raw.split():
        if tok.startswith("mode:"):
            mode = tok[5:].rstrip(",").lower()
    return mode


def _bracket_union(raw, key):
    # 引擎逐 token 累積 key[...]（InputScript.java:742-750），first-match 會漏第二個。
    # 各項不 strip：flags 走 InputFlag.valueOf 無 trim。
    out = set()
    for mm in re.finditer(rf"(?<!\w){key}\[([^\]]+)\]", raw):
        out.update(mm.group(1).split(";"))
    return out


def _norm_tag(t):
    # tag 走 ResourceLocation.of：lower＋無 namespace 補 base:（ResourceLocation.java:18-29）
    t = t.strip().lower()
    return t if ":" in t else f"base:{t}"


for m in MEDIA_DIRS:
    sdir = os.path.join(m, "scripts")
    if not os.path.isdir(sdir):
        continue
    texts = []
    for f in iter_files(sdir, {".txt"}):
        with open(f, encoding="utf-8") as fh:
            texts.append((os.path.relpath(f, REPO),
                          re.sub(r"/\*.*?\*/", "", fh.read(), flags=re.S)))
    drain_keep, drain_tag_map = {}, {}
    for _, txt in texts:
        for mm in re.finditer(r"(?<![\w.])item\s+(\w+)\s*\{([^{}]*)\}", txt):
            body = mm.group(2)
            if not re.search(r"ItemType\s*=\s*base:drainable", body, re.I):
                continue
            drain_keep[mm.group(1)] = \
                re.search(r"KeepOnDeplete\s*=\s*true", body, re.I) is not None
            tg = re.search(r"Tags\s*=\s*([^,\r\n]+)", body)
            drain_tag_map[mm.group(1)] = {_norm_tag(t) for t in tg.group(1).split(";")
                                          if t.strip()} if tg else set()
    if not drain_keep:
        continue
    for rel, txt in texts:
        for lineno, line in enumerate(txt.splitlines(), 1):
            im = re.match(r"\s*item\s+\d+\s+(.*)", line)
            if not im:
                continue
            rest = im.group(1)
            br = re.search(r"(?<!\w)\[([^\]]+)\]", rest)
            hit = set()
            if br:
                hit = {t.strip().rsplit(".", 1)[-1]
                       for t in br.group(1).split(";")} & set(drain_keep)
            in_tags = {_norm_tag(t) for t in _bracket_union(rest, "tags")}
            if in_tags:
                hit |= {name for name, tags in drain_tag_map.items() if in_tags & tags}
            if not hit:
                continue
            flags = _bracket_union(rest, "flags")
            if "ItemCount" in flags or "IsFull" in flags:
                continue
            mode = _input_mode(rest)
            if mode == "destroy":
                drain_bad.append(f"{rel}:{lineno} `{rest.strip()}` —— drainable 走 destroy "
                                 f"必須帶 flags[ItemCount]（或 IsFull），否則空件被連坐吞噬")
            elif any(not drain_keep[s] for s in hit):
                drain_bad.append(f"{rel}:{lineno} `{rest.strip()}` —— 非 destroy 的 drainable "
                                 f"輸入（mode={mode}）需該物品宣告 KeepOnDeplete = true，"
                                 f"否則空件耗盡被靜默移除（ItemUser.java:69-72）且同樣被連坐收集")
fail("drainable 輸入消耗語意（ItemCount/IsFull/KeepOnDeplete）", drain_bad) if drain_bad \
    else ok("drainable 輸入消耗語意（ItemCount/IsFull/KeepOnDeplete；本 MOD drainable）")

# ---- 總結 ----
print()
print(f"PASS {len(passed)} / FAIL {len(failed)} / SKIP {len(skipped)}")
sys.exit(1 if failed else 0)
