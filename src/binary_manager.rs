use crate::logger::Logger;
use fs_extra::dir;
use std::sync::OnceLock;
use tempfile::TempDir;
use zed_extension_api::{self as zed, DownloadedFileType, GithubReleaseOptions};

/// GitHub release version information
#[derive(Debug, Clone)]
pub struct AdapterVersion {
    /// Release tag name (version)
    pub tag_name: String,
    /// Download URL for the release asset
    pub download_url: String,
}

/// NetCoreDbg binary manager - handles downloading, extracting, and locating the netcoredbg binary
pub struct BinaryManager {
    /// Cached path to the netcoredbg binary - set once and reused
    cached_binary_path: OnceLock<String>,
    /// Logger instance for debug logging
    logger: Logger,
}

impl Default for BinaryManager {
    fn default() -> Self {
        Self::new()
    }
}

impl BinaryManager {
    const GITHUB_OWNER: &str = "qwadrox";
    const GITHUB_REPO: &str = "netcoredbg";

    pub fn new() -> Self {
        Self {
            cached_binary_path: OnceLock::new(),
            logger: Logger::new(),
        }
    }

    fn get_executable_name() -> &'static str {
        if zed::current_platform().0 == zed::Os::Windows {
            "netcoredbg.exe"
        } else {
            "netcoredbg"
        }
    }

    /// Determines the appropriate asset name for the current platform
    /// Supported assets:
    /// - netcoredbg-linux-amd64.tar.gz
    /// - netcoredbg-linux-arm64.tar.gz  
    /// - netcoredbg-osx-amd64.tar.gz
    /// - netcoredbg-osx-arm64.tar.gz
    /// - netcoredbg-win64.zip
    fn get_platform_asset_name() -> Result<String, String> {
        let (platform, arch) = zed::current_platform();

        let (platform_arch, extension) = match (platform, arch) {
            (zed::Os::Linux, zed::Architecture::X8664) => ("linux-amd64", ".tar.gz"),
            (zed::Os::Linux, zed::Architecture::Aarch64) => ("linux-arm64", ".tar.gz"),
            (zed::Os::Mac, zed::Architecture::X8664) => ("osx-amd64", ".tar.gz"),
            (zed::Os::Mac, zed::Architecture::Aarch64) => ("osx-arm64", ".tar.gz"),
            (zed::Os::Windows, zed::Architecture::X8664) => ("win64", ".zip"),
            (zed::Os::Windows, zed::Architecture::Aarch64) => {
                // Windows ARM64 is not officially supported by netcoredbg,
                // but we can try the x64 version as a fallback
                ("win64", ".zip")
            }
            (_, zed::Architecture::X86) => {
                return Err("Unsupported architecture: x86 (32-bit). NetCoreDbg only supports 64-bit architectures (amd64/arm64).".to_string());
            }
        };

        Ok(format!("netcoredbg-{}{}", platform_arch, extension))
    }

    /// Fetches the latest release information from GitHub
    fn fetch_latest_release(&self) -> Result<AdapterVersion, String> {
        let release = zed::latest_github_release(
            &format!("{}/{}", Self::GITHUB_OWNER, Self::GITHUB_REPO),
            GithubReleaseOptions {
                require_assets: true,
                pre_release: false,
            },
        )
        .map_err(|e| format!("Failed to fetch latest release: {}", e))?;

        let asset_name = Self::get_platform_asset_name()?;

        let asset = release
            .assets
            .iter()
            .find(|asset| asset.name == asset_name)
            .ok_or_else(|| {
                format!(
                    "No compatible asset found for platform. Looking for: '{}'. Available assets: [{}]",
                    asset_name,
                    release.assets.iter().map(|a| a.name.as_str()).collect::<Vec<_>>().join(", ")
                )
            })?;

        Ok(AdapterVersion {
            tag_name: release.version,
            download_url: asset.download_url.clone(),
        })
    }

    /// Creates a secure temporary directory for extraction
    fn create_secure_temp_dir(&self, version: &str) -> Result<TempDir, String> {
        tempfile::Builder::new()
            .prefix(&format!("netcoredbg_v{}_", version))
            .tempdir()
            .map_err(|e| format!("Failed to create secure temp directory: {}", e))
    }

    /// Downloads and extracts the netcoredbg binary, returning the path to the executable
    fn download_and_extract_binary(&self) -> Result<String, String> {
        let version = self.fetch_latest_release()?;
        let asset_name = Self::get_platform_asset_name()?;

        let file_type = if asset_name.ends_with(".zip") {
            DownloadedFileType::Zip
        } else if asset_name.ends_with(".tar.gz") {
            DownloadedFileType::GzipTar
        } else {
            return Err(format!("Unsupported file type for asset: {}", asset_name));
        };

        // Version-specific directory in current working directory
        let version_dir = std::path::PathBuf::from(format!("netcoredbg_v{}", version.tag_name));

        let temp_dir = self.create_secure_temp_dir(&version.tag_name)?;
        self.logger.debug_log(&format!(
            "Created secure temp directory: {}",
            temp_dir.path().display()
        ));

        zed::download_file(
            &version.download_url,
            &temp_dir.path().to_string_lossy(),
            file_type,
        )
        .map_err(|e| format!("Failed to download netcoredbg: {}", e))?;

        std::fs::create_dir_all(&version_dir)
            .map_err(|e| format!("Failed to create version directory: {}", e))?;

        self.copy_extracted_content(temp_dir.path(), &version_dir)?;

        let exe_name = Self::get_executable_name();

        let binary_path = version_dir.join(exe_name);

        if !binary_path.exists() {
            return Err(format!(
                "netcoredbg executable not found at: {}",
                binary_path.display()
            ));
        }

        zed::make_file_executable(&binary_path.to_string_lossy())
            .map_err(|e| format!("Failed to make file executable: {}", e))?;

        let current_dir = std::env::current_dir()
            .map_err(|e| format!("Failed to get current directory: {}", e))?;
        let absolute_path = current_dir.join(&binary_path);
        Ok(absolute_path.to_string_lossy().to_string())
    }

    /// Copies extracted content from temp_dir into version_dir
    fn copy_extracted_content(
        &self,
        temp_dir: &std::path::Path,
        version_dir: &std::path::Path,
    ) -> Result<(), String> {
        let copy_options = dir::CopyOptions::new().content_only(true);

        dir::copy(temp_dir, version_dir, &copy_options)
            .map_err(|e| format!("Failed to copy extracted content: {}", e))?;

        Ok(())
    }

    /// Gets the netcoredbg binary path, downloading if necessary
    pub fn get_binary_path(&self, user_provided_path: Option<String>) -> Result<String, String> {
        self.logger.debug_log("Starting get_binary_path");

        // Priority 1: User-provided path
        if let Some(user_path) = user_provided_path {
            self.logger
                .debug_log(&format!("Using user-provided path: {}", user_path));
            let path = std::path::Path::new(&user_path);
            if !path.exists() {
                return Err(format!(
                    "User-provided netcoredbg binary not found at: {}",
                    user_path
                ));
            }
            if !path.is_file() {
                return Err(format!("User-provided path is not a file: {}", user_path));
            }
            // Convert to absolute path for consistency
            let current_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            let absolute_path = if path.is_absolute() {
                path.to_path_buf()
            } else {
                current_dir.join(path)
            };
            return Ok(absolute_path.to_string_lossy().to_string());
        }

        // Priority 2: Check in-memory cache
        if let Some(cached_path) = self.cached_binary_path.get() {
            if std::path::Path::new(cached_path).exists() {
                self.logger
                    .debug_log(&format!("Using cached binary path: {}", cached_path));
                return Ok(cached_path.clone());
            }
            self.logger
                .debug_log("Cached binary no longer exists, will re-download");
        }

        // Priority 3: Check existing binary on disk before downloading
        self.logger
            .debug_log("Fetching latest release info from GitHub to check for existing binary");
        let version = self.fetch_latest_release()?;
        self.logger
            .debug_log(&format!("Found latest version: {}", version.tag_name));

        // Version-specific directory in current working directory
        let version_dir = std::path::PathBuf::from(format!("netcoredbg_v{}", version.tag_name));
        let exe_name = Self::get_executable_name();
        let existing_binary_path = version_dir.join(exe_name);
        if existing_binary_path.exists() {
            self.logger.debug_log(&format!(
                "Found existing binary on disk: {}",
                existing_binary_path.display()
            ));
            let current_dir = std::env::current_dir()
                .map_err(|e| format!("Failed to get current directory: {}", e))?;
            let absolute_path = current_dir.join(&existing_binary_path);
            let path_str = absolute_path.to_string_lossy().to_string();
            let _ = self.cached_binary_path.set(path_str.clone());
            return Ok(path_str);
        }

        // Priority 4: Download and extract from GitHub releases
        self.logger
            .debug_log("No existing binary found, downloading from GitHub");
        let binary_path = self.download_and_extract_binary()?;
        self.logger.debug_log(&format!(
            "Successfully downloaded and extracted to: {}",
            binary_path
        ));

        let _ = self.cached_binary_path.set(binary_path.clone());

        Ok(binary_path)
    }

    /// Validates that the binary exists
    pub fn validate_binary(&self, binary_path: &str) -> Result<(), String> {
        let path = std::path::Path::new(binary_path);

        if !path.exists() {
            return Err(format!("netcoredbg binary not found at: {}", binary_path));
        }

        if !path.is_file() {
            return Err(format!("netcoredbg path is not a file: {}", binary_path));
        }

        Ok(())
    }
}
