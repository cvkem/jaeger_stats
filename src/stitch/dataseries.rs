use crate::{utils::Counted, StatsRec};
use std::mem;

pub struct DataSeries<'a>(pub &'a mut Vec<Option<StatsRec>>);

impl<'a> DataSeries<'a> {
    /// filter a Dataseries to drop out Process/Operation-combination that occur seldom (possible only part of testing-flows, or reversed deployments)
    /// The traces are not yet filtered out at the call-chain specific level (To be determined whether that is independent filtering, or based on Process/Operation)
    pub fn drop_low_volume_traces(&mut self, drop_count: usize) -> usize {
        // Determine the count per process
        let proc_count = self.0.iter().fold(Counted::new(), |mut proc_count, sro| {
            sro.as_ref().map(|sr| {
                sr.stats.iter().for_each(|(k, v)| {
                    //                    proc_count.add_item_count(&k[..], v.num_received_calls);
                    proc_count.add_item_count(
                        k.to_owned(),
                        v.num_received_calls + v.num_unknown_calls,  // unknown calls included as these might be inbound calls (trying to be conservative in excluding Processes.s)
                    );
                });
            });
            proc_count
        });

        let mut num_dropped = 0;

        self.0.iter_mut().for_each(|sro| {
            sro.as_mut().map(|mut sr| {
                let orig_stats = mem::take(&mut sr.stats);
                sr.stats = orig_stats
                    .into_iter()
                    .filter(|(k, v)| {
                        if proc_count.get_count(k.to_owned()) > drop_count {
                            true
                        } else {
                            num_dropped += 1;
                            //println!("DROPPING {k}:  {v:?}");
                            false
                        }
                    })
                    .collect()
            });
        });

        num_dropped
    }

}
