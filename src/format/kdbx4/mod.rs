mod dump;
mod parse;

use crate::{
    config::{CompressionConfig, InnerCipherConfig, KdfConfig, OuterCipherConfig},
    format::DatabaseVersion,
};

#[allow(unused_imports)]
pub(crate) use crate::format::kdbx4::dump::dump_kdbx4;
pub(crate) use crate::format::kdbx4::parse::{decrypt_kdbx4, parse_kdbx4};

/// Size for a master seed in bytes
pub const HEADER_MASTER_SEED_SIZE: usize = 32;

/// Header entry denoting the end of the header
pub const HEADER_END: u8 = 0;
/// Header entry denoting a comment
pub const HEADER_COMMENT: u8 = 1;
/// A UUID specifying which cipher suite should be used to encrypt the payload
pub const HEADER_OUTER_ENCRYPTION_ID: u8 = 2;
/// First byte determines compression of payload
pub const HEADER_COMPRESSION_ID: u8 = 3;
/// Master seed for deriving the master key
pub const HEADER_MASTER_SEED: u8 = 4;
/// Initialization Vector for decrypting the payload
pub const HEADER_ENCRYPTION_IV: u8 = 7;
/// Parameters for the key derivation function
pub const HEADER_KDF_PARAMS: u8 = 11;

/// Inner header entry denoting the end of the inner header
pub const INNER_HEADER_END: u8 = 0x00;
/// Inner header entry denoting the UUID of the inner cipher
pub const INNER_HEADER_RANDOM_STREAM_ID: u8 = 0x01;
/// Inner header entry denoting the key of the inner cipher
pub const INNER_HEADER_RANDOM_STREAM_KEY: u8 = 0x02;
/// Inner header entry denoting a binary attachment
pub const INNER_HEADER_BINARY_ATTACHMENTS: u8 = 0x03;

struct KDBX4OuterHeader {
    version: DatabaseVersion,
    outer_cipher_config: OuterCipherConfig,
    compression_config: CompressionConfig,
    master_seed: Vec<u8>,
    outer_iv: Vec<u8>,
    kdf_config: KdfConfig,
    kdf_seed: Vec<u8>,
}

struct KDBX4InnerHeader {
    inner_random_stream: InnerCipherConfig,
    inner_random_stream_key: Vec<u8>,
}

#[cfg(test)]
mod kdbx4_tests {
    use super::*;
    use crate::{
        config::{CompressionConfig, DatabaseConfig, InnerCipherConfig, KdfConfig, OuterCipherConfig},
        db::{group_add_child, node::*, Database, Entry, Group, HeaderAttachment},
        format::{kdbx4::dump::dump_kdbx4, KDBX4_CURRENT_MINOR_VERSION},
        key::DatabaseKey,
        rc_refcell_node,
    };

    #[cfg(feature = "challenge_response")]
    #[test]
    fn test_with_challenge_response() {
        let mut db = Database::new(DatabaseConfig::default());

        let mut root_group = Group::new("Root");
        root_group.add_child(rc_refcell_node!(Entry::default()), 0);
        root_group.add_child(rc_refcell_node!(Entry::default()), 0);
        root_group.add_child(rc_refcell_node!(Entry::default()), 0);
        db.root = rc_refcell_node!(root_group).into();

        let mut password_bytes: Vec<u8> = vec![];
        let mut password: String = "".to_string();
        password_bytes.resize(40, 0);
        getrandom::getrandom(&mut password_bytes).unwrap();
        for random_char in password_bytes {
            password += &std::char::from_u32(random_char as u32).unwrap().to_string();
        }

        let db_key =
            DatabaseKey::new()
                .with_password(&password)
                .with_challenge_response_key(crate::key::ChallengeResponseKey::LocalChallenge(
                    "0102030405060708090a0b0c0d0e0f1011121314".to_string(),
                ));

        let mut encrypted_db = Vec::new();
        dump_kdbx4(&db, &db_key, &mut encrypted_db).unwrap();

        let decrypted_db = parse_kdbx4(&encrypted_db, &db_key).unwrap();

        assert_eq!(group_get_children(&decrypted_db.root).unwrap().len(), 3);
    }

