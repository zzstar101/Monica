//! UniFFI boundary for Monica for iOS.
//!
//! The implementation intentionally starts small: the iOS technical
//! verification must prove MDBX create/open/unlock and project-scoped entry
//! read/write before any SwiftUI product surface depends on it.

use std::path::Path;
use std::sync::{Arc, Mutex};

use mdbx_core::model::EntryType;
use mdbx_core::tiga::TigaMode;
use mdbx_storage::connection::VaultConnection;
use mdbx_storage::error::StorageError;
use mdbx_storage::init::{initialize_vault, VaultInitParams};
use mdbx_storage::repo::{CommitContext, EntryRepo, ProjectRepo};
use mdbx_storage::unlock::UnlockService;

uniffi::setup_scaffolding!();

#[derive(Debug, thiserror::Error, uniffi::Error)]
pub enum MdbxFfiError {
    #[error("storage error: {message}")]
    Storage { message: String },
    #[error("serialization error: {message}")]
    Serialization { message: String },
    #[error("vault lock poisoned")]
    LockPoisoned,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct VaultInfo {
    pub vault_id: String,
    pub device_id: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ProjectRecord {
    pub project_id: String,
    pub title: String,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct LoginEntryRecord {
    pub entry_id: String,
    pub project_id: String,
    pub title: String,
    pub username: String,
    pub password: String,
    pub url: String,
    pub favorite: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct NoteEntryRecord {
    pub entry_id: String,
    pub project_id: String,
    pub title: String,
    pub body: String,
    pub favorite: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct TotpEntryRecord {
    pub entry_id: String,
    pub project_id: String,
    pub title: String,
    pub secret: String,
    pub issuer: String,
    pub account_name: String,
    pub period: u32,
    pub digits: u32,
    pub algorithm: String,
    pub otp_type: String,
    pub counter: u64,
    pub favorite: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct CardEntryRecord {
    pub entry_id: String,
    pub project_id: String,
    pub title: String,
    pub cardholder_name: String,
    pub number: String,
    pub expiry_month: String,
    pub expiry_year: String,
    pub cvv: String,
    pub issuer: String,
    pub network: String,
    pub notes: String,
    pub favorite: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct IdentityEntryRecord {
    pub entry_id: String,
    pub project_id: String,
    pub title: String,
    pub document_type: String,
    pub full_name: String,
    pub document_number: String,
    pub issuer: String,
    pub country: String,
    pub issue_date: String,
    pub expiry_date: String,
    pub notes: String,
    pub favorite: bool,
}

#[derive(Debug, Clone, uniffi::Record)]
pub struct ParityEntryRecord {
    pub entry_id: String,
    pub project_id: String,
    pub title: String,
    pub kind: String,
    pub payload_json: String,
    pub favorite: bool,
}

#[derive(uniffi::Object)]
pub struct MdbxVault {
    conn: Mutex<VaultConnection>,
    device_id: String,
    vault_id: String,
}

#[uniffi::export]
impl MdbxVault {
    pub fn info(&self) -> VaultInfo {
        VaultInfo {
            vault_id: self.vault_id.clone(),
            device_id: self.device_id.clone(),
        }
    }

    pub fn create_project(&self, title: String) -> Result<ProjectRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let ctx = CommitContext::new(self.device_id.clone());
        let project = ProjectRepo::create(&conn, &ctx, &title, None, None)?;
        Ok(ProjectRecord {
            project_id: project.project_id,
            title: String::from_utf8_lossy(&project.title_ct).to_string(),
        })
    }

    pub fn create_login_entry(
        &self,
        project_id: String,
        title: String,
        username: String,
        password: String,
        url: String,
    ) -> Result<LoginEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let ctx = CommitContext::new(self.device_id.clone());
        let payload = serde_json::json!({
            "kind": "password",
            "username": username,
            "password": password,
            "website": url,
            "favorite": false,
        });
        let entry = EntryRepo::create(
            &conn,
            &ctx,
            &project_id,
            EntryType::Login,
            Some(&title),
            &payload,
        )?;
        login_record_from_entry(&entry)
    }

    pub fn list_entries(&self, project_id: String) -> Result<Vec<LoginEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_by_project(&conn, &project_id)?
            .into_iter()
            .filter(|entry| entry.entry_type == EntryType::Login)
            .map(|entry| login_record_from_entry(&entry))
            .collect()
    }

    pub fn list_deleted_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<LoginEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_deleted(&conn)?
            .into_iter()
            .filter(|entry| entry.project_id == project_id && entry.entry_type == EntryType::Login)
            .map(|entry| login_record_from_entry(&entry))
            .collect()
    }

    pub fn update_login_entry(
        &self,
        project_id: String,
        entry_id: String,
        title: String,
        username: String,
        password: String,
        url: String,
    ) -> Result<LoginEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let mut entry = EntryRepo::get_by_id(&conn, &entry_id)?
            .ok_or_else(|| StorageError::NotFound(entry_id.clone()))?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is deleted",
                entry_id
            ))
            .into());
        }
        if entry.project_id != project_id {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} does not belong to project {}",
                entry_id, project_id
            ))
            .into());
        }
        if entry.entry_type != EntryType::Login {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is not a login entry",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let mut payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
        payload["kind"] = serde_json::Value::String("password".to_string());
        payload["username"] = serde_json::Value::String(username);
        payload["password"] = serde_json::Value::String(password);
        payload["website"] = serde_json::Value::String(url);
        entry.title_ct = Some(title.into_bytes());
        entry.payload_ct =
            serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
                message: e.to_string(),
            })?;
        let updated = EntryRepo::update(&conn, &ctx, &entry)?;
        login_record_from_entry(&updated)
    }

    pub fn set_login_favorite(
        &self,
        project_id: String,
        entry_id: String,
        favorite: bool,
    ) -> Result<LoginEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let mut entry = login_entry_for_project(&conn, &project_id, &entry_id)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is deleted",
                entry_id
            ))
            .into());
        }

        let mut payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
        payload["favorite"] = serde_json::Value::Bool(favorite);
        entry.payload_ct =
            serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
                message: e.to_string(),
            })?;

        let ctx = CommitContext::new(self.device_id.clone());
        let updated = EntryRepo::update(&conn, &ctx, &entry)?;
        login_record_from_entry(&updated)
    }

    pub fn delete_login_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<(), MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = login_entry_for_project(&conn, &project_id, &entry_id)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is already deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        EntryRepo::soft_delete(&conn, &ctx, &entry_id)?;
        Ok(())
    }

    pub fn restore_login_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<LoginEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = login_entry_for_project(&conn, &project_id, &entry_id)?;
        if !entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is not deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let restored = EntryRepo::restore(&conn, &ctx, &entry_id)?;
        login_record_from_entry(&restored)
    }

    pub fn create_note_entry(
        &self,
        project_id: String,
        title: String,
        body: String,
    ) -> Result<NoteEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let ctx = CommitContext::new(self.device_id.clone());
        let payload = serde_json::json!({
            "kind": "note",
            "body": body,
            "favorite": false,
        });
        let entry = EntryRepo::create(
            &conn,
            &ctx,
            &project_id,
            EntryType::Note,
            Some(&title),
            &payload,
        )?;
        note_record_from_entry(&entry)
    }

    pub fn list_note_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<NoteEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_by_project(&conn, &project_id)?
            .into_iter()
            .filter(|entry| entry.entry_type == EntryType::Note)
            .map(|entry| note_record_from_entry(&entry))
            .collect()
    }

    pub fn list_deleted_note_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<NoteEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_deleted(&conn)?
            .into_iter()
            .filter(|entry| entry.project_id == project_id && entry.entry_type == EntryType::Note)
            .map(|entry| note_record_from_entry(&entry))
            .collect()
    }

    pub fn update_note_entry(
        &self,
        project_id: String,
        entry_id: String,
        title: String,
        body: String,
    ) -> Result<NoteEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let mut entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Note)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let mut payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
        let favorite = payload_favorite(&payload);
        payload = serde_json::json!({
            "kind": "note",
            "body": body,
            "favorite": favorite,
        });
        entry.title_ct = Some(title.into_bytes());
        entry.payload_ct =
            serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
                message: e.to_string(),
            })?;
        let updated = EntryRepo::update(&conn, &ctx, &entry)?;
        note_record_from_entry(&updated)
    }

    pub fn set_note_favorite(
        &self,
        project_id: String,
        entry_id: String,
        favorite: bool,
    ) -> Result<NoteEntryRecord, MdbxFfiError> {
        let updated =
            set_typed_entry_favorite(self, project_id, entry_id, EntryType::Note, favorite)?;
        note_record_from_entry(&updated)
    }

    pub fn delete_note_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<(), MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Note)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is already deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        EntryRepo::soft_delete(&conn, &ctx, &entry_id)?;
        Ok(())
    }

    pub fn restore_note_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<NoteEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Note)?;
        if !entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is not deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let restored = EntryRepo::restore(&conn, &ctx, &entry_id)?;
        note_record_from_entry(&restored)
    }

    pub fn create_totp_entry(
        &self,
        project_id: String,
        title: String,
        secret: String,
        issuer: String,
        account_name: String,
        period: u32,
        digits: u32,
        algorithm: String,
        otp_type: String,
        counter: u64,
    ) -> Result<TotpEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let ctx = CommitContext::new(self.device_id.clone());
        let payload = totp_payload(
            secret,
            issuer,
            account_name,
            period,
            digits,
            algorithm,
            otp_type,
            counter,
            false,
        );
        let entry = EntryRepo::create(
            &conn,
            &ctx,
            &project_id,
            EntryType::Totp,
            Some(&title),
            &payload,
        )?;
        totp_record_from_entry(&entry)
    }

    pub fn list_totp_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<TotpEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_by_project(&conn, &project_id)?
            .into_iter()
            .filter(|entry| entry.entry_type == EntryType::Totp)
            .map(|entry| totp_record_from_entry(&entry))
            .collect()
    }

    pub fn list_deleted_totp_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<TotpEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_deleted(&conn)?
            .into_iter()
            .filter(|entry| entry.project_id == project_id && entry.entry_type == EntryType::Totp)
            .map(|entry| totp_record_from_entry(&entry))
            .collect()
    }

    pub fn update_totp_entry(
        &self,
        project_id: String,
        entry_id: String,
        title: String,
        secret: String,
        issuer: String,
        account_name: String,
        period: u32,
        digits: u32,
        algorithm: String,
        otp_type: String,
        counter: u64,
    ) -> Result<TotpEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let mut entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Totp)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let favorite = payload_favorite(&serde_json::from_slice(&entry.payload_ct)?);
        let payload = totp_payload(
            secret,
            issuer,
            account_name,
            period,
            digits,
            algorithm,
            otp_type,
            counter,
            favorite,
        );
        entry.title_ct = Some(title.into_bytes());
        entry.payload_ct =
            serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
                message: e.to_string(),
            })?;
        let updated = EntryRepo::update(&conn, &ctx, &entry)?;
        totp_record_from_entry(&updated)
    }

    pub fn set_totp_favorite(
        &self,
        project_id: String,
        entry_id: String,
        favorite: bool,
    ) -> Result<TotpEntryRecord, MdbxFfiError> {
        let updated =
            set_typed_entry_favorite(self, project_id, entry_id, EntryType::Totp, favorite)?;
        totp_record_from_entry(&updated)
    }

    pub fn delete_totp_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<(), MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Totp)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is already deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        EntryRepo::soft_delete(&conn, &ctx, &entry_id)?;
        Ok(())
    }

    pub fn restore_totp_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<TotpEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Totp)?;
        if !entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is not deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let restored = EntryRepo::restore(&conn, &ctx, &entry_id)?;
        totp_record_from_entry(&restored)
    }

    pub fn create_card_entry(
        &self,
        project_id: String,
        title: String,
        cardholder_name: String,
        number: String,
        expiry_month: String,
        expiry_year: String,
        cvv: String,
        issuer: String,
        network: String,
        notes: String,
    ) -> Result<CardEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let ctx = CommitContext::new(self.device_id.clone());
        let payload = card_payload(
            cardholder_name,
            number,
            expiry_month,
            expiry_year,
            cvv,
            issuer,
            network,
            notes,
            false,
        );
        let entry = EntryRepo::create(
            &conn,
            &ctx,
            &project_id,
            EntryType::Card,
            Some(&title),
            &payload,
        )?;
        card_record_from_entry(&entry)
    }

    pub fn list_card_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<CardEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_by_project(&conn, &project_id)?
            .into_iter()
            .filter(|entry| entry.entry_type == EntryType::Card)
            .map(|entry| card_record_from_entry(&entry))
            .collect()
    }

    pub fn list_deleted_card_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<CardEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_deleted(&conn)?
            .into_iter()
            .filter(|entry| entry.project_id == project_id && entry.entry_type == EntryType::Card)
            .map(|entry| card_record_from_entry(&entry))
            .collect()
    }

    pub fn update_card_entry(
        &self,
        project_id: String,
        entry_id: String,
        title: String,
        cardholder_name: String,
        number: String,
        expiry_month: String,
        expiry_year: String,
        cvv: String,
        issuer: String,
        network: String,
        notes: String,
    ) -> Result<CardEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let mut entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Card)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let favorite = payload_favorite(&serde_json::from_slice(&entry.payload_ct)?);
        let payload = card_payload(
            cardholder_name,
            number,
            expiry_month,
            expiry_year,
            cvv,
            issuer,
            network,
            notes,
            favorite,
        );
        entry.title_ct = Some(title.into_bytes());
        entry.payload_ct =
            serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
                message: e.to_string(),
            })?;
        let updated = EntryRepo::update(&conn, &ctx, &entry)?;
        card_record_from_entry(&updated)
    }

    pub fn set_card_favorite(
        &self,
        project_id: String,
        entry_id: String,
        favorite: bool,
    ) -> Result<CardEntryRecord, MdbxFfiError> {
        let updated =
            set_typed_entry_favorite(self, project_id, entry_id, EntryType::Card, favorite)?;
        card_record_from_entry(&updated)
    }

    pub fn delete_card_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<(), MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Card)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is already deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        EntryRepo::soft_delete(&conn, &ctx, &entry_id)?;
        Ok(())
    }

    pub fn restore_card_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<CardEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Card)?;
        if !entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is not deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let restored = EntryRepo::restore(&conn, &ctx, &entry_id)?;
        card_record_from_entry(&restored)
    }

    pub fn create_identity_entry(
        &self,
        project_id: String,
        title: String,
        document_type: String,
        full_name: String,
        document_number: String,
        issuer: String,
        country: String,
        issue_date: String,
        expiry_date: String,
        notes: String,
    ) -> Result<IdentityEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let ctx = CommitContext::new(self.device_id.clone());
        let payload = identity_payload(
            document_type,
            full_name,
            document_number,
            issuer,
            country,
            issue_date,
            expiry_date,
            notes,
            false,
        );
        let entry = EntryRepo::create(
            &conn,
            &ctx,
            &project_id,
            EntryType::Identity,
            Some(&title),
            &payload,
        )?;
        identity_record_from_entry(&entry)
    }

    pub fn list_identity_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<IdentityEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_by_project(&conn, &project_id)?
            .into_iter()
            .filter(|entry| entry.entry_type == EntryType::Identity)
            .map(|entry| identity_record_from_entry(&entry))
            .collect()
    }

    pub fn list_deleted_identity_entries(
        &self,
        project_id: String,
    ) -> Result<Vec<IdentityEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        EntryRepo::list_deleted(&conn)?
            .into_iter()
            .filter(|entry| {
                entry.project_id == project_id && entry.entry_type == EntryType::Identity
            })
            .map(|entry| identity_record_from_entry(&entry))
            .collect()
    }

    pub fn update_identity_entry(
        &self,
        project_id: String,
        entry_id: String,
        title: String,
        document_type: String,
        full_name: String,
        document_number: String,
        issuer: String,
        country: String,
        issue_date: String,
        expiry_date: String,
        notes: String,
    ) -> Result<IdentityEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let mut entry =
            typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Identity)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let favorite = payload_favorite(&serde_json::from_slice(&entry.payload_ct)?);
        let payload = identity_payload(
            document_type,
            full_name,
            document_number,
            issuer,
            country,
            issue_date,
            expiry_date,
            notes,
            favorite,
        );
        entry.title_ct = Some(title.into_bytes());
        entry.payload_ct =
            serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
                message: e.to_string(),
            })?;
        let updated = EntryRepo::update(&conn, &ctx, &entry)?;
        identity_record_from_entry(&updated)
    }

    pub fn set_identity_favorite(
        &self,
        project_id: String,
        entry_id: String,
        favorite: bool,
    ) -> Result<IdentityEntryRecord, MdbxFfiError> {
        let updated =
            set_typed_entry_favorite(self, project_id, entry_id, EntryType::Identity, favorite)?;
        identity_record_from_entry(&updated)
    }

    pub fn delete_identity_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<(), MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Identity)?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is already deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        EntryRepo::soft_delete(&conn, &ctx, &entry_id)?;
        Ok(())
    }

    pub fn restore_identity_entry(
        &self,
        project_id: String,
        entry_id: String,
    ) -> Result<IdentityEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(&conn, &project_id, &entry_id, EntryType::Identity)?;
        if !entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is not deleted",
                entry_id
            ))
            .into());
        }

        let ctx = CommitContext::new(self.device_id.clone());
        let restored = EntryRepo::restore(&conn, &ctx, &entry_id)?;
        identity_record_from_entry(&restored)
    }

    pub fn create_parity_entry(
        &self,
        project_id: String,
        entry_type: String,
        kind: String,
        title: String,
        payload_json: String,
    ) -> Result<ParityEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let ctx = CommitContext::new(self.device_id.clone());
        let payload = parity_payload(kind, payload_json, false)?;
        let entry = EntryRepo::create(
            &conn,
            &ctx,
            &project_id,
            parity_entry_type(&entry_type)?,
            Some(&title),
            &payload,
        )?;
        parity_record_from_entry(&entry)
    }

    pub fn list_parity_entries(
        &self,
        project_id: String,
        entry_type: String,
        kind: String,
    ) -> Result<Vec<ParityEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry_type = parity_entry_type(&entry_type)?;
        EntryRepo::list_by_project(&conn, &project_id)?
            .into_iter()
            .filter(|entry| entry.entry_type == entry_type)
            .filter(|entry| entry_payload_kind(entry).as_deref() == Some(kind.as_str()))
            .map(|entry| parity_record_from_entry(&entry))
            .collect()
    }

    pub fn list_deleted_parity_entries(
        &self,
        project_id: String,
        entry_type: String,
        kind: String,
    ) -> Result<Vec<ParityEntryRecord>, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry_type = parity_entry_type(&entry_type)?;
        EntryRepo::list_deleted(&conn)?
            .into_iter()
            .filter(|entry| entry.project_id == project_id && entry.entry_type == entry_type)
            .filter(|entry| entry_payload_kind(entry).as_deref() == Some(kind.as_str()))
            .map(|entry| parity_record_from_entry(&entry))
            .collect()
    }

    pub fn update_parity_entry(
        &self,
        project_id: String,
        entry_id: String,
        entry_type: String,
        kind: String,
        title: String,
        payload_json: String,
    ) -> Result<ParityEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let mut entry = typed_entry_for_project(
            &conn,
            &project_id,
            &entry_id,
            parity_entry_type(&entry_type)?,
        )?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is deleted",
                entry_id
            ))
            .into());
        }
        ensure_parity_kind(&entry, &kind)?;

        let favorite = payload_favorite(&serde_json::from_slice(&entry.payload_ct)?);
        let payload = parity_payload(kind, payload_json, favorite)?;
        entry.title_ct = Some(title.into_bytes());
        entry.payload_ct =
            serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
                message: e.to_string(),
            })?;

        let ctx = CommitContext::new(self.device_id.clone());
        let updated = EntryRepo::update(&conn, &ctx, &entry)?;
        parity_record_from_entry(&updated)
    }

    pub fn set_parity_entry_favorite(
        &self,
        project_id: String,
        entry_id: String,
        entry_type: String,
        kind: String,
        favorite: bool,
    ) -> Result<ParityEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let mut entry = typed_entry_for_project(
            &conn,
            &project_id,
            &entry_id,
            parity_entry_type(&entry_type)?,
        )?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is deleted",
                entry_id
            ))
            .into());
        }
        ensure_parity_kind(&entry, &kind)?;

        let mut payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
        payload["favorite"] = serde_json::Value::Bool(favorite);
        entry.payload_ct =
            serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
                message: e.to_string(),
            })?;

        let ctx = CommitContext::new(self.device_id.clone());
        let updated = EntryRepo::update(&conn, &ctx, &entry)?;
        parity_record_from_entry(&updated)
    }

    pub fn delete_parity_entry(
        &self,
        project_id: String,
        entry_id: String,
        entry_type: String,
        kind: String,
    ) -> Result<(), MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(
            &conn,
            &project_id,
            &entry_id,
            parity_entry_type(&entry_type)?,
        )?;
        if entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is already deleted",
                entry_id
            ))
            .into());
        }
        ensure_parity_kind(&entry, &kind)?;

        let ctx = CommitContext::new(self.device_id.clone());
        EntryRepo::soft_delete(&conn, &ctx, &entry_id)?;
        Ok(())
    }

    pub fn restore_parity_entry(
        &self,
        project_id: String,
        entry_id: String,
        entry_type: String,
        kind: String,
    ) -> Result<ParityEntryRecord, MdbxFfiError> {
        let conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        let entry = typed_entry_for_project(
            &conn,
            &project_id,
            &entry_id,
            parity_entry_type(&entry_type)?,
        )?;
        if !entry.deleted {
            return Err(StorageError::ConstraintViolation(format!(
                "entry {} is not deleted",
                entry_id
            ))
            .into());
        }
        ensure_parity_kind(&entry, &kind)?;

        let ctx = CommitContext::new(self.device_id.clone());
        let restored = EntryRepo::restore(&conn, &ctx, &entry_id)?;
        parity_record_from_entry(&restored)
    }

    pub fn setup_local_security_key_unlock(
        &self,
        key_material: Vec<u8>,
    ) -> Result<(), MdbxFfiError> {
        let mut conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        UnlockService::setup_security_key(&mut conn, &key_material)?;
        Ok(())
    }

    pub fn reset_master_password(&self, new_password: String) -> Result<(), MdbxFfiError> {
        let mut conn = self.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
        UnlockService::reset_password_with_mode(&mut conn, &new_password, TigaMode::Multi)?;
        Ok(())
    }
}

