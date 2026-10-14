//! Native Android entry point for Apelles.
//!
//! This shell deliberately starts with the shared engine and egui UI. Desktop and web shells
//! remain separate; Android-specific file and lifecycle services are added in later milestones.

#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, clippy::unimplemented, clippy::todo, clippy::unreachable)]

#[cfg(target_os = "android")]
use std::cell::RefCell;
#[cfg(target_os = "android")]
use std::path::{Path, PathBuf};
#[cfg(target_os = "android")]
use std::rc::Rc;
#[cfg(target_os = "android")]
use std::sync::Arc;

#[cfg(target_os = "android")]
use android_activity::AndroidApp;
#[cfg(target_os = "android")]
use jni::EnvUnowned;
#[cfg(target_os = "android")]
use jni::errors::LogErrorAndDefault;
#[cfg(target_os = "android")]
use jni::objects::{JByteArray, JClass, JObject, JString};
#[cfg(target_os = "android")]
use apelles_doc::Document;
#[cfg(target_os = "android")]
use apelles_engine::Session;
#[cfg(target_os = "android")]
use apelles_format::RecoveryStore;
#[cfg(target_os = "android")]
use apelles_ui_egui::{ApellesApp, Recoverable, Services};

#[cfg(target_os = "android")]
fn export_options(settings: &apelles_ui_egui::ExportSettings) -> apelles_io::ExportOptions {
    let mut options = apelles_io::ExportOptions::default();
    options.encode.jpeg_quality = settings.jpeg_quality.unwrap_or(options.encode.jpeg_quality);
    options.encode.webp_lossless = settings.webp_lossless;
    options.encode.webp_quality = settings.webp_quality.unwrap_or(options.encode.webp_quality);
    options.tiff_layers = settings.tiff_layers;
    options.xmp = if settings.xmp_all { apelles_io::XmpEmbed::All } else { apelles_io::XmpEmbed::None };
    options
}

#[cfg(target_os = "android")]
fn load_android_fonts(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else { return };
    for entry in entries.flatten() {
        let path = entry.path();
        let ext = path.extension().and_then(|e| e.to_str()).map(str::to_ascii_lowercase);
        if !matches!(ext.as_deref(), Some("ttf" | "otf" | "ttc")) {
            continue;
        }
        if let Ok(bytes) = std::fs::read(&path) {
            if !bytes.is_empty() {
                let _ = apelles_ui_egui::android_register_font_bytes(bytes);
            }
        }
    }
}

#[cfg(target_os = "android")]
fn android_save_file(
    doc: &Document,
    path: &str,
    settings: &apelles_ui_egui::ExportSettings,
    ctl: &apelles_ui_egui::jobs_ui::SaveCtl,
) -> Result<Vec<String>, String> {
    ctl.progress(0.1, "Encoding");
    let result = apelles_io::export(doc, path, &export_options(settings)).map_err(|e| e.to_string())?;
    if result.bytes.is_empty() {
        return Err(format!("export produced 0 bytes for {path}"));
    }
    ctl.check()?;
    ctl.commit()?;
    ctl.progress(0.8, "Writing");
    use std::io::Write;
    let mut file = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(path).map_err(|e| e.to_string())?;
    file.write_all(&result.bytes).map_err(|e| e.to_string())?;
    file.sync_all().map_err(|e| e.to_string())?;
    let written = file.metadata().map_err(|e| e.to_string())?.len();
    if written != result.bytes.len() as u64 || written == 0 {
        return Err(format!("short or empty save for {path}: expected {}, wrote {written}", result.bytes.len()));
    }
    apelles_ui_egui::android_mark_save_complete(path);
    Ok(result.warnings)
}

#[cfg(target_os = "android")]
type SharedRecovery = Rc<RefCell<RecoveryStore>>;

#[cfg(target_os = "android")]
fn with_recovery<R>(store: &SharedRecovery, f: impl FnOnce(&mut RecoveryStore) -> R) -> Result<R, String> {
    let mut slot = store.try_borrow_mut().map_err(|_| "crash recovery is busy".to_string())?;
    Ok(f(&mut slot))
}