    fn test_with_config(config: DatabaseConfig) {
        let mut db = Database::new(config);

        let root_group = rc_refcell_node!(Group::new("Root"));

        let entry_with_password = rc_refcell_node!(Entry::default());
        entry_with_password.borrow_mut().set_title(Some("Demo Entry"));

        if let Some(entry) = entry_with_password.borrow_mut().as_any_mut().downcast_mut::<Entry>() {
            entry.set_password(Some("secret"));
        }

        group_add_child(&root_group, entry_with_password, 0).unwrap();
        group_add_child(&root_group, rc_refcell_node!(Entry::default()), 0).unwrap();
        group_add_child(&root_group, rc_refcell_node!(Entry::default()), 0).unwrap();
        db.root = root_group.into();

        let mut password_bytes: Vec<u8> = vec![];
        let mut password: String = "".to_string();
        password_bytes.resize(40, 0);
        getrandom::getrandom(&mut password_bytes).unwrap();
        for random_char in password_bytes {
            password += &std::char::from_u32(random_char as u32).unwrap().to_string();
        }

        let db_key = DatabaseKey::new().with_password(&password);

        let mut encrypted_db = Vec::new();
        dump_kdbx4(&db, &db_key, &mut encrypted_db).unwrap();

        let decrypted_db = parse_kdbx4(&encrypted_db, &db_key).unwrap();

        assert_eq!(group_get_children(&decrypted_db.root).unwrap().len(), 3);

        let entry = Group::get(&decrypted_db.root, &["Demo Entry"]).unwrap();
        assert_eq!(
            entry.borrow().as_any().downcast_ref::<Entry>().unwrap().get_password(),
            Some("secret")
        );
    }

    #[test]
    pub fn test_config_matrix() {
        let outer_cipher_configs = [OuterCipherConfig::AES256, OuterCipherConfig::Twofish, OuterCipherConfig::ChaCha20];

        let compression_configs = [CompressionConfig::None, CompressionConfig::GZip];

        let inner_cipher_configs = [InnerCipherConfig::Plain, InnerCipherConfig::Salsa20, InnerCipherConfig::ChaCha20];

        let kdf_configs = [
            KdfConfig::Aes { rounds: 10 },
            KdfConfig::Argon2 {
                iterations: 10,
                memory: 65536,
                parallelism: 2,
                version: argon2::Version::Version13,
            },
            KdfConfig::Argon2id {
                iterations: 10,
                memory: 65536,
                parallelism: 2,
                version: argon2::Version::Version13,
            },
        ];

        for outer_cipher_config in &outer_cipher_configs {
            for compression_config in &compression_configs {
                for inner_cipher_config in &inner_cipher_configs {
                    for kdf_config in &kdf_configs {
                        let config = DatabaseConfig {
                            version: DatabaseVersion::KDB4(KDBX4_CURRENT_MINOR_VERSION),
                            outer_cipher_config: outer_cipher_config.clone(),
                            compression_config: compression_config.clone(),
                            inner_cipher_config: inner_cipher_config.clone(),
                            kdf_config: kdf_config.clone(),
                        };

                        println!("Testing with config: {config:?}");

                        test_with_config(config);
                    }
                }
            }
        }
    }

    #[test]
    pub fn header_attachments() {
        let root_group = rc_refcell_node!(Group::new("Root"));
        group_add_child(&root_group, rc_refcell_node!(Entry::default()), 0).unwrap();

        let mut db = Database::new(DatabaseConfig::default());

        db.header_attachments = vec![
            HeaderAttachment {
                flags: 1,
                content: vec![0x01, 0x02, 0x03, 0x04],
            },
            HeaderAttachment {
                flags: 2,
                content: vec![0x04, 0x03, 0x02, 0x01],
            },
        ];

        let entry = rc_refcell_node!(Entry::default());
        entry.borrow_mut().set_title(Some("Demo entry"));
        group_add_child(&db.root, entry, 0).unwrap();

        let db_key = DatabaseKey::new().with_password("test");

        let mut encrypted_db = Vec::new();
        dump_kdbx4(&db, &db_key, &mut encrypted_db).unwrap();

        let decrypted_db = parse_kdbx4(&encrypted_db, &db_key).unwrap();

        assert_eq!(group_get_children(&decrypted_db.root).unwrap().len(), 1);

        let header_attachments = &decrypted_db.header_attachments;
        assert_eq!(header_attachments.len(), 2);
        assert_eq!(header_attachments[0].flags, 1);
        assert_eq!(header_attachments[0].content, [0x01, 0x02, 0x03, 0x04]);
    }
}
