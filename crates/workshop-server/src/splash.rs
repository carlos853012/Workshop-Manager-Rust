use native_windows_gui as nwg;
use std::sync::{Arc, Mutex};
use tokio::sync::oneshot;

#[cfg(all(target_os = "windows", not(test)))]
pub fn show(ready_rx: oneshot::Receiver<Result<String, String>>) {
    if nwg::init().is_err() {
        return;
    }
    nwg::Font::set_global_family("Segoe UI").ok();

    let mut window = Default::default();
    if nwg::Window::builder()
        .size((480, 280))
        .center(true)
        .title("WorkshopManager Server")
        .flags(nwg::WindowFlags::WINDOW | nwg::WindowFlags::VISIBLE)
        .build(&mut window)
        .is_err()
    {
        return;
    }

    let hwnd_ptr = window.handle.hwnd().expect("splash HWND") as *mut std::ffi::c_void;

    let mut title_label = Default::default();
    nwg::Label::builder()
        .text("WorkshopManager Server")
        .size((420, 35))
        .position((30, 30))
        .h_align(nwg::HTextAlign::Center)
        .parent(&window)
        .build(&mut title_label)
        .ok();

    let mut title_font = Default::default();
    nwg::Font::builder()
        .size(22)
        .weight(700)
        .family("Segoe UI")
        .build(&mut title_font)
        .ok();
    title_label.set_font(Some(&title_font));

    let mut version_label = Default::default();
    nwg::Label::builder()
        .text("v0.1.0")
        .size((420, 20))
        .position((30, 70))
        .h_align(nwg::HTextAlign::Center)
        .parent(&window)
        .build(&mut version_label)
        .ok();

    let mut version_font = Default::default();
    nwg::Font::builder()
        .size(12)
        .family("Segoe UI")
        .build(&mut version_font)
        .ok();
    version_label.set_font(Some(&version_font));

    let mut status_label = Default::default();
    nwg::Label::builder()
        .text("Iniciando servidor...")
        .size((420, 25))
        .position((30, 170))
        .h_align(nwg::HTextAlign::Center)
        .parent(&window)
        .build(&mut status_label)
        .ok();

    let mut status_font = Default::default();
    nwg::Font::builder()
        .size(11)
        .family("Segoe UI")
        .build(&mut status_font)
        .ok();
    status_label.set_font(Some(&status_font));

    let mut progress = Default::default();
    nwg::ProgressBar::builder()
        .size((400, 24))
        .position((40, 210))
        .range(0..100)
        .parent(&window)
        .build(&mut progress)
        .ok();

    let mut timer = Default::default();
    nwg::AnimationTimer::builder()
        .interval(std::time::Duration::from_millis(50))
        .parent(&window)
        .build(&mut timer)
        .ok();

    let server_ready = Arc::new(Mutex::new(false));
    let server_error: Arc<Mutex<Option<String>>> = Arc::new(Mutex::new(None));

    let server_ready_clone = server_ready.clone();
    let server_error_clone = server_error.clone();
    std::thread::spawn(move || {
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async move {
            match ready_rx.await {
                Ok(Ok(_addr)) => {
                    tracing::info!("[splash] server ready signal received");
                    if let Ok(mut guard) = server_ready_clone.lock() {
                        *guard = true;
                    }
                }
                Ok(Err(e)) => {
                    tracing::error!("[splash] error: {}", e);
                    if let Ok(mut guard) = server_error_clone.lock() {
                        *guard = Some(e);
                    }
                }
                Err(_) => {
                    tracing::warn!("[splash] channel closed");
                    if let Ok(mut guard) = server_error_clone.lock() {
                        *guard = Some("Channel closed".to_string());
                    }
                }
            }
        });
    });

    let server_ready_clone = server_ready.clone();
    let server_error_clone = server_error.clone();

    let handler = nwg::full_bind_event_handler(&window.handle, move |evt, _evt_data, _handle| {
        use nwg::Event as E;
        if evt == E::OnTimerTick {
            if let Ok(err) = server_error_clone.lock() {
                if let Some(msg) = err.as_ref() {
                    tracing::error!("[splash] error detected: {}", msg);
                    status_label.set_text(&format!("Error: {}", msg));
                    close_splash(hwnd_ptr);
                    return;
                }
            }

            if let Ok(guard) = server_ready_clone.lock() {
                if *guard {
                    progress.set_pos(100);
                    status_label.set_text("Listo!");
                    tracing::info!("[splash] closing splash...");
                    close_splash(hwnd_ptr);
                    return;
                }
            }

            let pos = progress.pos();
            if pos < 90 {
                progress.set_pos(pos + 1);
            }
        }
    });

    timer.start();
    nwg::dispatch_thread_events();
    nwg::unbind_event_handler(&handler);
    tracing::info!("[splash] dispatch returned, splash closed");
}

fn close_splash(hwnd: *mut std::ffi::c_void) {
    use windows_sys::Win32::UI::WindowsAndMessaging::{PostQuitMessage, ShowWindow, SW_HIDE};
    unsafe {
        ShowWindow(hwnd, SW_HIDE);
        PostQuitMessage(0);
    }
}