#[cfg(target_os = "android")]
fn recovery_services(dir: PathBuf) -> Services {
    let recover_dir = dir.clone();
    let store = Rc::new(RefCell::new(RecoveryStore::new(dir)));
    let autosave_store = store.clone();
    let results_store = store.clone();
    let discard_store = store.clone();
    let adopt_store = store;
    Services {
        import: Some(Box::new(|name: &str, bytes: &[u8], max_svg_group_depth: usize| {
            apelles_io::import_with_svg_group_depth(name, bytes, max_svg_group_depth)
                .map(|result| (result.document, result.warnings))
                .map_err(|e| e.to_string())
        })),
        export: Some(Box::new(|doc: &Document, path: &str, settings: &apelles_ui_egui::ExportSettings| {
            let result = apelles_io::export(doc, path, &export_options(settings)).map_err(|e| e.to_string())?;
            Ok((result.bytes, result.warnings))
        })),
        save_file: Some(Arc::new(android_save_file)),
        write: Some(Box::new(|path: &str, bytes: &[u8]| {
            if bytes.is_empty() {
                return Err(format!("refusing to write 0 bytes to {path}"));
            }
            use std::io::Write;
            let mut file = std::fs::OpenOptions::new().create(true).truncate(true).write(true).open(path).map_err(|e| e.to_string())?;
            file.write_all(bytes).map_err(|e| e.to_string())?;
            file.sync_all().map_err(|e| e.to_string())?;
            let written = file.metadata().map_err(|e| e.to_string())?.len();
            if written != bytes.len() as u64 {
                return Err(format!("short write to {path}: expected {}, wrote {written}", bytes.len()));
            }
            apelles_ui_egui::android_mark_save_complete(path);
            Ok(())
        })),
        clipboard_set_image: Some(Box::new(|width, height, pixels| {
            apelles_ui_egui::android_set_clipboard_image(width, height, pixels);
            Ok(())
        })),
        clipboard_get_image: Some(Box::new(apelles_ui_egui::android_get_clipboard_image)),
        autosave: Some(Box::new(move |doc: &Arc<Document>, revision: u64, path: Option<&str>| {
            with_recovery(&autosave_store, |s| s.autosave_checked(doc, revision, path.map(str::to_string)))?
        })),
        autosave_results: Some(Box::new(move || with_recovery(&results_store, RecoveryStore::take_completed).unwrap_or_default())),
        discard_autosave: Some(Box::new(move |id: u64| {
            let _ = with_recovery(&discard_store, |s| s.discard(id));
        })),
        recover: Some(Box::new(move || {
            apelles_format::list_recovery(&recover_dir)
                .into_iter()
                .map(|entry| Recoverable {
                    key: entry.info.key.clone(),
                    name: entry.info.document_name.clone(),
                    path: entry.info.original_path.clone(),
                    load: Box::new(move || apelles_format::recover(&entry).map_err(|e| e.to_string())),
                })
                .collect()
        })),
        adopt_autosave: Some(Box::new(move |id: u64, key: &str| {
            let _ = with_recovery(&adopt_store, |s| s.adopt(id, key));
        })),
        ..Default::default()
    }
}

