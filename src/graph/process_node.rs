use crate::stats::call_chain::{Call, CallChain, CallDirection};
use std::{collections::HashMap, iter};

#[derive(Default, Debug)]
pub enum ProcessNodeType {
    /// represents a prcoess that is directly exposed (via an API gateway), so this is the starting point of a service-process
    EndPoint,
    /// an internal service that supports (public) services (endpoints)
    Intermediate,
    /// a terminal typically receives calls but does not have any outbound calls
    Terminal,
    /// to be determined
    #[default]
    Unknown,
}

#[derive(Default, Debug)]
pub struct ProcessNode {
    pub name: String,
    pub ptype: ProcessNodeType,
    /// Operations are the endpoints exposed by the proces (so this is the inbound traffic)
    pub operations: HashMap<String, i32>, //LinkCount>,
    /// methods are the outbound calls to others
    pub methods: HashMap<String, i32>, //LinkCount>,
    // This category is only used for call-chains that have not been fixed, to set call_direction for CallDirection::Unknown.
    pub direction_unknown: HashMap<String, i32>, //LinkCount>,
}

impl ProcessNode {
    pub fn new(name: String) -> Self {
        Self {
            name,
            ..Default::default()
        }
    }

    /// for the operation 'oper_name' add count calls.
    pub fn add_operation(&mut self, oper_name: String, count: i32) {
        self.operations
            .entry(oper_name)
            .and_modify(|cnt| *cnt += count)
            .or_insert(count);
    }

    /// for the operation 'oper_name' add count calls.
    pub fn add_method(&mut self, method_name: String, count: i32) {
        self.methods
            .entry(method_name)
            .and_modify(|cnt| *cnt += count)
            .or_insert(count);
    }

    /// for the operation 'oper_name' add count calls.
    pub fn add_direction_unknown(&mut self, method_name: String, count: i32) {
        self.direction_unknown
            .entry(method_name)
            .and_modify(|cnt| *cnt += count)
            .or_insert(count);
    }
}

#[derive(Debug, Default)]
pub struct ProcessNodes(HashMap<String, ProcessNode>);

impl ProcessNodes {
    // pub fn new() -> Self {
    //     Self(HashMap::new())
    // }

    /// termmporary function to check callchains whether they are aloways alternating between server and Client
    pub fn tmp_check_cc(call_chain: &CallChain, is_leaf: bool, rooted: bool, looped: &Vec<String>) {
        let unexpected: Vec<_> = call_chain
            .iter()
            .enumerate()
            .filter_map(|(idx, call)| {
                if (idx == 0 && call.call_direction != CallDirection::Inbound)
                    || call.call_direction == CallDirection::Unknown
                {
                    Some((idx, &call.call_direction))
                } else {
                    None
                }
            })
            .collect();
        if !unexpected.is_empty() {
            let cc = call_chain
                .iter()
                .enumerate()
                .map(|c| format!("{c:?}"))
                .collect::<Vec<_>>()
                .join("\n\t");
            println!("\nFAILED on call_chain: is_leaf={is_leaf}  rooted={rooted}  looped={looped:?}\n\t{cc}\non indexes: {unexpected:?}");
        }
        // else {
        //     println!("\nSUCCESS on call_chain: {call_chain:?}");
        // }
    }

    pub fn add_call_chain(&mut self, call_chain: &CallChain, count: i32) {
        let initial_call = Call {
            process: "User".to_owned(),
            method: "method".to_owned(),
            call_direction: CallDirection::Outbound,
        };
        iter::once(&initial_call)
            .chain(call_chain.iter())
            .zip(call_chain.iter())
            // call_chain.iter()
            //     .scan(initial_call, |prev, call| {
            //         let pair = (prev.clone(), call);
            //         prev = &mut &call;
            //         Some(pair)
            //     })
            .for_each(|(_pred, call)| {
                // work in progress: pred will be used
                let amend = |pn: &mut ProcessNode| match call.call_direction {
                    CallDirection::Outbound => pn.add_method(call.method.clone(), count),
                    CallDirection::Inbound => pn.add_operation(call.method.clone(), count),
                    CallDirection::Unknown => pn.add_direction_unknown(call.method.clone(), count), // panic!("Unknown call-direction, this should have been fixed earlier!")
                };
                self.0
                    .entry(call.process.clone())
                    .and_modify(|pn| {
                        amend(pn);
                    })
                    .or_insert_with(|| {
                        let mut pn = ProcessNode::new(call.process.clone());
                        amend(&mut pn);
                        pn
                    });
            })
    }
}

#[derive(Hash, Eq, PartialEq, Debug, PartialOrd, Ord)]
struct LinkKey {
    process: String,
    end_point: String,
}
