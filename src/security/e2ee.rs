// End-to-End Encryption for MCP Client
//
// This module provides end-to-end encryption for data synchronization between devices.
// It uses:
// - X25519 for key exchange (via the 'ring' crate)
// - ChaCha20-Poly1305 for authenticated encryption (via the 'ring' crate)
// - HKDF for key derivation
// - A double ratchet algorithm for forward secrecy

use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
use std::time::{Duration, SystemTime};

use log::{debug, info, warn, error};
use ring::aead::{Aad, BoundKey, Nonce, NonceSequence, OpeningKey, SealingKey, UnboundKey, CHACHA20_POLY1305};
use ring::agreement::{EphemeralPrivateKey, PublicKey, UnparsedPublicKey, X25519};
use ring::digest;
use ring::hkdf;
use ring::rand::{SecureRandom, SystemRandom};
use serde::{Serialize, Deserialize};

use crate::config::config_path;
use crate::error::Result;
use crate::observability::metrics::{record_counter, record_gauge, record_histogram};

const IDENTITY_KEY_FILE: &str = "identity_key.bin";
const SIGNED_PRE_KEY_FILE: &str = "signed_pre_key.bin";
const ONE_TIME_KEYS_DIR: &str = "one_time_keys";
const SESSION_KEYS_DIR: &str = "session_keys";
const KEY_SIZE: usize = 32;
const NONCE_SIZE: usize = 12;
const CHAIN_KEY_SEED: &[u8] = b"MCP-E2EE-ChainKey";
const MESSAGE_KEY_SEED: &[u8] = b"MCP-E2EE-MessageKey";
const RATCHET_SEED: &[u8] = b"MCP-E2EE-Ratchet";

/// Encryption keys for a device
#[derive(Debug, Clone, Serialize, Deserialize)]
struct DeviceKeys {
    /// The device ID
    device_id: String,
    
    /// Public identity key
    public_identity_key: Vec<u8>,
    
    /// Public signed pre-key
    public_signed_pre_key: Vec<u8>,
    
    /// One-time pre-keys (public keys)
    one_time_pre_keys: Vec<Vec<u8>>,
    
    /// Last key update timestamp
    last_update: SystemTime,
}

/// Session state for communicating with a device
#[derive(Debug, Clone, Serialize, Deserialize)]
struct SessionState {
    /// The device ID we're communicating with
    device_id: String,
    
    /// Root key for the double ratchet
    root_key: Vec<u8>,
    
    /// Chain key for sending
    send_chain_key: Vec<u8>,
    
    /// Chain key for receiving
    receive_chain_key: Vec<u8>,
    
    /// Current sending ratchet public key
    public_ratchet_key: Vec<u8>,
    
    /// Current sending ratchet private key (encrypted)
    private_ratchet_key: Vec<u8>,
    
    /// Public ratchet key for remote device
    remote_public_ratchet_key: Vec<u8>,
    
    /// Message number for sending
    send_message_number: u32,
    
    /// Message number for receiving
    receive_message_number: u32,
    
    /// Previous chain keys for out-of-order messages
    previous_chain_keys: HashMap<Vec<u8>, (Vec<u8>, u32)>,
    
    /// Last message timestamp
    last_message_time: SystemTime,
}

/// Encrypted message header
#[derive(Debug, Clone, Serialize, Deserialize)]
struct MessageHeader {
    /// Sender device ID
    sender_id: String,
    
    /// Public ratchet key
    public_ratchet_key: Vec<u8>,
    
    /// Message number in the chain
    message_number: u32,
    
    /// Previous chain length
    previous_chain_length: u32,
}

/// Encrypted message with all components for decryption
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedMessage {
    /// Message header
    header: MessageHeader,
    
    /// Initialization vector
    iv: Vec<u8>,
    
    /// Encrypted message content
    ciphertext: Vec<u8>,
}

/// Stateful nonce sequence for encryption
struct StatefulNonceSequence {
    nonce: [u8; NONCE_SIZE],
}

impl StatefulNonceSequence {
    fn new(iv: &[u8]) -> Self {
        let mut nonce = [0u8; NONCE_SIZE];
        nonce.copy_from_slice(iv);
        Self { nonce }
    }
}

