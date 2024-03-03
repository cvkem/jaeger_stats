use super::{
    super::{call_chain_data::CallChainData, stitch_list::StitchSources, Stitched, StitchedSet},
    call_chain_data::{LegacyCallChainData, VecLegacyCallChainData},
    stitched_set::LegacyStitchedSet,
};
use crate::ServiceOperString;
use serde::{Deserialize, Serialize};
use serde_json;
use std::{error::Error, fs, io, path::Path};

#[derive(Default, Serialize, Deserialize)]
pub struct LegacyStitched {
    /// the list of input-files (one per analysis) that are used.
    pub sources: StitchSources,
    pub basic: LegacyStitchedSet,
    pub process_operation: Vec<(ServiceOperString, LegacyStitchedSet)>,
    ///  call-chain is keyed by the Service/Operation and the values is a series of call-chains that end in this process/Oper
    /// The values is a Vector call-chains represent all different paths (call-chains) that terminate in de Process/Oper of the key of this vector.
    pub call_chain: Vec<(ServiceOperString, VecLegacyCallChainData)>,
}

impl LegacyStitched {
    // read the legacy-stitched and return a Stiched or an error
    pub fn from_json(file_name: &str) -> Result<Self, Box<dyn Error>> {
        let path_str = Path::new(file_name);
        let f = fs::File::open(path_str)?;
        let reader = io::BufReader::new(f);

        let legacy_stitched = serde_json::from_reader(reader)?;
        Ok(legacy_stitched)
    }

    // read the legacy-stitched and return a Stiched or an error
    pub fn from_bincode(file_name: &str) -> Result<Self, Box<dyn Error>> {
        let path_str = Path::new(file_name);
        let f = fs::File::open(path_str)?;
        let reader = io::BufReader::new(f);

        let legacy_stitched = bincode::deserialize_from(reader)?;
        Ok(legacy_stitched)
    }
}

/// Implement Into directly as we do not want or need/accept the Operation from Stitched to LegacyStitched.
impl TryInto<Stitched> for LegacyStitched {
    type Error = &'static str;

    fn try_into(self) -> Result<Stitched, Self::Error> {
        let sources = self.sources;
        let basic = self.basic.try_into()?;
        let service_operation = vec_try_into(self.process_operation)?;
        let call_chain = vec_try_into(self.call_chain)?;

        Ok(Stitched::new(sources, basic, service_operation, call_chain))
    }
}

// /// Try to extract a service-operation from a Legacy-servie_oper.
// fn try_service_operation(proc_oper: Vec<(String, LegacyStitchedSet)>) -> Result<Vec<(String, StitchedSet)>, &'static str> {
//     let mut error: Result<(), &'static str> = Ok(());
//     let serv_oper = proc_oper
//         .into_iter()
//         .scan(&mut error, |error, (key, lset)| match lset.try_into() {
//             Ok(stitched_set) => Some((key, stitched_set)),
//             Err(err) => {
//                 **error = Err(err);
//                 None  // short circuit
//             }
//         })
//         .collect();
//     error?; // check the error
//     Ok(serv_oper)
// }

//type static_str = &'static str;

/// Using try_into to translate a vector of type Vec<K, A> to a Vec<K, B> or return the first error. The proces is aborted on the first error.
/// This operation is used to handle service_oper and handle call-chain fields when migrating from LegacyStitched to Stitched.
fn vec_try_into<K, A, B>(proc_oper: Vec<(K, A)>) -> Result<Vec<(K, B)>, &'static str>
where
    A: TryInto<B, Error = &'static str>,
{
    let mut error: Result<(), &'static str> = Ok(());
    let serv_oper = proc_oper
        .into_iter()
        .scan(&mut error, |error, (key, lset)| match lset.try_into() {
            Ok(stitched_set) => Some((key, stitched_set)),
            Err(err) => {
                **error = Err(err);
                None // short circuit
            }
        })
        .collect();
    error?; // check the error
    Ok(serv_oper)
}
