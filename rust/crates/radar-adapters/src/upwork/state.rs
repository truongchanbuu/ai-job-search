use super::*;
use chacha20poly1305::{
    ChaCha20Poly1305, KeyInit,
    aead::{Aead, AeadCore, OsRng},
};
use rmcp::transport::auth::{AuthError, CredentialStore, StoredCredentials};
use std::{
    fs::{self, File, OpenOptions},
    io::Write,
    path::{Path, PathBuf},
    sync::Arc,
};

#[derive(Clone)]
pub struct PrivateState {
    directory: PathBuf,
    key: [u8; 32],
}
pub struct StateGuard {
    _file: File,
}
impl PrivateState {
    pub fn from_settings(settings: &UpworkSettings) -> Result<Self, UpworkError> {
        let directory = std::env::var_os(&settings.state_dir_env)
            .ok_or_else(|| UpworkError::new("state_not_configured"))?;
        let key = std::env::var(&settings.encryption_key_env)
            .map_err(|_| UpworkError::new("encryption_key_missing"))?;
        let key: [u8; 32] = hex::decode(key)
            .ok()
            .and_then(|v| v.try_into().ok())
            .ok_or_else(|| UpworkError::new("encryption_key_invalid"))?;
        Self::new(PathBuf::from(directory), key)
    }
    pub fn new(directory: PathBuf, key: [u8; 32]) -> Result<Self, UpworkError> {
        #[cfg(unix)]
        use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
        if !directory.exists() {
            let mut builder = fs::DirBuilder::new();
            #[cfg(unix)]
            builder.mode(0o700);
            builder
                .recursive(true)
                .create(&directory)
                .map_err(|_| UpworkError::new("state_unavailable"))?;
        }
        let metadata =
            fs::symlink_metadata(&directory).map_err(|_| UpworkError::new("state_unavailable"))?;
        if !metadata.is_dir() || metadata.file_type().is_symlink() {
            return Err(UpworkError::new("state_directory_unsafe"));
        }
        #[cfg(unix)]
        if metadata.permissions().mode() & 0o077 != 0 {
            return Err(UpworkError::new("state_permissions_require_0700"));
        }
        Ok(Self { directory, key })
    }
    pub fn lock(&self) -> Result<StateGuard, UpworkError> {
        let path = self.directory.join("session.lock");
        let file = private_open(&path)?;
        fs2::FileExt::try_lock_exclusive(&file)
            .map_err(|_| UpworkError::new("upwork_session_busy"))?;
        Ok(StateGuard { _file: file })
    }
    pub fn read<T: serde::de::DeserializeOwned>(
        &self,
        name: &str,
    ) -> Result<Option<T>, UpworkError> {
        let path = self.directory.join(name);
        if !path.exists() {
            return Ok(None);
        }
        check_file(&path)?;
        let bytes = fs::read(path).map_err(|_| UpworkError::new("state_unavailable"))?;
        if bytes.len() < 12 || bytes.len() > 4 * 1024 * 1024 {
            return Err(UpworkError::new("state_invalid"));
        }
        let cipher = ChaCha20Poly1305::new((&self.key).into());
        let plain = cipher
            .decrypt(
                (&bytes[..12]).into(),
                chacha20poly1305::aead::Payload {
                    msg: &bytes[12..],
                    aad: name.as_bytes(),
                },
            )
            .map_err(|_| UpworkError::new("state_decryption_failed"))?;
        serde_json::from_slice::<Option<T>>(&plain).map_err(|_| UpworkError::new("state_invalid"))
    }
    pub fn write<T: Serialize>(&self, name: &str, value: &T) -> Result<(), UpworkError> {
        let cipher = ChaCha20Poly1305::new((&self.key).into());
        let nonce = ChaCha20Poly1305::generate_nonce(&mut OsRng);
        let plain = serde_json::to_vec(value).map_err(|_| UpworkError::new("state_invalid"))?;
        let encrypted = cipher
            .encrypt(
                &nonce,
                chacha20poly1305::aead::Payload {
                    msg: &plain,
                    aad: name.as_bytes(),
                },
            )
            .map_err(|_| UpworkError::new("state_encryption_failed"))?;
        let path = self
            .directory
            .join(format!(".{name}.{}", uuid::Uuid::new_v4()));
        let mut options = OpenOptions::new();
        options.write(true).create_new(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        let mut file = options
            .open(&path)
            .map_err(|_| UpworkError::new("state_unavailable"))?;
        file.write_all(&nonce)
            .and_then(|_| file.write_all(&encrypted))
            .and_then(|_| file.sync_all())
            .map_err(|_| UpworkError::new("state_unavailable"))?;
        fs::rename(path, self.directory.join(name))
            .map_err(|_| UpworkError::new("state_unavailable"))?;
        File::open(&self.directory)
            .and_then(|f| f.sync_all())
            .map_err(|_| UpworkError::new("state_unavailable"))?;
        Ok(())
    }
}
fn check_file(path: &Path) -> Result<(), UpworkError> {
    let meta = fs::symlink_metadata(path).map_err(|_| UpworkError::new("state_unavailable"))?;
    if !meta.is_file() || meta.file_type().is_symlink() {
        return Err(UpworkError::new("state_file_unsafe"));
    }
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        if meta.permissions().mode() & 0o077 != 0 {
            return Err(UpworkError::new("state_permissions_require_0600"));
        }
    }
    Ok(())
}
fn private_open(path: &Path) -> Result<File, UpworkError> {
    if path.exists() {
        check_file(path)?
    }
    let mut options = OpenOptions::new();
    options.read(true).write(true).create(true).truncate(false);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    options
        .open(path)
        .map_err(|_| UpworkError::new("state_unavailable"))
}
#[async_trait::async_trait]
impl CredentialStore for PrivateState {
    async fn load(&self) -> Result<Option<StoredCredentials>, AuthError> {
        self.read("oauth.enc").map_err(|_| {
            AuthError::CredentialStoreError("Radar credential storage unavailable".into())
        })
    }
    async fn save(&self, c: StoredCredentials) -> Result<(), AuthError> {
        self.write("oauth.enc", &c).map_err(|_| {
            AuthError::CredentialStoreError("Radar credential storage unavailable".into())
        })
    }
    async fn clear(&self) -> Result<(), AuthError> {
        self.write("oauth.enc", &Option::<StoredCredentials>::None)
            .map_err(|_| {
                AuthError::CredentialStoreError("Radar credential storage unavailable".into())
            })
    }
}
#[derive(Default, Serialize, Deserialize)]
struct Budget {
    minute: i64,
    hour: i64,
    day: i64,
    m: u32,
    h: u32,
    d: u32,
    blocked_until: i64,
    #[serde(default)]
    failures: u32,
}
impl PrivateState {
    pub fn reserve(&self, s: &UpworkSettings, now: i64) -> Result<(), UpworkError> {
        let mut b: Budget = self.read("budget.enc")?.unwrap_or_default();
        if now < b.blocked_until {
            return Err(UpworkError {
                category: "rate_limited".into(),
                retry_after_seconds: Some((b.blocked_until - now) as u64),
            });
        }
        for (window, used, current) in [
            (&mut b.minute, &mut b.m, now / 60),
            (&mut b.hour, &mut b.h, now / 3600),
            (&mut b.day, &mut b.d, now / 86400),
        ] {
            if *window != current {
                *window = current;
                *used = 0
            }
        }
        if b.m >= s.max_requests_per_minute
            || b.h >= s.max_requests_per_hour
            || b.d >= s.max_requests_per_day
        {
            return Err(UpworkError {
                category: "request_budget_reached".into(),
                retry_after_seconds: Some(60),
            });
        }
        b.m += 1;
        b.h += 1;
        b.d += 1;
        self.write("budget.enc", &b)
    }
    pub fn reset_backoff(&self) -> Result<(), UpworkError> {
        let mut budget: Budget = self.read("budget.enc")?.unwrap_or_default();
        budget.failures = 0;
        self.write("budget.enc", &budget)
    }
    pub fn defer(&self, settings: &UpworkSettings, error: &UpworkError) -> Result<(), UpworkError> {
        let mut b: Budget = self.read("budget.enc")?.unwrap_or_default();
        let backoff = settings
            .retry_backoff_seconds
            .saturating_mul(1u64 << b.failures.min(8))
            .min(3600);
        b.failures = b.failures.saturating_add(1);
        let seconds = backoff.max(error.retry_after_seconds.unwrap_or(0));
        b.blocked_until = chrono::Utc::now()
            .timestamp()
            .saturating_add(i64::try_from(seconds).unwrap_or(i64::MAX));
        self.write("budget.enc", &b)
    }
}
pub type SharedState = Arc<PrivateState>;
