use std::fs;

use mdbx_ios_ffi::{create_vault, open_vault, open_vault_with_security_key};
use uuid::Uuid;

#[test]
fn creates_reopens_and_reads_project_scoped_login_entry() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("GitHub".to_string()).unwrap();
    let created = vault
        .create_login_entry(
            project.project_id.clone(),
            "GitHub main login".to_string(),
            "alice".to_string(),
            "correct horse battery staple".to_string(),
            "https://github.com".to_string(),
        )
        .unwrap();
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let entries = reopened.list_entries(project.project_id).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].entry_id, created.entry_id);
    assert_eq!(entries[0].title, "GitHub main login");
    assert_eq!(entries[0].username, "alice");
    assert_eq!(entries[0].password, "correct horse battery staple");
    assert_eq!(entries[0].url, "https://github.com");

    let _ = fs::remove_file(vault_path);
}

#[test]
fn updates_reopens_and_reads_project_scoped_login_entry() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("GitHub".to_string()).unwrap();
    let created = vault
        .create_login_entry(
            project.project_id.clone(),
            "GitHub main login".to_string(),
            "alice".to_string(),
            "old-password".to_string(),
            "https://github.com".to_string(),
        )
        .unwrap();

    let updated = vault
        .update_login_entry(
            project.project_id.clone(),
            created.entry_id.clone(),
            "GitHub work login".to_string(),
            "alice@example.com".to_string(),
            "new-password".to_string(),
            "https://github.com/settings/profile".to_string(),
        )
        .unwrap();
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let entries = reopened.list_entries(project.project_id).unwrap();

    assert_eq!(updated.entry_id, created.entry_id);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].entry_id, created.entry_id);
    assert_eq!(entries[0].title, "GitHub work login");
    assert_eq!(entries[0].username, "alice@example.com");
    assert_eq!(entries[0].password, "new-password");
    assert_eq!(entries[0].url, "https://github.com/settings/profile");

    let _ = fs::remove_file(vault_path);
}

#[test]
fn favorites_reopens_and_preserves_project_scoped_login_entry() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("GitHub".to_string()).unwrap();
    let created = vault
        .create_login_entry(
            project.project_id.clone(),
            "GitHub main login".to_string(),
            "alice".to_string(),
            "old-password".to_string(),
            "https://github.com".to_string(),
        )
        .unwrap();
    assert!(!created.favorite);

    let favorited = vault
        .set_login_favorite(project.project_id.clone(), created.entry_id.clone(), true)
        .unwrap();
    assert!(favorited.favorite);

    let updated = vault
        .update_login_entry(
            project.project_id.clone(),
            created.entry_id.clone(),
            "GitHub work login".to_string(),
            "alice@example.com".to_string(),
            "new-password".to_string(),
            "https://github.com/settings/profile".to_string(),
        )
        .unwrap();
    assert!(updated.favorite);
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let entries = reopened.list_entries(project.project_id).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].entry_id, created.entry_id);
    assert_eq!(entries[0].title, "GitHub work login");
    assert_eq!(entries[0].password, "new-password");
    assert!(entries[0].favorite);

    let _ = fs::remove_file(vault_path);
}