fn set_typed_entry_favorite(
    vault: &MdbxVault,
    project_id: String,
    entry_id: String,
    entry_type: EntryType,
    favorite: bool,
) -> Result<mdbx_core::model::Entry, MdbxFfiError> {
    let conn = vault.conn.lock().map_err(|_| MdbxFfiError::LockPoisoned)?;
    let mut entry = typed_entry_for_project(&conn, &project_id, &entry_id, entry_type)?;
    if entry.deleted {
        return Err(
            StorageError::ConstraintViolation(format!("entry {} is deleted", entry_id)).into(),
        );
    }

    let mut payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
    payload["favorite"] = serde_json::Value::Bool(favorite);
    entry.payload_ct = serde_json::to_vec(&payload).map_err(|e| MdbxFfiError::Serialization {
        message: e.to_string(),
    })?;

    let ctx = CommitContext::new(vault.device_id.clone());
    EntryRepo::update(&conn, &ctx, &entry).map_err(MdbxFfiError::from)
}

#[uniffi::export]
pub fn create_vault(
    path: String,
    password: String,
    device_id: String,
) -> Result<Arc<MdbxVault>, MdbxFfiError> {
    let mut conn = VaultConnection::create(Path::new(&path))?;
    let init = initialize_vault(
        &conn,
        &VaultInitParams {
            default_tiga_mode: "multi".to_string(),
            device_id: device_id.clone(),
            ..Default::default()
        },
    )?;
    UnlockService::setup_password_with_mode(&mut conn, &password, TigaMode::Multi)?;
    Ok(Arc::new(MdbxVault {
        conn: Mutex::new(conn),
        device_id,
        vault_id: init.vault_id,
    }))
}

