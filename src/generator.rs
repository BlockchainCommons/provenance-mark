use bc_rand::RandomNumberGenerator;
use dcbor::{ CBOREncodable, Date };
use serde::{ Serialize, Deserialize };
use crate::util::{ serialize_base64, deserialize_base64 };

use crate::{ ProvenanceSeed, RngState };
use crate::{
    crypto_utils::extend_key,
    xoshiro256starstar::Xoshiro256StarStar,
    ProvenanceMark,
    ProvenanceMarkResolution,
};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProvenanceMarkGenerator {
    res: ProvenanceMarkResolution,
    seed: ProvenanceSeed,
    #[serde(rename = "chainID")]
    #[serde(serialize_with = "serialize_base64", deserialize_with = "deserialize_base64")]
    chain_id: Vec<u8>,
    #[serde(rename = "nextSeq")]
    next_seq: u32,
    #[serde(rename = "rngState")]
    rng_state: RngState,
}

impl ProvenanceMarkGenerator {
    pub fn new_with_seed(res: ProvenanceMarkResolution, seed: ProvenanceSeed) -> Self {
        let chain_id = seed.to_bytes()[..res.link_length()].to_vec();
        Self::new(res, seed.clone(), chain_id, 0, seed.to_bytes().into())
    }

    pub fn new_with_passphrase(res: ProvenanceMarkResolution, passphrase: &str) -> Self {
        let seed_data = extend_key(passphrase.as_bytes());
        let seed = ProvenanceSeed::from_bytes(seed_data);
        Self::new_with_seed(res, seed)
    }

    pub fn new_using(res: ProvenanceMarkResolution, rng: &mut impl RandomNumberGenerator) -> Self {
        let seed = ProvenanceSeed::new_using(rng);
        Self::new_with_seed(res, seed)
    }

    pub fn new(
        res: ProvenanceMarkResolution,
        seed: ProvenanceSeed,
        chain_id: Vec<u8>,
        next_seq: u32,
        rng_state: RngState
    ) -> Self {
        assert!(chain_id.len() == res.link_length());
        Self { res, seed, chain_id, next_seq, rng_state }
    }

    pub fn next(&mut self, date: Date, info: Option<impl CBOREncodable>) -> ProvenanceMark {
        let data: [u8; 32] = self.rng_state.clone().into();
        let mut rng = Xoshiro256StarStar::from_data(&data);

        let seq = self.next_seq;
        self.next_seq += 1;

        let key;
        if seq == 0 {
            key = self.chain_id.clone();
        } else {
            // The randomness generated by the PRNG should be portable across implementations.
            key = rng.next_bytes(self.res.link_length());
            self.rng_state = rng.to_data().into();
        }

        let mut next_rng = rng.clone();
        let next_key = next_rng.next_bytes(self.res.link_length());

        ProvenanceMark::new(
            self.res,
            key,
            next_key,
            self.chain_id.clone(),
            seq,
            date,
            info
        ).unwrap()
    }
}

impl std::fmt::Display for ProvenanceMarkGenerator {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "ProvenanceMarkGenerator(chainID: {}, res: {}, seed: {}, nextSeq: {}, rngState: {:?})",
            hex::encode(&self.chain_id),
            self.res,
            self.seed.hex(),
            self.next_seq,
            self.rng_state
        )
    }
}