#[test]
fn favorites_reopens_and_preserves_project_scoped_non_login_entries() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("Personal".to_string()).unwrap();

    let note = vault
        .create_note_entry(
            project.project_id.clone(),
            "Recovery codes".to_string(),
            "code-1\ncode-2".to_string(),
        )
        .unwrap();
    let totp = vault
        .create_totp_entry(
            project.project_id.clone(),
            "GitHub TOTP".to_string(),
            "JBSWY3DPEHPK3PXP".to_string(),
            "GitHub".to_string(),
            "alice".to_string(),
            30,
            6,
            "SHA1".to_string(),
            "TOTP".to_string(),
            0,
        )
        .unwrap();
    let card = vault
        .create_card_entry(
            project.project_id.clone(),
            "Everyday Visa".to_string(),
            "Alice Example".to_string(),
            "4111111111111111".to_string(),
            "12".to_string(),
            "2031".to_string(),
            "123".to_string(),
            "Monica Bank".to_string(),
            "Visa".to_string(),
            "Primary checking card".to_string(),
        )
        .unwrap();
    let identity = vault
        .create_identity_entry(
            project.project_id.clone(),
            "Passport".to_string(),
            "passport".to_string(),
            "Alice Example".to_string(),
            "P1234567".to_string(),
            "Monica Authority".to_string(),
            "US".to_string(),
            "2026-01-02".to_string(),
            "2036-01-01".to_string(),
            "Primary travel document".to_string(),
        )
        .unwrap();

    assert!(!note.favorite);
    assert!(!totp.favorite);
    assert!(!card.favorite);
    assert!(!identity.favorite);

    let note = vault
        .set_note_favorite(project.project_id.clone(), note.entry_id.clone(), true)
        .unwrap();
    let totp = vault
        .set_totp_favorite(project.project_id.clone(), totp.entry_id.clone(), true)
        .unwrap();
    let card = vault
        .set_card_favorite(project.project_id.clone(), card.entry_id.clone(), true)
        .unwrap();
    let identity = vault
        .set_identity_favorite(project.project_id.clone(), identity.entry_id.clone(), true)
        .unwrap();

    assert!(note.favorite);
    assert!(totp.favorite);
    assert!(card.favorite);
    assert!(identity.favorite);

    let updated_note = vault
        .update_note_entry(
            project.project_id.clone(),
            note.entry_id.clone(),
            "Recovery codes updated".to_string(),
            "code-3\ncode-4".to_string(),
        )
        .unwrap();
    let updated_totp = vault
        .update_totp_entry(
            project.project_id.clone(),
            totp.entry_id.clone(),
            "GitHub Work TOTP".to_string(),
            "JBSWY3DPEHPK3PXQ".to_string(),
            "GitHub".to_string(),
            "alice@example.com".to_string(),
            60,
            8,
            "SHA256".to_string(),
            "TOTP".to_string(),
            0,
        )
        .unwrap();
    let updated_card = vault
        .update_card_entry(
            project.project_id.clone(),
            card.entry_id.clone(),
            "Travel Mastercard".to_string(),
            "Alice Q. Example".to_string(),
            "5555555555554444".to_string(),
            "01".to_string(),
            "2032".to_string(),
            "456".to_string(),
            "Monica Credit Union".to_string(),
            "Mastercard".to_string(),
            "No foreign transaction fee".to_string(),
        )
        .unwrap();
    let updated_identity = vault
        .update_identity_entry(
            project.project_id.clone(),
            identity.entry_id.clone(),
            "Driver License".to_string(),
            "driver_license".to_string(),
            "Alice Q. Example".to_string(),
            "D7654321".to_string(),
            "Monica DMV".to_string(),
            "US-CA".to_string(),
            "2026-05-31".to_string(),
            "2031-05-30".to_string(),
            "State license metadata".to_string(),
        )
        .unwrap();
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let notes = reopened
        .list_note_entries(project.project_id.clone())
        .unwrap();
    let totps = reopened
        .list_totp_entries(project.project_id.clone())
        .unwrap();
    let cards = reopened
        .list_card_entries(project.project_id.clone())
        .unwrap();
    let identities = reopened
        .list_identity_entries(project.project_id.clone())
        .unwrap();

    assert!(updated_note.favorite);
    assert!(updated_totp.favorite);
    assert!(updated_card.favorite);
    assert!(updated_identity.favorite);
    assert_eq!(notes[0].body, "code-3\ncode-4");
    assert_eq!(totps[0].secret, "JBSWY3DPEHPK3PXQ");
    assert_eq!(cards[0].number, "5555555555554444");
    assert_eq!(identities[0].document_number, "D7654321");
    assert!(notes[0].favorite);
    assert!(totps[0].favorite);
    assert!(cards[0].favorite);
    assert!(identities[0].favorite);

    let _ = fs::remove_file(vault_path);
}