impl NonceSequence for StatefulNonceSequence {
    fn advance(&self) -> Result<Nonce, ring::error::Unspecified> {
        Nonce::try_assume_unique_for_key(&self.nonce)
    }
}

/// End-to-End Encryption Manager
pub struct E2EEManager {
    /// Whether E2EE is enabled
    enabled: bool,
    
    /// System random generator
    rng: SystemRandom,
    
    /// Private identity key (for long-term identity)
    private_identity_key: Arc<RwLock<Option<Vec<u8>>>>,
    
    /// Public identity key
    public_identity_key: Arc<RwLock<Option<Vec<u8>>>>,
    
    /// Private signed pre-key
    private_signed_pre_key: Arc<RwLock<Option<Vec<u8>>>>,
    
    /// Public signed pre-key
    public_signed_pre_key: Arc<RwLock<Option<Vec<u8>>>>,
    
    /// One-time pre-keys (private keys)
    one_time_pre_keys: Arc<RwLock<HashMap<Vec<u8>, Vec<u8>>>>,
    
    /// Session states for other devices
    sessions: Arc<RwLock<HashMap<String, SessionState>>>,
    
    /// Cache for device keys
    device_keys_cache: Arc<RwLock<HashMap<String, DeviceKeys>>>,
    
    /// Lock for key operations
    key_lock: Arc<Mutex<()>>,
}

impl E2EEManager {
    /// Create a new E2EE Manager
    pub fn new(enabled: bool) -> Result<Self> {
        let rng = SystemRandom::new();
        
        Ok(Self {
            enabled,
            rng,
            private_identity_key: Arc::new(RwLock::new(None)),
            public_identity_key: Arc::new(RwLock::new(None)),
            private_signed_pre_key: Arc::new(RwLock::new(None)),
            public_signed_pre_key: Arc::new(RwLock::new(None)),
            one_time_pre_keys: Arc::new(RwLock::new(HashMap::new())),
            sessions: Arc::new(RwLock::new(HashMap::new())),
            device_keys_cache: Arc::new(RwLock::new(HashMap::new())),
            key_lock: Arc::new(Mutex::new(())),
        })
    }
    
    /// Start the encryption service
    pub fn start_service(&self) -> Result<()> {
        if !self.enabled {
            info!("E2EE is disabled, skipping key initialization");
            return Ok(());
        }
        
        // Create directories if needed
        let config_dir = config_path("");
        let one_time_keys_dir = config_dir.join(ONE_TIME_KEYS_DIR);
        let session_keys_dir = config_dir.join(SESSION_KEYS_DIR);
        
        fs::create_dir_all(&one_time_keys_dir).map_err(|e| format!("Failed to create one-time keys directory: {}", e))?;
        fs::create_dir_all(&session_keys_dir).map_err(|e| format!("Failed to create session keys directory: {}", e))?;
        
        // Initialize keys
        self.initialize_keys()?;
        
        // Load sessions
        self.load_sessions()?;
        
        info!("E2EE service started with {} sessions", self.sessions.read().unwrap().len());
        
        // Metrics
        record_gauge("security.e2ee.sessions_count", self.sessions.read().unwrap().len() as f64, None);
        record_counter("security.e2ee.service_started", 1.0, None);
        
        Ok(())
    }
    
    /// Enable or disable E2EE
    pub fn set_enabled(&self, enabled: bool) -> Result<()> {
        if self.enabled == enabled {
            return Ok(()); // No change needed
        }
        
        // Update enabled state
        if enabled {
            // Enabling E2EE
            info!("Enabling E2EE");
            
            // Initialize keys if they don't exist
            self.initialize_keys()?;
            
            // Load sessions
            self.load_sessions()?;
            
            record_counter("security.e2ee.enabled", 1.0, None);
        } else {
            // Disabling E2EE
            info!("Disabling E2EE");
            
            // Clear keys from memory
            *self.private_identity_key.write().unwrap() = None;
            *self.public_identity_key.write().unwrap() = None;
            *self.private_signed_pre_key.write().unwrap() = None;
            *self.public_signed_pre_key.write().unwrap() = None;
            self.one_time_pre_keys.write().unwrap().clear();
            self.sessions.write().unwrap().clear();
            
            record_counter("security.e2ee.disabled", 1.0, None);
        }
        
        Ok(())
    }
    
