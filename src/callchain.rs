

#[derive(Hash, Eq, PartialEq, PartialOrd, Ord, Debug, Default)]
pub struct Call {
    pub process: String,
    pub method: String,
}

pub type CallChain = Vec<Call>;


/// build a call-chain-key based on parameters
pub fn call_chain_key(call_chain: &CallChain, caching_process: &str, is_leaf: bool) -> String {
    let call_chain_str = call_chain
        .iter()
        .fold(String::new(), |a, b| {
            let sep = if a.len() > 0 { " | " } else { "" };
            a + sep + &b.process + "/" + &b.method
        });
    let leaf_str = if is_leaf { " *LEAF*" } else { "" };
    call_chain_str + &caching_process + &leaf_str
}


/// get_cache_suffix determines whether cached processes are in the call-chain and if so returns a suffix to represent it.
pub fn caching_process_label(caching_process: &Vec<String>, call_chain: &CallChain) -> String {
    if caching_process.len() == 0 {
        return "".to_owned()
    }
    let mut cached = Vec::new();

    call_chain.iter()
    .for_each(|Call{process, method}| {
        match &method[..] {
            "GET" | "POST" | "HEAD" | "QUERY" => (),  // ignore these methods as the inbound call has been matched already. (prevent duplicates of cached names)
            _ => match caching_process.iter().find(|&s| *s == *process) {
                Some(_) => {
                    cached.push(process.to_owned())},
                None => ()
            }
        }
    });
    if cached.len() > 0 {
        format!(" [{}]", cached.join(", "))
    } else {
        "".to_owned()
    }
}
