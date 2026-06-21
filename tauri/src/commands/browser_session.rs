use std::{
    collections::HashMap,
    fs,
    sync::{
        Arc,
        atomic::{
            AtomicBool,
            Ordering,
        },
    },
};

use shinden_to_anilist_grpc::cloudflare::SHINDEN_ORIGIN;
use tauri::{
    AppHandle,
    Manager,
    Url,
    WebviewUrl,
    WindowEvent,
    webview::{
        Cookie,
        WebviewWindow,
        WebviewWindowBuilder,
    },
};
use tracing::{
    info,
    warn,
};

use crate::{
    dto::ShindenCloudflareClearanceDto,
    state::PRODUCT_NAME,
};

const SHINDEN_VERIFICATION_LABEL: &str = "shinden-cloudflare-verification";
const CF_CLEARANCE_COOKIE: &str = "cf_clearance";

#[tauri::command]
pub(crate) async fn open_shinden_cloudflare_verification(
    app: AppHandle,
) -> Result<ShindenCloudflareClearanceDto, String> {
    if let Some(window) = app.get_webview_window(SHINDEN_VERIFICATION_LABEL) {
        let _ = window.set_focus();
        return Err("Okno weryfikacji Shinden jest już otwarte.".to_string());
    }

    let (tx, rx) = std::sync::mpsc::channel::<Result<ShindenCloudflareClearanceDto, String>>();
    let tx = Arc::new(std::sync::Mutex::new(Some(tx)));
    let capture_started = Arc::new(AtomicBool::new(false));
    let shinden_url = Url::parse(SHINDEN_ORIGIN).map_err(|err| err.to_string())?;
    let data_directory = app
        .path()
        .data_dir()
        .map_err(|err| err.to_string())?
        .join(PRODUCT_NAME)
        .join("shinden-cloudflare-webview");
    fs::create_dir_all(&data_directory).map_err(|err| err.to_string())?;

    info!(path = %data_directory.display(), "opening Shinden Cloudflare verification webview");
    let window = WebviewWindowBuilder::new(
        &app,
        SHINDEN_VERIFICATION_LABEL,
        WebviewUrl::External(shinden_url),
    )
    .title("Shinden - weryfikacja Cloudflare")
    .inner_size(960.0, 760.0)
    .min_inner_size(720.0, 600.0)
    .data_directory(data_directory)
    .build()
    .map_err(|err| err.to_string())?;

    let capture_window = window.clone();
    window.on_window_event(move |event| {
        let WindowEvent::CloseRequested { api, .. } = event else {
            return;
        };

        if capture_started.swap(true, Ordering::SeqCst) {
            return;
        }

        api.prevent_close();
        info!("Shinden Cloudflare verification window closed by user; capturing cookies");
        let tx = Arc::clone(&tx);
        let window = capture_window.clone();
        tauri::async_runtime::spawn(async move {
            let result = capture_tauri_clearance(window.clone()).await;
            if let Some(tx) = tx.lock().expect("Shinden Cloudflare sender lock poisoned").take() {
                let _ = tx.send(result);
            }
            let _ = window.close();
        });
    });

    tauri::async_runtime::spawn_blocking(move || {
        rx.recv()
            .map_err(|_| "Okno weryfikacji Shinden zostało zamknięte bez wyniku.".to_string())?
    })
    .await
    .map_err(|err| err.to_string())?
}

async fn capture_tauri_clearance(window: WebviewWindow) -> Result<ShindenCloudflareClearanceDto, String> {
    let user_agent = webview_user_agent(&window).await.unwrap_or_else(|error| {
        warn!(error = %error, "failed to read Shinden verification user agent");
        String::new()
    });

    tauri::async_runtime::spawn_blocking(move || capture_tauri_cookies(&window, user_agent))
        .await
        .map_err(|err| err.to_string())?
}

fn capture_tauri_cookies(
    window: &WebviewWindow,
    user_agent: String,
) -> Result<ShindenCloudflareClearanceDto, String> {
    let url = Url::parse(SHINDEN_ORIGIN).map_err(|err| err.to_string())?;
    let cookies = window
        .cookies_for_url(url)
        .map_err(|err| format!("Nie udało się odczytać ciasteczek Shinden: {err}"))?;
    let cookie = best_clearance_cookie(cookies)
        .ok_or_else(|| "Nie udało się odczytać ciasteczka Cloudflare z okna weryfikacji.".to_string())?;

    info!(
        user_agent_len = user_agent.len(),
        cookie_len = cookie.value().len(),
        domain = ?cookie.domain(),
        path = ?cookie.path(),
        "captured Shinden Cloudflare clearance from Tauri webview"
    );

    Ok(ShindenCloudflareClearanceDto {
        user_agent,
        cf_clearance: cookie.value().to_string(),
        domain: cookie.domain().unwrap_or("lista.shinden.pl").to_string(),
        path: cookie.path().unwrap_or("/").to_string(),
        expires_unix_seconds: cookie
            .expires_datetime()
            .map(|expires| expires.unix_timestamp() as f64),
        captured_at_ms: current_timestamp_ms(),
    })
}

fn best_clearance_cookie(cookies: Vec<Cookie<'static>>) -> Option<Cookie<'static>> {
    let mut by_key = HashMap::new();
    for cookie in cookies {
        by_key.insert(
            format!(
                "{}\n{}\n{}",
                cookie.domain().unwrap_or("lista.shinden.pl"),
                cookie.path().unwrap_or("/"),
                cookie.name()
            ),
            cookie,
        );
    }

    by_key
        .into_values()
        .filter(|cookie| cookie.name().eq_ignore_ascii_case(CF_CLEARANCE_COOKIE))
        .max_by_key(|cookie| {
            (
                shinden_domain_score(cookie.domain().unwrap_or("lista.shinden.pl")),
                usize::from(cookie.path().unwrap_or("/") == "/"),
                cookie.value().len(),
            )
        })
}

fn shinden_domain_score(domain: &str) -> usize {
    let domain = domain.trim().trim_start_matches('.').to_ascii_lowercase();
    match domain.as_str() {
        "lista.shinden.pl" => 3,
        "shinden.pl" => 2,
        domain if domain.ends_with(".shinden.pl") => 1,
        _ => 0,
    }
}

async fn webview_user_agent(window: &WebviewWindow) -> Result<String, String> {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    window
        .eval_with_callback("navigator.userAgent", move |value| {
            let _ = tx.send(value);
        })
        .map_err(|err| format!("Nie udało się odczytać user agenta Shindena: {err}"))?;

    let value = tauri::async_runtime::spawn_blocking(move || {
        rx.recv()
            .map_err(|_| "Nie udało się odebrać user agenta Shindena.".to_string())
    })
    .await
    .map_err(|err| err.to_string())??;

    serde_json::from_str::<String>(&value).or(Ok(value))
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}
