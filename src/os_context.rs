use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use anyhow::{Result, anyhow};

/// Operating system types with primary focus on Linux distributions
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum OSType {
    ArchLinux,
    Ubuntu,
    Debian,
    // Non-Linux support is best-effort in v1.0
    MacOS,      // Limited support
    Windows,    // Limited support
    Unknown,
}

/// Package manager types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum PackageManager {
    Pacman,     // Arch Linux
    Apt,        // Ubuntu/Debian
    // Limited support for others in v1.0
    Yum,        // Best-effort
    Brew,       // Best-effort
    Unknown,
}

/// Shell types
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
    Sh,
    Unknown,
}

/// System paths for different operating systems
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemPaths {
    pub config_dir: PathBuf,
    pub home_dir: PathBuf,
    pub temp_dir: PathBuf,
    pub bin_dirs: Vec<PathBuf>,
    pub os_release_file: Option<PathBuf>,
}

/// OS Context containing all operating system awareness information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OSContext {
    pub os_type: OSType,
    pub package_manager: PackageManager,
    pub shell: Shell,
    pub paths: SystemPaths,
    pub version_info: String,
    pub architecture: String,
}

#[allow(dead_code)]
impl OSContext {
    /// Detect the operating system and create OS context
    /// This is cached for performance after first detection
    pub fn detect() -> Self {
        // Try to load cached context first
        if let Ok(cached) = Self::load_cached() {
            return cached;
        }

        // Perform fresh detection
        let context = Self::perform_detection();
        
        // Cache the result for future use
        let _ = context.save_cache();
        
        context
    }

    /// Perform actual OS detection
    fn perform_detection() -> Self {
        let os_type = Self::detect_os_type();
        let package_manager = Self::detect_package_manager(&os_type);
        let shell = Self::detect_shell();
        let paths = Self::detect_system_paths(&os_type);
        let version_info = Self::get_version_info(&os_type);
        let architecture = Self::get_architecture();

        OSContext {
            os_type,
            package_manager,
            shell,
            paths,
            version_info,
            architecture,
        }
    }

    /// Detect operating system type
    fn detect_os_type() -> OSType {
        // Check for Linux distributions first
        if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
            if os_release.contains("Arch Linux") || os_release.contains("ID=arch") {
                return OSType::ArchLinux;
            }
            if os_release.contains("Ubuntu") || os_release.contains("ID=ubuntu") {
                return OSType::Ubuntu;
            }
            if os_release.contains("Debian") || os_release.contains("ID=debian") {
                return OSType::Debian;
            }
        }

        // Check for Arch-specific files
        if fs::metadata("/etc/arch-release").is_ok() {
            return OSType::ArchLinux;
        }

        // Check for Debian-specific files
        if fs::metadata("/etc/debian_version").is_ok() {
            return OSType::Debian;
        }

