//! pz-ime-guard — MinidoracatIMEGuardFor42 的 Windows 配套常駐工具。
//!
//! MOD 一寫 state.txt 就醒來（目錄變更通知），否則每 100 ms 輪詢一次：PZ 視窗在前景時，依 typing 狀態
//! 決定它該用哪個鍵盤配置，不對就送 WM_INPUTLANGCHANGEREQUEST，下一輪回讀確認（請求 ≠ 已接受）。
//!   沒在打字 → en-US（GLFW 才收得到按鍵）；打字中 → 玩家原本的 IME。
//! 「原本的 IME」＝最近一次在 PZ 視窗上看到的非英文配置（玩家按 Alt+Shift 切走時順便記住，再切回）。
//! 不裝鍵盤 hook、不代送按鍵、不改登錄檔、不碰其他視窗。
//!
//! 檔案協定（%USERPROFILE%\Zomboid\Lua\MinidoracatIMEGuard\）：
//!   state.txt      MOD 寫，"1"＝打字中、"0"＝沒有；空檔／其他內容視為未變
//!   heartbeat.txt  本工具每 2 s 寫 epoch 秒；MOD 進遊戲時讀不到或過期就提醒玩家
#![windows_subsystem = "windows"]

use std::{
    fs,
    path::PathBuf,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};
use windows::{
    core::{w, BOOL, HSTRING},
    Win32::Foundation::{HWND, LPARAM, WAIT_OBJECT_0, WPARAM},
    Win32::Globalization::GetUserDefaultUILanguage,
    Win32::Storage::FileSystem::{
        FindFirstChangeNotificationW, FindNextChangeNotification, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE,
    },
    Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL},
    Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, EnumWindows, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
        IsWindowVisible, MessageBoxW, MsgWaitForMultipleObjects, PeekMessageW, PostMessageW, TranslateMessage,
        MB_ICONINFORMATION, MB_OK, MSG, PM_REMOVE, QS_ALLINPUT,
    },
};

const WM_INPUTLANGCHANGEREQUEST: u32 = 0x0050;
const LANG_EN_US: u16 = 0x0409;
const TICK: Duration = Duration::from_millis(100); // 輪詢 fallback；state.txt 變更由目錄通知即時喚醒
const RESEND_AFTER: Duration = Duration::from_millis(500);
const HEARTBEAT_EVERY: Duration = Duration::from_secs(2);

fn lang(hkl: HKL) -> u16 {
    (hkl.0 as usize & 0xffff) as u16
}

fn installed_layouts() -> Vec<HKL> {
    unsafe {
        let count = GetKeyboardLayoutList(None).max(0) as usize;
        let mut list = vec![HKL::default(); count];
        let got = GetKeyboardLayoutList(Some(&mut list)).max(0) as usize;
        list.truncate(got);
        list
    }
}

/// 第一個可見且標題為 "Project Zomboid" 的頂層視窗（GLFW 視窗 FindWindowW 找不到，EnumWindows 才看得到）。
/// ponytail: 兩個 client 同機時只守第一個。
fn find_game_window() -> Option<HWND> {
    const TITLE: &[u16] = &[
        b'P' as u16, b'r' as u16, b'o' as u16, b'j' as u16, b'e' as u16, b'c' as u16, b't' as u16, b' ' as u16,
        b'Z' as u16, b'o' as u16, b'm' as u16, b'b' as u16, b'o' as u16, b'i' as u16, b'd' as u16,
    ];
    unsafe extern "system" fn visit(hwnd: HWND, found: LPARAM) -> BOOL {
        unsafe {
            if !IsWindowVisible(hwnd).as_bool() {
                return true.into();
            }
            let mut buf = [0u16; 32];
            let len = GetWindowTextW(hwnd, &mut buf) as usize;
            if &buf[..len] == TITLE {
                *(found.0 as *mut Option<HWND>) = Some(hwnd);
                return false.into();
            }
            true.into()
        }
    }
    let mut found: Option<HWND> = None;
    unsafe {
        let _ = EnumWindows(Some(visit), LPARAM(&mut found as *mut _ as isize));
    }
    found
}

