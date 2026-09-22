use url::Url;

use crate::core::domain::BrowserProfile;
use crate::ports::UrlLauncherPort;

pub struct ProcessLauncher;

impl ProcessLauncher {
    pub fn new() -> Self {
        Self
    }
}

impl Default for ProcessLauncher {
    fn default() -> Self {
        Self::new()
    }
}

impl UrlLauncherPort for ProcessLauncher {
    fn launch(
        &self,
        profile: &BrowserProfile,
        url: &Url,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut command = std::process::Command::new(&profile.executable);
        command.args(&profile.args).arg(url.as_str());

        #[cfg(unix)]
        {
            use std::os::unix::process::CommandExt;
            command.process_group(0);
        }

        #[cfg(target_os = "windows")]
        {
            use std::os::windows::process::CommandExt;
            const CREATE_NO_WINDOW: u32 = 0x0800_0000;
            const DETACHED_PROCESS: u32 = 0x0000_0008;
            command.creation_flags(CREATE_NO_WINDOW | DETACHED_PROCESS);
        }

        command
            .spawn()
            .map(|_| ())
            .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_missing_binary_instead_of_silently_failing() {
        let launcher = ProcessLauncher::new();
        let profile = BrowserProfile {
            id: "t".into(),
            name: "t".into(),
            browser_name: "test".into(),
            executable: "/nonexistent/linkrouter-target".into(),
            profile_dir: None,
            icon: None,
            args: vec![],
        };
        let url = Url::parse("https://example.com").unwrap();
        assert!(launcher.launch(&profile, &url).is_err());
    }
}
