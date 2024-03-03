use super::{super::StitchedSet, stitched_line::LegacyStitchedLine};
use serde::{Deserialize, Serialize};

#[derive(Default, Debug, Deserialize, Serialize)]
pub struct LegacyStitchedSet(pub Vec<LegacyStitchedLine>);

/// Implement Into directly as we do not want or need/accept the Operation from Stitched to LegacyStitched.
impl TryInto<StitchedSet> for LegacyStitchedSet {
    type Error = &'static str;

    fn try_into(self) -> Result<StitchedSet, Self::Error> {
        let mut error = "";
        let stitched_set = self
            .0
            .into_iter()
            .scan(&mut error, |error, lsl| match lsl.try_into() {
                Ok(sl) => Some(sl),
                Err(err) => {
                    **error = err;
                    None
                }
            })
            .collect();

        // check for an error
        if error.is_empty() {
            Ok(StitchedSet(stitched_set))
        } else {
            Err(error)
        }
    }
}
