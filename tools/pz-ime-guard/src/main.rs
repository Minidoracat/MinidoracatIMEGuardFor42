//! pz-ime-guard — MinidoracatIMEGuardFor42 的 Windows 配套常駐工具。
//!
//! MOD 一寫 state.txt 就醒來（目錄變更通知），否則每 100 ms 輪詢一次：PZ 視窗在前景時，依 typing 狀態
//! 決定它該用哪個鍵盤配置，不對就送 WM_INPUTLANGCHANGEREQUEST，下一輪回讀確認（請求 ≠ 已接受）。
//!   沒在打字 → en-US（GLFW 才收得到按鍵）；打字中 → 玩家原本的 IME。
//! 「原本的 IME」＝最近一次在 PZ 視窗上看到的非英文配置（玩家按 Alt+Shift 切走時順便記住，再切回）。
//! 離開 PZ（alt-tab、關遊戲、正常退出）後，若我們真的把它切走過，就等一個穩定的新前景，把 PZ 進前景前
//! 桌面用的配置還回去（有限重試＋回讀確認，可關）。沒切過就沒有還原責任，絕不把 en-US 推給桌面。
//! 不裝鍵盤 hook、不代送按鍵、不改登錄檔；除了上述還原之外不碰其他視窗。
//!
//! 檔案協定（%USERPROFILE%\Zomboid\Lua\MinidoracatIMEGuard\）：
//!   state.txt      MOD 寫，"1"＝打字中、"0"＝沒有；空檔／其他內容視為未變
//!   exiting.txt    MOD 寫，"1"＝玩家已確認關程序（視窗還會活很久），"0"／空＝正常；本工具讀到就消費掉
//!   heartbeat.txt  本工具每 2 s 寫 epoch 秒；MOD 進遊戲時讀不到或過期就提醒玩家
//!   running.txt    執行中那一份的版本號；較新版啟動時據此決定要不要請它退出（見 main）
#![windows_subsystem = "windows"]

use std::{
    fs,
    os::windows::process::CommandExt,
    path::{Path, PathBuf},
    sync::mpsc,
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tray_icon::{
    menu::{CheckMenuItem, Menu, MenuEvent, MenuItem, PredefinedMenuItem},
    Icon, TrayIcon, TrayIconBuilder,
};
use windows::{
    core::{w, BOOL, HSTRING},
    Win32::Foundation::{CloseHandle, GetLastError, ERROR_ALREADY_EXISTS, HANDLE, HWND, LPARAM, WAIT_OBJECT_0, WPARAM},
    Win32::Globalization::GetUserDefaultUILanguage,
    Win32::Storage::FileSystem::{
        FindFirstChangeNotificationW, FindNextChangeNotification, FILE_NOTIFY_CHANGE_LAST_WRITE, FILE_NOTIFY_CHANGE_SIZE,
    },
    Win32::UI::Input::KeyboardAndMouse::{GetKeyboardLayout, GetKeyboardLayoutList, HKL},
    Win32::System::Threading::{CreateEventW, CreateMutexW, OpenEventW, SetEvent, WaitForSingleObject, EVENT_MODIFY_STATE},
    Win32::UI::Shell::ShellExecuteW,
    Win32::UI::WindowsAndMessaging::{
        DispatchMessageW, EnumWindows, GetClassNameW, GetForegroundWindow, GetWindowTextW, GetWindowThreadProcessId,
        IsWindow, IsWindowVisible, MessageBoxW, MsgWaitForMultipleObjects, PeekMessageW, PostMessageW, TranslateMessage,
        IDYES, MB_ICONINFORMATION, MB_ICONWARNING, MB_OK, MB_YESNO, MSG, PM_REMOVE, QS_ALLINPUT, SW_SHOWNORMAL,
    },
};

mod startup;

const WM_INPUTLANGCHANGEREQUEST: u32 = 0x0050;
const LANG_EN_US: u16 = 0x0409;
const TICK: Duration = Duration::from_millis(100); // 輪詢 fallback；state.txt 變更由目錄通知即時喚醒
const RESTORE_SETTLE: Duration = Duration::from_millis(300); // 前景要真的待滿這麼久才算穩定（兩輪可能只差 1 ms）
const RESTORE_WINDOW: Duration = Duration::from_secs(2); // 前景穩定後，還原桌面配置的時限
const RESTORE_GIVE_UP: Duration = Duration::from_secs(10); // 有可用前景之後的絕對放棄上限
const RESTORE_TRIES: u8 = 3; // 還原請求的重送次數上限（請求 ≠ 已接受）
const RESEND_AFTER: Duration = Duration::from_millis(500);
const HEARTBEAT_EVERY: Duration = Duration::from_secs(2);

fn lang(hkl: HKL) -> u16 {
    (hkl.0 as usize & 0xffff) as u16
}

/// 會把按鍵攔成 VK_PROCESSKEY 的是「輸入法（IME）」而不是「鍵盤配置」。Windows 內建 IME 的語系（primary language）：
/// 中文 0x04、日文 0x11、韓文 0x12、越南文 0x2A（Telex／VNI）、印度語系 Phonetic（印地 0x39、孟加拉 0x45、
/// 旁遮普 0x46、古吉拉特 0x47、奧里亞 0x48、泰米爾 0x49、泰盧固 0x4A、卡納達 0x4B、馬拉雅拉姆 0x4C、馬拉地 0x4E）、
/// 切羅基 0x5C、阿姆哈拉 0x5E、提格利尼亞 0x73。其他配置 GLFW 都收得到，一律視為安全、不介入。
/// 中日韓有 PZ 玩家實證；其餘依微軟文件同為 IME [未實機驗證]。
/// ponytail: ImmIsIME 對已安裝的 en-US 也回 true（實測），不能拿來判。
fn is_ime_lang(hkl: HKL) -> bool {
    matches!(
        lang(hkl) & 0x3ff,
        0x04 | 0x11 | 0x12 | 0x2A | 0x39 | 0x45 | 0x46 | 0x47 | 0x48 | 0x49 | 0x4A | 0x4B | 0x4C | 0x4E | 0x5C | 0x5E | 0x73
    )
}

/// 玩遊戲時要切去的配置：en-US 優先，沒有就任一英文，再沒有就任一非 IME 配置。
fn pick_safe_layout(layouts: &[HKL]) -> Option<HKL> {
    let by = |f: &dyn Fn(HKL) -> bool| layouts.iter().copied().find(|&h| f(h));
    by(&|h| lang(h) == LANG_EN_US)
        .or_else(|| by(&|h| lang(h) & 0x3ff == 0x09))
        .or_else(|| by(&|h| !is_ime_lang(h)))
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

/// Alt+Tab 切換器、工作檢視、開始功能表那類一閃而過的殼層視窗。它們會短暫成為前景，
/// 拿它們當還原對象不但沒用（訊息大多被丟掉），還會把還原機會白白吃掉。
/// ponytail: 只比類別名，不呼叫 shell COM；名單以外一律當成真正的視窗。
fn is_transient_shell(hwnd: HWND) -> bool {
    const SHELLS: [&str; 6] = [
        "XamlExplorerHostIslandWindow", // Win11 Alt+Tab／工作檢視
        "MultitaskingViewFrame",        // Win10 工作檢視
        "TaskSwitcherWnd",              // 傳統 Alt+Tab
        "TaskSwitcherOverlayWnd",
        "ForegroundStaging",              // 切換動畫的暫存前景
        "Windows.UI.Core.CoreWindow",     // 開始功能表／搜尋等 UWP 覆蓋層
    ];
    let mut buf = [0u16; 64];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) } as usize;
    let class = String::from_utf16_lossy(&buf[..len]);
    SHELLS.contains(&class.as_str())
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
    Closing,
    NoEnglish,
    Guarding,
    Typing,
}

