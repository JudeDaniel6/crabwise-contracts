extern crate std;
use soroban_sdk::{
    contract, contractimpl, testutils::{Address as _, Events as _, Ledger as _},
    Address, Bytes, Env, String, IntoVal,
};

use crate::dispute_helpers::DisputeHelper;
use crate::errors::DisputeError;
use crate::types::{DisputeRecord, DisputeStatus};
use crate::{DisputeArbiter, DisputeArbiterClient};

#[contract]
pub struct MockEscrow;

#[contractimpl]
impl MockEscrow {
    pub fn open_dis(_env: Env, _commission_id: Bytes, _initiator: Address) {}
    pub fn refund_cl(_env: Env, _commission_id: Bytes, _config_contract: Address) {}
    pub fn release_p(_env: Env, _commission_id: Bytes, _config_contract: Address) {}
    pub fn rel_pay(_env: Env, _commission_id: Bytes, _config_contract: Address) {}
}

#[contract]
pub struct MockConfig;

#[contractimpl]
impl MockConfig {
    pub fn get_usdc(env: Env) -> Address {
        env.register_contract(None, MockToken)
    }
}

#[contract]
pub struct MockToken;

#[contractimpl]
impl MockToken {
    pub fn balance(_env: Env, _id: Address) -> i128 { 1000 }
    pub fn transfer(_env: Env, _from: Address, _to: Bytes, _amount: i128) {}
}

fn create_test_env() -> (Env, Address, Address, Address, Address) {
    let env = Env::default();
    let admin = Address::generate(&env);
    let escrow_contract = env.register_contract(None, MockEscrow);
    let config_contract = env.register_contract(None, MockConfig);
    let token_admin = Address::generate(&env);
    (env, admin, escrow_contract, config_contract, token_admin)
}

fn setup_initialized(
    env: &Env,
    admin: &Address,
    escrow: &Address,
    config: &Address,
    auto_resolve: u32,
) {
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(env, &arbiter);
    client.initialize(admin, escrow, config, &auto_resolve);
}

// ========== initialize ==========

#[test]
fn test_initialize_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
}

#[test]
fn test_initialize_double_init_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let result = client.try_initialize(&admin, &escrow, &config, &100u32);
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::AlreadyInitialized);
}

// ========== open_dispute ==========

#[test]
fn test_open_dispute_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    let record = client.get_dispute(&commission_id);
    assert_eq!(record.status, DisputeStatus::Open);
}

#[test]
fn test_open_dispute_already_exists_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    let result = client.try_open_dispute(&commission_id, &initiator);
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::AlreadyResolved);
}

#[test]
fn test_open_dispute_not_initialized_fails() {
    let (env, _admin, _escrow, _config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.try_open_dispute(&commission_id, &initiator);
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::NotInitialized);
}

// ========== resolve_for_client ==========

#[test]
#[ignore = "requires a live escrow contract"]
fn test_resolve_for_client_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.resolve_for_client(&commission_id, &String::from_str(&env, "Refunded"));
    let record = client.get_dispute(&commission_id);
    assert_eq!(record.status, DisputeStatus::ResolvedForClient);
}

#[test]
fn test_resolve_for_client_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.try_resolve_for_client(&commission_id, &String::from_str(&env, "note"));
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::NotFound);
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_resolve_for_client_wrong_status_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.resolve_for_client(&commission_id, &String::from_str(&env, "first"));
    let result = client.try_resolve_for_client(&commission_id, &String::from_str(&env, "second"));
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::InvalidStatus);
}

// ========== resolve_for_artist ==========

#[test]
#[ignore = "requires a live escrow contract"]
fn test_resolve_for_artist_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.resolve_for_artist(&commission_id, &String::from_str(&env, "Paid"));
    let record = client.get_dispute(&commission_id);
    assert_eq!(record.status, DisputeStatus::ResolvedForArtist);
}

#[test]
fn test_resolve_for_artist_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.try_resolve_for_artist(&commission_id, &String::from_str(&env, "note"));
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::NotFound);
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_resolve_for_artist_wrong_status_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.resolve_for_artist(&commission_id, &String::from_str(&env, "first"));
    let result = client.try_resolve_for_artist(&commission_id, &String::from_str(&env, "second"));
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::InvalidStatus);
}

// ========== partial_resolve ==========

#[test]
#[ignore = "requires a live escrow contract"]
fn test_partial_resolve_4000_bps() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.partial_resolve(&commission_id, &4000u32, &String::from_str(&env, "40pct client"));
    let record = client.get_dispute(&commission_id);
    assert_eq!(record.status, DisputeStatus::PartiallyResolved);
}

#[test]
fn test_partial_resolve_invalid_bps_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    let result = client.try_partial_resolve(&commission_id, &10001u32, &String::from_str(&env, "bad"));
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::InvalidShareBps);
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_partial_resolve_valid_bps_boundary() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.partial_resolve(&commission_id, &10000u32, &String::from_str(&env, "all_client"));
}