        // Check for other OS types using std::env::consts
        match std::env::consts::OS {
            "macos" => OSType::MacOS,
            "windows" => OSType::Windows,
            _ => OSType::Unknown,
        }
    }

    /// Detect package manager based on OS type and available commands
    fn detect_package_manager(os_type: &OSType) -> PackageManager {
        match os_type {
            OSType::ArchLinux => PackageManager::Pacman,
            OSType::Ubuntu | OSType::Debian => PackageManager::Apt,
            OSType::MacOS => {
                // Check if brew is available
                if Command::new("which").arg("brew").output().is_ok() {
                    PackageManager::Brew
                } else {
                    PackageManager::Unknown
                }
            }
            _ => {
                // Try to detect by checking for available commands
                if Command::new("which").arg("pacman").output().is_ok() {
                    PackageManager::Pacman
                } else if Command::new("which").arg("apt").output().is_ok() {
                    PackageManager::Apt
                } else if Command::new("which").arg("yum").output().is_ok() {
                    PackageManager::Yum
                } else if Command::new("which").arg("brew").output().is_ok() {
                    PackageManager::Brew
                } else {
                    PackageManager::Unknown
                }
            }
        }
    }

    /// Detect current shell
    fn detect_shell() -> Shell {
        if let Ok(shell_path) = std::env::var("SHELL") {
            if shell_path.contains("bash") {
                return Shell::Bash;
            }
            if shell_path.contains("zsh") {
                return Shell::Zsh;
            }
            if shell_path.contains("fish") {
                return Shell::Fish;
            }
            if shell_path.ends_with("/sh") {
                return Shell::Sh;
            }
        }
        
        // Default to bash on Unix-like systems
        if cfg!(unix) {
            Shell::Bash
        } else {
            Shell::Unknown
        }
    }

    /// Detect system paths based on OS type
    fn detect_system_paths(os_type: &OSType) -> SystemPaths {
        let home_dir = dirs::home_dir().unwrap_or_else(|| PathBuf::from("/"));
        let config_dir = dirs::config_dir().unwrap_or_else(|| home_dir.join(".config"));
        let temp_dir = std::env::temp_dir();

        let bin_dirs = match os_type {
            OSType::ArchLinux | OSType::Ubuntu | OSType::Debian => {
                vec![
                    PathBuf::from("/usr/bin"),
                    PathBuf::from("/usr/local/bin"),
                    PathBuf::from("/bin"),
                    home_dir.join(".local/bin"),
                ]
            }
            OSType::MacOS => {
                vec![
                    PathBuf::from("/usr/bin"),
                    PathBuf::from("/usr/local/bin"),
                    PathBuf::from("/bin"),
                    PathBuf::from("/opt/homebrew/bin"),
                    home_dir.join(".local/bin"),
                ]
            }
            OSType::Windows => {
                vec![
                    PathBuf::from("C:\\Windows\\System32"),
                    PathBuf::from("C:\\Windows"),
                ]
            }
            OSType::Unknown => {
                vec![
                    PathBuf::from("/usr/bin"),
                    PathBuf::from("/usr/local/bin"),
                    PathBuf::from("/bin"),
                ]
            }
        };

        let os_release_file = match os_type {
            OSType::ArchLinux | OSType::Ubuntu | OSType::Debian => {
                Some(PathBuf::from("/etc/os-release"))
            }
            _ => None,
        };

        SystemPaths {
            config_dir,
            home_dir,
            temp_dir,
            bin_dirs,
            os_release_file,
        }
    }

    /// Get version information for the OS
    fn get_version_info(os_type: &OSType) -> String {
        match os_type {
            OSType::ArchLinux => {
                // Try to get Arch version info
                if let Ok(output) = Command::new("uname").arg("-r").output() {
                    if let Ok(version) = String::from_utf8(output.stdout) {
                        return format!("Arch Linux (kernel {})", version.trim());
                    }
                }
                "Arch Linux".to_string()
            }
            OSType::Ubuntu | OSType::Debian => {
                // Try to read from os-release
                if let Ok(os_release) = fs::read_to_string("/etc/os-release") {
                    for line in os_release.lines() {
                        if line.starts_with("PRETTY_NAME=") {
                            let name = line.trim_start_matches("PRETTY_NAME=")
                                .trim_matches('"');
                            return name.to_string();
                        }
                    }
                }
                format!("{:?}", os_type)
            }
            _ => {
                // Try uname for generic info
                if let Ok(output) = Command::new("uname").arg("-a").output() {
                    if let Ok(info) = String::from_utf8(output.stdout) {
                        return info.trim().to_string();
                    }
                }
                format!("{:?}", os_type)
            }
        }
    }

    /// Get system architecture
    fn get_architecture() -> String {
        std::env::consts::ARCH.to_string()
    }

    /// Get installation command for a package
    pub fn get_install_command(&self, package: &str) -> String {
        match self.package_manager {
            PackageManager::Pacman => format!("sudo pacman -S {}", package),
            PackageManager::Apt => format!("sudo apt install {}", package),
            PackageManager::Yum => format!("sudo yum install {}", package),
            PackageManager::Brew => format!("brew install {}", package),
            PackageManager::Unknown => format!("# Package manager not detected. Please install {} manually", package),
        }
    }

    /// Get system information command
    pub fn get_system_info_command(&self) -> String {
        match self.os_type {
            OSType::ArchLinux => "uname -a && cat /etc/os-release".to_string(),
            OSType::Ubuntu | OSType::Debian => "uname -a && lsb_release -a".to_string(),
            OSType::MacOS => "uname -a && sw_vers".to_string(),
            OSType::Windows => "systeminfo".to_string(),
            OSType::Unknown => "uname -a".to_string(),
        }
    }

    /// Get package search command
    pub fn get_package_search_command(&self, query: &str) -> String {
        match self.package_manager {
            PackageManager::Pacman => format!("pacman -Ss {}", query),
            PackageManager::Apt => format!("apt search {}", query),
            PackageManager::Yum => format!("yum search {}", query),
            PackageManager::Brew => format!("brew search {}", query),
            PackageManager::Unknown => format!("# Package manager not detected. Cannot search for {}", query),
        }
    }

    /// Get package list command
    pub fn get_package_list_command(&self) -> String {
        match self.package_manager {
            PackageManager::Pacman => "pacman -Q".to_string(),
            PackageManager::Apt => "dpkg -l".to_string(),
            PackageManager::Yum => "yum list installed".to_string(),
            PackageManager::Brew => "brew list".to_string(),
            PackageManager::Unknown => "# Package manager not detected".to_string(),
        }
    }

    /// Get update system command
    pub fn get_update_command(&self) -> String {
        match self.package_manager {
            PackageManager::Pacman => "sudo pacman -Syu".to_string(),
            PackageManager::Apt => "sudo apt update && sudo apt upgrade".to_string(),
            PackageManager::Yum => "sudo yum update".to_string(),
            PackageManager::Brew => "brew update && brew upgrade".to_string(),
            PackageManager::Unknown => "# Package manager not detected".to_string(),
        }
    }

    /// Get Arch Linux specific commands (for task 6.3)
    pub fn get_arch_specific_commands(&self) -> Option<ArchCommands> {
        if self.os_type == OSType::ArchLinux {
            Some(ArchCommands::new())
        } else {
            None
        }
    }

    /// Get configuration path for CLIAI
    pub fn get_config_path(&self) -> PathBuf {
        self.paths.config_dir.join("cliai")
    }

    /// Load cached OS context
    fn load_cached() -> Result<Self> {
        let cache_path = Self::get_cache_path()?;
        if cache_path.exists() {
            let content = fs::read_to_string(&cache_path)?;
            let context: OSContext = serde_json::from_str(&content)?;
            Ok(context)
        } else {
            Err(anyhow!("No cached context found"))
        }
    }

    /// Save OS context to cache
    fn save_cache(&self) -> Result<()> {
        let cache_path = Self::get_cache_path()?;
        if let Some(parent) = cache_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let content = serde_json::to_string_pretty(self)?;
        fs::write(cache_path, content)?;
        Ok(())
    }

    /// Get cache file path
    fn get_cache_path() -> Result<PathBuf> {
        let cache_dir = dirs::cache_dir()
            .or_else(|| dirs::config_dir())
            .ok_or_else(|| anyhow!("Could not find cache directory"))?;
        Ok(cache_dir.join("cliai").join("os_context.json"))
    }

    /// Clear cached OS context (useful for testing or when system changes)
    pub fn clear_cache() -> Result<()> {
        let cache_path = Self::get_cache_path()?;
        if cache_path.exists() {
            fs::remove_file(cache_path)?;
        }
        Ok(())
    }

    /// Force refresh of OS context (bypass cache)
    pub fn refresh() -> Self {
        let _ = Self::clear_cache();
        Self::detect()
    }
}

