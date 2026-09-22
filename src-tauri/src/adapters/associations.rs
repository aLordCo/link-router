use tauri::AppHandle;

pub fn register_scheme(app: &AppHandle, scheme: &str) -> Result<(), String> {
    #[cfg(target_os = "linux")]
    {
        linux::register(app, scheme)
    }

    #[cfg(windows)]
    {
        use tauri_plugin_deep_link::DeepLinkExt;
        app.deep_link()
            .register(scheme)
            .map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (app, scheme);
        Err("el registro de esquemas no está soportado en esta plataforma".into())
    }
}

pub fn check_scheme(app: &AppHandle, scheme: &str) -> Result<bool, String> {
    #[cfg(target_os = "linux")]
    {
        linux::check(app, scheme)
    }

    #[cfg(windows)]
    {
        use tauri_plugin_deep_link::DeepLinkExt;
        app.deep_link().is_registered(scheme).map_err(|e| e.to_string())
    }

    #[cfg(not(any(target_os = "linux", windows)))]
    {
        let _ = (app, scheme);
        Err("el control de esquemas no está soportado en esta plataforma".into())
    }
}

/// Native Linux scheme registration.
///
/// `xdg-mime default` silently no-ops inside KDE sessions when the `qtpaths`
/// helper is missing, so instead of relying on it we write the desktop entry
/// and update the freedesktop `mimeapps.list` files directly. Those are the
/// files Qt/KDE/GNOME read to resolve default handlers.
#[cfg(target_os = "linux")]
mod linux {
    use std::path::{Path, PathBuf};
    use std::{fs, process::Command};

    use tauri::{AppHandle, Manager};

    fn scheme_key(scheme: &str) -> String {
        format!("x-scheme-handler/{scheme}")
    }

    fn handler_name() -> Result<String, String> {
        let bin = tauri::utils::platform::current_exe()
            .map_err(|e| format!("no se pudo localizar el ejecutable: {e}"))?;
        let name = bin
            .file_name()
            .ok_or_else(|| "el ejecutable no tiene nombre".to_string())?;
        Ok(format!("{}-handler.desktop", name.to_string_lossy()))
    }

    fn data_dir(app: &AppHandle) -> Result<PathBuf, String> {
        app.path()
            .data_dir()
            .map_err(|e| format!("no se pudo resolver el directorio de datos: {e}"))
    }

    fn config_dir(app: &AppHandle) -> Result<PathBuf, String> {
        app.path()
            .config_dir()
            .map_err(|e| format!("no se pudo resolver el directorio de configuración: {e}"))
    }

    fn applications_dir(app: &AppHandle) -> Result<PathBuf, String> {
        Ok(data_dir(app)?.join("applications"))
    }

    fn desktop_path(app: &AppHandle) -> Result<PathBuf, String> {
        Ok(applications_dir(app)?.join(handler_name()?))
    }

    fn desktop_contents(exe: &Path) -> String {
        format!(
            "[Desktop Entry]\n\
             Type=Application\n\
             Name=LinkRouter\n\
             Exec=\"{}\" %u\n\
             TryExec={}\n\
             Terminal=false\n\
             NoDisplay=true\n\
             MimeType=x-scheme-handler/http;x-scheme-handler/https;\n",
            exe.display(),
            exe.display()
        )
    }

    fn ensure_header(lines: &mut Vec<String>, header: &str) {
        if lines.iter().any(|l| l.trim() == header) {
            return;
        }
        if let Some(last) = lines.last() {
            if !last.trim().is_empty() {
                lines.push(String::new());
            }
        }
        lines.push(header.to_string());
    }

    fn set_pair(lines: &mut Vec<String>, header: &str, key: &str, value: &str) {
        ensure_header(lines, header);
        let mut in_sec = false;
        for line in lines.iter_mut() {
            let t = line.trim();
            if t.starts_with('[') {
                in_sec = t == header;
                continue;
            }
            if in_sec {
                if let Some(eq) = t.find('=') {
                    if t[..eq].trim() == key {
                        *line = format!("{key}={value}");
                        return;
                    }
                }
            }
        }
        let at = lines
            .iter()
            .position(|l| l.trim() == header)
            .expect("header ensured");
        lines.insert(at + 1, format!("{key}={value}"));
    }

    fn add_association(lines: &mut Vec<String>, key: &str, desktop: &str) {
        ensure_header(lines, "[Added Associations]");
        let mut in_sec = false;
        for (i, line) in lines.iter().enumerate() {
            let t = line.trim();
            if t.starts_with('[') {
                in_sec = t == "[Added Associations]";
                continue;
            }
            if in_sec {
                if let Some(eq) = t.find('=') {
                    if t[..eq].trim() == key {
                        let names = t[eq + 1..].trim();
                        if !names.split(';').any(|n| n == desktop) {
                            let sep = if names.is_empty() || names.ends_with(';') {
                                ""
                            } else {
                                ";"
                            };
                            lines[i] = format!("{key}={names}{sep}{desktop};");
                        }
                        return;
                    }
                }
            }
        }
        let at = lines
            .iter()
            .position(|l| l.trim() == "[Added Associations]")
            .expect("header ensured");
        lines.insert(at + 1, format!("{key}={desktop};"));
    }

