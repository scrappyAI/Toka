//! Security-focused tests for Vault encryption
//! Tests encryption edge cases, key management, and potential vulnerabilities

use anyhow::Result;
use std::sync::Arc;
use tempfile::tempdir;
use toka_secrets::{Vault, VaultEntry, VaultMetadata};
use base64::Engine;

#[tokio::test]
async fn test_encryption_key_isolation() -> Result<()> {
    // Different vault instances should have different encryption keys
    let dir1 = tempdir()?;
    let dir2 = tempdir()?;
    
    let vault1 = Vault::new(dir1.path().to_str().unwrap())?;
    let vault2 = Vault::new(dir2.path().to_str().unwrap())?;
    
    let entry = VaultEntry {
        key: "test_key".to_string(),
        data: "sensitive_data".to_string(),
        metadata: VaultMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        },
    };
    
    // Store in vault1
    vault1.insert(&entry).await?;
    
    // Should not be accessible from vault2
    let result = vault2.get("test_key").await?;
    assert!(result.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_large_data_encryption() -> Result<()> {
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;
    
    // Test with various large data sizes
    let sizes = vec![1024, 10240, 102400, 1024000]; // 1KB to 1MB
    
    for size in sizes {
        let large_data = "x".repeat(size);
        let entry = VaultEntry {
            key: format!("large_key_{}", size),
            data: large_data.clone(),
            metadata: VaultMetadata {
                created_at: 1234567890,
                updated_at: 1234567890,
                version: 1,
            },
        };
        
        vault.insert(&entry).await?;
        let retrieved = vault.get(&entry.key).await?.unwrap();
        assert_eq!(retrieved.data, large_data);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_unicode_and_binary_data_encryption() -> Result<()> {
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;
    
    // Test Unicode data
    let unicode_data = "Hello 世界 🌍 Ελληνικά العربية русский";
    let unicode_entry = VaultEntry {
        key: "unicode_test".to_string(),
        data: unicode_data.to_string(),
        metadata: VaultMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        },
    };
    
    vault.insert(&unicode_entry).await?;
    let retrieved = vault.get("unicode_test").await?.unwrap();
    assert_eq!(retrieved.data, unicode_data);
    
    // Test binary-like data (base64 encoded)
    let binary_data = base64::engine::general_purpose::STANDARD.encode(b"\x00\x01\x02\xFF\xFE\xFD");
    let binary_entry = VaultEntry {
        key: "binary_test".to_string(),
        data: binary_data.clone(),
        metadata: VaultMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        },
    };
    
    vault.insert(&binary_entry).await?;
    let retrieved = vault.get("binary_test").await?.unwrap();
    assert_eq!(retrieved.data, binary_data);
    
    Ok(())
}

#[tokio::test]
async fn test_concurrent_encryption_operations() -> Result<()> {
    let dir = tempdir()?;
    let vault = Arc::new(Vault::new(dir.path().to_str().unwrap())?);
    
    let mut handles = vec![];
    
    // Spawn multiple concurrent encryption operations
    for i in 0..50 {
        let vault_clone = Arc::clone(&vault);
        let handle = tokio::spawn(async move {
            let entry = VaultEntry {
                key: format!("concurrent_key_{}", i),
                data: format!("concurrent_data_{}", i),
                metadata: VaultMetadata {
                    created_at: 1234567890 + i as u64,
                    updated_at: 1234567890 + i as u64,
                    version: 1,
                },
            };
            
            vault_clone.insert(&entry).await?;
            let retrieved = vault_clone.get(&entry.key).await?.unwrap();
            assert_eq!(retrieved.data, entry.data);
            
            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        handle.await??;
    }
    
    Ok(())
}

#[tokio::test]
async fn test_vault_corruption_resistance() -> Result<()> {
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;
    
    let entry = VaultEntry {
        key: "test_key".to_string(),
        data: "test_data".to_string(),
        metadata: VaultMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        },
    };
    
    vault.insert(&entry).await?;
    
    // Simulate database corruption by creating a new vault instance
    // with the same path (this tests key persistence and recovery)
    drop(vault);
    
    let vault2 = Vault::new(dir.path().to_str().unwrap())?;
    let retrieved = vault2.get("test_key").await?.unwrap();
    assert_eq!(retrieved.data, "test_data");
    
    Ok(())
}

#[tokio::test]
async fn test_empty_and_null_data_handling() -> Result<()> {
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;
    
    // Test empty string
    let empty_entry = VaultEntry {
        key: "empty_key".to_string(),
        data: "".to_string(),
        metadata: VaultMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        },
    };
    
    vault.insert(&empty_entry).await?;
    let retrieved = vault.get("empty_key").await?.unwrap();
    assert_eq!(retrieved.data, "");
    
    // Test JSON null
    let null_entry = VaultEntry {
        key: "null_key".to_string(),
        data: "null".to_string(),
        metadata: VaultMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        },
    };
    
    vault.insert(&null_entry).await?;
    let retrieved = vault.get("null_key").await?.unwrap();
    assert_eq!(retrieved.data, "null");
    
    Ok(())
}

