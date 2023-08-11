use crate::stats::call_chain::{CallChain, CallDirection};

/// The fix_call_chain returns a copy of the call-chain where CallDirection is always set (no more Unknown)
/// Each time a new Process appears on the call-chain it will be an Inbound call, the subsequent calls are the outbound calls.
/// For CallChain this assumption hold, as the callChain is a path to a leaf, and is not a full trace that allows back-tracking.
pub fn fix_call_chain(call_chain: &CallChain) -> CallChain {
    let mut issues = Vec::new();
    let fixed_call_chain: Vec<_> = call_chain
        .iter()
        .cloned()
        .enumerate()
        .scan("".to_owned(), |last_proc, (idx, mut call)| {
            if call.process == *last_proc {
                if call.call_direction == CallDirection::Inbound {
                    issues.push(idx);
                };
                //            call.call_direction = CallDirection::Outbound;
                Some(call)
            } else {
                last_proc.clear();
                last_proc.push_str(&call.process[..]);
                if call.call_direction == CallDirection::Outbound {
                    issues.push(idx);
                } else {
                    call.call_direction = CallDirection::Inbound;
                };
                //            call.call_direction = CallDirection::Inbound;
                Some(call)
            }
        })
        .collect();
    if !issues.is_empty() {
        println!("\nFailed on idx = {issues:?}:");
        for idx in 0..call_chain.len() {
            println!(
                "{idx}: {:?}  ->  {:?}",
                call_chain[idx], fixed_call_chain[idx]
            );
        }
    }

    fixed_call_chain
}