/// ponytail: 只認 %USERPROFILE%\Zomboid；PZ 的 -cachedir 改路徑時要改這裡。
fn state_dir() -> PathBuf {
    let home = std::env::var_os("USERPROFILE").unwrap_or_default();
    PathBuf::from(home).join("Zomboid").join("Lua").join("MinidoracatIMEGuard")
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Status {
    Paused,
    NoGame,
    Background,
    NoEnglish,
    Guarding,
    Typing,
}

impl Status {
    fn rgb(self) -> [u8; 3] {
        match self {
            Status::Paused | Status::NoGame | Status::Background => [128, 128, 128],
            Status::NoEnglish => [220, 50, 50],
            Status::Guarding => [40, 180, 80],
            Status::Typing => [240, 160, 30],
        }
    }
    fn text(self, s: &Strings) -> String {
        let body = match self {
            Status::Paused => s.paused,
            Status::NoGame => s.no_game,
            Status::Background => s.background,
            Status::NoEnglish => s.no_english,
            Status::Guarding => s.guarding,
            Status::Typing => s.typing,
        };
        format!("pz-ime-guard: {body}")
    }
}

/// 介面文字：依 Windows 顯示語言挑一組（繁中／簡中／日文，其餘英文），與 MOD 的四語翻譯槽一致。
struct Strings {
    paused: &'static str,
    no_game: &'static str,
    background: &'static str,
    no_english: &'static str,
    guarding: &'static str,
    typing: &'static str,
    menu_pause: &'static str,
    menu_quit: &'static str,
    notice: &'static str,
}

const EN: Strings = Strings {
    paused: "paused",
    no_game: "waiting for Project Zomboid",
    background: "PZ not in foreground",
    no_english: "no English (US) keyboard installed",
    guarding: "English layout active (movement keys safe)",
    typing: "typing, your IME restored",
    menu_pause: "Pause",
    menu_quit: "Quit",
    notice: "pz-ime-guard is now running in the system tray (it may be hidden under the ^ arrow).\n\
             The lamp on the keycap icon: green = English layout, orange = typing (your IME restored), grey = waiting for Project Zomboid.\n\
             Right-click the icon to pause or quit. This notice is shown only once.",
};
const TW: Strings = Strings {
    paused: "已暫停",
    no_game: "等待 Project Zomboid 啟動",
    background: "PZ 不在前景",
    no_english: "系統未安裝英文（美國）鍵盤",
    guarding: "英文鍵盤中，移動鍵安全",
    typing: "打字中，已切回你的輸入法",
    menu_pause: "暫停",
    menu_quit: "結束",
    notice: "pz-ime-guard 已在系統匣運作（可能收在 ^ 隱藏區）。\n\
             鍵帽圖示上的小燈：綠＝英文鍵盤、橘＝打字中已切回你的輸入法、灰＝等待 Project Zomboid。\n\
             右鍵圖示可暫停或結束。此訊息只顯示一次。",
};
const CN: Strings = Strings {
    paused: "已暂停",
    no_game: "等待 Project Zomboid 启动",
    background: "PZ 不在前台",
    no_english: "系统未安装英语（美国）键盘",
    guarding: "英文键盘中，移动键安全",
    typing: "打字中，已切回你的输入法",
    menu_pause: "暂停",
    menu_quit: "退出",
    notice: "pz-ime-guard 已在系统托盘运行（可能收在 ^ 隐藏区）。\n\
             键帽图标上的小灯：绿＝英文键盘、橙＝打字中已切回你的输入法、灰＝等待 Project Zomboid。\n\
             右键图标可暂停或退出。此消息只显示一次。",
};
const JP: Strings = Strings {
    paused: "一時停止中",
    no_game: "Project Zomboid の起動を待機中",
    background: "PZ が前面にありません",
    no_english: "英語（米国）キーボードが未インストール",
    guarding: "英語配列中、移動キーは安全",
    typing: "入力中、IME を復帰済み",
    menu_pause: "一時停止",
    menu_quit: "終了",
    notice: "pz-ime-guard はタスクトレイで動作中です（^ の中に隠れている場合があります）。\n\
             キーキャップアイコンのランプ：緑＝英語配列、橙＝入力中（IME 復帰済み）、灰＝Project Zomboid を待機中。\n\
             アイコンを右クリックで一時停止・終了。この案内は初回のみ表示されます。",
};

fn strings() -> &'static Strings {
    let id = unsafe { GetUserDefaultUILanguage() };
    match id {
        0x0404 | 0x0C04 | 0x1404 => &TW, // 台灣／香港／澳門
        0x0804 | 0x1004 => &CN,          // 中國／新加坡
        _ if id & 0x3ff == 0x11 => &JP,
        _ => &EN,
    }
}

/// 32×32 鍵帽底圖（assets/icon-src.png 縮圖，raw RGBA）＋右下角狀態燈（直徑 12、深色 1px 描邊）。
const TRAY_BASE: &[u8] = include_bytes!("../assets/tray-32.rgba");

fn icon(status: Status) -> Icon {
    let [r, g, b] = status.rgb();
    let size = 32u32;
    let mut rgba = TRAY_BASE.to_vec();
    let (cx, cy, radius) = (25.0f32, 25.0f32, 6.0f32);
    for y in 0..size {
        for x in 0..size {
            let d = ((x as f32 + 0.5 - cx).powi(2) + (y as f32 + 0.5 - cy).powi(2)).sqrt();
            if d > radius + 1.0 {
                continue;
            }
            let i = ((y * size + x) * 4) as usize;
            let px = if d <= radius { [r, g, b, 255] } else { [30, 30, 30, 255] };
            rgba[i..i + 4].copy_from_slice(&px);
        }
    }
    Icon::from_rgba(rgba, size, size).expect("32x32 rgba icon")
}