/// Arch Linux specific commands and functionality
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ArchCommands {
    pub aur_helper: Option<String>,
}

#[allow(dead_code)]
impl ArchCommands {
    pub fn new() -> Self {
        let aur_helper = Self::detect_aur_helper();
        Self { aur_helper }
    }

    /// Detect available AUR helper
    fn detect_aur_helper() -> Option<String> {
        let helpers = ["yay", "paru", "trizen", "yaourt"];
        
        for helper in &helpers {
            if Command::new("which").arg(helper).output().is_ok() {
                return Some(helper.to_string());
            }
        }
        None
    }

    /// Get AUR installation command
    pub fn get_aur_install_command(&self, package: &str) -> String {
        if let Some(helper) = &self.aur_helper {
            format!("{} -S {}", helper, package)
        } else {
            format!("# No AUR helper detected. Install {} manually from AUR", package)
        }
    }

    /// Get AUR search command
    pub fn get_aur_search_command(&self, query: &str) -> String {
        if let Some(helper) = &self.aur_helper {
            format!("{} -Ss {}", helper, query)
        } else {
            format!("# No AUR helper detected. Search for {} manually on AUR", query)
        }
    }

    /// Get system service management commands
    pub fn get_service_command(&self, service: &str, action: &str) -> String {
        match action {
            "start" => format!("sudo systemctl start {}", service),
            "stop" => format!("sudo systemctl stop {}", service),
            "restart" => format!("sudo systemctl restart {}", service),
            "enable" => format!("sudo systemctl enable {}", service),
            "disable" => format!("sudo systemctl disable {}", service),
            "status" => format!("systemctl status {}", service),
            _ => format!("systemctl {} {}", action, service),
        }
    }

    /// Get Arch-specific system information commands
    pub fn get_arch_info_commands(&self) -> Vec<String> {
        vec![
            "uname -a".to_string(),
            "cat /etc/os-release".to_string(),
            "pacman -Q | wc -l".to_string(), // Package count
            "df -h".to_string(),
            "free -h".to_string(),
            "systemctl --failed".to_string(), // Failed services
        ]
    }

    /// Get kernel information
    pub fn get_kernel_info_command(&self) -> String {
        "uname -r && pacman -Q linux".to_string()
    }

    /// Get package file listing command
    pub fn get_package_files_command(&self, package: &str) -> String {
        format!("pacman -Ql {}", package)
    }