#[test]
fn test_partial_resolve_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.try_partial_resolve(&commission_id, &5000u32, &String::from_str(&env, "note"));
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::NotFound);
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_partial_resolve_wrong_status_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.resolve_for_client(&commission_id, &String::from_str(&env, "done"));
    let result = client.try_partial_resolve(&commission_id, &5000u32, &String::from_str(&env, "note"));
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::InvalidStatus);
}

// ========== auto_resolve ==========

#[test]
fn test_auto_resolve_before_timeout_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    let result = client.try_auto_resolve(&commission_id);
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::AutoResolveNotDue);
}

#[test]
fn test_auto_resolve_at_timeout_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    env.ledger().with_mut(|l| l.sequence_number = 101);
    client.auto_resolve(&commission_id);
    let record = client.get_dispute(&commission_id);
    assert_eq!(record.status, DisputeStatus::AutoResolved);
}

#[test]
fn test_auto_resolve_after_timeout_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    env.ledger().with_mut(|l| l.sequence_number = 200);
    client.auto_resolve(&commission_id);
}

#[test]
fn test_auto_resolve_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.try_auto_resolve(&commission_id);
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::NotFound);
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_auto_resolve_wrong_status_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    env.ledger().with_mut(|l| l.sequence_number = 101);
    client.auto_resolve(&commission_id);
    let result = client.try_auto_resolve(&commission_id);
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::InvalidStatus);
}

// ========== get_dispute ==========

#[test]
fn test_get_dispute_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    let record = client.get_dispute(&commission_id);
    assert_eq!(record.commission_id, commission_id);
    assert_eq!(record.status, DisputeStatus::Open);
}

#[test]
fn test_get_dispute_not_found_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    let result = client.try_get_dispute(&commission_id);
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::NotFound);
}

// ========== error codes ==========

#[test]
fn test_error_codes() {
    assert_eq!(DisputeError::AlreadyInitialized as u32, 1);
    assert_eq!(DisputeError::NotInitialized as u32, 2);
    assert_eq!(DisputeError::Unauthorized as u32, 3);
    assert_eq!(DisputeError::NotFound as u32, 4);
    assert_eq!(DisputeError::InvalidStatus as u32, 5);
    assert_eq!(DisputeError::AlreadyResolved as u32, 6);
    assert_eq!(DisputeError::AutoResolveNotDue as u32, 7);
    assert_eq!(DisputeError::InvalidShareBps as u32, 8);
    assert_eq!(DisputeError::ArithmeticOverflow as u32, 9);
    assert_eq!(DisputeError::NoteTooLong as u32, 10);
    assert_eq!(DisputeError::EvidenceWindowClosed as u32, 11);
    assert_eq!(DisputeError::EvidenceTooLong as u32, 12);
    assert_eq!(DisputeError::InvalidTtl as u32, 13);
}

// ========== dispute status values ==========

#[test]
fn test_dispute_status_values() {
    assert_eq!(DisputeStatus::Open as u32, 0);
    assert_eq!(DisputeStatus::ResolvedForClient as u32, 1);
    assert_eq!(DisputeStatus::ResolvedForArtist as u32, 2);
    assert_eq!(DisputeStatus::PartiallyResolved as u32, 3);
    assert_eq!(DisputeStatus::AutoResolved as u32, 4);
}

// ========== events ==========

#[test]
fn test_open_dispute_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    let events = env.events().all();
    assert!(!events.is_empty());
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_resolve_for_client_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.resolve_for_client(&commission_id, &String::from_str(&env, "Refunded"));
    let events = env.events().all();
    assert!(events.len() >= 2);
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_resolve_for_artist_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.resolve_for_artist(&commission_id, &String::from_str(&env, "Paid"));
    let events = env.events().all();
    assert!(events.len() >= 2);
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_partial_resolve_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.partial_resolve(&commission_id, &4000u32, &String::from_str(&env, "split"));
    let events = env.events().all();
    assert!(events.len() >= 2);
}

#[test]
#[ignore = "requires a live escrow contract"]
fn test_auto_resolve_emits_event() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);
    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    env.ledger().with_mut(|l| l.sequence_number = 101);
    client.auto_resolve(&commission_id);
    let events = env.events().all();
    assert!(events.len() >= 2);
}

// ========== version queries ==========

#[test]
fn test_version_is_0_1_0() {
    let v = DisputeArbiter::get_version(Env::default());
    assert_eq!(v.major, 0);
    assert_eq!(v.minor, 1);
    assert_eq!(v.patch, 0);
    assert!(DisputeArbiter::is_version_compatible(Env::default(), 0, 1, 0));
    assert!(!DisputeArbiter::is_version_compatible(Env::default(), 0, 2, 0));
    let meta = DisputeArbiter::get_version_metadata(Env::default());
    assert_eq!(meta.storage_schema, 1);
}