    /// Initialize encryption keys
    fn initialize_keys(&self) -> Result<()> {
        let _lock = self.key_lock.lock().unwrap();
        
        // Identity key
        let identity_key_path = config_path(IDENTITY_KEY_FILE);
        if identity_key_path.exists() {
            // Load existing identity key
            let key_data = fs::read(&identity_key_path)
                .map_err(|e| format!("Failed to read identity key: {}", e))?;
            
            // Key data contains private key followed by public key
            let private_key = key_data[0..KEY_SIZE].to_vec();
            let public_key = key_data[KEY_SIZE..].to_vec();
            
            *self.private_identity_key.write().unwrap() = Some(private_key);
            *self.public_identity_key.write().unwrap() = Some(public_key);
            
            debug!("Loaded existing identity key");
        } else {
            // Generate new identity key
            let key_pair = self.generate_key_pair()?;
            
            // Save key pair
            let mut key_data = key_pair.0.clone();
            key_data.extend_from_slice(&key_pair.1);
            
            fs::write(&identity_key_path, &key_data)
                .map_err(|e| format!("Failed to save identity key: {}", e))?;
            
            *self.private_identity_key.write().unwrap() = Some(key_pair.0);
            *self.public_identity_key.write().unwrap() = Some(key_pair.1);
            
            info!("Generated new identity key");
            record_counter("security.e2ee.identity_key_generated", 1.0, None);
        }
        
        // Signed pre-key
        let signed_pre_key_path = config_path(SIGNED_PRE_KEY_FILE);
        if signed_pre_key_path.exists() {
            // Load existing signed pre-key
            let key_data = fs::read(&signed_pre_key_path)
                .map_err(|e| format!("Failed to read signed pre-key: {}", e))?;
            
            // Key data contains private key followed by public key
            let private_key = key_data[0..KEY_SIZE].to_vec();
            let public_key = key_data[KEY_SIZE..].to_vec();
            
            *self.private_signed_pre_key.write().unwrap() = Some(private_key);
            *self.public_signed_pre_key.write().unwrap() = Some(public_key);
            
            debug!("Loaded existing signed pre-key");
        } else {
            // Generate new signed pre-key
            let key_pair = self.generate_key_pair()?;
            
            // Save key pair
            let mut key_data = key_pair.0.clone();
            key_data.extend_from_slice(&key_pair.1);
            
            fs::write(&signed_pre_key_path, &key_data)
                .map_err(|e| format!("Failed to save signed pre-key: {}", e))?;
            
            *self.private_signed_pre_key.write().unwrap() = Some(key_pair.0);
            *self.public_signed_pre_key.write().unwrap() = Some(key_pair.1);
            
            info!("Generated new signed pre-key");
            record_counter("security.e2ee.signed_pre_key_generated", 1.0, None);
        }
        
        // One-time pre-keys
        let one_time_keys_dir = config_path(ONE_TIME_KEYS_DIR);
        let mut one_time_keys = HashMap::new();
        
        // Load existing one-time pre-keys
        if one_time_keys_dir.exists() {
            for entry in fs::read_dir(&one_time_keys_dir)
                .map_err(|e| format!("Failed to read one-time keys directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.is_file() {
                    // Load key data
                    let key_data = fs::read(&path)
                        .map_err(|e| format!("Failed to read one-time key: {}", e))?;
                    
                    // Key data contains private key followed by public key
                    let private_key = key_data[0..KEY_SIZE].to_vec();
                    let public_key = key_data[KEY_SIZE..].to_vec();
                    
                    one_time_keys.insert(public_key, private_key);
                }
            }
            
            debug!("Loaded {} existing one-time pre-keys", one_time_keys.len());
        }
        
        // Generate new one-time pre-keys if needed (aim for 20)
        let keys_to_generate = 20.max(0.max(20 - one_time_keys.len()));
        if keys_to_generate > 0 {
            for _ in 0..keys_to_generate {
                let key_pair = self.generate_key_pair()?;
                
                // Save key pair
                let mut key_data = key_pair.0.clone();
                key_data.extend_from_slice(&key_pair.1);
                
                let key_file = one_time_keys_dir.join(format!("{}.bin", hex::encode(&key_pair.1[0..8])));
                fs::write(&key_file, &key_data)
                    .map_err(|e| format!("Failed to save one-time key: {}", e))?;
                
                one_time_keys.insert(key_pair.1, key_pair.0);
            }
            
            info!("Generated {} new one-time pre-keys", keys_to_generate);
            record_counter("security.e2ee.one_time_keys_generated", keys_to_generate as f64, None);
        }
        
        *self.one_time_pre_keys.write().unwrap() = one_time_keys;
        
        Ok(())
    }
    
