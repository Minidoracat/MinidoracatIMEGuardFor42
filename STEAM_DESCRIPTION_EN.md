[h1]⌨️ Minidoracat IME Guard for B42[/h1]
[h3]By Minidoracat[/h3]

[hr][/hr]

[b]⚠️ Two-part mod: subscribing alone does nothing. You also need the small Windows companion tool (see below).[/b]

[h2]✨ What it is[/h2]
Keeps Project Zomboid on an English keyboard layout while you play, and switches back to your own IME the moment you start typing (chat, search, naming). Works with Bopomofo, Pinyin, Japanese, Korean — any Windows IME.

[h2]🧟 Why your IME gets you killed[/h2]
On Windows, whenever a CJK IME is in composition mode every keystroke goes to the IME first and PZ simply drops it — WASD, Shift to run, Space, E, all dead. Worse, PZ's own controls are Windows' layout hotkeys (Shift+Alt, Ctrl+Space), so you get switched back mid-game. The developers have said this will not be handled in-game. This mod exists for that.

[h2]🧰 How to use[/h2]
[olist]
[*] Subscribe and enable this mod.
[*] Download [b]pz-ime-guard.exe[/b] from [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] and run it. No window — just a keycap icon in the system tray (usually under the ^ overflow), with a small lamp in its corner: green = English layout active, orange = typing, your IME restored, grey = PZ not found or paused, red = no English (US) keyboard installed.
[*] Play. If the tool is not running when you enter a world, you get one reminder.
[/olist]
Tip: enable Windows "Let me set a different input method for each app window" so the switch only affects PZ.

[h2]🌐 Supported IMEs and languages[/h2]
[list]
[*] [b]Guarded IMEs:[/b] Chinese (Bopomofo, Cangjie, Boshiamy, Microsoft/Sogou Pinyin, RIME…), Japanese (Microsoft IME, Google Japanese Input…), Korean — Chinese and Japanese confirmed by players, Korean confirmed by Microsoft's IME documentation. Vietnamese Telex/VNI and Indic Phonetic IMEs built into Windows use the same mechanism but are not yet verified in-game.
[*] [b]Left alone:[/b] plain keyboard layouts (any English variant, Russian, German, French, Thai…) — they never had this problem.
[*] [b]UI languages:[/b] tool tooltips, menu and in-game notice in Traditional/Simplified Chinese, Japanese, Korean; English otherwise. Steam page in EN/繁/简/日/한.
[/list]

[h2]🦀 Why Rust, why open source[/h2]
[list]
[*] One ~450 KB exe. No .NET, Java or any runtime to install — download and run.
[*] Rust is memory-safe, so no buffer-overflow class of bugs; idle cost is ~0.3% of one core and under 2 MB RAM.
[*] Only public Windows APIs: enumerate windows, read the keyboard layout, post one switch message. No keyboard hook, no reading your keystrokes, no network, no registry writes, no other windows touched. The mod itself is pure Lua and only writes a "typing" flag into your Zomboid folder.
[*] Source on [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url] — anyone can audit it or build their own. The exe is not code-signed, so SmartScreen may warn about an unknown publisher; if in doubt, cargo build it yourself.
[/list]

[h2]📋 Mod info[/h2]
[list]
[*] [b]Mod ID:[/b] MinidoracatIMEGuardFor42
[*] [b]Version:[/b] Build 42.20.4+
[*] [b]Singleplayer / Multiplayer:[/b] both (client-side only; servers need nothing)
[*] [b]Platform:[/b] tool is Windows-only; macOS/Linux do not have this problem
[/list]

[h2]💬 Feedback[/h2]
[list]
[*] [url=https://discord.gg/Gur2V67]Discord[/url]
[/list]


[b]#Minidoracat[/b]