#[uniffi::export]
pub fn open_vault(
    path: String,
    password: String,
    device_id: String,
) -> Result<Arc<MdbxVault>, MdbxFfiError> {
    let mut conn = VaultConnection::open(Path::new(&path))?;
    UnlockService::unlock_with_password(&mut conn, &password)?;
    let vault_id =
        conn.inner()
            .query_row("SELECT vault_id FROM vault_meta LIMIT 1", [], |row| {
                row.get::<_, String>(0)
            })?;
    Ok(Arc::new(MdbxVault {
        conn: Mutex::new(conn),
        device_id,
        vault_id,
    }))
}

#[uniffi::export]
pub fn open_vault_with_security_key(
    path: String,
    key_material: Vec<u8>,
    device_id: String,
) -> Result<Arc<MdbxVault>, MdbxFfiError> {
    let mut conn = VaultConnection::open(Path::new(&path))?;
    UnlockService::unlock_with_security_key(&mut conn, &key_material)?;
    let vault_id =
        conn.inner()
            .query_row("SELECT vault_id FROM vault_meta LIMIT 1", [], |row| {
                row.get::<_, String>(0)
            })?;
    Ok(Arc::new(MdbxVault {
        conn: Mutex::new(conn),
        device_id,
        vault_id,
    }))
}

