//! WebView2 settings on Windows (Tech Stack 7, PRD 7.3).
//!
//! The browser's own keys (reload, find, print, zoom) and its right-click menu are
//! turned off, so every key does only what PRD 7.3 lists. Browser zoom is also
//! turned off safely by `zoomHotkeysEnabled: false` in `tauri.conf.json`; it is set
//! here again so all three settings sit in one place.
//!
//! Tauri 2 has no safe setting for the accelerator keys or the context menu, so
//! this module makes the WebView2 COM calls itself. It is the only place in the
//! app where `unsafe` is allowed (plan 01 T1.6).

use tauri::WebviewWindow;
use webview2_com::Microsoft::Web::WebView2::Win32::{
    ICoreWebView2Controller, ICoreWebView2Settings3,
};
use windows_core::Interface;

/// Applies the settings to one window's webview.
///
/// A failure here is not fatal: the app still works, only the browser keys stay on. It is
/// written to the log, so a support file shows which call failed (NFR-OBS-1).
pub fn apply(window: &WebviewWindow) -> tauri::Result<()> {
    window.with_webview(|webview| {
        if let Err(error) = turn_off_browser_features(&webview.controller()) {
            tracing::warn!(
                error = %error,
                "a WebView2 setting could not be turned off; the browser keys may still work"
            );
        }
    })
}

/// Tech Stack 7: `AreBrowserAcceleratorKeysEnabled = false`,
/// `IsZoomControlEnabled = false`, `AreDefaultContextMenusEnabled = false`.
#[allow(unsafe_code)]
fn turn_off_browser_features(controller: &ICoreWebView2Controller) -> windows_core::Result<()> {
    // SAFETY: `controller` is a live WebView2 controller handed to us by Tauri on
    // the thread that owns the webview. Each call is a plain COM property setter
    // with a bool argument; no pointers or lifetimes leave this block.
    unsafe {
        let settings = controller.CoreWebView2()?.Settings()?;
        settings.SetAreDefaultContextMenusEnabled(false)?;
        settings.SetIsZoomControlEnabled(false)?;
        settings
            .cast::<ICoreWebView2Settings3>()?
            .SetAreBrowserAcceleratorKeysEnabled(false)?;
    }
    Ok(())
}