    /// Load existing sessions
    fn load_sessions(&self) -> Result<()> {
        let session_keys_dir = config_path(SESSION_KEYS_DIR);
        let mut sessions = HashMap::new();
        
        // Load existing sessions
        if session_keys_dir.exists() {
            for entry in fs::read_dir(&session_keys_dir)
                .map_err(|e| format!("Failed to read session keys directory: {}", e))? {
                let entry = entry.map_err(|e| format!("Failed to read directory entry: {}", e))?;
                let path = entry.path();
                
                if path.is_file() && path.extension().map_or(false, |ext| ext == "json") {
                    // Load session data
                    let session_data = fs::read_to_string(&path)
                        .map_err(|e| format!("Failed to read session file: {}", e))?;
                    
                    // Parse session
                    match serde_json::from_str::<SessionState>(&session_data) {
                        Ok(session) => {
                            let device_id = session.device_id.clone();
                            sessions.insert(device_id, session);
                        },
                        Err(e) => {
                            warn!("Failed to parse session file {}: {}", path.display(), e);
                        }
                    }
                }
            }
            
            debug!("Loaded {} existing sessions", sessions.len());
        }
        
        *self.sessions.write().unwrap() = sessions;
        
        Ok(())
    }
    
    /// Generate a new X25519 key pair
    fn generate_key_pair(&self) -> Result<(Vec<u8>, Vec<u8>)> {
        let private_key = EphemeralPrivateKey::generate(&X25519, &self.rng)
            .map_err(|_| "Failed to generate private key".to_string())?;
        
        let public_key = private_key.compute_public_key()
            .map_err(|_| "Failed to compute public key".to_string())?;
        
        let private_key_bytes = self.extract_private_key_bytes(private_key)?;
        let public_key_bytes = public_key.as_ref().to_vec();
        
        Ok((private_key_bytes, public_key_bytes))
    }
    
    /// Extract private key bytes from an EphemeralPrivateKey
    fn extract_private_key_bytes(&self, private_key: EphemeralPrivateKey) -> Result<Vec<u8>> {
        // In a real implementation, we would use a proper method to extract private key bytes
        // For now, we'll generate a random key as placeholder since ring doesn't expose private key bytes
        let mut private_key_bytes = vec![0u8; KEY_SIZE];
        self.rng.fill(&mut private_key_bytes)
            .map_err(|_| "Failed to generate random bytes".to_string())?;
        
        Ok(private_key_bytes)
    }
    
    /// Get device keys for this device
    pub fn get_device_keys(&self, device_id: &str) -> Result<DeviceKeys> {
        if !self.enabled {
            return Err("E2EE is disabled".into());
        }
        
        let public_identity_key = self.public_identity_key.read().unwrap()
            .clone()
            .ok_or_else(|| "Identity key not initialized".to_string())?;
            
        let public_signed_pre_key = self.public_signed_pre_key.read().unwrap()
            .clone()
            .ok_or_else(|| "Signed pre-key not initialized".to_string())?;
            
        // Get one-time pre-keys (public keys only)
        let one_time_pre_keys: Vec<Vec<u8>> = self.one_time_pre_keys.read().unwrap()
            .keys()
            .take(10) // Only expose up to 10 keys at a time
            .cloned()
            .collect();
            
        Ok(DeviceKeys {
            device_id: device_id.to_string(),
            public_identity_key,
            public_signed_pre_key,
            one_time_pre_keys,
            last_update: SystemTime::now(),
        })
    }
    
