//! 登入 Windows 時啟動：當前使用者 Startup 資料夾（FOLDERID_Startup）裡一個固定名的 `pz-ime-guard.lnk`。
//!
//! 「已開啟」的唯一真相就是那個捷徑存在、參數剛好是 `--startup`、描述是我們的品牌標記；沒有第二份狀態檔。
//! 不寫登錄檔（Run/RunOnce）、不讀不寫 StartupApproved、不建服務或排程、不提權，也只管當前使用者。
//!
//! 已知界線：「登入」不是「開機」，而且 Windows 可能延後啟動；exe 搬走之後捷徑仍指向舊路徑
//! （捷徑追蹤不可靠，所以不用 Resolve 補，改由搬完之後手動跑一次時 `set_enabled(true)` 就地改寫）；
//! 使用者若從 設定 > 應用程式 > 啟動 把它關掉，這裡仍讀到「已開啟」——那個狀態沒有公開 API 可讀。

use std::{
    fs,
    io::ErrorKind,
    os::windows::ffi::OsStrExt,
    path::{Path, PathBuf},
};
use windows::{
    core::{HSTRING, Interface},
    Win32::Foundation::RPC_E_CHANGED_MODE,
    Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
        CoUninitialize, IPersistFile, STGM_READ,
    },
    Win32::UI::Shell::{
        FOLDERID_Startup, IShellLinkW, KF_FLAG_DEFAULT, SHGetKnownFolderPath, SLGP_RAWPATH, ShellLink,
    },
};

/// 固定 ASCII 檔名，不隨介面語言變；使用者要手動解除就是去 `shell:startup` 刪掉它。
const LINK_NAME: &str = "pz-ime-guard.lnk";
/// 捷徑帶的參數：登入副本撞到已在跑的那份時可以安靜退出。
const ARG: &str = "--startup";
/// 描述欄位當品牌標記：光看參數不夠，別人的捷徑也可能叫 `--startup`，認錯就會刪到別人的東西。
const BRAND: &str = "MinidoracatIMEGuardFor42 pz-ime-guard (start at logon)";
/// 捷徑的路徑欄位存不下 MAX_PATH 以上（GetPath 也只吐得回這麼多），超過就別假裝寫得成。
const MAX_PATH_WIDE: usize = 260;

/// 登入啟動目前是否開著。純讀取，不寫檔。
///
/// 捷徑不存在回 `false`；讀得到檔卻解不開（權限、壞檔、COM 失敗）回 `Err`，讓呼叫端顯示錯誤而不是誤勾。
pub fn enabled() -> Result<bool, String> {
    enabled_at(&link_path()?)
}

/// 開啟／關閉登入啟動。開啟＝（重）寫捷徑指向這份 exe；關閉＝刪掉我們那份捷徑。
///
/// 成功回傳代表回讀確認過：開啟後重新載入捷徑，目標等於 `current_exe()`、參數與描述都是我們的；
/// 關閉後確認檔案真的不見了。檔案存在或 API 回 Ok 都不算數（`Save`／`GetPath` 的 S_FALSE 也是 Ok）。
/// 已經是要的狀態就什麼都不寫。那個檔名若不是我們建的，一律不覆寫也不刪，直接回 `Err`。
pub fn set_enabled(enabled: bool) -> Result<(), String> {
    set_enabled_at(&link_path()?, enabled)
}

fn enabled_at(link: &Path) -> Result<bool, String> {
    Ok(read_link(link)?.is_some_and(|l| is_ours(&l) && !l.target.is_empty()))
}

