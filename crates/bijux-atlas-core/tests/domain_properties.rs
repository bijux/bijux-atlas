use bijux_atlas_core::canonical;
use proptest::prelude::*;
use serde_json::json;

fn identifier_strategy() -> impl Strategy<Value = String> {
    let charset = prop_oneof![
        Just('a'),
        Just('b'),
        Just('c'),
        Just('d'),
        Just('e'),
        Just('f'),
        Just('g'),
        Just('h'),
        Just('i'),
        Just('j'),
        Just('k'),
        Just('l'),
        Just('m'),
        Just('n'),
        Just('o'),
        Just('p'),
        Just('q'),
        Just('r'),
        Just('s'),
        Just('t'),
        Just('u'),
        Just('v'),
        Just('w'),
        Just('x'),
        Just('y'),
        Just('z'),
        Just('0'),
        Just('1'),
        Just('2'),
        Just('3'),
        Just('4'),
        Just('5'),
        Just('6'),
        Just('7'),
        Just('8'),
        Just('9'),
        Just('-'),
        Just('_')
    ];

    proptest::collection::vec(charset, 1..32).prop_map(|chars| chars.into_iter().collect())
}

proptest! {
    #[test]
    fn stable_hash_bytes_is_deterministic(payload in proptest::collection::vec(any::<u8>(), 0..256)) {
        let h1 = canonical::stable_hash_hex(&payload);
        let h2 = canonical::stable_hash_hex(&payload);
        prop_assert_eq!(h1, h2);
    }

    #[test]
    fn stable_json_bytes_are_independent_of_object_key_order(a in identifier_strategy(), b in identifier_strategy(), av in any::<u32>(), bv in any::<u32>()) {
        prop_assume!(a != b);

        let left = json!({a.clone(): av, b.clone(): bv});
        let right = json!({b: bv, a: av});

        let left_bytes = canonical::stable_json_bytes(&left).expect("canonical left");
        let right_bytes = canonical::stable_json_bytes(&right).expect("canonical right");

        prop_assert_eq!(left_bytes, right_bytes);
    }
}