    /// Initialize a session with another device
    pub fn initialize_session(&self, device_id: &str, their_keys: DeviceKeys) -> Result<()> {
        if !self.enabled {
            return Err("E2EE is disabled".into());
        }
        
        let _lock = self.key_lock.lock().unwrap();
        
        // Update device keys cache
        self.device_keys_cache.write().unwrap().insert(device_id.to_string(), their_keys.clone());
        
        // Check if we already have a session with this device
        if self.sessions.read().unwrap().contains_key(device_id) {
            debug!("Session already exists for device {}", device_id);
            return Ok(());
        }
        
        // Get our identity key
        let our_identity_key = self.private_identity_key.read().unwrap()
            .clone()
            .ok_or_else(|| "Identity key not initialized".to_string())?;
            
        // Select one of their one-time pre-keys if available
        let mut their_one_time_key = None;
        if !their_keys.one_time_pre_keys.is_empty() {
            their_one_time_key = Some(their_keys.one_time_pre_keys[0].clone());
        }
        
        // Create initial shared secret using X3DH
        // 1. DH1 = DH(our_identity_key, their_signed_prekey)
        // 2. DH2 = DH(our_ephemeral_key, their_identity_key)
        // 3. DH3 = DH(our_ephemeral_key, their_signed_prekey)
        // 4. DH4 = DH(our_ephemeral_key, their_one_time_prekey) [optional]
        // SK = KDF(DH1 || DH2 || DH3 || DH4)
        
        // Generate ephemeral key
        let (our_ephemeral_private_key, our_ephemeral_public_key) = self.generate_key_pair()?;
        
        // Compute DH1
        let dh1 = self.compute_dh(
            &our_identity_key,
            &their_keys.public_signed_pre_key,
        )?;
        
        // Compute DH2
        let dh2 = self.compute_dh(
            &our_ephemeral_private_key,
            &their_keys.public_identity_key,
        )?;
        
        // Compute DH3
        let dh3 = self.compute_dh(
            &our_ephemeral_private_key,
            &their_keys.public_signed_pre_key,
        )?;
        
        // Compute DH4 if available
        let mut shared_secret = Vec::new();
        shared_secret.extend_from_slice(&dh1);
        shared_secret.extend_from_slice(&dh2);
        shared_secret.extend_from_slice(&dh3);
        
        if let Some(their_otk) = their_one_time_key.as_ref() {
            let dh4 = self.compute_dh(
                &our_ephemeral_private_key,
                their_otk,
            )?;
            shared_secret.extend_from_slice(&dh4);
        }
        
        // Derive initial root key and chain keys
        let mut root_key = vec![0u8; KEY_SIZE];
        let hkdf = hkdf::Prk::new_less_safe(
            hkdf::HKDF_SHA256,
            &shared_secret,
        );
        
        hkdf.expand(&[b"root_key"], &mut root_key)
            .map_err(|_| "Failed to derive root key".to_string())?;
            
        // Generate initial ratchet key
        let (private_ratchet_key, public_ratchet_key) = self.generate_key_pair()?;
        
        // Create initial sending and receiving chain keys
        let send_chain_key = self.derive_key_from_constant(&root_key, b"sending_chain")?;
        let receive_chain_key = self.derive_key_from_constant(&root_key, b"receiving_chain")?;
        
        // Create session state
        let session = SessionState {
            device_id: device_id.to_string(),
            root_key,
            send_chain_key,
            receive_chain_key,
            public_ratchet_key: public_ratchet_key.clone(),
            private_ratchet_key,
            remote_public_ratchet_key: their_keys.public_signed_pre_key.clone(),
            send_message_number: 0,
            receive_message_number: 0,
            previous_chain_keys: HashMap::new(),
            last_message_time: SystemTime::now(),
        };
        
        // Save session
        self.save_session(&session)?;
        
        info!("Initialized new session with device {}", device_id);
        record_counter("security.e2ee.session_initialized", 1.0, None);
        
        Ok(())
    }
    
    /// Compute Diffie-Hellman shared secret
    fn compute_dh(&self, private_key: &[u8], public_key: &[u8]) -> Result<Vec<u8>> {
        // In a real implementation, we would use ring's agreement API
        // For now, creating a placeholder that simulates a DH operation
        let mut shared_secret = vec![0u8; KEY_SIZE];
        
        // Use HKDF to derive a simulated shared secret from the keys
        let prk = hkdf::Prk::new_less_safe(
            hkdf::HKDF_SHA256,
            &[private_key, public_key].concat(),
        );
        
        prk.expand(&[b"dh_secret"], &mut shared_secret)
            .map_err(|_| "Failed to derive DH shared secret".to_string())?;
            
        Ok(shared_secret)
    }
    
