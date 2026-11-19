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

use shinden_to_anilist_grpc::{
    cloudflare::SHINDEN_ORIGIN,
    error::ServiceError,
    pb::ErrorKind,
};
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

use crate::{
    dto::{
        CommandErrorDto,
        ShindenCloudflareClearanceDto,
        ShindenCloudflareVerificationOptionsDto,
        command_error,
    },
    state::PRODUCT_NAME,
};

const SHINDEN_VERIFICATION_LABEL: &str = "shinden-cloudflare-verification";
const SHINDEN_HOMEPAGE_URL: &str = "https://shinden.pl/";
const CLOUDFLARE_CLEARANCE_COOKIE: &str = "cf_clearance";
const AUTO_CLOSE_TEST_COOKIE: &str = "sess_shinden";
const CLEARANCE_POLLING_INTERVAL: Duration = Duration::from_millis(750);
const BLOCKED_SHINDEN_AD_HOST: &str = "reklama.shinden.eu";

type ClearanceResult = Result<ShindenCloudflareClearanceDto, ServiceError>;
type ClearanceSender = Sender<ClearanceResult>;

#[tauri::command]
pub(crate) async fn open_shinden_cloudflare_verification(
    app: AppHandle,
    options: Option<ShindenCloudflareVerificationOptionsDto>,
) -> Result<ShindenCloudflareClearanceDto, CommandErrorDto> {
    open_verification(app, options).await.map_err(command_error)
}

async fn open_verification(
    app: AppHandle,
    options: Option<ShindenCloudflareVerificationOptionsDto>,
) -> ClearanceResult {
    if let Some(window) = app.get_webview_window(SHINDEN_VERIFICATION_LABEL) {
        let _ = window.set_focus();
        return Err(ServiceError::new(
            ErrorKind::VerificationWindowAlreadyOpen,
            "Shinden verification window already open",
        ));
    }

    let (tx, rx) = std::sync::mpsc::channel::<ClearanceResult>();
    let tx = Arc::new(Mutex::new(Some(tx)));
    let capture_started = Arc::new(AtomicBool::new(false));
    let cookie_name = verification_cookie_name(options.as_ref()).to_string();
    let shinden_url = Url::parse(SHINDEN_HOMEPAGE_URL)
        .map_err(|error| verification_failure("parse Shinden homepage URL", error))?;
    let data_directory = app
        .path()
        .data_dir()
        .map_err(|error| verification_failure("resolve verification data directory", error))?
        .join(PRODUCT_NAME)
        .join("shinden-cloudflare-webview");
    fs::create_dir_all(&data_directory)
        .map_err(|error| verification_failure("create verification data directory", error))?;

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
            false
        } else {
            true
        }
    })
    .on_new_window(|_url, _features| NewWindowResponse::Deny)
    .build()
    .map_err(|error| verification_failure("build verification window", error))?;

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
        rx.recv().map_err(|error| {
            ServiceError::caused_by(
                ErrorKind::VerificationWindowFailed,
                "receive verification window result",
                error,
            )
        })?
    })
    .await
    .map_err(|error| verification_failure("join verification result task", error))?
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

async fn capture_tauri_clearance(window: WebviewWindow, cookie_name: String) -> ClearanceResult {
    let cookie_window = window.clone();
    let cookie = tauri::async_runtime::spawn_blocking(move || {
        capture_tauri_clearance_cookie(&cookie_window, &cookie_name)
    })
    .await
    .map_err(|error| verification_failure("join verification cookie task", error))??;
    let user_agent = webview_user_agent(&window).await?;

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
) -> Result<Cookie<'static>, ServiceError> {
    let url = Url::parse(SHINDEN_ORIGIN)
        .map_err(|error| verification_failure("parse Shinden origin URL", error))?;
    let homepage_url = Url::parse(SHINDEN_HOMEPAGE_URL)
        .map_err(|error| verification_failure("parse Shinden homepage URL", error))?;
    let mut cookies = window
        .cookies_for_url(homepage_url)
        .map_err(|error| verification_failure("read Shinden verification cookies", error))?;
    cookies.extend(
        window
            .cookies_for_url(url)
            .map_err(|error| verification_failure("read Shinden verification cookies", error))?,
    );

    best_cookie(cookies, cookie_name).ok_or_else(|| {
        ServiceError::new(
            ErrorKind::Cancelled,
            format!("verification closed before receiving {cookie_name}"),
        )
    })
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

async fn webview_user_agent(window: &WebviewWindow) -> Result<String, ServiceError> {
    let (tx, rx) = std::sync::mpsc::channel::<String>();
    window
        .eval_with_callback("navigator.userAgent", move |value| {
            let _ = tx.send(value);
        })
        .map_err(|error| verification_failure("request verification browser user agent", error))?;

    let value = tauri::async_runtime::spawn_blocking(move || {
        rx.recv_timeout(Duration::from_secs(5))
            .map_err(|error| verification_failure("receive verification browser user agent", error))
    })
    .await
    .map_err(|error| verification_failure("join verification user agent task", error))??;

    serde_json::from_str::<String>(&value).or(Ok(value))
}

fn current_timestamp_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|duration| u64::try_from(duration.as_millis()).unwrap_or(u64::MAX))
        .unwrap_or_default()
}

fn verification_failure(
    context: &str,
    error: impl std::error::Error + Send + Sync + 'static,
) -> ServiceError {
    ServiceError::caused_by(ErrorKind::VerificationWindowFailed, context, error)
}
