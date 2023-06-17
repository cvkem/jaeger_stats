use jaeger_stats::{
    read_jaeger_trace_file, JaegerTrace};
use std::{
    collections::HashMap,
    error::Error};


const SHOW_STDOUT: bool = false;
const INPUT_FILE: &str = "/home/ceesvk/Downloads/372e70a4e259978e.json";

fn collect_span_tags(jt: &JaegerTrace) -> (u32, HashMap<String, u32>) {
    let mut span_tags = HashMap::new();
    let mut num_spans = 0;
    jt.data
        .iter()
        .for_each(|ji| {
            ji.spans
                .iter()
                .for_each(|span| {
                    num_spans += 1;
                    span.tags
                        .iter()
                        .for_each(|tag| {
                            // increment or insert 1
                            span_tags.entry(tag.key.to_owned()).and_modify(|counter| *counter += 1).or_insert(1);

                        })
                })
        });
    (num_spans, span_tags)
}


fn show_span_percentage(num_spans: u32, span_tag_cnt: &HashMap<String, u32>) {
    let multiplier = 100.0 / num_spans as f64;
    let mut data = span_tag_cnt
        .into_iter()
        .collect::<Vec<_>>();
 
    // sort on Count decending
    data.sort_by(|a,b| b.1.cmp(&a.1));
    data
        .iter()
        .enumerate()
        .for_each(|(idx, (key, cnt))| {
            let key = *key;
            let cnt = **cnt;
            let perc = cnt as f64 * multiplier;
            let prefix = format!("{idx}:  {key}");
            let tabs = match prefix.len() {
                0..=7 => "\t\t\t\t\t",
                8..=15 => "\t\t\t\t",
                16..=23 => "\t\t\t",
                24..=31 => "\t\t",
                32..=39 => "\t",
                _ => ""
            };
            println!("{prefix}{tabs}has count {cnt}  ({perc:.1}%)");
        })
}


fn main() -> Result<(), Box<dyn Error>> {
    println!("Reading a Jaeger-trace from '{INPUT_FILE}'");
    let jt = read_jaeger_trace_file(INPUT_FILE).unwrap();

    if SHOW_STDOUT {
        println!("{:#?}", jt);
    }


    let (num_spans, span_tag_cnt) =  collect_span_tags(&jt);
    println!("For input_file {INPUT_FILE} observed {num_spans} spans in total");
    println!("span-tag-counts are: {span_tag_cnt:#?}");

    show_span_percentage(num_spans, &span_tag_cnt);
    // let mut file = File::create(OUTPUT_FILE)?;
    // file.write_all(s.as_bytes())?;

Ok(())
}