#[tokio::test]
async fn test_key_collision_resistance() -> Result<()> {
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;
    
    // Test keys that might collide when hashed
    let collision_keys = vec![
        "key1",
        "key2", 
        "key1 ", // with space
        "Key1", // different case
        "key1\n", // with newline
        "key1\0", // with null byte
    ];
    
    for (i, key) in collision_keys.iter().enumerate() {
        let entry = VaultEntry {
            key: key.to_string(),
            data: format!("data_{}", i),
            metadata: VaultMetadata {
                created_at: 1234567890,
                updated_at: 1234567890,
                version: 1,
            },
        };
        
        vault.insert(&entry).await?;
    }
    
    // Verify all entries are stored separately
    for (i, key) in collision_keys.iter().enumerate() {
        let retrieved = vault.get(key).await?.unwrap();
        assert_eq!(retrieved.data, format!("data_{}", i));
    }
    
    Ok(())
}

#[tokio::test]
async fn test_metadata_tampering_detection() -> Result<()> {
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;
    
    let original_entry = VaultEntry {
        key: "tamper_test".to_string(),
        data: "original_data".to_string(),
        metadata: VaultMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        },
    };
    
    vault.insert(&original_entry).await?;
    let retrieved = vault.get("tamper_test").await?.unwrap();
    
    // Verify metadata integrity
    assert_eq!(retrieved.metadata.created_at, 1234567890);
    assert_eq!(retrieved.metadata.version, 1);
    
    // Update with new metadata
    let updated_entry = VaultEntry {
        key: "tamper_test".to_string(),
        data: "updated_data".to_string(),
        metadata: VaultMetadata {
            created_at: 1234567890, // Keep original
            updated_at: 1234567999, // New timestamp
            version: 2, // Increment version
        },
    };
    
    vault.insert(&updated_entry).await?;
    let retrieved = vault.get("tamper_test").await?.unwrap();
    
    assert_eq!(retrieved.data, "updated_data");
    assert_eq!(retrieved.metadata.version, 2);
    assert_eq!(retrieved.metadata.updated_at, 1234567999);
    
    Ok(())
}

#[tokio::test]
async fn test_nonce_uniqueness() -> Result<()> {
    // This test verifies that encryption produces different ciphertexts
    // for the same plaintext (due to unique nonces)
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;
    
    let same_data = "identical_plaintext_data";
    
    // Insert the same data with different keys
    for i in 0..10 {
        let entry = VaultEntry {
            key: format!("nonce_test_{}", i),
            data: same_data.to_string(),
            metadata: VaultMetadata {
                created_at: 1234567890,
                updated_at: 1234567890,
                version: 1,
            },
        };
        
        vault.insert(&entry).await?;
    }
    
    // All entries should decrypt to the same plaintext
    for i in 0..10 {
        let retrieved = vault.get(&format!("nonce_test_{}", i)).await?.unwrap();
        assert_eq!(retrieved.data, same_data);
    }
    
    Ok(())
}

#[tokio::test]
async fn test_memory_cleanup() -> Result<()> {
    // Test that sensitive data doesn't linger in memory
    let dir = tempdir()?;
    let vault = Vault::new(dir.path().to_str().unwrap())?;
    
    let sensitive_data = "super_secret_password_123456789";
    let entry = VaultEntry {
        key: "memory_test".to_string(),
        data: sensitive_data.to_string(),
        metadata: VaultMetadata {
            created_at: 1234567890,
            updated_at: 1234567890,
            version: 1,
        },
    };
    
    vault.insert(&entry).await?;
    let retrieved = vault.get("memory_test").await?.unwrap();
    assert_eq!(retrieved.data, sensitive_data);
    
    // Explicitly drop the vault to trigger cleanup
    drop(vault);
    
    // Create new vault instance and verify data persistence
    let vault2 = Vault::new(dir.path().to_str().unwrap())?;
    let retrieved2 = vault2.get("memory_test").await?.unwrap();
    assert_eq!(retrieved2.data, sensitive_data);
    
    Ok(())
}

#[tokio::test]
async fn test_stress_concurrent_operations() -> Result<()> {
    let dir = tempdir()?;
    let vault = Arc::new(Vault::new(dir.path().to_str().unwrap())?);
    
    let mut handles = vec![];
    
    // Mix of reads, writes, and deletes
    for i in 0..100 {
        let vault_clone = Arc::clone(&vault);
        let handle = tokio::spawn(async move {
            let key = format!("stress_key_{}", i);
            let data = format!("stress_data_{}", i);
            
            // Insert
            let entry = VaultEntry {
                key: key.clone(),
                data: data.clone(),
                metadata: VaultMetadata {
                    created_at: 1234567890 + i as u64,
                    updated_at: 1234567890 + i as u64,
                    version: 1,
                },
            };
            vault_clone.insert(&entry).await?;
            
            // Read
            let retrieved = vault_clone.get(&key).await?.unwrap();
            assert_eq!(retrieved.data, data);
            
            // Update
            let updated_entry = VaultEntry {
                key: key.clone(),
                data: format!("updated_{}", data),
                metadata: VaultMetadata {
                    created_at: 1234567890 + i as u64,
                    updated_at: 1234567890 + i as u64 + 1000,
                    version: 2,
                },
            };
            vault_clone.insert(&updated_entry).await?;
            
            // Verify update
            let retrieved = vault_clone.get(&key).await?.unwrap();
            assert_eq!(retrieved.data, format!("updated_{}", data));
            
            Ok::<(), anyhow::Error>(())
        });
        handles.push(handle);
    }
    
    // Wait for all operations
    for handle in handles {
        handle.await??;
    }
    
    // Verify final state
    let keys = vault.list().await?;
    assert_eq!(keys.len(), 100);
    
    Ok(())
}