// ========== dispute deadline and evidence window (#977) ==========

#[test]
fn test_open_dispute_records_a_deadline() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);

    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);

    assert_eq!(client.get_dispute_deadline(&commission_id), 100);
    assert_eq!(client.ledgers_until_deadline(&commission_id), 100);
    assert_eq!(client.get_dispute(&commission_id).deadline_ledger, 100);
}

#[test]
fn test_default_ttl_matches_the_shared_dispute_constant() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    // No override: the arbiter reports the shared default.
    client.initialize(&admin, &escrow, &config, &shared::dispute::DEFAULT_DISPUTE_TTL_LEDGERS);
    assert_eq!(
        client.get_dispute_ttl_ledgers(),
        shared::dispute::DEFAULT_DISPUTE_TTL_LEDGERS
    );
}

#[test]
fn test_submit_evidence_within_the_window_succeeds() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    let submitter = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);

    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    client.submit_evidence(&commission_id, &submitter, &Bytes::from_slice(&env, b"proof"));

    assert_eq!(client.get_evidence_count(&commission_id), 1);
    let entry = client.get_evidence(&commission_id, &0u32);
    assert_eq!(entry.submitter, submitter);
    assert_eq!(entry.submitted_ledger, 0);
}

#[test]
fn test_submit_evidence_exactly_at_the_deadline_is_allowed() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    let submitter = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);

    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.submit_evidence(&commission_id, &submitter, &Bytes::from_slice(&env, b"last minute"));
    assert_eq!(client.get_evidence_count(&commission_id), 1);
}

#[test]
fn test_submit_evidence_after_the_deadline_fails() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    let submitter = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);

    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    env.ledger().with_mut(|l| l.sequence_number = 101);
    let result = client.try_submit_evidence(&commission_id, &submitter, &Bytes::from_slice(&env, b"late"));
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::EvidenceWindowClosed);
}

#[test]
fn test_oversized_evidence_is_rejected() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    let submitter = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);

    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    let big = Bytes::from_slice(&env, &[7u8; 1025]);
    let result = client.try_submit_evidence(&commission_id, &submitter, &big);
    assert_eq!(result.unwrap_err().unwrap(), DisputeError::EvidenceTooLong);
}

#[test]
fn test_auto_resolve_fires_at_exactly_the_deadline() {
    let (env, admin, escrow, config, _token) = create_test_env();
    let initiator = Address::generate(&env);
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);

    let commission_id = Bytes::from_array(&env, &[1u8, 2, 3]);
    client.open_dispute(&commission_id, &initiator);
    env.ledger().with_mut(|l| l.sequence_number = 100);
    client.auto_resolve(&commission_id);
    assert_eq!(client.get_dispute(&commission_id).status, DisputeStatus::AutoResolved);
}

// ========== resolver eligibility (#39) ==========

const RESOLVED_CODE: u32 = 2;

/// `DisputeHelper` calls `require_auth`, which the test host only permits
/// inside a contract invocation, so the helper is exposed through a thin
/// contract and every case below exercises it on-chain.
#[contract]
pub struct DisputeHelperHarness;

#[contractimpl]
impl DisputeHelperHarness {
    pub fn raise_dispute(env: Env, participant: Address, dispute_id: u64) {
        DisputeHelper::raise_dispute(&env, &participant, dispute_id);
    }

    pub fn register_parties(env: Env, dispute_id: u64, client: Address, artist: Address) {
        DisputeHelper::register_parties(&env, dispute_id, &client, &artist);
    }

    pub fn authorize_arbiter(env: Env, who: Address) {
        DisputeHelper::authorize_arbiter(&env, &who);
    }

    pub fn resolve_dispute(
        env: Env,
        resolver: Address,
        dispute_id: u64,
        resolution_code: u32,
    ) -> Result<(), DisputeError> {
        DisputeHelper::resolve_dispute(&env, &resolver, dispute_id, resolution_code)
    }

    pub fn is_party(env: Env, dispute_id: u64, who: Address) -> bool {
        DisputeHelper::is_party(&env, dispute_id, &who)
    }

    pub fn is_arbiter(env: Env, who: Address) -> bool {
        DisputeHelper::is_arbiter(&env, &who)
    }

    pub fn dispute_status(env: Env, dispute_id: u64) -> u32 {
        DisputeHelper::dispute_status(&env, dispute_id)
    }
}

