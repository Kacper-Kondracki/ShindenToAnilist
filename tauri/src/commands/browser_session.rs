use std::{
    collections::HashMap,
    fs,
    sync::{
        Arc,
        Mutex,
        atomic::{
            AtomicBool,
            Ordering,
        },
        mpsc::Sender,
    },
    time::Duration,
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
        NewWindowResponse,
        WebviewWindow,
        WebviewWindowBuilder,
    },
};
use tracing::{
    info,
    warn,
};

use crate::{
    dto::{
        ShindenCloudflareClearanceDto,
        ShindenCloudflareVerificationOptionsDto,
    },
    state::PRODUCT_NAME,
};

const SHINDEN_VERIFICATION_LABEL: &str = "shinden-cloudflare-verification";
const SHINDEN_HOMEPAGE_URL: &str = "https://shinden.pl/";
const CLOUDFLARE_CLEARANCE_COOKIE: &str = "cf_clearance";
const AUTO_CLOSE_TEST_COOKIE: &str = "sess_shinden";
const CLEARANCE_POLLING_INTERVAL: Duration = Duration::from_millis(750);
const BLOCKED_SHINDEN_AD_HOST: &str = "reklama.shinden.eu";

type ClearanceResult = Result<ShindenCloudflareClearanceDto, String>;
type ClearanceSender = Sender<ClearanceResult>;