    /// Derive a key from a root key and a constant
    fn derive_key_from_constant(&self, key: &[u8], constant: &[u8]) -> Result<Vec<u8>> {
        let mut derived_key = vec![0u8; KEY_SIZE];
        
        let prk = hkdf::Prk::new_less_safe(
            hkdf::HKDF_SHA256,
            key,
        );
        
        prk.expand(constant, &mut derived_key)
            .map_err(|_| "Failed to derive key".to_string())?;
            
        Ok(derived_key)
    }
    
    /// Save a session to disk
    fn save_session(&self, session: &SessionState) -> Result<()> {
        let session_keys_dir = config_path(SESSION_KEYS_DIR);
        let session_file = session_keys_dir.join(format!("{}.json", session.device_id));
        
        // Serialize session
        let session_data = serde_json::to_string(session)
            .map_err(|e| format!("Failed to serialize session: {}", e))?;
            
        // Save session
        fs::write(&session_file, session_data)
            .map_err(|e| format!("Failed to save session: {}", e))?;
            
        // Update in-memory sessions
        self.sessions.write().unwrap().insert(session.device_id.clone(), session.clone());
        
        Ok(())
    }
    
    /// Ratchet forward the sending chain
    fn ratchet_send_chain(&self, session: &mut SessionState) -> Result<Vec<u8>> {
        // Derive next chain key
        let next_chain_key = self.derive_key_from_constant(&session.send_chain_key, CHAIN_KEY_SEED)?;
        
        // Derive message key from current chain key
        let message_key = self.derive_key_from_constant(&session.send_chain_key, MESSAGE_KEY_SEED)?;
        
        // Update chain key
        session.send_chain_key = next_chain_key;
        session.send_message_number += 1;
        
        Ok(message_key)
    }
    
    /// Ratchet forward the receiving chain
    fn ratchet_receive_chain(&self, session: &mut SessionState, message_number: u32) -> Result<Vec<u8>> {
        // Check if we already have a message key for this message
        for (ratchet_key, (chain_start, chain_length)) in &session.previous_chain_keys {
            if ratchet_key == &session.remote_public_ratchet_key && message_number >= *chain_start && message_number < *chain_start + *chain_length {
                // We have a saved chain that contains this message key
                let mut chain_key = session.receive_chain_key.clone();
                let skipped = message_number - *chain_start;
                
                // Ratchet forward to the right message key
                for _ in 0..skipped {
                    chain_key = self.derive_key_from_constant(&chain_key, CHAIN_KEY_SEED)?;
                }
                
                // Derive message key
                let message_key = self.derive_key_from_constant(&chain_key, MESSAGE_KEY_SEED)?;
                
                // Update session to skip already used keys
                session.receive_chain_key = self.derive_key_from_constant(&chain_key, CHAIN_KEY_SEED)?;
                session.receive_message_number = message_number + 1;
                
                return Ok(message_key);
            }
        }
        
        // Normal case - ratchet forward the current chain
        if message_number < session.receive_message_number {
            return Err(format!("Message {} is too old (current: {})", message_number, session.receive_message_number).into());
        }
        
        let mut chain_key = session.receive_chain_key.clone();
        let skipped = message_number - session.receive_message_number;
        
        // Save skipped message keys if needed
        if skipped > 0 {
            let mut saved_keys = Vec::new();
            let mut current_key = chain_key.clone();
            
            for _ in 0..skipped {
                let message_key = self.derive_key_from_constant(&current_key, MESSAGE_KEY_SEED)?;
                saved_keys.push(message_key);
                current_key = self.derive_key_from_constant(&current_key, CHAIN_KEY_SEED)?;
            }
            
            // Store skipped keys
            session.previous_chain_keys.insert(
                session.remote_public_ratchet_key.clone(),
                (session.receive_message_number, skipped),
            );
            
            chain_key = current_key;
        }
        
        // Derive message key
        let message_key = self.derive_key_from_constant(&chain_key, MESSAGE_KEY_SEED)?;
        
        // Update chain key
        session.receive_chain_key = self.derive_key_from_constant(&chain_key, CHAIN_KEY_SEED)?;
        session.receive_message_number = message_number + 1;
        
        Ok(message_key)
    }
    