/// A raised dispute between two generated parties.
fn setup_dispute_with_parties<'a>(
    env: &'a Env,
) -> (DisputeHelperHarnessClient<'a>, u64, Address, Address) {
    let dispute_id = 7u64;
    let client = Address::generate(env);
    let artist = Address::generate(env);
    env.mock_all_auths();
    let harness = env.register_contract(None, DisputeHelperHarness);
    let helper = DisputeHelperHarnessClient::new(env, &harness);
    helper.raise_dispute(&client, &dispute_id);
    helper.register_parties(&dispute_id, &client, &artist);
    (helper, dispute_id, client, artist)
}

#[test]
fn test_client_party_cannot_resolve_its_own_dispute() {
    let env = Env::default();
    let (helper, dispute_id, client, _artist) = setup_dispute_with_parties(&env);

    let result = helper.try_resolve_dispute(&client, &dispute_id, &RESOLVED_CODE);

    assert_eq!(result.unwrap_err().unwrap(), DisputeError::Unauthorized);
    // Refused before the write: the dispute is still open.
    assert_eq!(helper.dispute_status(&dispute_id), 1);
}

#[test]
fn test_artist_party_cannot_resolve_its_own_dispute() {
    let env = Env::default();
    let (helper, dispute_id, _client, artist) = setup_dispute_with_parties(&env);

    let result = helper.try_resolve_dispute(&artist, &dispute_id, &RESOLVED_CODE);

    assert_eq!(result.unwrap_err().unwrap(), DisputeError::Unauthorized);
    assert_eq!(helper.dispute_status(&dispute_id), 1);
}

#[test]
fn test_unauthorized_caller_cannot_resolve() {
    let env = Env::default();
    let (helper, dispute_id, _client, _artist) = setup_dispute_with_parties(&env);
    let stranger = Address::generate(&env);

    let result = helper.try_resolve_dispute(&stranger, &dispute_id, &RESOLVED_CODE);

    assert_eq!(result.unwrap_err().unwrap(), DisputeError::Unauthorized);
    assert_eq!(helper.dispute_status(&dispute_id), 1);
}

#[test]
fn test_authorized_arbiter_can_resolve() {
    let env = Env::default();
    let (helper, dispute_id, _client, _artist) = setup_dispute_with_parties(&env);
    let arbiter = Address::generate(&env);
    helper.authorize_arbiter(&arbiter);

    helper.resolve_dispute(&arbiter, &dispute_id, &RESOLVED_CODE);

    assert_eq!(helper.dispute_status(&dispute_id), RESOLVED_CODE);
}

#[test]
fn test_party_separately_authorized_as_arbiter_can_resolve() {
    let env = Env::default();
    let (helper, dispute_id, _client, artist) = setup_dispute_with_parties(&env);

    // Still a party, and still refused until the role is granted separately.
    let refused = helper.try_resolve_dispute(&artist, &dispute_id, &RESOLVED_CODE);
    assert_eq!(refused.unwrap_err().unwrap(), DisputeError::Unauthorized);

    helper.authorize_arbiter(&artist);
    helper.resolve_dispute(&artist, &dispute_id, &RESOLVED_CODE);

    assert_eq!(helper.dispute_status(&dispute_id), RESOLVED_CODE);
}

#[test]
fn test_arbiter_authorization_is_per_address_and_idempotent() {
    let env = Env::default();
    let (helper, dispute_id, client, artist) = setup_dispute_with_parties(&env);
    let arbiter = Address::generate(&env);

    helper.authorize_arbiter(&arbiter);
    helper.authorize_arbiter(&arbiter);

    assert!(helper.is_arbiter(&arbiter));
    // Being an arbiter is not the same as being one of the parties.
    assert!(helper.is_party(&dispute_id, &client));
    assert!(helper.is_party(&dispute_id, &artist));
    assert!(!helper.is_party(&dispute_id, &arbiter));
}

#[test]
fn test_eligibility_is_per_dispute() {
    let env = Env::default();
    let (helper, dispute_id, client, _artist) = setup_dispute_with_parties(&env);
    let other_dispute = dispute_id + 1;

    assert!(helper.is_party(&dispute_id, &client));
    assert!(!helper.is_party(&other_dispute, &client));
}

#[test]
fn test_set_dispute_ttl_ledgers_bounds_and_persists() {
    let (env, admin, escrow, config, _token) = create_test_env();
    env.mock_all_auths();
    let arbiter = env.register_contract(None, DisputeArbiter);
    let client = DisputeArbiterClient::new(&env, &arbiter);
    client.initialize(&admin, &escrow, &config, &100u32);

    client.set_dispute_ttl_ledgers(&admin, &500u32);
    assert_eq!(client.get_dispute_ttl_ledgers(), 500);

    for bad in [0u32, shared::dispute::MAX_DISPUTE_TTL_LEDGERS + 1] {
        let result = client.try_set_dispute_ttl_ledgers(&admin, &bad);
        assert_eq!(result.unwrap_err().unwrap(), DisputeError::InvalidTtl);
    }
}