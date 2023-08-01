use lazy_static::lazy_static;
use regex::Regex;
use std::fs;

// lazy_static! {
//     static ref RE: Regex = Regex::new(r"(?x)
//         ^(?P<login>[^@\s]+)@
//         ([[:word:]]+\.)*
//         [[:word:]]+$
//         ").unwrap();
// }

fn main() {
    // yVsG3MmWC1e_PtwgOnAPsfE26-ZcONkUP-eRZsk=
    // Yq_Pf0vealPhJkccUmAecW4WHTd_mXQOZhRRByF2
    // iFuSy02923vSn8lTePUoKAVwbb2yzry_1bfwSkc=
    let haystack = fs::read_to_string("ble").unwrap();

    //    let re_rek = Regex::new(r"/\d{4,10}/").unwrap();
    //    let re_rek = Regex::new(r"/T\d{4}-\d{2}-\d{2}_\d{5,10};").unwrap();  // should be EOL
    //let re_rek = Regex::new(r"/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{4}-[0-99-f]{12}").unwrap(); // should be EOL
    lazy_static! {
        static ref re_rek: Regex = Regex::new(r"/\d{8,11}/").unwrap();  // actually 9-10
        static ref re_time: Regex = Regex::new(r"(?x)
        /T\d{4}-\d{2}-\d{2}_
        \d{5,10};").unwrap();
        static ref re_base: Regex = Regex::new(r"(?x)
        /[a-zA-Z0-9\-_]{39,40}
        ={0,1}
        /").unwrap();

    }
    println!("Finding matches");
    //let re_rek = Regex::new(r"/[0-9a-f]{8}-[0-9a-f]{4}-[0-9a-f]{4}-[0-9a-f]{2,14};").unwrap();
    // let mut results = vec![];
    // for (_, [path, lineno, line]) in re_rek.captures_iter(&haystack).map(|c| c.extract()) {
    //     println!("path={path}   lineno={lineno} and lines='{line}");
    //     results.push((path, lineno.parse::<u64>().unwrap(), line));
    // }
    let found: Vec<&str> = re_time.find_iter(&haystack).map(|m| m.as_str()).collect();
    let lens: Vec<_> = found.iter().map(|s| s.len()).collect();

    println!("found {} matches", found.len());
    println!("{found:?}");
    println!("lengths = {lens:?}");

    let f = found
        .iter()
        .filter(|s| !s.ends_with("=/"))
        .collect::<Vec<_>>();
    println!("NON= {f:?}");
}