impl Status {
    fn rgb(self) -> [u8; 3] {
        match self {
            Status::Paused | Status::NoGame | Status::Background | Status::Closing => [128, 128, 128],
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
            Status::Closing => s.closing,
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
    closing: &'static str,
    no_english: &'static str,
    guarding: &'static str,
    typing: &'static str,
    no_safe_notice: &'static str,
    already_running: &'static str,
    old_running: &'static str,
    menu_pause: &'static str,
    menu_quit: &'static str,
    menu_workshop: &'static str,
    menu_github: &'static str,
    menu_check_updates: &'static str,
    menu_restore_desktop: &'static str,
    menu_startup: &'static str,
    startup_failed: &'static str,
    update_available: &'static str,
    notice: &'static str,
}

const EN: Strings = Strings {
    paused: "paused",
    no_game: "waiting for Project Zomboid",
    background: "PZ not in foreground",
    closing: "Project Zomboid is closing — guard stopped",
    no_english: "no non-IME keyboard installed — add English (US) in Windows settings",
    guarding: "safe layout active (movement keys work)",
    typing: "typing, your IME restored",
    menu_pause: "Pause",
    menu_workshop: "Workshop page",
    menu_github: "GitHub (source / issues)",
    menu_check_updates: "Check for updates (on start, daily)",
    menu_restore_desktop: "Restore my IME when leaving the game",
    menu_startup: "Start when I sign in to Windows",
    startup_failed: "Could not change the Windows startup shortcut:\n\n{error}",
    update_available: "pz-ime-guard {tag} is available (you have {current}).

Open the download page?",
    menu_quit: "Quit",
    notice: "pz-ime-guard is now running in the system tray (it may be hidden under the ^ arrow).\n\
             The lamp on the keycap icon: green = English layout, orange = typing (your IME restored), grey = waiting for Project Zomboid.\n\
             Right-click the icon to pause or quit. This notice is shown only once.",
    no_safe_notice: "No English (US) — or any other non-IME — keyboard is installed, so pz-ime-guard has nothing to switch to and cannot protect your keys.\n\n\
                     Windows Settings → Time & language → Language & region → Add a language → English (United States), or add the US keyboard under your current language's options.\n\n\
                     Open Settings now?",
    already_running: "pz-ime-guard is already running — check the system tray (^ overflow). This copy will exit.",
    old_running: "An older pz-ime-guard is still running and could not be closed automatically.\n\n\
                  Right-click its icon in the system tray (^ overflow) → Quit, then open this copy again.",
};
const TW: Strings = Strings {
    paused: "已暫停",
    no_game: "等待 Project Zomboid 啟動",
    background: "PZ 不在前景",
    closing: "Project Zomboid 正在關閉，已停止守護",
    no_english: "系統只有輸入法鍵盤，請到 Windows 設定新增英文（美國）鍵盤",
    guarding: "英文鍵盤中，移動鍵安全",
    typing: "打字中，已切回你的輸入法",
    menu_pause: "暫停",
    menu_workshop: "Workshop 頁面",
    menu_github: "GitHub（原始碼／回報問題）",
    menu_check_updates: "自動檢查更新（啟動時與每日）",
    menu_restore_desktop: "切出遊戲時還原桌面的輸入法",
    menu_startup: "登入 Windows 時自動啟動",
    startup_failed: "無法變更自動啟動的捷徑：\n\n{error}",
    update_available: "有新版 pz-ime-guard {tag}（目前 {current}）。

要開啟下載頁嗎？",
    menu_quit: "結束",
    notice: "pz-ime-guard 已在系統匣運作（可能收在 ^ 隱藏區）。\n\
             鍵帽圖示上的小燈：綠＝英文鍵盤、橘＝打字中已切回你的輸入法、灰＝等待 Project Zomboid。\n\
             右鍵圖示可暫停或結束。此訊息只顯示一次。",
    no_safe_notice: "系統沒有安裝英文（美國）或其他非輸入法的鍵盤，pz-ime-guard 沒有可以切過去的配置，無法保護你的按鍵。\n\n\
                     請到 Windows 設定 → 時間與語言 → 語言與地區 → 新增語言 → English (United States)，或在「中文（台灣）」的語言選項裡新增「美式鍵盤」。\n\n\
                     要現在開啟設定嗎？",
    already_running: "pz-ime-guard 已經在執行中，請看系統匣（^ 隱藏區）。這個副本會直接結束。",
    old_running: "舊版 pz-ime-guard 還在執行，無法自動關閉。\n\n\
                  請在系統匣（^ 隱藏區）對它的圖示按右鍵 →「結束」，再重新開啟這一份。",
};
const CN: Strings = Strings {
    paused: "已暂停",
    no_game: "等待 Project Zomboid 启动",
    background: "PZ 不在前台",
    closing: "Project Zomboid 正在关闭，已停止守护",
    no_english: "系统只有输入法键盘，请到 Windows 设置新增英语（美国）键盘",
    guarding: "英文键盘中，移动键安全",
    typing: "打字中，已切回你的输入法",
    menu_pause: "暂停",
    menu_workshop: "创意工坊页面",
    menu_github: "GitHub（源码／反馈问题）",
    menu_check_updates: "自动检查更新（启动时与每日）",
    menu_restore_desktop: "切出游戏时还原桌面的输入法",
    menu_startup: "登录 Windows 时自动启动",
    startup_failed: "无法变更自动启动的快捷方式：\n\n{error}",
    update_available: "有新版 pz-ime-guard {tag}（当前 {current}）。

要打开下载页吗？",
    menu_quit: "退出",
    notice: "pz-ime-guard 已在系统托盘运行（可能收在 ^ 隐藏区）。\n\
             键帽图标上的小灯：绿＝英文键盘、橙＝打字中已切回你的输入法、灰＝等待 Project Zomboid。\n\
             右键图标可暂停或退出。此消息只显示一次。",
    no_safe_notice: "系统没有安装英语（美国）或其他非输入法的键盘，pz-ime-guard 没有可以切换过去的布局，无法保护你的按键。\n\n\
                     请到 Windows 设置 → 时间和语言 → 语言和区域 → 添加语言 → English (United States)，或在「中文（简体，中国）」的语言选项里添加「美式键盘」。\n\n\
                     现在打开设置吗？",
    already_running: "pz-ime-guard 已经在运行中，请看系统托盘（^ 隐藏区）。此副本将直接退出。",
    old_running: "旧版 pz-ime-guard 仍在运行，无法自动关闭。\n\n\
                  请在系统托盘（^ 隐藏区）右键它的图标 →「退出」，然后重新打开这一份。",
};
const JP: Strings = Strings {
    paused: "一時停止中",
    no_game: "Project Zomboid の起動を待機中",
    background: "PZ が前面にありません",
    closing: "Project Zomboid を終了中、保護を停止しました",
    no_english: "IME 以外のキーボードがありません。Windows 設定で英語（米国）を追加してください",
    guarding: "英語配列中、移動キーは安全",
    typing: "入力中、IME を復帰済み",
    menu_pause: "一時停止",
    menu_workshop: "Workshop ページ",
    menu_github: "GitHub（ソース／不具合報告）",
    menu_check_updates: "更新を自動確認（起動時と毎日）",
    menu_restore_desktop: "ゲームから離れたら元の IME に戻す",
    menu_startup: "Windows サインイン時に自動実行",
    startup_failed: "スタートアップのショートカットを変更できませんでした：\n\n{error}",
    update_available: "新しい pz-ime-guard {tag} があります（現在 {current}）。

ダウンロードページを開きますか？",
    menu_quit: "終了",
    notice: "pz-ime-guard はタスクトレイで動作中です（^ の中に隠れている場合があります）。\n\
             キーキャップアイコンのランプ：緑＝英語配列、橙＝入力中（IME 復帰済み）、灰＝Project Zomboid を待機中。\n\
             アイコンを右クリックで一時停止・終了。この案内は初回のみ表示されます。",
    no_safe_notice: "英語（米国）などの IME 以外のキーボードがインストールされていないため、pz-ime-guard には切り替え先がなく、キーを保護できません。\n\n\
                     Windows 設定 → 時刻と言語 → 言語と地域 → 言語の追加 → English (United States)、または「日本語」の言語オプションで「英語キーボード」を追加してください。\n\n\
                     今すぐ設定を開きますか？",
    already_running: "pz-ime-guard はすでに動作中です。タスクトレイ（^ の中）を確認してください。このコピーは終了します。",
    old_running: "古いバージョンの pz-ime-guard が動作中で、自動で終了できませんでした。\n\n\
                  タスクトレイ（^ の中）のアイコンを右クリック →「終了」してから、このコピーをもう一度開いてください。",
};
// 韓文由非母語者撰寫，待母語者校對
const KO: Strings = Strings {
    paused: "일시 정지됨",
    no_game: "Project Zomboid 실행 대기 중",
    background: "PZ가 전면에 있지 않음",
    closing: "Project Zomboid 종료 중, 보호를 중지했습니다",
    no_english: "IME가 아닌 키보드가 없습니다. Windows 설정에서 영어(미국)를 추가하세요",
    guarding: "영어 배열 활성, 이동 키 안전",
    typing: "입력 중, IME 복원됨",
    menu_pause: "일시 정지",
    menu_workshop: "Workshop 페이지",
    menu_github: "GitHub(소스／문제 제보)",
    menu_check_updates: "업데이트 자동 확인(시작 시·매일)",
    menu_restore_desktop: "게임에서 벗어나면 원래 IME로 복원",
    menu_startup: "Windows 로그인 시 자동 실행",
    startup_failed: "시작 프로그램 바로 가기를 변경하지 못했습니다:\n\n{error}",
    update_available: "새 버전 pz-ime-guard {tag}가 있습니다(현재 {current}).

다운로드 페이지를 열까요?",
    menu_quit: "종료",
    notice: "pz-ime-guard가 시스템 트레이에서 실행 중입니다(^ 안에 숨겨져 있을 수 있음).\n\
             키캡 아이콘의 램프: 초록＝영어 배열, 주황＝입력 중(IME 복원됨), 회색＝Project Zomboid 대기 중.\n\
             아이콘을 우클릭하면 일시 정지／종료할 수 있습니다. 이 안내는 처음 한 번만 표시됩니다.",
    no_safe_notice: "영어(미국) 등 IME가 아닌 키보드가 설치되어 있지 않아 pz-ime-guard가 전환할 배열이 없고 키를 보호할 수 없습니다.\n\n\
                     Windows 설정 → 시간 및 언어 → 언어 및 지역 → 언어 추가 → English (United States), 또는 「한국어」 언어 옵션에서 「영어 키보드」를 추가하세요.\n\n\
                     지금 설정을 열까요?",
    already_running: "pz-ime-guard가 이미 실행 중입니다. 시스템 트레이(^ 안)를 확인하세요. 이 사본은 종료됩니다.",
    old_running: "이전 버전의 pz-ime-guard가 실행 중이며 자동으로 종료하지 못했습니다.\n\n\
                  시스템 트레이(^ 안)의 아이콘을 우클릭 →「종료」한 뒤 이 사본을 다시 여세요.",
};

fn strings() -> &'static Strings {
    let id = unsafe { GetUserDefaultUILanguage() };
    match id {
        0x0404 | 0x0C04 | 0x1404 => &TW, // 台灣／香港／澳門
        0x0804 | 0x1004 => &CN,          // 中國／新加坡
        _ if id & 0x3ff == 0x11 => &JP,
        _ if id & 0x3ff == 0x12 => &KO,
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

/// 沒有任何非 IME 配置時每次啟動都彈：這是阻礙性錯誤，「是」直接開 Windows 語言設定頁。
fn no_safe_layout_notice(s: &Strings) {
    let answer = unsafe { MessageBoxW(None, &HSTRING::from(s.no_safe_notice), w!("pz-ime-guard"), MB_YESNO | MB_ICONWARNING) };
    if answer == IDYES {
        unsafe {
            ShellExecuteW(None, w!("open"), w!("ms-settings:regionlanguage"), None, None, SW_SHOWNORMAL);
        }
    }
}

/// 第一次啟動才彈：Windows 11 預設把新圖示收進「^」隱藏區，不講玩家不知道它跑起來了。
fn first_run_notice(dir: &Path, s: &Strings) {
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

/// PZ 不在前景時看到的前景視窗。`thread` 一起記下來：HWND 會被回收再利用，送出前重驗「同一個
/// 視窗、同一條執行緒、而且還是前景」才不會把配置塞給別人的視窗。
/// 沒有前景、前景是 PZ 自己（關程序等待中）、或前景是 Alt+Tab 切換器之類的過渡殼層，都餵 None。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Foreground {
    window: isize,
    thread: u32,
    layout: HKL,
}

#[derive(PartialEq, Eq, Debug)]
enum Step {
    Nothing,
    Post { window: isize, thread: u32, layout: HKL },
}

/// 待還原。兩個時鐘都不從「離開 PZ 那一刻」起算：
///   `armed`   第一次看到可用的（非遊戲、非過渡）前景才開始，之後 RESTORE_GIVE_UP 內沒搞定就放棄。
///             關程序等待可能很長（存檔、關 Steam、壓日誌），那段時間前景還是 PZ，不能讓時鐘空轉。
///   `deadline` 該前景真正待滿 RESTORE_SETTLE 之後才起算，這才是「可以動手」的時刻。
struct Pending {
    armed: Option<Instant>,
    window: Option<isize>, // 綁定的前景視窗；換一個就重新起算，絕不對舊視窗繼續送
    deadline: Option<Instant>,
    posted: Option<Instant>,
    tries: u8,
}

/// 還原桌面配置的決策：純邏輯、不碰 Win32（觀察值由 tick 餵進來），所以可以單測。
/// 責任來源只有一個：我們真的把 PZ 從玩家的配置切走過（回讀確認），否則一律不動桌面，
/// 免得把 en-US 推給從來沒被我們碰過的環境。
#[derive(Default)]
struct Restorer {
    desktop: Option<HKL>, // PZ 進前景前，桌面前景視窗用的（非 safe）配置＝還原目標
    owed: bool,           // 有還原責任：我們真的把 PZ 切走過（回讀確認）
    away: bool,           // 目前人不在 PZ 裡
    done: bool,           // 這一次離開已經處理完（還原成功、或已放棄），不再重新武裝
    pending: Option<Pending>,
    seen: Option<(isize, Instant)>, // 目前的前景視窗與第一次看到它的時間；靠真實時間判斷穩定
}

impl Restorer {
    /// PZ 在前景，且回讀確認我們送出的安全配置已經生效。
    fn took_over(&mut self) {
        self.owed = true;
    }

    /// 收手。`settled` 代表桌面已經回到玩家要的配置，責任了結；放棄（逾時、重送用完、沒有還原目標）
    /// 則留著責任，下次離開再試一次。同一次離開只處理一輪，不會反覆重新武裝。
    fn finish(&mut self, settled: bool) {
        self.pending = None;
        self.done = true;
        if settled {
            self.owed = false;
        }
    }

    /// PZ 回到前景：待還原作廢（現在該守，不該還原），等下次離開再重新起算。
    fn in_game(&mut self) {
        self.pending = None;
        self.away = false;
        self.seen = None;
    }

    /// 暫停、或玩家把「還原」選項關掉：連還原責任一起放掉。只清 pending 不夠——暫停期間玩家切出去，
    /// 恢復後那一輪又會重新武裝，變成很久以後才補送一次；恢復守護後下一輪回讀確認會重新背上責任。
    fn cancel(&mut self) {
        self.finish(true);
    }

    /// PZ 從前景離開（alt-tab、關遊戲、正常退出訊號）。這裡不直接武裝：回讀確認可能晚幾輪才到
    /// （剛送出切換就 alt-tab 的話），用「離開那一瞬間」當邊緣會整個漏掉，改由 observe 依條件武裝。
    fn left(&mut self) {
        self.away = true;
        self.done = false;
        self.seen = None;
    }

    /// PZ 不在前景的每一輪。回傳這一輪要送出的還原請求；請求 ≠ 已接受，下一輪看回讀結果決定重送或收手。
    fn observe(&mut self, now: Instant, fg: Option<Foreground>, safe: HKL, enabled: bool) -> Step {
        if !enabled {
            self.cancel();
        }
        // 武裝條件（不是邊緣）：責任可能在離開之後才確認下來，那時「離開那一瞬間」早就過去了
        if self.away && !self.done && self.owed && self.pending.is_none() {
            self.pending = Some(Pending { armed: None, window: None, deadline: None, posted: None, tries: 0 });
        }
        let Some(fg) = fg else {
            self.seen = None; // 過渡狀態：時鐘不起算，待還原也不被消耗掉
            return Step::Nothing;
        };
        let first_seen = match self.seen {
            Some((window, at)) if window == fg.window => at,
            _ => {
                self.seen = Some((fg.window, now));
                now
            }
        };
        // 這裡才算「有可用的前景」：絕對放棄上限從現在起算，關程序等待多久都不會吃掉它
        if let Some(p) = self.pending.as_mut() {
            if p.armed.is_none() {
                p.armed = Some(now);
            }
        }
        if self.pending.as_ref().is_some_and(|p| p.armed.is_some_and(|a| now.duration_since(a) > RESTORE_GIVE_UP)) {
            self.finish(false);
        }
        if fg.layout != safe {
            // 前景不是被我們帶走的配置：玩家本來就用這個、或剛手動切成別的——兩種都是他的意圖，
            // 記下來當下次的還原目標，並結束這次的責任（同時也是還原成功的回讀確認）。
            self.desktop = Some(fg.layout);
            if now.duration_since(first_seen) >= RESTORE_SETTLE {
                self.finish(true); // 桌面已經是玩家要的配置，責任了結（這也是還原成功的回讀確認）
            }
            return Step::Nothing;
        }
        // 連續兩輪同一個 HWND 不等於穩定：目錄變更通知可能在 1 ms 內叫醒兩輪。要真的待滿一段時間。
        if now.duration_since(first_seen) < RESTORE_SETTLE {
            return Step::Nothing;
        }
        let Some(desktop) = self.desktop else {
            self.finish(false); // 從沒看過桌面的配置＝沒有還原目標，永遠不動
            return Step::Nothing;
        };
        let Some(p) = self.pending.as_mut() else { return Step::Nothing };
        if p.window != Some(fg.window) {
            p.window = Some(fg.window);
            p.deadline = Some(now + RESTORE_WINDOW);
            p.posted = None;
            p.tries = 0;
        }
        if p.deadline.is_some_and(|d| now > d) || p.tries >= RESTORE_TRIES {
            self.finish(false); // 這次沒成功，但責任還在：下次離開再試
            return Step::Nothing;
        }
        if p.posted.is_some_and(|t| now.duration_since(t) < RESEND_AFTER) {
            return Step::Nothing;
        }
        p.posted = Some(now);
        p.tries += 1;
        Step::Post { window: fg.window, thread: fg.thread, layout: desktop }
    }
}

/// exiting.txt 的三種判讀。**"0" 不是「沒事」，是「新的一局開始了」**：MOD 每次 OnGameBoot 都寫它，
/// 所以它能把上一局殘留的 "1" 造成的鎖存解開。少了這個分別，「工具暫停→上一局寫 1 沒被處理→新一局
/// 開窗→解除暫停讀到舊的 1 鎖住新視窗」之後就再也解不開，新的一局永遠停在 Closing。
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum ExitSignal {
    None,     // 沒有可信的新版本：維持現狀
    Quitting, // "1"：玩家已確認關程序
    Reset,    // "0"：MOD 剛開機／回主選單，這一局是乾淨的
}

/// exiting.txt 的判讀。回傳（這一輪的訊號、新的水位）。
/// **只有讀到完整有效值才推進水位**：MOD 用 getFileWriter 先截斷再寫，中間那一瞬間是空檔，
/// 把空檔或讀失敗當成「這個版本看過了」會整個漏掉退出訊號。也不刪檔——那是 MOD 正在寫的檔，
/// 刪它只會製造競態；水位本身就足以保證同一個版本不會被認第二次。
/// 水位起始值＝本工具啟動時間，所以上一局留在磁碟上的舊旗標一律不認。
fn exit_verdict(content: Option<&str>, mtime: SystemTime, seen: SystemTime) -> (ExitSignal, SystemTime) {
    if mtime <= seen {
        return (ExitSignal::None, seen);
    }
    match content.map(str::trim) {
        Some("1") => (ExitSignal::Quitting, mtime),
        Some("0") => (ExitSignal::Reset, mtime),
        _ => (ExitSignal::None, seen),
    }
}

/// 退出鎖存。MOD 只在四條已確認的 quitToDesktop 路徑寫 exiting.txt（回主選單走 quit()／exitToMenu()，
/// 視窗還在，不寫）。旗標綁在當時那個遊戲視窗上：視窗消失或換成下一局就解除，免得一個沒被消費掉的
/// 旗標把下一局整局停守。
#[derive(Default)]
struct ExitLatch {
    window: Option<isize>,
}

impl ExitLatch {
    /// 回傳「這個遊戲視窗已經在關程序」。鎖存住就不會再回 Guarding，直到視窗真的不見、換了一局，
    /// 或是 MOD 在新的一局 OnGameBoot 寫了 "0"（Reset）把它解開。
    fn update(&mut self, game: Option<isize>, signal: ExitSignal) -> bool {
        if self.window.is_some() && self.window != game {
            self.window = None;
        }
        match signal {
            ExitSignal::Quitting => self.window = game,
            ExitSignal::Reset => self.window = None,
            ExitSignal::None => {}
        }
        self.window.is_some() && self.window == game
    }
}

/// 責任來源的追蹤：我們對哪個視窗送過「切到安全配置」，以及那個請求有沒有被接受（請求 ≠ 已接受）。
/// 確認過就消費掉，暫停／關掉還原就整個丟掉——留著的話，暫停期間切出去、解除暫停那一輪的回讀
/// 會把舊請求當成新責任，事後補送一次還原。
#[derive(Default)]
struct Takeover {
    requested: Option<isize>,
}

impl Takeover {
    fn requested(&mut self, window: isize) {
        self.requested = Some(window);
    }
    fn forget(&mut self) {
        self.requested = None;
    }
    fn pending_for(&self, window: isize) -> bool {
        self.requested == Some(window)
    }
    /// 回讀確認：同一個視窗、而且配置真的已經是安全配置，才算我們把它切走過。
    fn confirm(&mut self, window: isize, layout_is_safe: bool) -> bool {
        if layout_is_safe && self.requested == Some(window) {
            self.requested = None;
            return true;
        }
        false
    }
}

struct Guard {
    dir: PathBuf,
    hwnd: Option<HWND>, // 快取；IsWindow 失效才重掃（EnumWindows＋跨程序 GetWindowText 是主要 CPU 來源）
    safe: Option<HKL>,
    ime: Option<HKL>,
    typing: bool,
    last_post: Option<(HKL, Instant)>, // 只對「同一目標」限流重送；目標一換立刻送
    last_heartbeat: Option<Instant>,
    was_fg: bool,                      // 上一輪 PZ 是否在前景；true→false 就進入待還原
    takeover: Takeover, // 責任來源：這輪前景期間對哪個視窗送過「切到安全配置」
    restorer: Restorer,
    exit: ExitLatch,
    exit_seen: SystemTime, // exiting.txt 已經看過的最新 mtime；起始值＝本工具啟動時間
}

impl Guard {
    fn new() -> Self {
        let mut guard = Self {
            dir: state_dir(),
            hwnd: None,
            safe: None,
            ime: None,
            typing: false,
            last_post: None,
            last_heartbeat: None,
            was_fg: false,
            takeover: Takeover::default(),
            restorer: Restorer::default(),
            exit: ExitLatch::default(),
            exit_seen: SystemTime::now(),
        };
        guard.rescan_layouts();
        guard
    }

    /// 重讀系統鍵盤清單。啟動時一次；紅燈狀態下每輪再讀，玩家照提示新增英文鍵盤後不用重啟。
    fn rescan_layouts(&mut self) {
        let layouts = installed_layouts();
        self.safe = pick_safe_layout(&layouts);
        if self.ime.is_none() {
            self.ime = layouts.iter().copied().find(|&h| is_ime_lang(h));
        }
    }

    fn read_typing(&mut self) {
        match fs::read_to_string(self.dir.join("state.txt")).map(|s| s.trim().to_owned()).as_deref() {
            Ok("1") => self.typing = true,
            Ok("0") => self.typing = false,
            _ => {} // 空檔（MOD 正在截斷重寫）或沒有 MOD：沿用上一個狀態
        }
    }

    /// 退出訊號：只認本工具這個工作階段之後寫的版本（比 mtime）。判讀規則見 `exit_verdict`。
    fn exit_signal(&mut self) -> ExitSignal {
        let path = self.dir.join("exiting.txt");
        let Ok(mtime) = fs::metadata(&path).and_then(|m| m.modified()) else { return ExitSignal::None };
        if mtime <= self.exit_seen {
            return ExitSignal::None; // 常見路徑：連讀都不用讀
        }
        let body = fs::read_to_string(&path).ok();
        let (signal, seen) = exit_verdict(body.as_deref(), mtime, self.exit_seen);
        self.exit_seen = seen;
        signal
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

    /// 暫停、或玩家把「離開時還原」關掉：責任與**責任來源**一起放掉。只清 Restorer 不夠——舊的
    /// 「已送出切換」請求還留著的話，暫停期間切出去、解除暫停那一輪的回讀會把它當成新責任補送一次。
    fn disown(&mut self) {
        self.takeover.forget();
        self.restorer.cancel();
    }

    /// PZ 在前景這一輪的責任追蹤。Win32 的部分（送出切換、回讀配置）由 tick 做完再把結果餵進來，
    /// 所以「送出→回讀→暫停→切出」這種跨元件順序可以整合測試。
    fn guard_tick(&mut self, window: isize, restore: bool, posted_safe: bool, is_safe: bool) {
        self.was_fg = true;
        self.restorer.in_game();
        if !restore {
            self.disown(); // 關掉還原：連舊請求一起丟，重新打開也不追舊責任
            return;
        }
        if posted_safe {
            self.takeover.requested(window);
        }
        if self.takeover.confirm(window, is_safe) {
            self.restorer.took_over();
        }
    }

    /// PZ 不在前景（含關程序等待）這一輪。`game` 是遊戲視窗與「它的配置現在是不是安全配置」——
    /// 剛送出切換就馬上 alt-tab 的話，回讀會落在離開之後這幾輪，這裡確認到了才算真的切走過。
    fn away_tick(&mut self, now: Instant, game: Option<(isize, bool)>, fg: Option<Foreground>, safe: HKL, restore: bool) -> Step {
        if !restore {
            self.disown();
        }
        if std::mem::take(&mut self.was_fg) {
            self.restorer.left();
        }
        match game {
            Some((pz, is_safe)) => {
                if self.takeover.confirm(pz, is_safe) {
                    self.restorer.took_over();
                }
            }
            None => self.takeover.forget(), // 視窗沒了：這次的請求不用再追
        }
        self.restorer.observe(now, fg, safe, restore)
    }

    fn tick(&mut self, paused: bool, restore: bool) -> Status {
        self.heartbeat();
        if paused {
            self.disown();
            return Status::Paused;
        }
        let Some(safe) = self.safe else {
            self.rescan_layouts();
            return Status::NoEnglish;
        };
        let cached = self.hwnd.filter(|h| unsafe { IsWindow(Some(*h)).as_bool() });
        let hwnd = cached.or_else(|| {
            self.hwnd = find_game_window();
            self.hwnd
        });
        let signal = self.exit_signal();
        let quitting = self.exit.update(hwnd.map(|h| h.0 as isize), signal);
        let fg = unsafe { GetForegroundWindow() };
        let now = Instant::now();
        // 玩家確認關程序後，視窗還會活很久（GameWindow.exit() 存檔／關 Steam／壓日誌），而且多半還是前景。
        // 那段時間遊戲已經不吃按鍵，繼續守只會把 en-US 留在桌面上，所以訊號一到就當作離開。
        if quitting || hwnd != Some(fg) {
            // 只有還有未確認的請求時才去讀 PZ 緒的配置
            let game = hwnd.map(|pz| {
                let id = pz.0 as isize;
                let is_safe = self.takeover.pending_for(id)
                    && unsafe { GetKeyboardLayout(GetWindowThreadProcessId(pz, None)) } == safe;
                (id, is_safe)
            });
            // 還原目標永遠不會是遊戲視窗自己（關程序等待中它還是前景，而且已經不處理訊息了），
            // 也不會是 Alt+Tab 切換器那種過渡殼層
            let target = (!fg.0.is_null() && Some(fg) != hwnd && !is_transient_shell(fg)).then(|| {
                let thread = unsafe { GetWindowThreadProcessId(fg, None) };
                Foreground { window: fg.0 as isize, thread, layout: unsafe { GetKeyboardLayout(thread) } }
            });
            if let Step::Post { window, thread, layout } = self.away_tick(now, game, target, safe, restore) {
                let win = HWND(window as *mut core::ffi::c_void);
                // 送出前重驗：決策到這一行之間前景可能已經換人，HWND 也可能被回收給別的程序
                let still_there = unsafe {
                    IsWindow(Some(win)).as_bool()
                        && GetForegroundWindow() == win
                        && GetWindowThreadProcessId(win, None) == thread
                };
                if still_there {
                    unsafe {
                        let _ = PostMessageW(Some(win), WM_INPUTLANGCHANGEREQUEST, WPARAM(0), LPARAM(layout.0 as isize));
                    }
                }
            }
            return match (quitting, hwnd.is_some()) {
                (true, _) => Status::Closing,
                (false, true) => Status::Background,
                (false, false) => Status::NoGame,
            };
        }
        let hwnd = fg;
        let thread = unsafe { GetWindowThreadProcessId(hwnd, None) };
        let current = unsafe { GetKeyboardLayout(thread) };
        if is_ime_lang(current) {
            self.ime = Some(current);
        }
        self.read_typing();
        // 打字 → 玩家的 IME（沒看過就不動）；沒打字 → 只有目前是 CJK IME 才切到安全配置，其他配置本來就安全
        let want = match (self.typing, self.ime) {
            (true, Some(ime)) => ime,
            (true, None) => current,
            (false, _) if is_ime_lang(current) => safe,
            (false, _) => current,
        };
        let same_target_recently = self.last_post.is_some_and(|(h, t)| h == want && t.elapsed() < RESEND_AFTER);
        let mut posted_safe = false;
        if current != want && !same_target_recently {
            self.last_post = Some((want, now));
            posted_safe = want == safe;
            unsafe {
                let _ = PostMessageW(Some(hwnd), WM_INPUTLANGCHANGEREQUEST, WPARAM(0), LPARAM(want.0 as isize));
            }
        }
        // 回讀確認我們真的把這個視窗切走了，才背上還原責任（請求 ≠ 已接受）
        self.guard_tick(hwnd.0 as isize, restore, posted_safe, current == safe);
        if self.typing { Status::Typing } else { Status::Guarding }
    }

    /// 還原還沒完成時不要放慢輪詢：前景穩定判定與回讀確認都要靠接下來這幾輪
    fn restoring(&self) -> bool {
        self.restorer.pending.is_some()
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

const RELEASES_API: &str = "https://api.github.com/repos/Minidoracat/MinidoracatIMEGuardFor42/releases/latest";
const RELEASES_PAGE: &str = "https://github.com/Minidoracat/MinidoracatIMEGuardFor42/releases/latest";
const UPDATE_EVERY: Duration = Duration::from_secs(24 * 60 * 60);

/// settings.txt 每行 `key=0|1`；缺檔／缺行＝開。存檔時整檔重寫（只有兩個 key）。
fn setting(dir: &Path, key: &str) -> bool {
    fs::read_to_string(dir.join("settings.txt")).map(|s| !s.contains(&format!("{key}=0"))).unwrap_or(true)
}

fn save_settings(dir: &Path, check_updates: bool, restore_desktop: bool) {
    let _ = fs::create_dir_all(dir);
    let body = format!("check_updates={}\nrestore_desktop={}\n", check_updates as u8, restore_desktop as u8);
    let _ = fs::write(dir.join("settings.txt"), body);
}

/// 背景執行緒用 Windows 內建 curl 抓最新 Release 的 tag（`v42.20.4-0.1.1`），只回傳比本版新的版本字串。
/// ponytail: 不加 HTTP 依賴；curl 缺席或離線就當沒新版。版本比較只看 tag 最後一段（工具版號，與 Cargo 同步 bump）。
fn spawn_update_check(tx: mpsc::Sender<String>) {
    std::thread::spawn(move || {
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let out = std::process::Command::new("curl")
            .args(["-sL", "--max-time", "15", "-H", "User-Agent: pz-ime-guard", RELEASES_API])
            .creation_flags(CREATE_NO_WINDOW)
            .output();
        let Ok(out) = out else { return };
        let body = String::from_utf8_lossy(&out.stdout);
        let Some(tag) = body.split("\"tag_name\":\"").nth(1).and_then(|r| r.split('"').next()) else { return };
        let latest = tag.rsplit('-').next().unwrap_or(tag);
        if is_newer(latest, env!("CARGO_PKG_VERSION")) {
            let _ = tx.send(tag.to_string());
        }
    });
}

fn is_newer(candidate: &str, current: &str) -> bool {
    let parse = |v: &str| -> Vec<u32> { v.split('.').map(|p| p.parse().unwrap_or(0)).collect() };
    parse(candidate) > parse(current)
}

const QUIT_EVENT: windows::core::PCWSTR = w!("Local\\pz-ime-guard-quit");

/// 單一實例：named mutex 由 OS 在程序結束時釋放。搶到就故意不關 handle；已有別份在跑就關掉自己的，免得拖住它釋放。
fn claim_single_instance() -> bool {
    unsafe {
        let h = CreateMutexW(None, false, w!("Local\\pz-ime-guard-single-instance"));
        if GetLastError() != ERROR_ALREADY_EXISTS {
            return true;
        }
        if let Ok(h) = h {
            let _ = CloseHandle(h);
        }
        false
    }
}

/// 請正在跑的那一份退出（它收到事件就走和選單「結束」同一條路）。加入本機制之前的舊版沒有這個事件 → false。
fn ask_running_to_quit() -> bool {
    unsafe {
        let Ok(ev) = OpenEventW(EVENT_MODIFY_STATE, false, QUIT_EVENT) else { return false };
        let ok = SetEvent(ev).is_ok();
        let _ = CloseHandle(ev);
        ok
    }
}

fn main() {
    let s = strings();
    let state_dir = state_dir();
    // 開機自動啟動的那一份帶 --startup：第二實例要安靜（開機時彈對話框很煩），手動點開的照常提示
    let from_startup = std::env::args().any(|a| a == "--startup");
    let say = |text: &str| {
        if !from_startup {
            unsafe { MessageBoxW(None, &HSTRING::from(text), w!("pz-ime-guard"), MB_OK | MB_ICONINFORMATION) };
        }
    };
    if !claim_single_instance() {
        // 已有一份在跑：比本版舊就請它退出、等它放掉 mutex 後接手（開機捷徑也因此改指向本 exe）；
        // 同版或更新就照舊提示後離開。沒有 quit 事件＝加入本機制之前的舊版，只能請玩家自己從系統匣結束。
        let running = fs::read_to_string(state_dir.join("running.txt")).unwrap_or_default();
        let has_quit_event = unsafe { OpenEventW(EVENT_MODIFY_STATE, false, QUIT_EVENT) }.map(|ev| unsafe { CloseHandle(ev) }).is_ok();
        if has_quit_event && !is_newer(env!("CARGO_PKG_VERSION"), running.trim()) {
            say(s.already_running);
            return;
        }
        if !ask_running_to_quit() || !(0..50).any(|_| { std::thread::sleep(TICK); claim_single_instance() }) {
            say(s.old_running);
            return;
        }
    }
    // manual-reset：MsgWaitForMultipleObjects 喚醒時不會把它吃掉，迴圈尾端再查一次
    let quit_event = unsafe { CreateEventW(None, true, false, QUIT_EVENT) }.ok();
    let _ = fs::create_dir_all(&state_dir);
    let _ = fs::write(state_dir.join("running.txt"), env!("CARGO_PKG_VERSION"));
    let about = MenuItem::new(format!("pz-ime-guard {} — MinidoracatIMEGuardFor42", env!("CARGO_PKG_VERSION")), false, None);
    let workshop = MenuItem::new(s.menu_workshop, true, None);
    let github = MenuItem::new(s.menu_github, true, None);
    let pause = CheckMenuItem::new(s.menu_pause, true, false, None);
    let quit = MenuItem::new(s.menu_quit, true, None);
    let updates = CheckMenuItem::new(s.menu_check_updates, true, setting(&state_dir, "check_updates"), None);
    let restore = CheckMenuItem::new(s.menu_restore_desktop, true, setting(&state_dir, "restore_desktop"), None);
    // 開機自動啟動：唯一真相是 Startup 資料夾裡的捷徑本身，不另外存設定，預設沒有捷徑＝關
    // exe 搬過家就就地改寫捷徑（已經指向本 exe 時 set_enabled 完全不寫檔）。修不動就把錯誤攤開、
    // 選項停用：與其給一個「以為還會自動啟動」的勾，不如講清楚它現在指向別的地方。
    let mut startup_state = startup::enabled();
    if startup_state.as_ref().is_ok_and(|on| *on) {
        startup_state = startup::set_enabled(true).and_then(|()| startup::enabled()); // 改寫後回讀確認
    }
    let startup_item = match &startup_state {
        Ok(on) => CheckMenuItem::new(s.menu_startup, true, *on, None),
        // 讀不到真實狀態就不能給一個會誤導的勾：停用該項，並把技術細節直接寫在字面上
        Err(e) => CheckMenuItem::new(format!("{} — {e}", s.menu_startup), false, false, None),
    };
    let menu = Menu::with_items(&[
        &about,
        &workshop,
        &github,
        &restore,
        &startup_item,
        &updates,
        &PredefinedMenuItem::separator(),
        &pause,
        &quit,
    ])
    .expect("tray menu");
    let mut status = Status::NoGame;
    let tray: TrayIcon = TrayIconBuilder::new()
        .with_menu(Box::new(menu))
        .with_tooltip(status.text(s))
        .with_icon(icon(status))
        .build()
        .expect("tray icon");

    let mut guard = Guard::new();
    first_run_notice(&guard.dir, s);
    if guard.safe.is_none() {
        no_safe_layout_notice(s);
    }
    // MOD 一寫 state.txt 就醒來，不用等下一輪；目錄通知拿不到就退回純輪詢（heartbeat 順便把目錄建好）
    guard.heartbeat();
    let watch = unsafe {
        FindFirstChangeNotificationW(
            &HSTRING::from(guard.dir.as_os_str()),
            false,
            FILE_NOTIFY_CHANGE_LAST_WRITE | FILE_NOTIFY_CHANGE_SIZE,
        )
    }
    .ok();
    let (update_tx, update_rx) = mpsc::channel();
    let mut last_update_check: Option<Instant> = None;
    let handles: Vec<HANDLE> = [watch, quit_event].into_iter().flatten().collect(); // watch 必須在第 0 位
    loop {
        pump_messages();
        while let Ok(event) = MenuEvent::receiver().try_recv() {
            let id = event.id();
            if id == quit.id() {
                return;
            }
            let url = if id == workshop.id() {
                Some(w!("https://steamcommunity.com/sharedfiles/filedetails/?id=3802890539"))
            } else if id == github.id() {
                Some(w!("https://github.com/Minidoracat/MinidoracatIMEGuardFor42"))
            } else {
                None
            };
            if let Some(url) = url {
                unsafe { ShellExecuteW(None, w!("open"), url, None, None, SW_SHOWNORMAL) };
            }
            if id == updates.id() || id == restore.id() {
                save_settings(&state_dir, updates.is_checked(), restore.is_checked());
            }
            if id == startup_item.id() {
                let want = startup_item.is_checked();
                if let Err(e) = startup::set_enabled(want) {
                    let text = s.startup_failed.replace("{error}", &e);
                    unsafe { MessageBoxW(None, &HSTRING::from(text), w!("pz-ime-guard"), MB_OK | MB_ICONWARNING) };
                    // 做不到就把勾勾撥回真實狀態，別讓它停在騙人的位置
                    startup_item.set_checked(startup::enabled().unwrap_or(false));
                }
            }
        }
        if updates.is_checked() && last_update_check.is_none_or(|t| t.elapsed() >= UPDATE_EVERY) {
            last_update_check = Some(Instant::now());
            spawn_update_check(update_tx.clone());
        }
        if let Ok(tag) = update_rx.try_recv() {
            let text = s.update_available.replace("{tag}", &tag).replace("{current}", env!("CARGO_PKG_VERSION"));
            let answer = unsafe { MessageBoxW(None, &HSTRING::from(text), w!("pz-ime-guard"), MB_YESNO | MB_ICONINFORMATION) };
            if answer == IDYES {
                unsafe { ShellExecuteW(None, w!("open"), &HSTRING::from(RELEASES_PAGE), None, None, SW_SHOWNORMAL) };
            }
        }
        let next = guard.tick(pause.is_checked(), restore.is_checked());
        if next != status {
            status = next;
            let _ = tray.set_icon(Some(icon(status)));
            let _ = tray.set_tooltip(Some(status.text(s)));
        }
        // PZ 沒開時沒有東西要守，放慢到 500 ms 省得一直掃視窗；還原進行中除外（關遊戲後正是這條路）
        let wait = if status == Status::NoGame && !guard.restoring() { TICK * 5 } else { TICK };
        unsafe {
            let woke = MsgWaitForMultipleObjects(Some(&handles), false, wait.as_millis() as u32, QS_ALLINPUT);
            if let Some(handle) = watch
                && woke == WAIT_OBJECT_0
            {
                let _ = FindNextChangeNotification(handle);
            }
            if quit_event.is_some_and(|ev| WaitForSingleObject(ev, 0) == WAIT_OBJECT_0) {
                return;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn hkl(v: usize) -> HKL {
        HKL(v as *mut core::ffi::c_void)
    }

    #[test]
    fn ime_languages_are_ime_everything_else_is_safe() {
        for v in [0x0404_0404, 0xE020_0404, 0x0804_0804, 0xE001_0411, 0x0412_0412, 0x042A_042A, 0x0439_0439] {
            assert!(is_ime_lang(hkl(v)), "{v:#x} should be IME");
        }
        for v in [0x0409_0409, 0x0809_0809, 0x0C09_0C09, 0x0407_0407, 0x0419_0419, 0x041E_041E] {
            assert!(!is_ime_lang(hkl(v)), "{v:#x} should be safe (Thai is a plain layout)");
        }
    }

    #[test]
    fn safe_layout_prefers_en_us_then_any_english_then_any_non_ime() {
        let us = hkl(0x0409_0409);
        let gb = hkl(0x0809_0809);
        let de = hkl(0x0407_0407);
        let tw = hkl(0x0404_0404);
        assert_eq!(pick_safe_layout(&[tw, gb, us]), Some(us));
        assert_eq!(pick_safe_layout(&[tw, de, gb]), Some(gb));
        assert_eq!(pick_safe_layout(&[tw, de]), Some(de));
        assert_eq!(pick_safe_layout(&[tw, hkl(0xE001_0411)]), None);
    }
}

#[cfg(test)]
mod update_tests {
    use super::is_newer;

    #[test]
    fn version_compare_is_numeric_not_lexical() {
        assert!(is_newer("0.1.1", "0.1.0"));
        assert!(is_newer("0.10.0", "0.9.9"));
        assert!(!is_newer("0.1.0", "0.1.0"));
        assert!(!is_newer("0.0.9", "0.1.0"));
        assert!(is_newer("v42.20.4-0.2.0".rsplit('-').next().unwrap(), "0.1.1"));
    }
}

/// 還原與退出的狀態轉移。Win32 的部分（GetForegroundWindow／GetKeyboardLayout／PostMessage）留在 tick，
/// 這裡測的是 tick 餵進來的觀察值怎麼變成「送不送、送給誰、送幾次」。
#[cfg(test)]
mod restore_tests {
    use super::*;

    pub(super) const SAFE: usize = 0x0409_0409; // en-US
    pub(super) const IME: usize = 0x0404_0404; // 繁中注音
    const DE: usize = 0x0407_0407; // 玩家自己改去的第三種配置

    pub(super) fn hkl(v: usize) -> HKL {
        HKL(v as *mut core::ffi::c_void)
    }
    fn th(window: isize) -> u32 {
        1000 + window as u32
    }
    pub(super) fn fg(window: isize, layout: usize) -> Option<Foreground> {
        Some(Foreground { window, thread: th(window), layout: hkl(layout) })
    }
    pub(super) fn post(window: isize, layout: usize) -> Step {
        Step::Post { window, thread: th(window), layout: hkl(layout) }
    }
    pub(super) fn ms(n: u64) -> Duration {
        Duration::from_millis(n)
    }

    /// 桌面本來用 IME、PZ 被我們切成 en-US、然後玩家離開：回到一個待得夠久的前景才還原。
    fn armed(t: Instant) -> Restorer {
        let mut r = Restorer::default();
        r.observe(t, fg(1, IME), hkl(SAFE), true);
        r.observe(t + RESTORE_SETTLE, fg(1, IME), hkl(SAFE), true); // 待滿才算數
        assert_eq!(r.desktop, Some(hkl(IME)));
        r.took_over();
        r.left();
        r
    }

    #[test]
    fn restore_waits_for_a_settled_foreground_then_confirms_by_read_back() {
        let t = Instant::now();
        let mut r = armed(t);
        // 過渡狀態（沒有前景、前景還是關程序中的 PZ、Alt+Tab 切換器）不起算時鐘、也不吃掉待還原
        assert_eq!(r.observe(t + ms(400), None, hkl(SAFE), true), Step::Nothing);
        assert_eq!(r.observe(t + ms(500), fg(9, SAFE), hkl(SAFE), true), Step::Nothing);
        assert_eq!(r.observe(t + ms(600), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
        // 連兩輪同一個 HWND 不算穩定：目錄變更通知可以在 1 ms 內叫醒第二輪
        assert_eq!(r.observe(t + ms(601), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
        assert_eq!(r.observe(t + ms(700), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
        // 真的待滿 RESTORE_SETTLE 才送，而且只送給它
        assert_eq!(r.observe(t + ms(900), fg(7, SAFE), hkl(SAFE), true), post(7, IME));
        // 回讀：配置真的回來了 → 責任結束，之後不再動
        assert_eq!(r.observe(t + ms(1000), fg(7, IME), hkl(SAFE), true), Step::Nothing);
        assert!(r.pending.is_none() && !r.owed);
        assert_eq!(r.observe(t + ms(1400), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
    }

    #[test]
    fn restore_retries_a_bounded_number_of_times_then_gives_up() {
        let t = Instant::now();
        let mut r = armed(t);
        let mut posts = 0;
        for step in 0..=40u64 {
            if r.observe(t + ms(600 + step * 100), fg(7, SAFE), hkl(SAFE), true) != Step::Nothing {
                posts += 1;
            }
        }
        assert_eq!(posts, RESTORE_TRIES, "請求 ≠ 已接受：重送有上限，不能一直吵");
        assert!(r.pending.is_none());
    }

    #[test]
    fn a_long_close_wait_does_not_burn_the_give_up_budget() {
        let t = Instant::now();
        let mut r = armed(t);
        // 關程序等待：GameWindow.exit() 存檔／關 Steam／壓日誌期間前景一直是 PZ 自己（餵 None）
        for step in 1..=60u64 {
            assert_eq!(r.observe(t + ms(step * 1000), None, hkl(SAFE), true), Step::Nothing);
        }
        // 視窗終於消失、桌面回來：責任還在，照樣還原
        r.observe(t + ms(61_000), fg(7, SAFE), hkl(SAFE), true);
        assert_eq!(r.observe(t + ms(61_400), fg(7, SAFE), hkl(SAFE), true), post(7, IME));
    }

    #[test]
    fn restore_window_starts_at_the_settled_foreground_not_at_leaving() {
        let t = Instant::now();
        let mut r = armed(t);
        // 離開後 1.9 秒前景才終於出現——舊寫法這裡已經逾時，現在才開始算
        r.observe(t + ms(1900), fg(7, SAFE), hkl(SAFE), true);
        assert_eq!(r.observe(t + ms(2300), fg(7, SAFE), hkl(SAFE), true), post(7, IME));
        // 但穩定之後就有時限：兩秒內沒生效就收手，不要幾秒後才突然改玩家的鍵盤
        assert_eq!(r.observe(t + ms(4500), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
        assert!(r.pending.is_none());
    }

    #[test]
    fn unstable_foreground_forever_eventually_gives_up() {
        let t = Instant::now();
        let mut r = armed(t);
        for step in 1..=20u64 {
            assert_eq!(r.observe(t + ms(step * 1000), fg(step as isize, SAFE), hkl(SAFE), true), Step::Nothing);
        }
        assert!(r.pending.is_none(), "前景一直換就放棄，不要無限期掛著等機會");
    }

    #[test]
    fn pausing_drops_the_obligation_so_leaving_later_never_fires_a_late_restore() {
        let t = Instant::now();
        let mut r = armed(t);
        r.cancel(); // 暫停／取消「還原」選項
        assert!(!r.owed, "取消的是責任本身，不只是這一次的待還原");
        r.left(); // 暫停期間玩家切出去，恢復後那一輪才走到這裡
        r.observe(t + ms(400), fg(7, SAFE), hkl(SAFE), true);
        assert_eq!(r.observe(t + ms(800), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
        assert!(r.pending.is_none(), "取消後不得重新武裝");
    }

    #[test]
    fn turning_the_option_off_mid_flight_drops_it_and_going_back_into_the_game_too() {
        let t = Instant::now();
        // 選項當場被關掉：這一輪就放掉，不會等一下才補送
        let mut r = armed(t);
        r.observe(t + ms(400), fg(7, SAFE), hkl(SAFE), false);
        assert_eq!(r.observe(t + ms(800), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
        // 玩家又切回 PZ：這次的待還原作廢（下次離開再重新起算）
        let mut r = armed(t);
        r.in_game();
        r.observe(t + ms(400), fg(7, SAFE), hkl(SAFE), true);
        assert_eq!(r.observe(t + ms(800), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
    }

    #[test]
    fn no_takeover_means_no_restore() {
        let t = Instant::now();
        let mut r = Restorer::default();
        r.observe(t, fg(1, IME), hkl(SAFE), true);
        r.observe(t + RESTORE_SETTLE, fg(1, IME), hkl(SAFE), true);
        r.left(); // 沒 took_over()：這一局從來沒切過 PZ
        r.observe(t + ms(400), fg(7, SAFE), hkl(SAFE), true);
        assert_eq!(r.observe(t + ms(800), fg(7, SAFE), hkl(SAFE), true), Step::Nothing);
        assert!(r.pending.is_none(), "沒切過就沒有還原責任，不能把配置推給桌面");
    }

    #[test]
    fn a_layout_the_player_picked_himself_wins_over_our_intent() {
        let t = Instant::now();
        let mut r = armed(t);
        // 玩家離開 PZ 後自己按 Alt+Shift 切到第三種配置
        r.observe(t + ms(400), fg(7, DE), hkl(SAFE), true);
        assert_eq!(r.observe(t + ms(800), fg(7, DE), hkl(SAFE), true), Step::Nothing);
        assert!(r.pending.is_none() && !r.owed);
        assert_eq!(r.desktop, Some(hkl(DE)), "之後要還原的是他新選的那個");
    }

    #[test]
    fn a_read_back_that_lands_after_leaving_still_arms_the_restore() {
        let t = Instant::now();
        let mut r = Restorer::default();
        r.observe(t, fg(1, IME), hkl(SAFE), true);
        r.observe(t + RESTORE_SETTLE, fg(1, IME), hkl(SAFE), true);
        // 送出切換的下一輪玩家就 alt-tab 了，這時還沒回讀到，責任還沒成立
        r.left();
        r.observe(t + ms(400), fg(7, SAFE), hkl(SAFE), true);
        r.took_over(); // 回讀確認落在離開之後（tick 會對還活著的 PZ 視窗繼續確認）
        r.observe(t + ms(500), fg(7, SAFE), hkl(SAFE), true);
        assert_eq!(r.observe(t + ms(800), fg(7, SAFE), hkl(SAFE), true), post(7, IME));
    }

    #[test]
    fn an_empty_or_unreadable_exit_file_never_advances_the_watermark() {
        let t0 = SystemTime::UNIX_EPOCH;
        let t1 = t0 + Duration::from_secs(1);
        let t2 = t0 + Duration::from_secs(2);
        // MOD 先截斷再寫，中間那一瞬間是空檔：認成「這個版本看過了」會整個漏掉退出訊號
        assert_eq!(exit_verdict(Some(""), t1, t0), (ExitSignal::None, t0));
        assert_eq!(exit_verdict(None, t1, t0), (ExitSignal::None, t0));
        // 同一個版本下一輪讀到完整內容，照樣認得出來
        assert_eq!(exit_verdict(Some("1\n"), t1, t0), (ExitSignal::Quitting, t1));
        // "0" 不是「沒事」，是「新的一局開始了」
        assert_eq!(exit_verdict(Some("0"), t2, t0), (ExitSignal::Reset, t2));
        // 水位之下（本工具啟動前寫的）一律不認：上一局的殘留不會停守下一局
        assert_eq!(exit_verdict(Some("1"), t1, t2), (ExitSignal::None, t2));
    }

    #[test]
    fn exit_latch_holds_for_the_window_that_quit_and_never_leaks_into_the_next_game() {
        let mut latch = ExitLatch::default();
        assert!(!latch.update(Some(5), ExitSignal::None));
        assert!(latch.update(Some(5), ExitSignal::Quitting), "訊號一到就鎖存");
        assert!(latch.update(Some(5), ExitSignal::None), "視窗還在（存檔／關 Steam 中）也不回頭當遊戲中");
        assert!(!latch.update(None, ExitSignal::None), "視窗消失＝解除");
        assert!(!latch.update(Some(7), ExitSignal::None), "下一局是乾淨的");
        // 沒有遊戲視窗時被消費掉的殘留旗標不會黏到下一局
        let mut stale = ExitLatch::default();
        assert!(!stale.update(None, ExitSignal::Quitting));
        assert!(!stale.update(Some(3), ExitSignal::None));
    }

    /// 工具暫停 → 上一局的 "1" 沒被處理 → 新的一局已經開窗 → 解除暫停那一輪才讀到舊的 "1"，
    /// 鎖在新視窗上。MOD 的 OnGameBoot "0" 必須把它解開，否則新的一局永遠停在 Closing。
    #[test]
    fn a_stale_quit_flag_latched_onto_a_fresh_game_is_released_by_the_next_boot_reset() {
        let mut latch = ExitLatch::default();
        assert!(latch.update(Some(9), ExitSignal::Quitting), "解除暫停那一輪讀到舊的 1");
        assert!(latch.update(Some(9), ExitSignal::None));
        assert!(!latch.update(Some(9), ExitSignal::Reset), "OnGameBoot 的 0 解鎖");
        assert!(!latch.update(Some(9), ExitSignal::None), "解開之後不會再滑回 Closing");
        // 同一局之後真的按退出，仍然鎖到視窗消失為止
        assert!(latch.update(Some(9), ExitSignal::Quitting));
        assert!(latch.update(Some(9), ExitSignal::None));
        assert!(!latch.update(None, ExitSignal::None));
    }

    #[test]
    fn a_takeover_is_consumed_on_confirmation_and_dropped_on_pause() {
        let mut t = Takeover::default();
        t.requested(5);
        assert!(!t.confirm(5, false), "請求 ≠ 已接受");
        assert!(!t.confirm(9, true), "別的視窗不算");
        assert!(t.confirm(5, true));
        assert!(!t.confirm(5, true), "確認過就消費掉，不能再認第二次");
        t.requested(5);
        t.forget();
        assert!(!t.confirm(5, true), "丟掉的請求不得在之後補成責任");
    }
}

/// Guard 層的整合：把 tick 的接線（Win32 以外的部分）照實跑一遍，驗「送出→回讀→暫停→切出」
/// 這種跨元件順序，不只單看 Restorer。
#[cfg(test)]
mod flow_tests {
    use super::restore_tests::*;
    use super::*;

    const PZ: isize = 77;

    /// 桌面本來用 IME，然後玩家進 PZ、我們把它切成 en-US 並回讀確認。
    fn in_game(t: Instant, safe: HKL) -> Guard {
        let mut g = Guard::new();
        g.away_tick(t, None, fg(1, IME), safe, true);
        g.away_tick(t + RESTORE_SETTLE, None, fg(1, IME), safe, true);
        assert_eq!(g.restorer.desktop, Some(hkl(IME)));
        g.guard_tick(PZ, true, true, false); // 送出切換
        g.guard_tick(PZ, true, false, true); // 下一輪回讀確認
        assert!(g.restorer.owed);
        g
    }

    #[test]
    fn the_normal_alt_tab_path_restores_end_to_end() {
        let t = Instant::now();
        let safe = hkl(SAFE);
        let mut g = in_game(t, safe);
        g.away_tick(t + ms(400), Some((PZ, false)), fg(7, SAFE), safe, true);
        assert_eq!(g.away_tick(t + ms(800), Some((PZ, false)), fg(7, SAFE), safe, true), post(7, IME));
    }

    #[test]
    fn pausing_then_leaving_never_resurrects_the_obligation_from_an_old_request() {
        let t = Instant::now();
        let safe = hkl(SAFE);
        let mut g = in_game(t, safe);
        g.disown(); // 暫停
        assert!(!g.restorer.owed);
        // 暫停期間玩家切出去；解除暫停後這幾輪 PZ 視窗還在、而且還是 en-US
        assert_eq!(g.away_tick(t + ms(400), Some((PZ, true)), fg(7, SAFE), safe, true), Step::Nothing);
        assert_eq!(g.away_tick(t + ms(800), Some((PZ, true)), fg(7, SAFE), safe, true), Step::Nothing);
        assert!(!g.restorer.owed, "暫停丟掉的責任不得靠舊的『已送出切換』復活");
    }

    #[test]
    fn turning_restore_off_and_on_again_does_not_pick_up_the_old_obligation() {
        let t = Instant::now();
        let safe = hkl(SAFE);
        let mut g = in_game(t, safe);
        g.guard_tick(PZ, false, false, true); // 玩家把「離開時還原」關掉
        assert!(!g.restorer.owed);
        // 馬上又打開：舊的那一次切換不算數，要等下一次真的切換＋回讀才重新背責任
        g.away_tick(t + ms(400), Some((PZ, true)), fg(7, SAFE), safe, true);
        assert_eq!(g.away_tick(t + ms(800), Some((PZ, true)), fg(7, SAFE), safe, true), Step::Nothing);
    }

    #[test]
    fn a_read_back_that_only_lands_after_alt_tab_still_restores() {
        let t = Instant::now();
        let safe = hkl(SAFE);
        let mut g = Guard::new();
        g.away_tick(t, None, fg(1, IME), safe, true);
        g.away_tick(t + RESTORE_SETTLE, None, fg(1, IME), safe, true);
        g.guard_tick(PZ, true, true, false); // 送出切換，這一輪還沒被接受
        // 玩家立刻 alt-tab：PZ 視窗還在、配置已經變成 en-US，責任在離開之後才確認
        g.away_tick(t + ms(400), Some((PZ, true)), fg(7, SAFE), safe, true);
        assert!(g.restorer.owed);
        assert_eq!(g.away_tick(t + ms(800), Some((PZ, false)), fg(7, SAFE), safe, true), post(7, IME));
    }
}
