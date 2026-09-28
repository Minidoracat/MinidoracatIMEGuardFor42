<!-- Steam 討論區貼文稿源（English）；簡介只放摘要，詳細內容以本串為準 -->
<!-- 討論串網址：https://steamcommunity.com/workshop/filedetails/discussion/3802890539/586187095760095692/ -->
<!-- 標題：📖 IME Guard Guide: How It Works, FAQ & Known Limitations -->

[b]繁體中文版：[/b][url=https://steamcommunity.com/workshop/filedetails/discussion/3802890539/586187095760095677/]IME Guard 完整說明：運作方式、常見問題與已知限制[/url]

[h1]📖 IME Guard Guide: How It Works, FAQ & Known Limitations[/h1]
Everything the Workshop page leaves out: how it works, settings, safety, FAQ and limitations.

[h2]🚀 Quick start[/h2]
[olist]
[*] Subscribe to and enable this mod (in multiplayer the server must list it).
[*] Download [b]pz-ime-guard-<version>.exe[/b] from [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest]GitHub Releases[/url] and run it (no installer, any folder).
[*] First launch shows SmartScreen's "Windows protected your PC" (normal for an unsigned exe): click "More info → Run anyway", or first tick "Unblock" in the exe's Properties.
[*] When the keycap icon is in the tray, play. If the tool is not running when you enter a world, you get one reminder.
[/olist]
35-second demo: [url=https://youtu.be/5q1bfbm3lgU]https://youtu.be/5q1bfbm3lgU[/url] (download → run → type Chinese in chat → close chat and walk)

[h2]🧟 Why you need it[/h2]
A Chinese, Japanese or Korean IME in composition mode takes every keystroke first and PZ drops it — WASD, Shift, Space, E all dead. PZ's controls also overlap Windows' IME hotkeys (Shift run + Alt sprint = Alt+Shift; Ctrl aim + Space = Ctrl+Space), so you get switched back mid-game. The developers will not handle this in-game; once the underlying fix reaches the game, this mod is no longer needed.

[h2]🔧 Features and how it works[/h2]
[b]You need both parts:[/b] the mod only reports whether you are typing and whether you confirmed quitting; it cannot switch input methods. The tool switches the PZ window's keyboard from those reports.
[list]
[*] [b]English while you play:[/b] when you are not typing, the PZ window gets a non-IME layout such as English (US).
[*] [b]Your IME when you type:[/b] follows the game's own "typing in a text box" state: click into chat, search, naming… and your IME comes back; leave and you are on English again. You get the last IME used in the PZ window, not the system default; to use another one, press Win+Space once in chat and it is remembered.
[*] [b]Restore on leave:[/b] on leaving the game it tries to restore the desktop IME it saw before you entered (see the Alt+Tab and quitting questions).
[/list]
[b]Tip:[/b] turn on Windows "Settings → Time & language → Typing → Advanced keyboard settings → Let me use a different input method for each app window" so the switch only affects PZ.

[h3]🌐 Supported IMEs (Chinese, Japanese, Korean)[/h3]
The tool looks at the language of the Windows keyboard layout, not the IME brand.
[list]
[*] [b]Guarded:[/b] Chinese (Bopomofo, Cangjie, Boshiamy, Microsoft/Sogou Pinyin, RIME…), Japanese (Microsoft IME, Google Japanese Input…), Korean. Chinese and Japanese confirmed by players, Korean by Microsoft's IME docs.
[*] [b]Same mechanism, not verified in-game:[/b] built-in Vietnamese Telex/VNI, Indic Phonetic.
[*] [b]Left alone:[/b] plain layouts (any English variant, Russian, German, French, Thai…) — they never had this problem.
[*] [b]UI languages:[/b] tool and in-game text in Traditional/Simplified Chinese, Japanese, Korean, else English (Korean by a non-native speaker — corrections welcome). Steam page in EN/繁/简/日/한.
[/list]

[h2]🚥 Tray icon and lamp[/h2]
No window, just a keycap icon in the system tray (Windows 11 usually hides it under the ^ overflow; the first launch explains this once). Hover it to see the state. The lamp:
[list]
[*] [b]Green:[/b] English layout active, movement keys work.
[*] [b]Orange:[/b] typing, your IME restored.
[*] [b]Grey:[/b] PZ not found, PZ not in the foreground, paused, or the game is closing (guard stopped).
[*] [b]Red:[/b] no English (US) or other non-IME keyboard to switch to (see FAQ).
[/list]
Right-click: version, Workshop page, GitHub, the settings below, Pause, Quit. Only one copy runs; a second one says so and exits.

[h2]⚙️ Settings (right-click menu)[/h2]
[list]
[*] [b]Restore my IME when leaving the game:[/b] on by default.
[*] [b]Check for updates (on start, daily):[/b] on by default; offers the download page when a new version is out.
[*] [b]Start when I sign in to Windows:[/b] off by default. Only a shortcut in your user Startup folder (no registry, no service); untick to remove it. Runs after sign-in, not at boot, maybe a bit late. Disabling it in Windows "Settings → Apps → Startup" keeps the shortcut, so the menu may still show it ticked. Moved the exe? Run it once from the new place (option ticked) to fix the shortcut.
[/list]

[h2]🔒 Safety and privacy[/h2]
[list]
[*] One standalone Rust exe, no runtime needed. No injection into the game, no reading keystrokes, no keyboard hooks, no registry writes; layouts switch through public Windows APIs.
[*] The mod is pure Lua and only writes typing and normal-exit signals to your Zomboid folder.
[*] The tool only goes online to check GitHub for updates (optional) and sends no user data.
[*] Source on [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42]GitHub[/url] for anyone to audit or build. Every exe is built by GitHub Actions in a clean environment and auto-submitted to VirusTotal; SHA-256 and scan link are on the Release page. The exe is unsigned, hence SmartScreen's unknown-publisher warning; if in doubt, cargo build it yourself.
[/list]

[h2]⚠️ Known limitations[/h2]
[list]
[*] The tool is Windows-only; macOS/Linux do not have this problem.
[*] With two copies of PZ open on one PC, only the first window is guarded.
[*] Restoring the desktop IME is best-effort, and closing with X or a crash cannot notify the tool (see the Alt+Tab and quitting questions).
[/list]

[h2]❓ FAQ[/h2]

[h3]Do search boxes switch back to my IME? Can it search items in containers?[/h3]
Yes, the lamp turns orange. But IME Guard only switches input methods: [b]it adds or extends no search feature[/b]; what search finds is up to the game (or other mods). If a text box does not turn the lamp orange, report the screen and field.

[h3]After Alt+Tab, my desktop sometimes stays in English[/h3]
Slow Alt+Tab picking and quick back-and-forth are improved: the tool waits for the real target window (up to ~2 s) before restoring, and never mistakes English for your original IME. Remaining limits:
[list]
[*] It restores only if it switched PZ to English itself and knows your desktop IME; it never guesses.
[*] Restoring has a retry and time limit; not every program accepts the switch.
[*] If you pause the tool or untick the restore option, pending restores are dropped.
[/list]
Most reliable: the per-app-window setting in the tip above. On the latest tool and it still happens? Report how you switched (held Alt+Tab, quick back-and-forth, which program).

[h3]Quitting the game switches my desktop to English[/h3]
When you confirm quitting to desktop from the game's menus (main menu, in-game quit, death or disconnect screen), the mod tells the tool, which stops guarding while the game shuts down (grey lamp) and restores your IME once you switch to another window. Needs [b]both the latest mod and tool[/b].
[list]
[*] X button, crash, or Task Manager: no signal, so the tool stops guarding only when the game window disappears.
[*] Returning to the main menu is not quitting; the tool keeps guarding.
[/list]

[h3]How do I update the tool?[/h3]
Run the new exe: it asks a running old copy to exit and takes over, and autostart follows the new version (very old versions cannot be taken over; quit them from the tray first). Delete old exe files freely.

[h3]SmartScreen blocks it / is it safe?[/h3]
Normal for an unsigned exe; see Quick start step 3 and "Safety and privacy".

[h3]"Companion tool pz-ime-guard is not running" in game[/h3]
The tool was not running when you entered the world. No keycap icon in the tray (check the ^ overflow)? Run the exe.

[h3]The icon lamp is red[/h3]
There is no plain keyboard to switch to. Add English (US) in Windows settings (or the US keyboard under your language's options); the tool picks it up without a restart.

[h3]Does it work in multiplayer?[/h3]
Yes, if the server lists this mod (PZ only loads mods on the server's list). Nothing runs server-side, saves are untouched; the tool runs only on your PC.

[h2]🗑️ Uninstalling[/h2]
No installer. If autostart is ticked, untick it before quitting the tool; if the exe is already gone, type [b]shell:startup[/b] in the File Explorer address bar and delete the pz-ime-guard shortcut. Then delete the exe. The MinidoracatIMEGuard folder under Zomboid\Lua only holds small state files and can go too. Unsubscribe from the mod on the Workshop.

[h2]💬 Reporting[/h2]
[list]
[*] Bugs: [url=https://github.com/Minidoracat/MinidoracatIMEGuardFor42/issues]GitHub Issues[/url] — include Windows version, IME, lamp color and steps to reproduce
[*] Usage questions: reply in this thread
[*] Chat: [url=https://discord.gg/Gur2V67]Discord[/url]
[/list]