    /// Get package information command
    pub fn get_package_info_command(&self, package: &str) -> String {
        format!("pacman -Qi {}", package)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_os_context_creation() {
        let context = OSContext::detect();
        
        // Basic validation that detection works
        assert!(!context.version_info.is_empty());
        assert!(!context.architecture.is_empty());
        assert!(!context.paths.home_dir.as_os_str().is_empty());
    }

    #[test]
    fn test_package_manager_commands() {
        let mut context = OSContext::detect();
        
        // Test with different package managers
        context.package_manager = PackageManager::Pacman;
        assert_eq!(context.get_install_command("vim"), "sudo pacman -S vim");
        assert_eq!(context.get_package_search_command("vim"), "pacman -Ss vim");
        assert_eq!(context.get_update_command(), "sudo pacman -Syu");
        
        context.package_manager = PackageManager::Apt;
        assert_eq!(context.get_install_command("vim"), "sudo apt install vim");
        assert_eq!(context.get_package_search_command("vim"), "apt search vim");
        assert_eq!(context.get_update_command(), "sudo apt update && sudo apt upgrade");
    }

    #[test]
    fn test_system_info_commands() {
        let mut context = OSContext::detect();
        
        context.os_type = OSType::ArchLinux;
        assert!(context.get_system_info_command().contains("uname -a"));
        assert!(context.get_system_info_command().contains("/etc/os-release"));
        
        context.os_type = OSType::Ubuntu;
        assert!(context.get_system_info_command().contains("lsb_release"));
    }

    #[test]
    fn test_arch_specific_commands() {
        let mut context = OSContext::detect();
        context.os_type = OSType::ArchLinux;
        
        if let Some(arch_commands) = context.get_arch_specific_commands() {
            assert!(arch_commands.get_service_command("nginx", "start").contains("systemctl start nginx"));
            assert!(arch_commands.get_package_info_command("vim").contains("pacman -Qi vim"));
            assert!(!arch_commands.get_arch_info_commands().is_empty());
        }
    }

    #[test]
    fn test_config_path() {
        let context = OSContext::detect();
        let config_path = context.get_config_path();
        
        assert!(config_path.to_string_lossy().contains("cliai"));
    }

    #[test]
    fn test_os_type_detection_logic() {
        // Test that OS detection doesn't panic
        let os_type = OSContext::detect_os_type();
        
        // Should be one of the valid types
        match os_type {
            OSType::ArchLinux | OSType::Ubuntu | OSType::Debian | 
            OSType::MacOS | OSType::Windows | OSType::Unknown => {
                // Valid OS type detected
            }
        }
    }

    #[test]
    fn test_shell_detection() {
        let shell = OSContext::detect_shell();
        
        // Should be one of the valid shell types
        match shell {
            Shell::Bash | Shell::Zsh | Shell::Fish | Shell::Sh | Shell::Unknown => {
                // Valid shell detected
            }
        }
    }

    #[test]
    fn test_system_paths() {
        let context = OSContext::detect();
        
        // Paths should be valid
        assert!(context.paths.home_dir.exists() || context.paths.home_dir == PathBuf::from("/"));
        assert!(!context.paths.bin_dirs.is_empty());
        assert!(context.paths.temp_dir.exists());
    }

    #[test]
    fn test_arch_commands_creation() {
        let arch_commands = ArchCommands::new();
        
        // Should create without panicking
        assert!(arch_commands.get_service_command("test", "start").contains("systemctl"));
        assert!(!arch_commands.get_arch_info_commands().is_empty());
    }

    #[test]
    fn test_cache_operations() {
        // Test that cache operations don't panic
        let _ = OSContext::clear_cache();
        
        let context = OSContext::detect();
        let _ = context.save_cache();
        
        // Try to load cached version
        let _ = OSContext::load_cached();
    }

    #[test]
    fn test_unknown_package_manager_handling() {
        let mut context = OSContext::detect();
        context.package_manager = PackageManager::Unknown;
        
        let install_cmd = context.get_install_command("vim");
        assert!(install_cmd.contains("not detected"));
        
        let search_cmd = context.get_package_search_command("vim");
        assert!(search_cmd.contains("not detected"));
    }

    #[test]
    fn test_aur_helper_commands() {
        let arch_commands = ArchCommands::new();
        
        let install_cmd = arch_commands.get_aur_install_command("yay-bin");
        // Should either use detected helper or show manual instruction
        assert!(install_cmd.contains("yay-bin") || install_cmd.contains("AUR"));
        
        let search_cmd = arch_commands.get_aur_search_command("browser");
        assert!(search_cmd.contains("browser") || search_cmd.contains("AUR"));
    }
}