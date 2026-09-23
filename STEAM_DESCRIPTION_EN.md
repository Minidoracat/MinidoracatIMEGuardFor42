[h1]⌨️ Minidoracat IME Guard for B42[/h1]
[h3]By Minidoracat[/h3]

[hr][/hr]

[b]⚠️ Two-part mod: subscribing alone does nothing. You also need the small Windows companion tool (see below).[/b]

[b]🎮 Works in singleplayer and multiplayer.[/b] SP: movement keys stop dying to the IME, and text boxes (search, naming) still switch back. [b]MP: the server must list this mod[/b] (PZ only loads mods on the server's list); it does nothing server-side and touches no saves.

[h2]✨ What it is[/h2]
Keeps Project Zomboid on an English keyboard layout while you play, and switches back to your own IME the moment you start typing (chat, search, naming). Works with Bopomofo, Pinyin, Japanese, Korean — any Windows IME.

[h2]🧟 Why your IME gets you killed[/h2]
On Windows, whenever a CJK IME is in composition mode every keystroke goes to the IME first and PZ simply drops it — WASD, Shift to run, Space, E, all dead. Worse, PZ's own controls are Windows' layout hotkeys (Shift+Alt, Ctrl+Space), so you get switched back mid-game. The developers have said this will not be handled in-game. This mod exists for that.

[h2]🎬 35-second demo[/h2]
[url=https://youtu.be/5q1bfbm3lgU]https://youtu.be/5q1bfbm3lgU[/url] (download → run → type Chinese in chat → close chat and walk)

[h2]🧰 How to use[/h2]
[olist]
[*] Subscribe and enable this mod.
[*] Download [b]pz-ime-guard-<version>.exe[/b] from [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] and run it. The first launch shows SmartScreen's "Windows protected your PC" (normal for an unsigned exe): click "More info → Run anyway", or right-click the exe → Properties → tick "Unblock" first. No window — just a keycap icon in the system tray (usually under the ^ overflow), with a small lamp in its corner: green = English layout active, orange = typing, your IME restored, grey = PZ not found or paused, red = no English (US) keyboard installed.
[*] Play. If the tool is not running when you enter a world, you get one reminder.
[/olist]
Tip: enable Windows "Let me set a different input method for each app window" so the switch only affects PZ.

[b]Right-click options:[/b] restore your input method when leaving the game (on by default), start when signing in to Windows (off by default), and automatic update checks. Autostart only creates a shortcut in your user Startup folder; uncheck it to remove the shortcut before deleting the tool. Update both the mod and companion tool for the new exit handling.

[h2]🌐 Supported IMEs and languages[/h2]
[list]
[*] [b]Guarded IMEs:[/b] Chinese (Bopomofo, Cangjie, Boshiamy, Microsoft/Sogou Pinyin, RIME…), Japanese (Microsoft IME, Google Japanese Input…), Korean — Chinese and Japanese confirmed by players, Korean confirmed by Microsoft's IME documentation. Vietnamese Telex/VNI and Indic Phonetic IMEs built into Windows use the same mechanism but are not yet verified in-game.
[*] [b]Left alone:[/b] plain keyboard layouts (any English variant, Russian, German, French, Thai…) — they never had this problem.
[*] [b]UI languages:[/b] tool tooltips, menu and in-game notice in Traditional/Simplified Chinese, Japanese, Korean; English otherwise. Steam page in EN/繁/简/日/한.
[/list]

[h2]🦀 Why Rust, why open source[/h2]
[list]
[*] One standalone exe. No .NET, Java or other runtime to install.
[*] Built with Rust. No injection into the game process and no reading your keystrokes.
[*] Uses public Windows APIs to switch layouts. No keyboard hooks or registry writes. Optional features restore the input method of the foreground window after leaving PZ and check GitHub for updates. The mod itself is pure Lua and writes typing and normal-exit signals to your Zomboid folder.
[*] Source on [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url] — anyone can audit it or build their own. Every exe is built by GitHub Actions in a clean environment and automatically submitted to VirusTotal; the SHA-256 and scan link are on the Release page. The exe is not code-signed, so SmartScreen may warn about an unknown publisher; if in doubt, cargo build it yourself.
[/list]

[h2]📋 Mod info[/h2]
[list]
[*] [b]Mod ID:[/b] MinidoracatIMEGuardFor42
[*] [b]Version:[/b] Build 42.20.4+
[*] [b]Singleplayer / Multiplayer:[/b] both (MP: server must list the mod, see top)
[*] [b]Platform:[/b] tool is Windows-only; macOS/Linux do not have this problem
[/list]

[h2]💬 Feedback[/h2]
[list]
[*] [url=https://discord.gg/Gur2V67]Discord[/url]
[/list]

[h2]☕ Support the author[/h2]
If this helped, a 👍 on this page and a ⭐ on GitHub help other players stuck with the same IME problem find it.
The mod is free and always will be, with the source public on GitHub. If you enjoy it, consider buying me a coffee - tips go straight into servers and mod development.
[url=https://ko-fi.com/minidoracat][img]https://raw.githubusercontent.com/Minidoracat/workshop-resources/refs/heads/main/badges/badge_kofi.png[/img][/url] [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42][img]https://raw.githubusercontent.com/Minidoracat/workshop-resources/refs/heads/main/badges/badge_github.png[/img][/url]

[b]#Minidoracat[/b]

Workshop ID: 3802890539
Mod ID: MinidoracatIMEGuardFor42
