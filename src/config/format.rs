//! Форматы конфигурации (бинарный и TOML)

use anyhow::{Context, Result};
use serde::{Serialize, de::DeserializeOwned};
use sha2::{Sha256, Digest};
use std::path::Path;
use tokio::fs;

/// Магические байты для бинарного формата ZIN
const ZIN_MAGIC: &[u8] = b"ZINCFG01";

/// Бинарный формат конфигурации
#[derive(Debug, Clone, Serialize)]
pub struct BinaryConfig<T> {
    pub magic: [u8; 8],
    pub version: u32,
    pub checksum: [u8; 32],
    pub timestamp: u64,
    pub data: T,
}

impl<T: Serialize> BinaryConfig<T> {
    pub fn new(data: T) -> Self {
        use chrono::Utc;
        
        // Сериализация данных для вычисления контрольной суммы
        let data_bytes = bincode::serialize(&data).unwrap_or_default();
        let checksum = Sha256::digest(&data_bytes);
        
        Self {
            magic: *ZIN_MAGIC,
            version: 1,
            checksum: checksum.into(),
            timestamp: Utc::now().timestamp() as u64,
            data,
        }
    }
    
    pub fn serialize(&self) -> Result<Vec<u8>> {
        bincode::serialize(self)
            .context("Ошибка сериализации бинарной конфигурации")
    }
    
    pub fn deserialize(bytes: &[u8]) -> Result<Self>
    where
        T: DeserializeOwned,
    {
        let config: BinaryConfig<T> = bincode::deserialize(bytes)
            .context("Ошибка десериализации бинарной конфигурации")?;
        
        // Проверка магических байтов
        if &config.magic != ZIN_MAGIC {
            anyhow::bail!("Неверные магические байты");
        }
        
        // Проверка версии
        if config.version != 1 {
            anyhow::bail!("Неподдерживаемая версия: {}", config.version);
        }
        
        // Проверка контрольной суммы
        let data_bytes = bincode::serialize(&config.data)?;
        let expected_checksum = Sha256::digest(&data_bytes);
        
        if config.checksum != expected_checksum.as_slice() {
            anyhow::bail!("Контрольная сумма не совпадает");
        }
        
        Ok(config)
    }
}

/// Сериализация в TOML
pub fn to_toml<T: Serialize>(data: &T) -> Result<String> {
    toml::to_string_pretty(data)
        .context("Ошибка сериализации в TOML")
}

/// Десериализация из TOML
pub fn from_toml<T: DeserializeOwned>(content: &str) -> Result<T> {
    toml::from_str(content)
        .context("Ошибка десериализации из TOML")
}

/// Сохранение в бинарном формате
pub async fn save_binary<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let binary_config = BinaryConfig::new(data);
    let bytes = binary_config.serialize()?;
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    fs::write(path, bytes).await?;
    Ok(())
}

/// Загрузка из бинарного формата
pub async fn load_binary<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let bytes = fs::read(path).await?;
    let config = BinaryConfig::<T>::deserialize(&bytes)?;
    Ok(config.data)
}

/// Сохранение в TOML формате
pub async fn save_toml<T: Serialize>(path: &Path, data: &T) -> Result<()> {
    let content = to_toml(data)?;
    
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).await?;
    }
    
    fs::write(path, content).await?;
    Ok(())
}

/// Загрузка из TOML формата
pub async fn load_toml<T: DeserializeOwned>(path: &Path) -> Result<T> {
    let content = fs::read_to_string(path).await?;
    from_toml(&content)
}

/// Проверка целостности бинарного файла
pub async fn verify_binary_integrity(path: &Path) -> Result<bool> {
    let bytes = fs::read(path).await?;
    
    match BinaryConfig::<serde_json::Value>::deserialize(&bytes) {
        Ok(_) => Ok(true),
        Err(_) => Ok(false),
    }
}