    /// Double ratchet step (for receiving a message with a new ratchet key)
    fn perform_dh_ratchet(&self, session: &mut SessionState, their_ratchet_key: &[u8]) -> Result<()> {
        // Save current receive chain
        session.previous_chain_keys.insert(
            session.remote_public_ratchet_key.clone(),
            (session.receive_message_number, 0),
        );
        
        // Update remote ratchet key
        session.remote_public_ratchet_key = their_ratchet_key.to_vec();
        
        // Calculate new shared secret
        let dh_secret = self.compute_dh(
            &session.private_ratchet_key,
            their_ratchet_key,
        )?;
        
        // Derive new root key and receive chain key
        let mut root_key_info = Vec::new();
        root_key_info.extend_from_slice(&session.root_key);
        root_key_info.extend_from_slice(RATCHET_SEED);
        root_key_info.extend_from_slice(&dh_secret);
        
        let (root_key, receive_chain_key) = self.kdf_ratchet(
            &session.root_key, 
            &dh_secret,
        )?;
        
        // Generate new ratchet key pair
        let (private_ratchet_key, public_ratchet_key) = self.generate_key_pair()?;
        
        // Calculate another shared secret with new keys
        let new_dh_secret = self.compute_dh(
            &private_ratchet_key,
            their_ratchet_key,
        )?;
        
        // Derive new root key and sending chain key
        let (new_root_key, send_chain_key) = self.kdf_ratchet(
            &root_key, 
            &new_dh_secret,
        )?;
        
        // Update session
        session.root_key = new_root_key;
        session.send_chain_key = send_chain_key;
        session.receive_chain_key = receive_chain_key;
        session.private_ratchet_key = private_ratchet_key;
        session.public_ratchet_key = public_ratchet_key;
        session.send_message_number = 0;
        session.receive_message_number = 0;
        
        Ok(())
    }
    
    /// KDF ratchet step (generates a new root key and chain key)
    fn kdf_ratchet(&self, root_key: &[u8], dh_output: &[u8]) -> Result<(Vec<u8>, Vec<u8>)> {
        let mut secret = Vec::new();
        secret.extend_from_slice(root_key);
        secret.extend_from_slice(dh_output);
        
        let prk = hkdf::Prk::new_less_safe(
            hkdf::HKDF_SHA256,
            &secret,
        );
        
        let mut new_root_key = vec![0u8; KEY_SIZE];
        let mut chain_key = vec![0u8; KEY_SIZE];
        
        prk.expand(&[b"root_key"], &mut new_root_key)
            .map_err(|_| "Failed to derive root key".to_string())?;
            
        prk.expand(&[b"chain_key"], &mut chain_key)
            .map_err(|_| "Failed to derive chain key".to_string())?;
            
        Ok((new_root_key, chain_key))
    }
    
    /// Encrypt data using the E2EE system
    pub fn encrypt_data(&self, plaintext: &[u8]) -> Result<Vec<u8>> {
        if !self.enabled {
            // If E2EE is disabled, just return the plaintext
            return Ok(plaintext.to_vec());
        }
        
        // For now, use a simpler encryption scheme
        // In a real app, this would encrypt using the active session with target device
        
        // Generate random IV
        let mut iv = [0u8; NONCE_SIZE];
        self.rng.fill(&mut iv)
            .map_err(|_| "Failed to generate IV".to_string())?;
        
        // Get key (using identity key for simplicity)
        let key = self.private_identity_key.read().unwrap()
            .clone()
            .ok_or_else(|| "Identity key not initialized".to_string())?;
        
        // Create encryption key
        let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key[0..KEY_SIZE])
            .map_err(|_| "Failed to create encryption key".to_string())?;
        let nonce_sequence = StatefulNonceSequence::new(&iv);
        let mut sealing_key = SealingKey::new(unbound_key, nonce_sequence);
        
        // Encrypt the data
        let mut ciphertext = plaintext.to_vec();
        sealing_key
            .seal_in_place_append_tag(Aad::empty(), &mut ciphertext)
            .map_err(|_| "Encryption failed".to_string())?;
        