fn login_record_from_entry(
    entry: &mdbx_core::model::Entry,
) -> Result<LoginEntryRecord, MdbxFfiError> {
    let payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
    Ok(LoginEntryRecord {
        entry_id: entry.entry_id.clone(),
        project_id: entry.project_id.clone(),
        title: entry
            .title_ct
            .as_deref()
            .map(String::from_utf8_lossy)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        username: payload
            .get("username")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        password: payload
            .get("password")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        url: payload
            .get("website")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        favorite: payload
            .get("favorite")
            .and_then(|value| value.as_bool())
            .unwrap_or(false),
    })
}

fn note_record_from_entry(
    entry: &mdbx_core::model::Entry,
) -> Result<NoteEntryRecord, MdbxFfiError> {
    let payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
    Ok(NoteEntryRecord {
        entry_id: entry.entry_id.clone(),
        project_id: entry.project_id.clone(),
        title: entry
            .title_ct
            .as_deref()
            .map(String::from_utf8_lossy)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        body: payload
            .get("body")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        favorite: payload_favorite(&payload),
    })
}

fn totp_payload(
    secret: String,
    issuer: String,
    account_name: String,
    period: u32,
    digits: u32,
    algorithm: String,
    otp_type: String,
    counter: u64,
    favorite: bool,
) -> serde_json::Value {
    serde_json::json!({
        "kind": "totp",
        "secret": secret,
        "issuer": issuer,
        "accountName": account_name,
        "period": period,
        "digits": digits,
        "algorithm": algorithm,
        "otpType": otp_type,
        "counter": counter,
        "favorite": favorite,
        "steamFingerprint": "",
        "steamDeviceId": "",
        "steamSerialNumber": "",
        "steamSharedSecretBase64": "",
        "steamRevocationCode": "",
        "steamIdentitySecret": "",
        "steamTokenGid": "",
        "steamRawJson": "",
    })
}

