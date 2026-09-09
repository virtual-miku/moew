//! Detecting the file or folder currently selected in Windows Explorer

use windows::{
    Win32::{
        System::{
            Com::{
                CLSCTX_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx,
                CoUninitialize, IDispatch, IServiceProvider,
            },
            SystemServices::SFGAO_FILESYSTEM,
            Variant::{VARIANT, VT_I4},
        },
        UI::{
            Shell::{
                IShellBrowser, IShellItemArray, IShellWindows, SIGDN_DESKTOPABSOLUTEPARSING,
                SIGDN_FILESYSPATH, SVGIO_SELECTION, ShellWindows,
            },
            WindowsAndMessaging,
        },
    },
    core::{Interface, w},
};

/// Return the path of the selected item in Windows Explorer
pub fn get_explorer_selected() -> Vec<String> {
    let mut result = Vec::new();

    unsafe {
        if CoInitializeEx(None, COINIT_APARTMENTTHREADED).is_err() {
            return result;
        }

        let foreground = WindowsAndMessaging::GetForegroundWindow();

        let tab_window = WindowsAndMessaging::FindWindowExW(
            Some(foreground),
            None,
            w!("ShellTabWindowClass"),
            None,
        );

        let shell_windows: IShellWindows =
            match CoCreateInstance(&ShellWindows, None, CLSCTX_SERVER) {
                Ok(w) => w,
                Err(_) => {
                    CoUninitialize();
                    return result;
                }
            };

        let count = shell_windows.Count().unwrap_or(0);
        for i in 0..count {
            let mut variant = VARIANT::default();
            let inner = &mut *variant.Anonymous.Anonymous;
            inner.vt = VT_I4;
            inner.Anonymous.lVal = i;

            let window: IDispatch = match shell_windows.Item(&variant) {
                Ok(w) => w,
                Err(_) => continue,
            };

            let mut service_provider: Option<IServiceProvider> = None;
            if window
                .query(
                    &IServiceProvider::IID,
                    &mut service_provider as *mut _ as *mut _,
                )
                .is_err()
            {
                continue;
            }

            let Some(service_provider) = service_provider else {
                continue;
            };

            let shell_browser =
                match service_provider.QueryService::<IShellBrowser>(&IShellBrowser::IID) {
                    Ok(sb) => sb,
                    Err(_) => continue,
                };

            let Ok(phwnd) = shell_browser.GetWindow() else {
                continue;
            };
            let is_match =
                foreground.0 == phwnd.0 || tab_window.as_ref().map_or(false, |tw| tw.0 == phwnd.0);
            if !is_match {
                continue;
            }

            let shell_view = match shell_browser.QueryActiveShellView() {
                Ok(sv) => sv,
                Err(_) => continue,
            };

            let shell_items: IShellItemArray = match shell_view.GetItemObject(SVGIO_SELECTION) {
                Ok(si) => si,
                Err(_) => continue,
            };

            let item_count = shell_items.GetCount().unwrap_or(0);
            for j in 0..item_count {
                let Ok(item) = shell_items.GetItemAt(j) else {
                    continue;
                };

                if let Ok(attrs) = item.GetAttributes(SFGAO_FILESYSTEM) {
                    if attrs.0 == 0 {
                        continue;
                    }
                }

                if let Ok(name) = item.GetDisplayName(SIGDN_FILESYSPATH) {
                    if let Ok(path) = name.to_string() {
                        result.push(path);
                        continue;
                    }
                }

                if let Ok(name) = item.GetDisplayName(SIGDN_DESKTOPABSOLUTEPARSING) {
                    if let Ok(path) = name.to_string() {
                        result.push(path);
                    }
                }
            }

            break;
        }

        CoUninitialize();
    }

    result
}

/// Tests for the `explorer` module
#[cfg(test)]
mod tests {
    use super::*;

    /// Ensures the function can be called without panicking.
    #[test]
    fn get_explorer_selected_does_not_panic() {
        let _ = get_explorer_selected();
    }
}
