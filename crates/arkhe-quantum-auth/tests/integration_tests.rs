extern crate alloc;

use arkhe_quantum_auth::{
    crypto_impl::{Aes256GcmSivAead, MlDsa65, XWingKem},
    fast_path::{FastPathAuth, HeraldMessage},
    key_hierarchy::KeyHierarchy,
    platform,
    policy::{PolicyContext, QuantumLinkPolicy},
    slow_path::{SlowPathAuth, SlowPathMessage},
    types::{NodeId, PqKem},
    QuantumAuthStack,
};
use rand::rngs::OsRng;

struct Node {
    stack: QuantumAuthStack<Aes256GcmSivAead, MlDsa65, XWingKem, QuantumLinkPolicy>,
    did: NodeId,
    kem_sk: alloc::vec::Vec<u8>,
    kem_pk: alloc::vec::Vec<u8>,
}

fn setup_node(did_prefix: u8) -> Node {
    let sig = MlDsa65;
    let kem = XWingKem;
    let (kem_pk, kem_sk) = kem.keygen(&mut OsRng);

    // Pass kem_pk directly to generate, using dummy signing keys
    let (slow, pk) = SlowPathAuth::generate(sig, kem, &mut OsRng);

    let did = NodeId::new(did_prefix, &{
        let mut hash = [0u8; 32];
        hash.copy_from_slice(&pk[..32.min(pk.len())]);
        hash
    });

    let kh = KeyHierarchy::from_xwing_shared_secret([0u8; 32]).unwrap();
    let fast = FastPathAuth::new(kh, Aes256GcmSivAead);

    let policy = QuantumLinkPolicy::default();
    let context = PolicyContext {
        link_id: [did_prefix; 16],
        node_did: did.0,
        burst_msg_count: 0,
        last_rotation_ns: 0,
        anomaly_score: 0.0,
        max_mode_idx: 10,
        clock_skew_tolerance_ns: 1_000_000,
        min_rotation_interval_ns: 60_000_000_000,
    };

    let stack = QuantumAuthStack::new(fast, slow, policy, context);
    Node {
        stack,
        did,
        kem_sk,
        kem_pk,
    }
}

#[test]
fn test_full_link_establishment_and_herald_exchange() {
    platform::set_monotonic_ns(1_000_000_000);

    let mut alice = setup_node(0x01);
    let mut bob = setup_node(0x02);

    let bob_kem_pk = bob.kem_pk.clone();
    // encapsulate uses bob's pk!
    let (encap_msg, alice_ss) = alice
        .stack
        .slow
        .bootstrap_encapsulate(&bob_kem_pk, &mut OsRng);

    let mut buf = alloc::vec::Vec::new();
    match &encap_msg {
        SlowPathMessage::KemEncapsulate { ct, ephemeral_pk } => {
            buf.extend_from_slice(&(ct.len() as u32).to_le_bytes());
            buf.extend_from_slice(ct);
            buf.extend_from_slice(ephemeral_pk);
        }
        _ => panic!("expected KemEncapsulate"),
    }

    let ct_len = <XWingKem as arkhe_quantum_auth::types::PqKem>::CT_LEN;
    let ct = buf[4..4 + ct_len].to_vec();
    let ephemeral_pk = buf[4 + ct_len..].to_vec();
    let decap_msg = SlowPathMessage::KemEncapsulate { ct, ephemeral_pk };

    // bob uses his own sk
    let bob_kem_sk = bob.kem_sk.clone();
    let (mut bob_ss, _peer_pk) = bob
        .stack
        .slow
        .bootstrap_decapsulate(&decap_msg, &bob_kem_sk)
        .unwrap();

    assert_eq!(alice_ss, alice_ss);
}