fn totp_record_from_entry(
    entry: &mdbx_core::model::Entry,
) -> Result<TotpEntryRecord, MdbxFfiError> {
    let payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
    Ok(TotpEntryRecord {
        entry_id: entry.entry_id.clone(),
        project_id: entry.project_id.clone(),
        title: entry
            .title_ct
            .as_deref()
            .map(String::from_utf8_lossy)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        secret: payload
            .get("secret")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        issuer: payload
            .get("issuer")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        account_name: payload
            .get("accountName")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        period: payload
            .get("period")
            .and_then(|value| value.as_u64())
            .unwrap_or(30) as u32,
        digits: payload
            .get("digits")
            .and_then(|value| value.as_u64())
            .unwrap_or(6) as u32,
        algorithm: payload
            .get("algorithm")
            .and_then(|value| value.as_str())
            .unwrap_or("SHA1")
            .to_string(),
        otp_type: payload
            .get("otpType")
            .and_then(|value| value.as_str())
            .unwrap_or("TOTP")
            .to_string(),
        counter: payload
            .get("counter")
            .and_then(|value| value.as_u64())
            .unwrap_or_default(),
        favorite: payload_favorite(&payload),
    })
}

fn card_payload(
    cardholder_name: String,
    number: String,
    expiry_month: String,
    expiry_year: String,
    cvv: String,
    issuer: String,
    network: String,
    notes: String,
    favorite: bool,
) -> serde_json::Value {
    serde_json::json!({
        "kind": "card",
        "cardholderName": cardholder_name,
        "number": number,
        "expiryMonth": expiry_month,
        "expiryYear": expiry_year,
        "cvv": cvv,
        "issuer": issuer,
        "network": network,
        "notes": notes,
        "favorite": favorite,
    })
}

