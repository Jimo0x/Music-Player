//! Téléchargement sur téléphone : le moteur Rust (file, choix du candidat, import) appelle
//! Kotlin par JNI (`YtDlpBridge.kt`, ajouté par scripts/android-patch.mjs), qui fait tourner
//! yt-dlp dans Python (Chaquopy). Compilé partout pour être vérifié, appelé seulement sur
//! Android (`cfg!(target_os = "android")` aux points d'entrée).

use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;
use std::time::Duration;

use jni::objects::{JClass, JObject, JString, JValue};
use jni::{JNIEnv, JavaVM};

use super::JobError;

/// Dossier des données de l'appli (yt-dlp y est rangé), noté au démarrage.
static DATA_DIR: OnceLock<PathBuf> = OnceLock::new();

pub fn set_data_dir(dir: &Path) {
    let _ = DATA_DIR.set(dir.to_path_buf());
}

pub fn data_dir() -> Result<&'static Path, JobError> {
    DATA_DIR.get().map(PathBuf::as_path).ok_or_else(|| JobError::Failed("Appli pas encore prête.".to_owned()))
}

fn failed(error: impl std::fmt::Display) -> JobError {
    JobError::Failed(format!("Pont Android : {error}"))
}

/// Machine Java de l'appli et son activité (fournies par Tauri au démarrage).
fn vm_and_activity() -> Result<(JavaVM, JObject<'static>), JobError> {
    let context = ndk_context::android_context();
    // SAFETY : pointeurs fournis par l'activité Android au lancement, valables toute la vie
    // du processus (ndk-context, initialisé par Tauri).
    let vm = unsafe { JavaVM::from_raw(context.vm().cast()) }.map_err(failed)?;
    let activity = unsafe { JObject::from_raw(context.context().cast()) };
    Ok((vm, activity))
}

/// Classe Kotlin `<paquet>.YtDlpBridge`, cherchée par le chargeur de classes de l'appli (celui
/// du fil natif ne voit pas les classes de l'appli).
fn bridge_class<'a>(env: &mut JNIEnv<'a>, activity: &JObject<'_>) -> jni::errors::Result<JClass<'a>> {
    let loader = env.call_method(activity, "getClassLoader", "()Ljava/lang/ClassLoader;", &[])?.l()?;
    let package = JString::from(env.call_method(activity, "getPackageName", "()Ljava/lang/String;", &[])?.l()?);
    let package: String = env.get_string(&package)?.into();
    let name = env.new_string(format!("{package}.YtDlpBridge"))?;
    let class = env.call_method(&loader, "loadClass", "(Ljava/lang/String;)Ljava/lang/Class;", &[JValue::Object(&name)])?.l()?;
    Ok(JClass::from(class))
}

/// Appelle une méthode statique de YtDlpBridge qui prend et renvoie des chaînes. Kotlin
/// renvoie « ERREUR:… » au lieu de lever une exception.
fn call_text(method: &str, args: &[&str]) -> Result<String, JobError> {
    let (vm, activity) = vm_and_activity()?;
    let mut env = vm.attach_current_thread().map_err(failed)?;
    let result = (|| -> jni::errors::Result<String> {
        let class = bridge_class(&mut env, &activity)?;
        let strings = args.iter().map(|a| env.new_string(a).map(JObject::from)).collect::<jni::errors::Result<Vec<_>>>()?;
        let values: Vec<JValue> = strings.iter().map(JValue::Object).collect();
        let signature = format!("({})Ljava/lang/String;", "Ljava/lang/String;".repeat(args.len()));
        let out = JString::from(env.call_static_method(&class, method, signature, &values)?.l()?);
        let text: String = env.get_string(&out)?.into();
        Ok(text)
    })();
    let text = result.map_err(|error| failed(java_error(&mut env, error)))?;
    match text.strip_prefix("ERREUR:") {
        Some(message) => Err(classify(message)),
        None => Ok(text),
    }
}

/// Texte de l'exception Java en attente (« java.lang.NoSuchMethodError: … »), effacée ensuite ;
/// sinon l'erreur JNI elle-même.
fn java_error(env: &mut JNIEnv<'_>, error: jni::errors::Error) -> String {
    if !matches!(error, jni::errors::Error::JavaException) {
        return error.to_string();
    }
    let Ok(throwable) = env.exception_occurred() else { return error.to_string() };
    let _ = env.exception_clear();
    let text = env
        .call_method(&throwable, "toString", "()Ljava/lang/String;", &[])
        .and_then(|v| v.l())
        .and_then(|o| env.get_string(&JString::from(o)).map(String::from));
    let _ = env.exception_clear();
    text.unwrap_or_else(|_| error.to_string())
}

fn classify(message: &str) -> JobError {
    let lower = message.to_lowercase();
    if lower.contains("arrêt demandé") {
        JobError::Cancelled
    } else if super::common::network_failure(&lower) {
        JobError::Network(message.to_owned())
    } else {
        JobError::Failed(format!("yt-dlp : {message}"))
    }
}

fn call_unit(method: &str) {
    let Ok((vm, activity)) = vm_and_activity() else { return };
    let Ok(mut env) = vm.attach_current_thread() else { return };
    let ok = bridge_class(&mut env, &activity).and_then(|class| env.call_static_method(&class, method, "()V", &[]).map(|_| ()));
    if ok.is_err() {
        let _ = env.exception_clear();
    }
}

fn progress() -> Option<f32> {
    let (vm, activity) = vm_and_activity().ok()?;
    let mut env = vm.attach_current_thread().ok()?;
    let value = bridge_class(&mut env, &activity).and_then(|class| env.call_static_method(&class, "progress", "()D", &[])?.d());
    match value {
        Ok(p) if (0.0..=1.0).contains(&p) => Some(p as f32),
        Ok(_) => None,
        Err(_) => {
            let _ = env.exception_clear();
            None
        }
    }
}

/// Candidats d'une recherche (même JSON que `yt-dlp --flat-playlist --dump-single-json`).
pub fn search(zip: &Path, query: &str) -> Result<String, JobError> {
    call_text("search", &[&zip.to_string_lossy(), query])
}

/// Télécharge le son du lien dans `dir`, en relayant l'avancement et l'arrêt demandé (un fil
/// de surveillance, limité à la durée de l'appel).
pub fn download(zip: &Path, url: &str, dir: &Path, cancel: &AtomicBool) -> Result<PathBuf, JobError> {
    let finished = AtomicBool::new(false);
    let sink = super::common::progress_sink();
    let result = std::thread::scope(|scope| {
        scope.spawn(|| {
            while !finished.load(Ordering::Relaxed) {
                if cancel.load(Ordering::Relaxed) {
                    call_unit("cancel");
                }
                if let (Some(sink), Some(p)) = (sink.as_ref(), progress()) {
                    sink(Some(p));
                }
                std::thread::sleep(Duration::from_millis(400));
            }
        });
        let result = call_text("download", &[&zip.to_string_lossy(), url, &dir.to_string_lossy()]);
        finished.store(true, Ordering::Relaxed);
        result
    });
    let path = PathBuf::from(result?);
    if !path.starts_with(dir) || !path.is_file() {
        return Err(JobError::Failed("yt-dlp n'a pas laissé de fichier audio.".to_owned()));
    }
    Ok(path)
}