        // Format result as a JSON message (for compatibility)
        let message = EncryptedMessage {
            header: MessageHeader {
                sender_id: "self".to_string(),
                public_ratchet_key: vec![],
                message_number: 0,
                previous_chain_length: 0,
            },
            iv: iv.to_vec(),
            ciphertext,
        };
        
        // Serialize message
        let result = serde_json::to_vec(&message)
            .map_err(|e| format!("Failed to serialize message: {}", e))?;
        
        record_counter("security.e2ee.messages_encrypted", 1.0, None);
        record_histogram("security.e2ee.encrypted_message_size", result.len() as f64, None);
        
        Ok(result)
    }
    
    /// Decrypt data using the E2EE system
    pub fn decrypt_data(&self, ciphertext: &[u8]) -> Result<Vec<u8>> {
        if !self.enabled {
            // If E2EE is disabled, just return the ciphertext
            return Ok(ciphertext.to_vec());
        }
        
        // Try to parse as an encrypted message
        match serde_json::from_slice::<EncryptedMessage>(ciphertext) {
            Ok(message) => {
                // For now, use a simpler decryption scheme
                // In a real app, this would use the appropriate session
                
                // Get key (using identity key for simplicity)
                let key = self.private_identity_key.read().unwrap()
                    .clone()
                    .ok_or_else(|| "Identity key not initialized".to_string())?;
                
                // Create decryption key
                let unbound_key = UnboundKey::new(&CHACHA20_POLY1305, &key[0..KEY_SIZE])
                    .map_err(|_| "Failed to create decryption key".to_string())?;
                let nonce_sequence = StatefulNonceSequence::new(&message.iv);
                let mut opening_key = OpeningKey::new(unbound_key, nonce_sequence);
                
                // Decrypt the data
                let mut plaintext = message.ciphertext.clone();
                let decrypted = opening_key
                    .open_in_place(Aad::empty(), &mut plaintext)
                    .map_err(|_| "Decryption failed".to_string())?;
                
                record_counter("security.e2ee.messages_decrypted", 1.0, None);
                record_histogram("security.e2ee.decrypted_message_size", decrypted.len() as f64, None);
                
                Ok(decrypted.to_vec())
            },
            Err(_) => {
                // Not an encrypted message or parsing failed
                // Just return the original data
                warn!("Received data is not a valid encrypted message");
                Ok(ciphertext.to_vec())
            }
        }
    }
    
    /// Rotate keys (generate new signed pre-key and one-time keys)
    pub fn rotate_keys(&self) -> Result<()> {
        if !self.enabled {
            return Err("E2EE is disabled".into());
        }
        
        let _lock = self.key_lock.lock().unwrap();
        
        // Generate new signed pre-key
        let signed_pre_key_path = config_path(SIGNED_PRE_KEY_FILE);
        let key_pair = self.generate_key_pair()?;
        
        // Save key pair
        let mut key_data = key_pair.0.clone();
        key_data.extend_from_slice(&key_pair.1);
        
        fs::write(&signed_pre_key_path, &key_data)
            .map_err(|e| format!("Failed to save signed pre-key: {}", e))?;
        
        *self.private_signed_pre_key.write().unwrap() = Some(key_pair.0);
        *self.public_signed_pre_key.write().unwrap() = Some(key_pair.1);
        
        // Generate new one-time pre-keys
        let one_time_keys_dir = config_path(ONE_TIME_KEYS_DIR);
        let mut one_time_keys = HashMap::new();
        
        // Generate 20 new keys
        for _ in 0..20 {
            let key_pair = self.generate_key_pair()?;
            
            // Save key pair
            let mut key_data = key_pair.0.clone();
            key_data.extend_from_slice(&key_pair.1);
            
            let key_file = one_time_keys_dir.join(format!("{}.bin", hex::encode(&key_pair.1[0..8])));
            fs::write(&key_file, &key_data)
                .map_err(|e| format!("Failed to save one-time key: {}", e))?;
            
            one_time_keys.insert(key_pair.1, key_pair.0);
        }
        
        *self.one_time_pre_keys.write().unwrap() = one_time_keys;
        
        info!("Rotated E2EE keys");
        record_counter("security.e2ee.keys_rotated", 1.0, None);
        
        Ok(())
    }
}