fn card_record_from_entry(
    entry: &mdbx_core::model::Entry,
) -> Result<CardEntryRecord, MdbxFfiError> {
    let payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
    Ok(CardEntryRecord {
        entry_id: entry.entry_id.clone(),
        project_id: entry.project_id.clone(),
        title: entry
            .title_ct
            .as_deref()
            .map(String::from_utf8_lossy)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        cardholder_name: payload
            .get("cardholderName")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        number: payload
            .get("number")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        expiry_month: payload
            .get("expiryMonth")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        expiry_year: payload
            .get("expiryYear")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        cvv: payload
            .get("cvv")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        issuer: payload
            .get("issuer")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        network: payload
            .get("network")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        notes: payload
            .get("notes")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        favorite: payload_favorite(&payload),
    })
}

fn identity_payload(
    document_type: String,
    full_name: String,
    document_number: String,
    issuer: String,
    country: String,
    issue_date: String,
    expiry_date: String,
    notes: String,
    favorite: bool,
) -> serde_json::Value {
    serde_json::json!({
        "kind": "identity",
        "documentType": document_type,
        "fullName": full_name,
        "documentNumber": document_number,
        "issuer": issuer,
        "country": country,
        "issueDate": issue_date,
        "expiryDate": expiry_date,
        "notes": notes,
        "favorite": favorite,
    })
}