fn set_enabled_at(link: &Path, enabled: bool) -> Result<(), String> {
    let existing = read_link(link)?;
    if let Some(found) = &existing {
        if !is_ours(found) {
            return Err(format!(
                "{} exists but was not created by pz-ime-guard (arguments {:?}, description {:?}); refusing to touch it",
                link.display(),
                found.args,
                found.description
            ));
        }
    }

    if !enabled {
        if existing.is_none() {
            return Ok(());
        }
        fs::remove_file(link).map_err(|e| format!("cannot delete {} ({e})", link.display()))?;
        if read_link(link)?.is_some() {
            return Err(format!("{} is still there after deleting it", link.display()));
        }
        return Ok(());
    }

    let exe = std::env::current_exe().map_err(|e| format!("cannot resolve current_exe ({e})"))?;
    if !exe.is_file() {
        return Err(format!("current_exe does not point at a file: {}", exe.display()));
    }
    let exe_str = exe.to_str().ok_or_else(|| format!("current_exe is not valid Unicode: {}", exe.display()))?;
    if exe.as_os_str().encode_wide().count() >= MAX_PATH_WIDE {
        return Err(format!("current_exe path is too long for a shortcut (>= {MAX_PATH_WIDE} chars): {exe_str}"));
    }
    // 已經指向這份 exe 就不要重寫，讓「每次啟動都修一下搬家」變成零寫入
    if existing.is_some_and(|l| same_path(&l.target, exe_str)) {
        return Ok(());
    }

    // 先寫暫存再換上去：寫壞了原本那份捷徑不會被毀掉。暫存檔名唯一、由我們 create_new 占用，
    // 失敗時只刪這一個（絕不照固定檔名亂刪），副檔名也不是 .lnk，萬一殘留在 Startup 也不會被登入執行。
    let temp = reserve_temp(link)?;
    let written = write_link(&temp, &exe).and_then(|()| verify(&temp, exe_str));
    if let Err(e) = written {
        let _ = fs::remove_file(&temp);
        return Err(e);
    }
    if let Err(e) = fs::rename(&temp, link) {
        let _ = fs::remove_file(&temp);
        return Err(format!("cannot move the new shortcut into {} ({e})", link.display()));
    }
    // 這裡失敗就只回報：檔案已經不是我們手上那份，可能被別的來源換掉了，不該連人家的一起刪
    verify(link, exe_str)
}

/// COM 生命週期：自己初始化的才自己解除。S_OK 與 S_FALSE 都要配一次 `CoUninitialize`；
/// RPC_E_CHANGED_MODE 代表這條執行緒已經是別的 apartment（例如 MTA），沿用它，但不能替它解除。
struct Com(bool);

impl Com {
    fn init() -> Result<Self, String> {
        let hr = unsafe { CoInitializeEx(None, COINIT_APARTMENTTHREADED) };
        if hr == RPC_E_CHANGED_MODE {
            return Ok(Self(false));
        }
        if hr.is_err() {
            return Err(format!("CoInitializeEx failed (0x{:08X})", hr.0 as u32));
        }
        Ok(Self(true))
    }
}

impl Drop for Com {
    fn drop(&mut self) {
        if self.0 {
            unsafe { CoUninitialize() };
        }
    }
}

struct Link {
    target: String,
    args: String,
    description: String,
}

fn link_path() -> Result<PathBuf, String> {
    // 不要自己拼 %APPDATA%：這個資料夾可能被重導向
    let _com = Com::init()?;
    let raw = unsafe { SHGetKnownFolderPath(&FOLDERID_Startup, KF_FLAG_DEFAULT, None) }
        .map_err(|e| format!("SHGetKnownFolderPath(FOLDERID_Startup) failed ({e})"))?;
    if raw.is_null() {
        return Err("SHGetKnownFolderPath(FOLDERID_Startup) returned null".into());
    }
    let dir = unsafe { raw.to_string() };
    unsafe { CoTaskMemFree(Some(raw.as_ptr() as *const core::ffi::c_void)) };
    let dir = dir.map_err(|e| format!("startup folder path is not valid UTF-16 ({e})"))?;
    if dir.is_empty() {
        return Err("startup folder path is empty".into());
    }
    Ok(PathBuf::from(dir).join(LINK_NAME))
}

/// 讀回捷徑內容；那個路徑沒有目錄項目時回 `Ok(None)`，其他讀取問題回 `Err`。
/// 用 SLGP_RAWPATH，不 Resolve（會彈對話框、也不保證找得到）。
fn read_link(path: &Path) -> Result<Option<Link>, String> {
    // symlink_metadata 不追連結：斷掉的符號連結仍是「這裡有東西」，不能當成不存在而被 rename 蓋過去
    match fs::symlink_metadata(path) {
        Err(e) if e.kind() == ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(format!("cannot stat {} ({e})", path.display())),
        Ok(meta) if meta.file_type().is_symlink() => {
            return Err(format!("{} is a symlink; refusing to read or replace it", path.display()));
        }
        Ok(_) => {}
    }
    let _com = Com::init()?;
    let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
        .map_err(|e| format!("CoCreateInstance(ShellLink) failed ({e})"))?;
    let file: IPersistFile = link.cast().map_err(|e| format!("QueryInterface(IPersistFile) failed ({e})"))?;
    unsafe { file.Load(&HSTRING::from(path.as_os_str()), STGM_READ) }
        .map_err(|e| format!("cannot load {} ({e})", path.display()))?;
    let mut buf = [0u16; 1024];
    unsafe { link.GetPath(&mut buf, std::ptr::null_mut(), SLGP_RAWPATH.0 as u32) }
        .map_err(|e| format!("IShellLinkW::GetPath failed on {} ({e})", path.display()))?;
    let target = wide(&buf);
    let mut buf = [0u16; 1024];
    unsafe { link.GetArguments(&mut buf) }
        .map_err(|e| format!("IShellLinkW::GetArguments failed on {} ({e})", path.display()))?;
    let args = wide(&buf);
    let mut buf = [0u16; 1024];
    unsafe { link.GetDescription(&mut buf) }
        .map_err(|e| format!("IShellLinkW::GetDescription failed on {} ({e})", path.display()))?;
    Ok(Some(Link { target, args, description: wide(&buf) }))
}