#[test]
fn deletes_restores_and_reopens_project_scoped_login_entry() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("GitHub".to_string()).unwrap();
    let created = vault
        .create_login_entry(
            project.project_id.clone(),
            "GitHub main login".to_string(),
            "alice".to_string(),
            "correct horse battery staple".to_string(),
            "https://github.com".to_string(),
        )
        .unwrap();

    vault
        .delete_login_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    assert!(vault
        .list_entries(project.project_id.clone())
        .unwrap()
        .is_empty());

    let deleted = vault
        .list_deleted_entries(project.project_id.clone())
        .unwrap();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].entry_id, created.entry_id);

    let restored = vault
        .restore_login_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let entries = reopened.list_entries(project.project_id.clone()).unwrap();
    let deleted_after_restore = reopened.list_deleted_entries(project.project_id).unwrap();

    assert_eq!(restored.entry_id, created.entry_id);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].entry_id, created.entry_id);
    assert_eq!(entries[0].title, "GitHub main login");
    assert!(deleted_after_restore.is_empty());

    let _ = fs::remove_file(vault_path);
}

#[test]
fn opens_vault_with_local_security_key_material_without_password() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";
    let security_key = vec![42u8; 32];

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("GitHub".to_string()).unwrap();
    vault
        .create_login_entry(
            project.project_id.clone(),
            "GitHub main login".to_string(),
            "alice".to_string(),
            "correct horse battery staple".to_string(),
            "https://github.com".to_string(),
        )
        .unwrap();
    vault
        .setup_local_security_key_unlock(security_key.clone())
        .unwrap();
    drop(vault);

    let reopened =
        open_vault_with_security_key(path.clone(), security_key, device_id.to_string()).unwrap();
    let entries = reopened.list_entries(project.project_id.clone()).unwrap();

    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].title, "GitHub main login");
    assert_eq!(entries[0].password, "correct horse battery staple");

    let wrong_key = vec![7u8; 32];
    assert!(open_vault_with_security_key(path.clone(), wrong_key, device_id.to_string()).is_err());

    let _ = fs::remove_file(vault_path);
}

#[test]
fn creates_updates_deletes_restores_and_reopens_project_scoped_note_entry() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("Personal".to_string()).unwrap();
    let created = vault
        .create_note_entry(
            project.project_id.clone(),
            "Recovery codes".to_string(),
            "code-1\ncode-2".to_string(),
        )
        .unwrap();

    let updated = vault
        .update_note_entry(
            project.project_id.clone(),
            created.entry_id.clone(),
            "Recovery codes updated".to_string(),
            "code-3\ncode-4".to_string(),
        )
        .unwrap();
    vault
        .delete_note_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    assert!(vault
        .list_note_entries(project.project_id.clone())
        .unwrap()
        .is_empty());

    let deleted = vault
        .list_deleted_note_entries(project.project_id.clone())
        .unwrap();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].entry_id, created.entry_id);
    assert_eq!(deleted[0].body, "code-3\ncode-4");

    let restored = vault
        .restore_note_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let entries = reopened
        .list_note_entries(project.project_id.clone())
        .unwrap();
    let deleted_after_restore = reopened
        .list_deleted_note_entries(project.project_id.clone())
        .unwrap();

    assert_eq!(updated.entry_id, created.entry_id);
    assert_eq!(restored.entry_id, created.entry_id);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].entry_id, created.entry_id);
    assert_eq!(entries[0].project_id, project.project_id);
    assert_eq!(entries[0].title, "Recovery codes updated");
    assert_eq!(entries[0].body, "code-3\ncode-4");
    assert!(deleted_after_restore.is_empty());

    let _ = fs::remove_file(vault_path);
}

