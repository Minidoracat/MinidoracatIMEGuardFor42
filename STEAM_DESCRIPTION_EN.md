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

[h2]🔒 What it does and does not do[/h2]
[list]
[*] The mod is pure Lua: it only writes a "typing" flag into your Zomboid folder. No native code.
[*] The tool only acts while PZ is the foreground window, using the official Windows message to change that window's layout. No keyboard hook, no key injection, no registry changes, no other windows touched.
[*] Open source: [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url]
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