#[cfg(target_os = "android")]
#[allow(unsafe_code)]
#[unsafe(no_mangle)]
pub fn android_main(android_app: AndroidApp) {
    if let Some(path) = android_app.internal_data_path() {
        load_android_fonts(&path.join("Apelles").join("fonts"));
    }
    let services = android_app.internal_data_path().map(|path| recovery_services(path.join("Apelles").join("Recovery"))).unwrap_or_default();
    let options = eframe::NativeOptions {
        android_app: Some(android_app),
        renderer: eframe::Renderer::Wgpu,
        viewport: egui::ViewportBuilder::default().with_title("Apelles"),
        persist_window: false,
        persistence_path: None,
        ..Default::default()
    };

    let result = eframe::run_native(
        "Apelles",
        options,
        Box::new(|cc| {
            ApellesApp::setup_context(&cc.egui_ctx, Default::default());
            let mut app = ApellesApp::new(Session::new(), services);
            app.integrated_titlebar = false;
            app.custom_titlebar = false;
            app.background_jobs = true;
            if let Some(render_state) = cc.wgpu_render_state.clone() {
                app.set_wgpu(render_state);
            }
            Ok(Box::new(app))
        }),
    );

    if let Err(error) = result {
        log::error!("Apelles Android stopped: {error}");
    }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeInsetsChanged(
    _env: *mut core::ffi::c_void,
    _activity: *mut core::ffi::c_void,
    left: i32,
    top: i32,
    right: i32,
    bottom: i32,
) {
    apelles_ui_egui::set_android_insets(left, top, right, bottom);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeShouldHideSystemBars(
    _env: *mut core::ffi::c_void,
    _activity: *mut core::ffi::c_void,
) -> u8 {
    u8::from(apelles_ui_egui::android_should_hide_system_bars())
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeRequestAutosave(
    _env: *mut core::ffi::c_void,
    _activity: *mut core::ffi::c_void,
) {
    apelles_ui_egui::android_request_autosave();
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeTakeFilePickerRequest(_env: EnvUnowned<'_>, _class: JClass<'_>) -> i32 {
    apelles_ui_egui::android_take_file_picker_request()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeTakeSaveSuggestedName<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> JObject<'local> {
    let Some(name) = apelles_ui_egui::android_take_save_suggested_name() else {
        return JObject::null();
    };
    env.with_env(|env| -> jni::errors::Result<JObject<'_>> { Ok(env.new_string(name)?.into()) }).resolve::<LogErrorAndDefault>()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeFontPickerResult(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    path: JString<'_>,
    name: JString<'_>,
) {
    let _ = env.with_env(|env| -> jni::errors::Result<()> {
        apelles_ui_egui::android_queue_font_result(Some(apelles_ui_egui::AndroidFontResult {
            path: path.try_to_string(env)?,
            name: name.try_to_string(env)?,
        }));
        Ok(())
    });
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeFontPickerCancelled(_env: EnvUnowned<'_>, _class: JClass<'_>) {
    apelles_ui_egui::android_queue_font_result(None);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeFilePickerOpenResult(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    path: JString<'_>,
    name: JString<'_>,
) {
    let _ = env.with_env(|env| -> jni::errors::Result<()> {
        let path = path.try_to_string(env)?;
        let name = name.try_to_string(env)?;
        apelles_ui_egui::android_queue_file_picker_result(Some(apelles_ui_egui::AndroidFilePickerResult::Open { path, name }));
        Ok(())
    });
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeFilePickerSaveResult(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    temp_path: JString<'_>,
    uri: JString<'_>,
) {
    let _ = env.with_env(|env| -> jni::errors::Result<()> {
        let temp_path = temp_path.try_to_string(env)?;
        let uri = uri.try_to_string(env)?;
        apelles_ui_egui::android_queue_file_picker_result(Some(apelles_ui_egui::AndroidFilePickerResult::Save { temp_path, uri }));
        Ok(())
    });
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeFilePickerCancelled(_env: EnvUnowned<'_>, _class: JClass<'_>) {
    apelles_ui_egui::android_queue_file_picker_result(None);
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeTakeSaveCompletion<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> JObject<'local> {
    let Some(path) = apelles_ui_egui::android_take_save_completion() else {
        return JObject::null();
    };
    env.with_env(|env| -> jni::errors::Result<JObject<'_>> { Ok(env.new_string(path)?.into()) }).resolve::<LogErrorAndDefault>()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeTakeClipboardSet<'local>(
    mut env: EnvUnowned<'local>,
    _class: JClass<'local>,
) -> JObject<'local> {
    let Some(payload) = apelles_ui_egui::android_take_clipboard_set() else {
        return JObject::null();
    };
    env.with_env(|env| -> jni::errors::Result<JObject<'_>> { Ok(env.byte_array_from_slice(&payload)?.into()) }).resolve::<LogErrorAndDefault>()
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeTakeTextClipboardRequest(
    _env: EnvUnowned<'_>,
    _class: JClass<'_>,
) -> u8 {
    if apelles_ui_egui::android_take_clipboard_text_request() { 1 } else { 0 }
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeTextClipboardResult(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    text: JString<'_>,
) {
    let _ = env.with_env(|env| -> jni::errors::Result<()> {
        let text: String = env.get_string(&text)?.into();
        apelles_ui_egui::android_set_clipboard_text_from_java(text);
        Ok(())
    });
}

#[cfg(target_os = "android")]
#[unsafe(no_mangle)]
pub extern "system" fn Java_ai_storyteller_apelles_nativeandroid_MainActivity_nativeClipboardImageResult(
    mut env: EnvUnowned<'_>,
    _class: JClass<'_>,
    pixels: JByteArray<'_>,
    width: i32,
    height: i32,
) {
    let _ = env.with_env(|env| -> jni::errors::Result<()> {
        let bytes = env.convert_byte_array(&pixels)?;
        if width > 0 && height > 0 && bytes.len() == (width as usize).saturating_mul(height as usize).saturating_mul(4) {
            apelles_ui_egui::android_set_clipboard_image_from_java(width as u32, height as u32, bytes);
        }
        Ok(())
    });
}