#[test]
fn creates_updates_deletes_restores_and_reopens_project_scoped_totp_entry() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("GitHub".to_string()).unwrap();
    let created = vault
        .create_totp_entry(
            project.project_id.clone(),
            "GitHub TOTP".to_string(),
            "JBSWY3DPEHPK3PXP".to_string(),
            "GitHub".to_string(),
            "alice".to_string(),
            30,
            6,
            "SHA1".to_string(),
            "TOTP".to_string(),
            0,
        )
        .unwrap();

    let updated = vault
        .update_totp_entry(
            project.project_id.clone(),
            created.entry_id.clone(),
            "GitHub Work TOTP".to_string(),
            "JBSWY3DPEHPK3PXQ".to_string(),
            "GitHub".to_string(),
            "alice@example.com".to_string(),
            60,
            8,
            "SHA256".to_string(),
            "TOTP".to_string(),
            0,
        )
        .unwrap();
    vault
        .delete_totp_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    assert!(vault
        .list_totp_entries(project.project_id.clone())
        .unwrap()
        .is_empty());

    let deleted = vault
        .list_deleted_totp_entries(project.project_id.clone())
        .unwrap();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].entry_id, created.entry_id);
    assert_eq!(deleted[0].secret, "JBSWY3DPEHPK3PXQ");

    let restored = vault
        .restore_totp_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let entries = reopened
        .list_totp_entries(project.project_id.clone())
        .unwrap();
    let deleted_after_restore = reopened
        .list_deleted_totp_entries(project.project_id.clone())
        .unwrap();

    assert_eq!(updated.entry_id, created.entry_id);
    assert_eq!(restored.entry_id, created.entry_id);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].entry_id, created.entry_id);
    assert_eq!(entries[0].project_id, project.project_id);
    assert_eq!(entries[0].title, "GitHub Work TOTP");
    assert_eq!(entries[0].secret, "JBSWY3DPEHPK3PXQ");
    assert_eq!(entries[0].issuer, "GitHub");
    assert_eq!(entries[0].account_name, "alice@example.com");
    assert_eq!(entries[0].period, 60);
    assert_eq!(entries[0].digits, 8);
    assert_eq!(entries[0].algorithm, "SHA256");
    assert_eq!(entries[0].otp_type, "TOTP");
    assert_eq!(entries[0].counter, 0);
    assert!(deleted_after_restore.is_empty());

    let _ = fs::remove_file(vault_path);
}

#[test]
fn creates_updates_deletes_restores_and_reopens_project_scoped_card_entry() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("Bank".to_string()).unwrap();
    let created = vault
        .create_card_entry(
            project.project_id.clone(),
            "Everyday Visa".to_string(),
            "Alice Example".to_string(),
            "4111111111111111".to_string(),
            "12".to_string(),
            "2031".to_string(),
            "123".to_string(),
            "Monica Bank".to_string(),
            "Visa".to_string(),
            "Primary checking card".to_string(),
        )
        .unwrap();

    let updated = vault
        .update_card_entry(
            project.project_id.clone(),
            created.entry_id.clone(),
            "Travel Mastercard".to_string(),
            "Alice Q. Example".to_string(),
            "5555555555554444".to_string(),
            "01".to_string(),
            "2032".to_string(),
            "456".to_string(),
            "Monica Credit Union".to_string(),
            "Mastercard".to_string(),
            "No foreign transaction fee".to_string(),
        )
        .unwrap();
    vault
        .delete_card_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    assert!(vault
        .list_card_entries(project.project_id.clone())
        .unwrap()
        .is_empty());

    let deleted = vault
        .list_deleted_card_entries(project.project_id.clone())
        .unwrap();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].entry_id, created.entry_id);
    assert_eq!(deleted[0].number, "5555555555554444");
    assert_eq!(deleted[0].cvv, "456");

    let restored = vault
        .restore_card_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let entries = reopened
        .list_card_entries(project.project_id.clone())
        .unwrap();
    let deleted_after_restore = reopened
        .list_deleted_card_entries(project.project_id.clone())
        .unwrap();

    assert_eq!(updated.entry_id, created.entry_id);
    assert_eq!(restored.entry_id, created.entry_id);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].entry_id, created.entry_id);
    assert_eq!(entries[0].project_id, project.project_id);
    assert_eq!(entries[0].title, "Travel Mastercard");
    assert_eq!(entries[0].cardholder_name, "Alice Q. Example");
    assert_eq!(entries[0].number, "5555555555554444");
    assert_eq!(entries[0].expiry_month, "01");
    assert_eq!(entries[0].expiry_year, "2032");
    assert_eq!(entries[0].cvv, "456");
    assert_eq!(entries[0].issuer, "Monica Credit Union");
    assert_eq!(entries[0].network, "Mastercard");
    assert_eq!(entries[0].notes, "No foreign transaction fee");
    assert!(deleted_after_restore.is_empty());

    let _ = fs::remove_file(vault_path);
}