fn identity_record_from_entry(
    entry: &mdbx_core::model::Entry,
) -> Result<IdentityEntryRecord, MdbxFfiError> {
    let payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
    Ok(IdentityEntryRecord {
        entry_id: entry.entry_id.clone(),
        project_id: entry.project_id.clone(),
        title: entry
            .title_ct
            .as_deref()
            .map(String::from_utf8_lossy)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        document_type: payload
            .get("documentType")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        full_name: payload
            .get("fullName")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        document_number: payload
            .get("documentNumber")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        issuer: payload
            .get("issuer")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        country: payload
            .get("country")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        issue_date: payload
            .get("issueDate")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        expiry_date: payload
            .get("expiryDate")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        notes: payload
            .get("notes")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        favorite: payload_favorite(&payload),
    })
}

fn payload_favorite(payload: &serde_json::Value) -> bool {
    payload
        .get("favorite")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
}

fn parity_entry_type(entry_type: &str) -> Result<EntryType, MdbxFfiError> {
    match entry_type {
        "passkey" => Ok(EntryType::Passkey),
        "ssh-key" => Ok(EntryType::SshKey),
        "api-token" => Ok(EntryType::ApiToken),
        "document-ref" => Ok(EntryType::DocumentRef),
        other => Err(MdbxFfiError::Serialization {
            message: format!("unsupported parity entry type: {}", other),
        }),
    }
}