fn write_link(path: &Path, exe: &Path) -> Result<(), String> {
    let _com = Com::init()?;
    let link: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }
        .map_err(|e| format!("CoCreateInstance(ShellLink) failed ({e})"))?;
    unsafe {
        // SetPath 吃的是「路徑」不是命令列：有空白也不加引號，中文路徑靠寬字元直接傳（不要轉 ACP）
        link.SetPath(&HSTRING::from(exe.as_os_str())).map_err(|e| format!("IShellLinkW::SetPath failed ({e})"))?;
        link.SetArguments(&HSTRING::from(ARG)).map_err(|e| format!("IShellLinkW::SetArguments failed ({e})"))?;
        link.SetDescription(&HSTRING::from(BRAND))
            .map_err(|e| format!("IShellLinkW::SetDescription failed ({e})"))?;
        let file: IPersistFile = link.cast().map_err(|e| format!("QueryInterface(IPersistFile) failed ({e})"))?;
        file.Save(&HSTRING::from(path.as_os_str()), true).map_err(|e| format!("cannot save {} ({e})", path.display()))?;
    }
    Ok(())
}

/// 回讀才算數：Save 的 Ok 包含 S_FALSE（沒存成），檔案存在也不代表內容對。
fn verify(path: &Path, exe: &str) -> Result<(), String> {
    let link = read_link(path)?.ok_or_else(|| format!("{} was not written", path.display()))?;
    if !same_path(&link.target, exe) {
        return Err(format!("shortcut target mismatch: wrote {exe:?}, read back {:?}", link.target));
    }
    if !is_ours(&link) {
        return Err(format!(
            "shortcut markers mismatch: read back arguments {:?}, description {:?}",
            link.args, link.description
        ));
    }
    Ok(())
}

/// 占一個唯一的暫存檔名（非 .lnk，登入不會執行它）。`create_new` 沒搶到就換下一個，
/// 所以我們永遠只會動自己剛建出來的那個檔，不會覆蓋或刪掉別人的同名檔。
fn reserve_temp(link: &Path) -> Result<PathBuf, String> {
    let pid = std::process::id();
    for n in 0..64u32 {
        let candidate = link.with_file_name(format!("pz-ime-guard.{pid}-{n}.tmp"));
        match fs::OpenOptions::new().write(true).create_new(true).open(&candidate) {
            Ok(_) => return Ok(candidate),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => continue,
            Err(e) => return Err(format!("cannot create a temp file next to {} ({e})", link.display())),
        }
    }
    Err(format!("cannot find a free temp file name next to {}", link.display()))
}

