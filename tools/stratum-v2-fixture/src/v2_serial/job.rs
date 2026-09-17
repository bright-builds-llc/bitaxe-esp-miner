use super::{
    model::{Error, Result, Submission},
    oracle,
};
use base64::{engine::general_purpose::STANDARD, Engine};
use bitaxe_stratum::v2::{
    frame::Frame,
    messages::{NewMiningJob, OpenStandardMiningChannelSuccess, SetNewPrevHash, SetTarget},
};
use rand::{rngs::OsRng, RngCore};
use serde::Serialize;
use sha2::{Digest, Sha256};
use std::time::{SystemTime, UNIX_EPOCH};
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub(super) struct Facts {
    pub connection_id: String,
    pub job_commitment: String,
    pub channel_success: String,
    pub new_mining_job: String,
    pub set_target: String,
    pub set_new_prev_hash: String,
}
pub(super) struct Job {
    pub channel: u32,
    pub id: u32,
    pub previous: [u8; 32],
    pub merkle: [u8; 32],
    pub ntime: u32,
    pub frames: [Frame; 4],
    pub facts: Facts,
}
impl Job {
    pub fn new(request: u32, connection: String) -> Result<Self> {
        let mut rng = OsRng;
        let channel = rng.next_u32().max(1);
        let id = rng.next_u32().max(1);
        let mut previous = [0; 32];
        let mut merkle = [0; 32];
        rng.fill_bytes(&mut previous);
        rng.fill_bytes(&mut merkle);
        let ntime = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| Error::new("job_sent", "clock"))?
            .as_secs()
            .try_into()
            .map_err(|_| Error::new("job_sent", "clock"))?;
        let encode = |value: std::result::Result<Frame, bitaxe_stratum::v2::StratumV2Error>| {
            value.map_err(|_| Error::new("job_sent", "protocol"))
        };
        let frames = [
            encode(
                OpenStandardMiningChannelSuccess {
                    request_id: request,
                    channel_id: channel,
                    target: oracle::TARGET,
                    extranonce_prefix: Vec::new(),
                    group_channel_id: 0,
                }
                .encode(),
            )?,
            encode(
                NewMiningJob {
                    channel_id: channel,
                    job_id: id,
                    maybe_min_ntime: None,
                    version: oracle::VERSION,
                    merkle_root: merkle,
                }
                .encode(),
            )?,
            encode(
                SetTarget {
                    channel_id: channel,
                    maximum_target: oracle::TARGET,
                }
                .encode(),
            )?,
            encode(
                SetNewPrevHash {
                    channel_id: channel,
                    job_id: id,
                    prev_hash: previous,
                    min_ntime: ntime,
                    nbits: oracle::NBITS,
                }
                .encode(),
            )?,
        ];
        let mut hash = Sha256::new();
        for frame in &frames {
            let bytes = frame.encode();
            hash.update((bytes.len() as u32).to_le_bytes());
            hash.update(bytes);
        }
        let facts = Facts {
            connection_id: connection,
            job_commitment: oracle::hex(&hash.finalize()),
            channel_success: STANDARD.encode(frames[0].encode()),
            new_mining_job: STANDARD.encode(frames[1].encode()),
            set_target: STANDARD.encode(frames[2].encode()),
            set_new_prev_hash: STANDARD.encode(frames[3].encode()),
        };
        Ok(Self {
            channel,
            id,
            previous,
            merkle,
            ntime,
            frames,
            facts,
        })
    }
    pub fn validate(&self, share: &Submission) -> Result<[u8; 32]> {
        if share.channel_id != self.channel {
            return Err(Error::new("share_received", "channel_mismatch"));
        }
        if share.job_id != self.id || share.ntime != self.ntime {
            return Err(Error::new("share_received", "job_mismatch"));
        }
        if !oracle::valid_version(share.version) {
            return Err(Error::new("share_received", "invalid_nonce"));
        }
        Ok(oracle::hash(&oracle::header(
            share.version,
            self.previous,
            self.merkle,
            share.ntime,
            oracle::NBITS,
            share.nonce,
        )))
    }
}