    fn upsert_mimeapps(content: &mut String, scheme: &str, desktop: &str) {
        let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
        let key = scheme_key(scheme);

        set_pair(&mut lines, "[Default Applications]", &key, desktop);
        add_association(&mut lines, &key, desktop);

        *content = format!("{}\n", lines.join("\n"));
    }

    fn is_default(content: &str, scheme: &str, desktop: &str) -> bool {
        let key = scheme_key(scheme);
        let mut in_sec = false;
        for line in content.lines() {
            let t = line.trim();
            if t.starts_with('[') {
                in_sec = t == "[Default Applications]";
                continue;
            }
            if in_sec {
                if let Some(eq) = t.find('=') {
                    if t[..eq].trim() == key {
                        let value = t[eq + 1..].trim();
                        return value.split(';').any(|n| n == desktop);
                    }
                }
            }
        }
        false
    }

    pub(super) fn register(app: &AppHandle, scheme: &str) -> Result<(), String> {
        let exe = tauri::utils::platform::current_exe().map_err(|e| e.to_string())?;
        let name = handler_name()?;
        let dir = applications_dir(app)?;
        fs::create_dir_all(&dir).map_err(|e| format!("no se pudo crear {dir:?}: {e}"))?;

        let file = dir.join(&name);
        fs::write(&file, desktop_contents(&exe))
            .map_err(|e| format!("no se pudo escribir {file:?}: {e}"))?;

        let _ = Command::new("update-desktop-database").arg(&dir).status();

        let config = config_dir(app)?.join("mimeapps.list");
        let mut content = fs::read_to_string(&config).unwrap_or_default();
        upsert_mimeapps(&mut content, scheme, &name);
        fs::write(&config, content).map_err(|e| format!("no se pudo escribir {config:?}: {e}"))?;

        let legacy = dir.join("mimeapps.list");
        if legacy.is_file() {
            let mut content = fs::read_to_string(&legacy).unwrap_or_default();
            upsert_mimeapps(&mut content, scheme, &name);
            let _ = fs::write(&legacy, content);
        }

        Ok(())
    }

    pub(super) fn check(app: &AppHandle, scheme: &str) -> Result<bool, String> {
        if !desktop_path(app)?.is_file() {
            return Ok(false);
        }
        let config = config_dir(app)?.join("mimeapps.list");
        let content = match fs::read_to_string(&config) {
            Ok(c) => c,
            Err(_) => return Ok(false),
        };
        Ok(is_default(&content, scheme, &handler_name()?))
    }

    #[cfg(test)]
    mod tests {
        use super::*;

        #[test]
        fn upsert_creates_default_and_association_from_scratch() {
            let mut content = String::new();
            upsert_mimeapps(&mut content, "https", "linkrouter-handler.desktop");

            assert!(content.contains("[Default Applications]"));
            assert!(content.contains("x-scheme-handler/https=linkrouter-handler.desktop"));
            assert!(content.contains("[Added Associations]"));
            assert!(content.contains("x-scheme-handler/https=linkrouter-handler.desktop;"));
            assert!(is_default(&content, "https", "linkrouter-handler.desktop"));
        }

        #[test]
        fn upsert_replaces_existing_default() {
            let mut content = "[Default Applications]\nx-scheme-handler/https=org.mozilla.firefox.desktop\n"
            .to_string();
            upsert_mimeapps(&mut content, "https", "linkrouter-handler.desktop");

            assert!(content.contains("x-scheme-handler/https=linkrouter-handler.desktop"));
            assert!(!content.contains("org.mozilla.firefox.desktop"));
            assert!(is_default(&content, "https", "linkrouter-handler.desktop"));
            assert!(!is_default(&content, "http", "linkrouter-handler.desktop"));
        }

        #[test]
        fn upsert_keeps_unrelated_entries() {
            let mut content = "[Default Applications]\n\
                                text/html=userapp-Firefox.desktop\n"
                .to_string();
            upsert_mimeapps(&mut content, "https", "linkrouter-handler.desktop");

            assert!(content.contains("text/html=userapp-Firefox.desktop"));
            assert!(content.contains("x-scheme-handler/https=linkrouter-handler.desktop"));
        }

        #[test]
        fn add_association_appends_second_desktop() {
            let mut content = String::new();
            upsert_mimeapps(&mut content, "https", "linkrouter-handler.desktop");

            let mut lines: Vec<String> = content.lines().map(str::to_string).collect();
            add_association(&mut lines, &scheme_key("https"), "org.mozilla.firefox.desktop");

            let mut content = lines.join("\n");
            upsert_mimeapps(&mut content, "https", "linkrouter-handler.desktop");
            assert!(content.contains(
                "x-scheme-handler/https=linkrouter-handler.desktop;org.mozilla.firefox.desktop;"
            ));
        }
    }
}