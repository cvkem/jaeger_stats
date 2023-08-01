use jaeger_stats::CChainEndPointCache;
use std::{
    env,
    path::{Path, PathBuf},
};

//const INPUT_FILE: &str = "/home/ceesvk/jaeger/loadTest-prodinz-prodGroep/";
const INPUT_FOLDER: &str = "/home/ceesvk/jaeger/prodinzicht-23-juni-14u/CallChain";
const INPUT_KEY: &str = "retail-gateway_GET:_services_apix-mobiel-accounts_api_cards";

fn main() {
    let args: Vec<String> = env::args().collect();

    let input_folder = if args.len() > 1 {
        args[1].to_owned()
    } else {
        INPUT_FOLDER.to_owned()
    };

    let file_key = if args.len() > 2 {
        args[2].clone()
    } else {
        INPUT_KEY.to_owned()
    };

    let path = Path::new(&input_folder).to_path_buf();
    let mut cache = CChainEndPointCache::new(path);

    let cchain = cache.get_cchain_key(&file_key);

    println!("Found:\n{cchain:#?}");
}
