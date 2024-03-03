use crate::stitch::call_chain_data::CallChainData;

use super::stitched_set::LegacyStitchedSet;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct LegacyCallChainData {
    /// unique key constructed by the concatenation of all steps of the trace.
    pub full_key: String,
    /// Key constructed from the inbound calls only (outbound calls, such as GET and POST are omitted). This key is likely unique, but is not guaranteed to be unique
    #[serde(alias = "inboud_process_key")]
    // fix as we had a typo in the past. I doubt whether this alias is needed.
    pub inbound_process_key: String,
    /// This process refers back to the (identified) root of the full trace
    pub rooted: bool,
    /// This Call-chain ends at a leaf, and thus covers a full chain (provided it is rooted)
    pub is_leaf: bool,
    /// The set of actual data associated with the Call-chain.
    pub data: LegacyStitchedSet,
}

/// Implement Into directly as we do not want or need/accept the Operation from Stitched to LegacyStitched.
impl TryInto<CallChainData> for LegacyCallChainData {
    type Error = &'static str;

    fn try_into(self) -> Result<CallChainData, Self::Error> {
        let data = self.data.try_into()?;

        Ok(CallChainData::new(
            self.full_key,
            self.inbound_process_key,
            self.rooted,
            self.is_leaf,
            data,
        ))
    }
}

/// introduce a new-type for this vector such that we can  implement the try-into on top (needed for a generic solution)
#[derive(Serialize, Deserialize, Debug)]
pub struct VecLegacyCallChainData(pub Vec<LegacyCallChainData>);

impl TryInto<Vec<CallChainData>> for VecLegacyCallChainData {
    type Error = &'static str;

    fn try_into(self) -> Result<Vec<CallChainData>, Self::Error> {
        let mut error: Result<(), &'static str> = Ok(());
        let vccd = self
            .0
            .into_iter()
            .scan(&mut error, |error, lccd| match lccd.data.try_into() {
                Ok(data) => Some(CallChainData::new(
                    lccd.full_key,
                    lccd.inbound_process_key,
                    lccd.rooted,
                    lccd.is_leaf,
                    data,
                )),
                Err(err) => {
                    **error = Err(err);
                    None // short circuit
                }
            })
            .collect();
        error?; // check the error
        Ok(vccd)
    }
}
