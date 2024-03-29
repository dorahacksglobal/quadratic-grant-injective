/// This module contains helper functions for mathematical operations.
pub mod math {
    use cosmwasm_std::{StdError, StdResult};
    pub fn log2_u64_with_decimal(x: u64) -> StdResult<u64> {
        if x == 0 {
            return Err(StdError::generic_err("log2(0) is undefined"));
        }
        let mut integer = 0;
        let mut t = x as u128;
        let mut m = 1;
        while t > 1 {
            t >>= 1;
            m <<= 1;
            integer += 1;
        }
        let mut fractional = 0;
        m *= 1000;
        t = x as u128 * 1000;
        // 71773463 / 1000000000 = 10^((lg2^(1/10)) - 1
        // The result of this formula is a specific numerical value, representing the proportional increase of 2^(1/10) over 1.
        // In other words, it indicates how much greater 2^(1/10) is compared to 1 in decimal terms.
        let mut step = m * 71773463 / 1000000000;
        while fractional < 10 {
            m += step;
            if m > t {
                break;
            } else {
                step = m * 71773463 / 1000000000;
                fractional = fractional + 1;
            };
        }
        Ok(integer * 10 + fractional)
    }

    pub fn sqrt(y: u128) -> u64 {
        if y < 4 {
            if y == 0 {
                0u64
            } else {
                1u64
            }
        } else {
            let mut z = y;
            let mut x = y / 2 + 1;
            while x < z {
                z = x;
                x = (y / x + x) / 2;
            }
            z as u64
        }
    }

    #[test]
    fn test_log2_u64_with_decimal() {
        assert_eq!(log2_u64_with_decimal(1), Ok(0));
        assert_eq!(log2_u64_with_decimal(2), Ok(10));
        assert_eq!(log2_u64_with_decimal(3), Ok(15));
        assert_eq!(log2_u64_with_decimal(25), Ok(46));
        assert_eq!(log2_u64_with_decimal(1024), Ok(100));
        assert_eq!(log2_u64_with_decimal(123143400), Ok(268));
    }
}

pub mod signature {
    use cosmwasm_std::{Deps, HexBinary, Uint128};
    use tiny_keccak::{Hasher, Keccak};

    pub fn build_msg(
        addr_bytes: &[u8],
        round_id: u64,
        project_ids: &Vec<u64>,
        amounts: &Vec<Uint128>,
        vcdora: u64,
        timestamp: u64,
        sig_chain_id: &str,
        sig_contract_addr: &str,
    ) -> Vec<u8> {
        let round_id_bytes = round_id.to_le_bytes();
        let mut project_ids_bytes = Vec::new();
        project_ids.iter().for_each(|id| {
            project_ids_bytes.extend_from_slice(&id.to_le_bytes());
        });
        let mut amounts_bytes = Vec::new();
        amounts.iter().for_each(|amount| {
            amounts_bytes.extend_from_slice(&amount.to_le_bytes());
        });

        let mut msg = Vec::new();
        msg.extend_from_slice(addr_bytes);
        msg.extend_from_slice(&round_id_bytes);
        msg.extend_from_slice(&project_ids_bytes);
        msg.extend_from_slice(&amounts_bytes);
        msg.extend_from_slice(&vcdora.to_le_bytes());
        msg.extend_from_slice(&timestamp.to_le_bytes());
        msg.extend_from_slice(sig_chain_id.as_bytes());
        msg.extend_from_slice(sig_contract_addr.as_bytes());
        msg
    }

    pub fn recover_pubkey(deps: Deps, msg: Vec<u8>, sig: Vec<u8>, recid: u8) -> Vec<u8> {
        let mut keccak256 = Keccak::v256();
        let mut hash = [0u8; 32];

        keccak256.update(msg.as_slice());
        keccak256.finalize(&mut hash);

        let signature = HexBinary::from(sig);
        deps.api
            .secp256k1_recover_pubkey(&hash, signature.as_slice(), recid)
            .unwrap_or_default()
    }

    #[test]
    fn test_verify() {
        use cosmwasm_std::testing::mock_dependencies;
        use hex;

        let deps = mock_dependencies();
        let msg = hex::decode("cd7fa009e29f21b3feb62c7091f38e7dad5270a08908583d037e25c3d987f1a902000000000000000108000000000000000180969800000000000000000000000000e817e37651ef0500").expect("Decoding failed");
        let sig = hex::decode("a9686a10a12b68ddcee5032a8e5e5486c59861a9fe62796c54ea468e67ede49b14b34fbfe3b51e7f16f2287dddd38492613319c8c5b4a0a7ada2de6585886b04").expect("Decoding failed");
        let key = recover_pubkey(deps.as_ref(), msg, sig, 0);
        assert_eq!(key, hex::decode("0479be667ef9dcbbac55a06295ce870b07029bfcdb2dce28d959f2815b16f81798483ada7726a3c4655da4fbfc0e1108a8fd17b448a68554199c47d08ffb10d4b8").expect("Decoding failed"));
    }

    #[test]
    fn test_build_msg() {
        let msg = build_msg(
            &hex::decode("4C87D8f31E3d6EE5969e4002E614a9c72C6A99B8")
                .expect("Decoding failed")
                .as_slice(),
            1,
            &vec![9, 8],
            &vec![100000000000000000u128.into(), 200000000000000000u128.into()],
            42,
            1682415684,
            "",
            "",
        );
        assert_eq!(msg, hex::decode("4c87d8f31e3d6ee5969e4002e614a9c72c6a99b801000000000000000900000000000000080000000000000000008a5d784563010000000000000000000014bbf08ac60200000000000000002a0000000000000044a0476400000000").expect("Decoding failed"));
    }
}