#[test]
fn creates_updates_deletes_restores_and_reopens_project_scoped_identity_entry() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("Identity".to_string()).unwrap();
    let created = vault
        .create_identity_entry(
            project.project_id.clone(),
            "Passport".to_string(),
            "passport".to_string(),
            "Alice Example".to_string(),
            "P1234567".to_string(),
            "Monica Authority".to_string(),
            "US".to_string(),
            "2026-01-02".to_string(),
            "2036-01-01".to_string(),
            "Primary travel document".to_string(),
        )
        .unwrap();

    let updated = vault
        .update_identity_entry(
            project.project_id.clone(),
            created.entry_id.clone(),
            "Driver License".to_string(),
            "driver_license".to_string(),
            "Alice Q. Example".to_string(),
            "D7654321".to_string(),
            "Monica DMV".to_string(),
            "US-CA".to_string(),
            "2026-05-31".to_string(),
            "2031-05-30".to_string(),
            "State license metadata".to_string(),
        )
        .unwrap();
    vault
        .delete_identity_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    assert!(vault
        .list_identity_entries(project.project_id.clone())
        .unwrap()
        .is_empty());

    let deleted = vault
        .list_deleted_identity_entries(project.project_id.clone())
        .unwrap();
    assert_eq!(deleted.len(), 1);
    assert_eq!(deleted[0].entry_id, created.entry_id);
    assert_eq!(deleted[0].document_number, "D7654321");

    let restored = vault
        .restore_identity_entry(project.project_id.clone(), created.entry_id.clone())
        .unwrap();
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let entries = reopened
        .list_identity_entries(project.project_id.clone())
        .unwrap();
    let deleted_after_restore = reopened
        .list_deleted_identity_entries(project.project_id.clone())
        .unwrap();

    assert_eq!(updated.entry_id, created.entry_id);
    assert_eq!(restored.entry_id, created.entry_id);
    assert_eq!(entries.len(), 1);
    assert_eq!(entries[0].entry_id, created.entry_id);
    assert_eq!(entries[0].project_id, project.project_id);
    assert_eq!(entries[0].title, "Driver License");
    assert_eq!(entries[0].document_type, "driver_license");
    assert_eq!(entries[0].full_name, "Alice Q. Example");
    assert_eq!(entries[0].document_number, "D7654321");
    assert_eq!(entries[0].issuer, "Monica DMV");
    assert_eq!(entries[0].country, "US-CA");
    assert_eq!(entries[0].issue_date, "2026-05-31");
    assert_eq!(entries[0].expiry_date, "2031-05-30");
    assert_eq!(entries[0].notes, "State license metadata");
    assert!(deleted_after_restore.is_empty());

    let _ = fs::remove_file(vault_path);
}