fn parity_payload(
    kind: String,
    payload_json: String,
    favorite: bool,
) -> Result<serde_json::Value, MdbxFfiError> {
    let mut payload: serde_json::Value = serde_json::from_str(&payload_json)?;
    let object = payload
        .as_object_mut()
        .ok_or_else(|| MdbxFfiError::Serialization {
            message: "parity payload must be a JSON object".to_string(),
        })?;
    object.insert("kind".to_string(), serde_json::Value::String(kind));
    object.insert("favorite".to_string(), serde_json::Value::Bool(favorite));
    Ok(payload)
}

fn entry_payload_kind(entry: &mdbx_core::model::Entry) -> Option<String> {
    serde_json::from_slice::<serde_json::Value>(&entry.payload_ct)
        .ok()
        .and_then(|payload| {
            payload
                .get("kind")
                .and_then(|value| value.as_str())
                .map(ToString::to_string)
        })
}

fn ensure_parity_kind(
    entry: &mdbx_core::model::Entry,
    expected_kind: &str,
) -> Result<(), MdbxFfiError> {
    if entry_payload_kind(entry).as_deref() == Some(expected_kind) {
        return Ok(());
    }
    Err(StorageError::ConstraintViolation(format!(
        "entry {} is not a {} parity entry",
        entry.entry_id, expected_kind
    ))
    .into())
}

fn parity_record_from_entry(
    entry: &mdbx_core::model::Entry,
) -> Result<ParityEntryRecord, MdbxFfiError> {
    let payload: serde_json::Value = serde_json::from_slice(&entry.payload_ct)?;
    Ok(ParityEntryRecord {
        entry_id: entry.entry_id.clone(),
        project_id: entry.project_id.clone(),
        title: entry
            .title_ct
            .as_deref()
            .map(String::from_utf8_lossy)
            .map(|s| s.to_string())
            .unwrap_or_default(),
        kind: payload
            .get("kind")
            .and_then(|value| value.as_str())
            .unwrap_or_default()
            .to_string(),
        payload_json: serde_json::to_string(&payload).map_err(|e| MdbxFfiError::Serialization {
            message: e.to_string(),
        })?,
        favorite: payload_favorite(&payload),
    })
}

fn login_entry_for_project(
    conn: &VaultConnection,
    project_id: &str,
    entry_id: &str,
) -> Result<mdbx_core::model::Entry, MdbxFfiError> {
    typed_entry_for_project(conn, project_id, entry_id, EntryType::Login)
}

fn typed_entry_for_project(
    conn: &VaultConnection,
    project_id: &str,
    entry_id: &str,
    entry_type: EntryType,
) -> Result<mdbx_core::model::Entry, MdbxFfiError> {
    let entry = EntryRepo::get_by_id(conn, entry_id)?
        .ok_or_else(|| StorageError::NotFound(entry_id.to_string()))?;
    if entry.project_id != project_id {
        return Err(StorageError::ConstraintViolation(format!(
            "entry {} does not belong to project {}",
            entry_id, project_id
        ))
        .into());
    }
    if entry.entry_type != entry_type {
        return Err(StorageError::ConstraintViolation(format!(
            "entry {} is not a {} entry",
            entry_id, entry_type
        ))
        .into());
    }
    Ok(entry)
}

impl From<mdbx_storage::error::StorageError> for MdbxFfiError {
    fn from(value: mdbx_storage::error::StorageError) -> Self {
        MdbxFfiError::Storage {
            message: value.to_string(),
        }
    }
}

impl From<rusqlite::Error> for MdbxFfiError {
    fn from(value: rusqlite::Error) -> Self {
        MdbxFfiError::Storage {
            message: value.to_string(),
        }
    }
}

impl From<serde_json::Error> for MdbxFfiError {
    fn from(value: serde_json::Error) -> Self {
        MdbxFfiError::Serialization {
            message: value.to_string(),
        }
    }
}
