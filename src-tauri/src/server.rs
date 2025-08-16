use warp::Filter;
use crate::commands::Pankti;
use tera::{Tera, Context};
use tauri::{AppHandle, Manager};
use std::sync::Arc;
use std::path::PathBuf;
use tokio::sync::Mutex;
use std::collections::HashMap;
use warp::http;
use crate::settings::{load_settings, save_settings, UserSettings};

pub async fn start_web_server(app_handle: AppHandle) {
    let tera = Tera::new("templates/**/*")
        .expect("Could not initialize Tera templates.");
    let tera = Arc::new(tera);

    let app_handle_clone = app_handle.clone();
    let app_handle_clone_for_save = app_handle.clone();
    let tera_clone = tera.clone();

    let static_files = warp::path("static").and(warp::fs::dir("static"));

    // Render settings page
    let settings_page = warp::path("settings").and_then(move || {
        let tera = tera.clone();
        async move {
            let mut context = Context::new();
            context.insert("title", "Settings");
            let html = tera.render("settings.html", &context).unwrap();
            Ok::<_, std::convert::Infallible>(warp::reply::html(html))
        }
    });

    // Save settings from form
    let save_settings_route = warp::path("save_settings")
        .and(warp::post())
        .and(warp::body::form())
        .map(move |form: HashMap<String, String>| {
            let settings = UserSettings {
                theme: form.get("theme").cloned().unwrap_or_default(),
                font: form.get("font").cloned().unwrap_or_default(),
            };
            let path = app_handle_clone_for_save.state::<PathBuf>();
            save_settings(&path, &settings);
            warp::redirect::see_other("/settings".parse::<http::Uri>().unwrap())
        });

    let custom_page = warp::path!("custom_page").and_then(move || {
        let app_handle = app_handle_clone.clone();
        let tera = tera_clone.clone();

        async move {
            let html = render_custom_page_with_settings(app_handle, tera).await;
            Ok::<_, std::convert::Infallible>(warp::reply::html(html))
        }
    });

    let routes = custom_page
        .or(settings_page)
        .or(save_settings_route)
        .or(static_files);

    warp::serve(routes)
        .run(([127, 0, 0, 1], 8080))
        .await;
}

async fn render_custom_page_with_settings(
    app_handle: AppHandle,
    tera: Arc<Tera>,
) -> String {
    let state = app_handle.state::<tokio::sync::Mutex<Pankti>>();
    let pankti = {
        let locked = state.lock().await;
        locked.clone()
    };

    let path = app_handle.state::<PathBuf>();
    let settings = load_settings(&path);

    let mut context = Context::new();
    context.insert("title", "Gurbani");
    context.insert("gurmukhi", &pankti.gurmukhi);
    context.insert("punjabi", &pankti.punjabi);
    context.insert("english", &pankti.english);
    context.insert("theme", &settings.theme);
    context.insert("font", &settings.font);

    tera.render("message.html", &context)
        .unwrap_or_else(|e| {
            eprintln!("Template error: {}", e);
            "Error rendering page.".to_string()
        })
}

// 🔄 Now renders the page *and* accesses the shared state inside
async fn render_custom_page(app_handle: AppHandle, tera: Arc<Tera>) -> String {
    // Access and lock the shared state
    let state = app_handle.state::<Mutex<Pankti>>();
    let locked_state = state.lock().await;

    // Clone the state contents
    let pankti = Pankti {
        gurmukhi: locked_state.gurmukhi.clone(),
        punjabi: locked_state.punjabi.clone(),
        english: locked_state.english.clone(),
    };

    // Build context for the template
    let mut context = Context::new();
    context.insert("title", "Gurbani");
    context.insert("gurmukhi", &pankti.gurmukhi);
    context.insert("punjabi", &pankti.punjabi);
    context.insert("english", &pankti.english);

    tera.render("message.html", &context)
        .unwrap_or_else(|e| {
            eprintln!("Template error: {}", e);
            "Error rendering page.".to_string()
        })
}