/// 固定長度緩衝區 → 字串：只截掉 NUL 之後的殘留，其他一律照原樣留著（多一個空白就是別人的字串）。
fn wide(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

/// 這份捷徑是不是我們建的：參數與描述都要一字不差，不做任何正規化——
/// 全形空白之類的細微差異代表那是別人寫的，寧可放過也不能刪錯。
fn is_ours(link: &Link) -> bool {
    link.args == ARG && link.description == BRAND
}

/// Windows 路徑比對：大小寫不敏感（只折 ASCII），斜線一律當反斜線，空字串永遠不相等。
/// 字面不同還要再問一次檔案系統：`SetPath` 會把 8.3 短檔名（`MINIDO~1`）之類的寫法展開成長名，
/// 回讀到的字串本來就可能跟我們寫進去的不一樣，但指的是同一個檔案。
fn same_path(a: &str, b: &str) -> bool {
    let clean = |p: &str| p.trim().trim_matches('"').to_string();
    let (a, b) = (clean(a), clean(b));
    if a.is_empty() {
        return false;
    }
    let norm = |p: &str| p.replace('/', "\\").to_ascii_lowercase();
    if norm(&a) == norm(&b) {
        return true;
    }
    matches!((fs::canonicalize(&a), fs::canonicalize(&b)), (Ok(x), Ok(y)) if x == y)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 全部隔離在 %TEMP% 自己的目錄，絕不碰真正的 Startup 資料夾。
    fn scratch(name: &str) -> PathBuf {
        let dir = std::env::temp_dir().join(format!("pz-ime-guard-test-{name}-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);
        fs::create_dir_all(&dir).unwrap();
        dir
    }

    /// 真的建、真的讀回來、真的刪掉；順便確認暫存檔不會留在資料夾裡。
    #[test]
    fn creates_reads_back_and_removes_its_own_shortcut() {
        let dir = scratch("roundtrip");
        let link = dir.join(LINK_NAME);
        assert!(!enabled_at(&link).unwrap());

        set_enabled_at(&link, true).unwrap();
        assert!(enabled_at(&link).unwrap());
        let written = read_link(&link).unwrap().unwrap();
        let exe = std::env::current_exe().unwrap();
        assert!(same_path(&written.target, exe.to_str().unwrap()), "target was {:?}", written.target);
        assert_eq!(written.args, ARG);
        assert_eq!(written.description, BRAND);

        set_enabled_at(&link, true).unwrap(); // 冪等：已經指對了就不重寫
        assert!(enabled_at(&link).unwrap());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 1, "沒有殘留暫存檔");

        set_enabled_at(&link, false).unwrap();
        assert!(!enabled_at(&link).unwrap());
        assert_eq!(fs::read_dir(&dir).unwrap().count(), 0);
        let _ = fs::remove_dir_all(&dir);
    }

    /// 用 COM 造一份任意內容的捷徑，模擬別的程式在 Startup 放的東西。
    fn foreign_link(path: &Path, target: &Path, args: &str, description: &str) {
        let _com = Com::init().unwrap();
        let shell: IShellLinkW = unsafe { CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER) }.unwrap();
        unsafe {
            shell.SetPath(&HSTRING::from(target.as_os_str())).unwrap();
            if !args.is_empty() {
                shell.SetArguments(&HSTRING::from(args)).unwrap();
            }
            if !description.is_empty() {
                shell.SetDescription(&HSTRING::from(description)).unwrap();
            }
            let file: IPersistFile = shell.cast().unwrap();
            file.Save(&HSTRING::from(path.as_os_str()), true).unwrap();
        }
    }

    /// 同名但不是我們建的捷徑：不覆寫、不刪除，兩個方向都要回 Err。
    #[test]
    fn never_touches_a_shortcut_it_did_not_create() {
        let dir = scratch("foreign");
        let other = dir.join("someone-elses.exe");
        fs::write(&other, b"MZ").unwrap();

        // 沒有我們的標記，以及「差一點」的標記：參數多一個全形空白、描述前後多空白都不算自家的
        for (name, args, description) in [
            ("plain.lnk", "", ""),
            ("near-miss-args.lnk", "--startup\u{3000}", BRAND),
            ("near-miss-brand.lnk", ARG, " MinidoracatIMEGuardFor42 pz-ime-guard (start at logon) "),
        ] {
            let link = dir.join(name);
            foreign_link(&link, &other, args, description);
            assert!(!enabled_at(&link).unwrap(), "{name} 被誤認成自家的");
            assert!(set_enabled_at(&link, true).is_err(), "{name} 會被覆寫");
            assert!(set_enabled_at(&link, false).is_err(), "{name} 會被刪掉");
            let survivor = read_link(&link).unwrap().unwrap();
            assert!(same_path(&survivor.target, other.to_str().unwrap()), "{name} 的目標被動到了");
        }

        // 根本不是捷徑的同名檔：解不開要回 Err（不是默默當成 false 然後覆蓋掉）
        let junk = dir.join("junk.lnk");
        fs::write(&junk, b"not a shortcut").unwrap();
        assert!(enabled_at(&junk).is_err());
        assert!(set_enabled_at(&junk, true).is_err());
        assert!(set_enabled_at(&junk, false).is_err());
        assert_eq!(fs::read(&junk).unwrap(), b"not a shortcut");
        let _ = fs::remove_dir_all(&dir);
    }
}