#[test]
fn creates_updates_deletes_restores_reopens_and_isolates_parity_entries() {
    let vault_path = std::env::temp_dir().join(format!("monica-ios-{}.mdbx", Uuid::new_v4()));
    let path = vault_path.to_string_lossy().to_string();
    let password = "中文 password 12345!";
    let device_id = "ios-test-device";

    let vault = create_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let project = vault.create_project("Android parity".to_string()).unwrap();

    let ssh = vault
        .create_parity_entry(
            project.project_id.clone(),
            "ssh-key".to_string(),
            "ssh-key".to_string(),
            "MacBook deploy key".to_string(),
            r#"{"username":"deploy","publicKey":"ssh-ed25519 AAAA","privateKey":"private-1"}"#
                .to_string(),
        )
        .unwrap();
    let api_token = vault
        .create_parity_entry(
            project.project_id.clone(),
            "api-token".to_string(),
            "api-token".to_string(),
            "Tiga API token".to_string(),
            r#"{"issuer":"Tiga","token":"tok_live_1","scopes":["sync","read"]}"#.to_string(),
        )
        .unwrap();
    let wifi = vault
        .create_parity_entry(
            project.project_id.clone(),
            "document-ref".to_string(),
            "wifi".to_string(),
            "Studio Wi-Fi".to_string(),
            r#"{"ssid":"Monica Studio","security":"WPA2","password":"wifi-secret"}"#.to_string(),
        )
        .unwrap();
    let send = vault
        .create_parity_entry(
            project.project_id.clone(),
            "document-ref".to_string(),
            "send".to_string(),
            "One-time send".to_string(),
            r#"{"recipient":"alice@example.com","secret":"shared-once"}"#.to_string(),
        )
        .unwrap();

    assert_eq!(ssh.kind, "ssh-key");
    assert_eq!(api_token.kind, "api-token");
    assert_eq!(wifi.kind, "wifi");
    assert_eq!(send.kind, "send");
    assert!(!ssh.favorite);

    let ssh_entries = vault
        .list_parity_entries(
            project.project_id.clone(),
            "ssh-key".to_string(),
            "ssh-key".to_string(),
        )
        .unwrap();
    let api_token_entries = vault
        .list_parity_entries(
            project.project_id.clone(),
            "api-token".to_string(),
            "api-token".to_string(),
        )
        .unwrap();
    let wifi_entries = vault
        .list_parity_entries(
            project.project_id.clone(),
            "document-ref".to_string(),
            "wifi".to_string(),
        )
        .unwrap();
    let send_entries = vault
        .list_parity_entries(
            project.project_id.clone(),
            "document-ref".to_string(),
            "send".to_string(),
        )
        .unwrap();

    assert_eq!(ssh_entries.len(), 1);
    assert_eq!(api_token_entries.len(), 1);
    assert_eq!(wifi_entries.len(), 1);
    assert_eq!(send_entries.len(), 1);
    assert_eq!(ssh_entries[0].entry_id, ssh.entry_id);
    assert_eq!(api_token_entries[0].entry_id, api_token.entry_id);
    assert_eq!(wifi_entries[0].entry_id, wifi.entry_id);
    assert_eq!(send_entries[0].entry_id, send.entry_id);

    assert!(vault
        .update_parity_entry(
            project.project_id.clone(),
            ssh.entry_id.clone(),
            "api-token".to_string(),
            "api-token".to_string(),
            "Wrong type".to_string(),
            r#"{"token":"should-not-write"}"#.to_string(),
        )
        .is_err());
    assert!(vault
        .update_parity_entry(
            project.project_id.clone(),
            wifi.entry_id.clone(),
            "document-ref".to_string(),
            "send".to_string(),
            "Wrong kind".to_string(),
            r#"{"secret":"should-not-write"}"#.to_string(),
        )
        .is_err());

    let favorited = vault
        .set_parity_entry_favorite(
            project.project_id.clone(),
            ssh.entry_id.clone(),
            "ssh-key".to_string(),
            "ssh-key".to_string(),
            true,
        )
        .unwrap();
    assert!(favorited.favorite);

    let updated_ssh = vault
        .update_parity_entry(
            project.project_id.clone(),
            ssh.entry_id.clone(),
            "ssh-key".to_string(),
            "ssh-key".to_string(),
            "MacBook deploy key rotated".to_string(),
            r#"{"username":"deploy","publicKey":"ssh-ed25519 BBBB","privateKey":"private-2"}"#
                .to_string(),
        )
        .unwrap();
    assert_eq!(updated_ssh.entry_id, ssh.entry_id);
    assert_eq!(updated_ssh.title, "MacBook deploy key rotated");
    assert!(updated_ssh.favorite);

    vault
        .delete_parity_entry(
            project.project_id.clone(),
            wifi.entry_id.clone(),
            "document-ref".to_string(),
            "wifi".to_string(),
        )
        .unwrap();
    assert!(vault
        .list_parity_entries(
            project.project_id.clone(),
            "document-ref".to_string(),
            "wifi".to_string(),
        )
        .unwrap()
        .is_empty());
    assert_eq!(
        vault
            .list_parity_entries(
                project.project_id.clone(),
                "document-ref".to_string(),
                "send".to_string(),
            )
            .unwrap()
            .len(),
        1
    );

    let deleted_wifi = vault
        .list_deleted_parity_entries(
            project.project_id.clone(),
            "document-ref".to_string(),
            "wifi".to_string(),
        )
        .unwrap();
    assert_eq!(deleted_wifi.len(), 1);
    assert_eq!(deleted_wifi[0].entry_id, wifi.entry_id);
    assert!(vault
        .restore_parity_entry(
            project.project_id.clone(),
            wifi.entry_id.clone(),
            "document-ref".to_string(),
            "send".to_string(),
        )
        .is_err());

    let restored_wifi = vault
        .restore_parity_entry(
            project.project_id.clone(),
            wifi.entry_id.clone(),
            "document-ref".to_string(),
            "wifi".to_string(),
        )
        .unwrap();
    assert_eq!(restored_wifi.entry_id, wifi.entry_id);
    drop(vault);

    let reopened = open_vault(path.clone(), password.to_string(), device_id.to_string()).unwrap();
    let reopened_ssh = reopened
        .list_parity_entries(
            project.project_id.clone(),
            "ssh-key".to_string(),
            "ssh-key".to_string(),
        )
        .unwrap();
    let reopened_wifi = reopened
        .list_parity_entries(
            project.project_id.clone(),
            "document-ref".to_string(),
            "wifi".to_string(),
        )
        .unwrap();
    let deleted_wifi_after_restore = reopened
        .list_deleted_parity_entries(
            project.project_id.clone(),
            "document-ref".to_string(),
            "wifi".to_string(),
        )
        .unwrap();
    let ssh_payload: serde_json::Value = serde_json::from_str(&reopened_ssh[0].payload_json)
        .expect("reopened ssh parity payload must be valid JSON");
    let wifi_payload: serde_json::Value = serde_json::from_str(&reopened_wifi[0].payload_json)
        .expect("reopened wifi parity payload must be valid JSON");

    assert_eq!(reopened_ssh.len(), 1);
    assert_eq!(reopened_ssh[0].entry_id, ssh.entry_id);
    assert_eq!(reopened_ssh[0].title, "MacBook deploy key rotated");
    assert_eq!(reopened_ssh[0].kind, "ssh-key");
    assert!(reopened_ssh[0].favorite);
    assert_eq!(ssh_payload["publicKey"], "ssh-ed25519 BBBB");
    assert_eq!(ssh_payload["favorite"], true);
    assert_eq!(ssh_payload["kind"], "ssh-key");

    assert_eq!(reopened_wifi.len(), 1);
    assert_eq!(reopened_wifi[0].entry_id, wifi.entry_id);
    assert_eq!(wifi_payload["ssid"], "Monica Studio");
    assert_eq!(wifi_payload["kind"], "wifi");
    assert!(deleted_wifi_after_restore.is_empty());

    let _ = fs::remove_file(vault_path);
}