#[tauri::command]
pub(crate) async fn open_shinden_cloudflare_verification(
    app: AppHandle,
    options: Option<ShindenCloudflareVerificationOptionsDto>,
) -> Result<ShindenCloudflareClearanceDto, String> {
    if let Some(window) = app.get_webview_window(SHINDEN_VERIFICATION_LABEL) {
        let _ = window.set_focus();
        return Err("Okno weryfikacji Shinden jest już otwarte.".to_string());
    }

    let (tx, rx) = std::sync::mpsc::channel::<ClearanceResult>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let capture_started = Arc::new(AtomicBool::new(false));
    let cookie_name = verification_cookie_name(options.as_ref()).to_string();
    let shinden_url = Url::parse(SHINDEN_HOMEPAGE_URL).map_err(|err| err.to_string())?;
    let data_directory = app
        .path()
        .data_dir()
        .map_err(|err| err.to_string())?
        .join(PRODUCT_NAME)
        .join("shinden-cloudflare-webview");
    fs::create_dir_all(&data_directory).map_err(|err| err.to_string())?;

    info!(path = %data_directory.display(), cookie_name, "opening Shinden Cloudflare verification webview");
    let window = WebviewWindowBuilder::new(
        &app,
        SHINDEN_VERIFICATION_LABEL,
        WebviewUrl::External(shinden_url),
    )
    .title("Shinden - weryfikacja Cloudflare")
    .inner_size(960.0, 760.0)
    .min_inner_size(720.0, 600.0)
    .data_directory(data_directory)
    .on_navigation(|url| {
        if is_blocked_shinden_ad_url(url) {
            info!(url = %url, "blocked Shinden ad navigation");
            false
        } else {
            true
        }
    })
    .on_new_window(|url, _features| {
        info!(url = %url, "blocked popup from Shinden verification webview");
        NewWindowResponse::Deny
    })
    .build()
    .map_err(|err| err.to_string())?;

    poll_for_tauri_clearance(
        window.clone(),
        Arc::clone(&tx),
        Arc::clone(&capture_started),
        cookie_name.clone(),
    );

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
        let cookie_name = cookie_name.clone();
        tauri::async_runtime::spawn(async move {
            let result = capture_tauri_clearance(window.clone(), cookie_name).await;
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

fn poll_for_tauri_clearance(
    window: WebviewWindow,
    tx: Arc<Mutex<Option<ClearanceSender>>>,
    capture_started: Arc<AtomicBool>,
    cookie_name: String,
) {
    tauri::async_runtime::spawn(async move {
        loop {
            sleep_clearance_polling_interval().await;

            if capture_started.load(Ordering::SeqCst) {
                break;
            }

            if !has_tauri_clearance_cookie(window.clone(), cookie_name.clone()).await {
                continue;
            }

            let result = capture_tauri_clearance(window.clone(), cookie_name.clone()).await;
            if result.is_err() || capture_started.swap(true, Ordering::SeqCst) {
                continue;
            }

            info!(
                cookie_name,
                "Shinden Cloudflare verification completed automatically; closing window"
            );
            if let Some(tx) = tx.lock().expect("Shinden Cloudflare sender lock poisoned").take() {
                let _ = tx.send(result);
            }
            let _ = window.close();
            break;
        }
    });
}

async fn sleep_clearance_polling_interval() {
    let _ = tauri::async_runtime::spawn_blocking(|| {
        std::thread::sleep(CLEARANCE_POLLING_INTERVAL);
    })
    .await;
}

async fn has_tauri_clearance_cookie(window: WebviewWindow, cookie_name: String) -> bool {
    tauri::async_runtime::spawn_blocking(move || {
        capture_tauri_clearance_cookie(&window, &cookie_name).is_ok()
    })
    .await
    .unwrap_or(false)
}

async fn capture_tauri_clearance(
    window: WebviewWindow,
    cookie_name: String,
) -> Result<ShindenCloudflareClearanceDto, String> {
    let user_agent = webview_user_agent(&window).await.unwrap_or_else(|error| {
        warn!(error = %error, "failed to read Shinden verification user agent");
        String::new()
    });

    tauri::async_runtime::spawn_blocking(move || capture_tauri_cookies(&window, user_agent, &cookie_name))
        .await
        .map_err(|err| err.to_string())?
}

fn capture_tauri_cookies(
    window: &WebviewWindow,
    user_agent: String,
    cookie_name: &str,
) -> Result<ShindenCloudflareClearanceDto, String> {
    let cookie = capture_tauri_clearance_cookie(window, cookie_name)?;

    info!(
        cookie_name,
        user_agent_len = user_agent.len(),
        cookie_len = cookie.value().len(),
        domain = ?cookie.domain(),
        path = ?cookie.path(),
        "captured Shinden Cloudflare clearance from Tauri webview"
    );
    if cookie_name == AUTO_CLOSE_TEST_COOKIE {
        info!(
            cookie_name,
            cookie_value = cookie.value(),
            "captured Shinden autoclose test cookie value"
        );
    }

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

fn capture_tauri_clearance_cookie(
    window: &WebviewWindow,
    cookie_name: &str,
) -> Result<Cookie<'static>, String> {
    let url = Url::parse(SHINDEN_ORIGIN).map_err(|err| err.to_string())?;
    let homepage_url = Url::parse(SHINDEN_HOMEPAGE_URL).map_err(|err| err.to_string())?;
    let mut cookies = window
        .cookies_for_url(homepage_url)
        .map_err(|err| format!("Nie udało się odczytać ciasteczek Shinden: {err}"))?;
    cookies.extend(
        window
            .cookies_for_url(url)
            .map_err(|err| format!("Nie udało się odczytać ciasteczek Shinden: {err}"))?,
    );

    best_cookie(cookies, cookie_name)
        .ok_or_else(|| format!("Nie udało się odczytać ciasteczka {cookie_name} z okna weryfikacji."))
}

fn best_cookie(cookies: Vec<Cookie<'static>>, cookie_name: &str) -> Option<Cookie<'static>> {
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
        .filter(|cookie| cookie.name().eq_ignore_ascii_case(cookie_name))
        .max_by_key(|cookie| {
            (
                shinden_domain_score(cookie.domain().unwrap_or("lista.shinden.pl")),
                usize::from(cookie.path().unwrap_or("/") == "/"),
                cookie.value().len(),
            )
        })
}

fn verification_cookie_name(options: Option<&ShindenCloudflareVerificationOptionsDto>) -> &'static str {
    if options.and_then(|options| options.mode.as_deref()) == Some("autocloseTest") {
        AUTO_CLOSE_TEST_COOKIE
    } else {
        CLOUDFLARE_CLEARANCE_COOKIE
    }
}

fn is_blocked_shinden_ad_url(url: &Url) -> bool {
    url.host_str()
        .map(|host| {
            let host = host.trim_end_matches('.').to_ascii_lowercase();
            host == BLOCKED_SHINDEN_AD_HOST || host.ends_with(&format!(".{BLOCKED_SHINDEN_AD_HOST}"))
        })
        .unwrap_or(false)
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