/// 第一次啟動才彈：Windows 11 預設把新圖示收進「^」隱藏區，不講玩家不知道它跑起來了。
fn first_run_notice(dir: &std::path::Path, s: &Strings) {
    let marker = dir.join("first-run-done.txt");
    if marker.exists() {
        return;
    }
    let _ = fs::create_dir_all(dir);
    let _ = fs::write(&marker, "1");
    unsafe {
        MessageBoxW(None, &HSTRING::from(s.notice), w!("pz-ime-guard"), MB_OK | MB_ICONINFORMATION);
    }
}

struct Guard {
    dir: PathBuf,
    english: Option<HKL>,
    ime: Option<HKL>,
    typing: bool,
    last_post: Option<(HKL, Instant)>, // 只對「同一目標」限流重送；目標一換立刻送
    last_heartbeat: Option<Instant>,
}

impl Guard {
    fn new() -> Self {
        let layouts = installed_layouts();
        Self {
            dir: state_dir(),
            english: layouts.iter().copied().find(|&h| lang(h) == LANG_EN_US),
            ime: layouts.iter().copied().find(|&h| lang(h) != LANG_EN_US),
            typing: false,
            last_post: None,
            last_heartbeat: None,
        }
    }

    fn read_typing(&mut self) {
        match fs::read_to_string(self.dir.join("state.txt")).map(|s| s.trim().to_owned()).as_deref() {
            Ok("1") => self.typing = true,
            Ok("0") => self.typing = false,
            _ => {} // 空檔（MOD 正在截斷重寫）或沒有 MOD：沿用上一個狀態
        }
    }

    fn heartbeat(&mut self) {
        if self.last_heartbeat.is_some_and(|t| t.elapsed() < HEARTBEAT_EVERY) {
            return;
        }
        self.last_heartbeat = Some(Instant::now());
        let secs = SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0);
        let tmp = self.dir.join("heartbeat.tmp");
        // 先寫暫存再 rename：MOD 讀到的永遠是完整的一行
        let _ = fs::create_dir_all(&self.dir);
        if fs::write(&tmp, secs.to_string()).is_ok() {
            let _ = fs::rename(&tmp, self.dir.join("heartbeat.txt"));
        }
    }

    fn tick(&mut self, paused: bool) -> Status {
        self.heartbeat();
        if paused {
            return Status::Paused;
        }
        let Some(english) = self.english else { return Status::NoEnglish };
        let Some(hwnd) = find_game_window() else { return Status::NoGame };
        if unsafe { GetForegroundWindow() } != hwnd {
            return Status::Background;
        }
        let thread = unsafe { GetWindowThreadProcessId(hwnd, None) };
        let current = unsafe { GetKeyboardLayout(thread) };
        if lang(current) != LANG_EN_US {
            self.ime = Some(current);
        }
        self.read_typing();
        let want = if self.typing { self.ime.unwrap_or(english) } else { english };
        let same_target_recently = self.last_post.is_some_and(|(h, t)| h == want && t.elapsed() < RESEND_AFTER);
        if current != want && !same_target_recently {
            self.last_post = Some((want, Instant::now()));
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_INPUTLANGCHANGEREQUEST, WPARAM(0), LPARAM(want.0 as isize));
            }
        }
        if self.typing { Status::Typing } else { Status::Guarding }
    }
}

fn pump_messages() {
    unsafe {
        let mut msg = MSG::default();
        while PeekMessageW(&mut msg, None, 0, 0, PM_REMOVE).as_bool() {
            let _ = TranslateMessage(&msg);
            DispatchMessageW(&msg);
        }
    }
}

fn main() {
    let s = strings();
    let pause = CheckMenuItem::new(s.menu_pause, true, false, None);
    let quit = MenuItem::new(s.menu_quit, true, None);
    let menu = Menu::with_items(&[&pause, &quit]).expect("tray menu");
    let mut status = Status::NoGame;
    let tray: TrayIcon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(status.text(s))
        .with_icon(icon(status))
        .build()
        .expect("tray icon");

    let mut guard = Guard::new();
    first_run_notice(&guard.dir, s);
    // MOD 一寫 state.txt 就醒來，不用等下一輪；目錄通知拿不到就退回純輪詢
    let _ = fs::create_dir_all(&guard.dir);
    let watch = unsafe {
        FindFirstChangeNotificationW(
            &HSTRING::from(guard.dir.as_os_str()),
            false,
            FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_SIZE,
        )
    }
    .ok();
    loop {
        pump_messages();
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            if event.id() == quit.id() {
                return;
            }
        }
        let next = guard.tick(pause.is_checked());
        if next != status {
            status = next;
            let _ = tray.set_icon(Some(icon(status)));
            let _ = tray.set_tooltip(Some(status.text(s)));
        }
        match watch {
            Some(handle) => unsafe {
                let woke = MsgWaitForMultipleObjects(Some(&[handle]), false, TICK.as_millis() as u32, QS_ALLINPUT);
                if woke == WAIT_OBJECT_0 {
                    let _ = FindNextChangeNotification(handle);
                }
            },
            None => std::thread::sleep(TICK),
        }
    }
}
