use lazy_static::lazy_static;
use regex::Regex;

fn replace_regex(s: String, re: &Regex, replacement: &str) -> (String, bool) {
    //let s_clone = s.clone();
    // TODO: this clone is needed to satisfy the borrow-checker.
    // Without Clone the 's' would be borrowed to build 'found'. S will be moved again as initial value for the iterator.
    // this might be a nice example where you trip if you do not have a borrow-checker in place
    // However, the clone above could move down as it is only needed in case we observe replacements (preventing needless clones)
    let found: Vec<&str> = re.find_iter(&s).map(|m| m.as_str()).collect();
    if !found.is_empty() {
        let s = found
            .into_iter()
            .fold(s.clone(), |s, obs| s.replace(obs, replacement));
        (s, true)
    } else {
        (s, false)
    }
}

pub fn unified_operation_name(js_operation: &str) -> (String, Option<String>) {
    lazy_static! {
        static ref REPLACEMENTS: Vec<(&'static str, Regex)> = {
            vec![("/{TIME}", Regex::new(r"(?x)
                    /T\d{4}-\d{2}-\d{2}_
                    \d{5,10}").unwrap() ),
                   // should possibly be merged with previous pattern
                 ("/{TIME2}", Regex::new(r"(?x)
                    /\d{4}-\d{2}-\d{2}_
                    \d{5,10}").unwrap() ),
                ("/{SAVINGS}", Regex::new(r"(?x)
                /[0-9a-f]{8}-
                [0-9a-f]{4}-
                [0-9a-f]{4}-
                [0-9a-f]{4}-
                [0-99-f]{12}").unwrap() ),
                ("/{BASE}/", Regex::new(r"(?x)
                    /[a-zA-Z0-9\-_]{39,40}
                    ={0,1}
                    /").unwrap()),
                ("-{VIEW}", Regex::new(r"\-\d{5,9}\-20\d{2}").unwrap() ),
                ("/{ACCOUNT}", Regex::new(r"/\d{6,10}").unwrap())
                ]
        };
    }

    let chained_update = |(oper_name, any_replaced), (label, pattern): &(&str, Regex)| {
        // Type annotation needed
        let (oper_name, replaced) = replace_regex(oper_name, pattern, label);
        (oper_name, any_replaced || replaced)
    };

    let (oper_name, replaced) = REPLACEMENTS
        .iter()
        .fold((js_operation.to_owned(), false), chained_update);

    if replaced {
        (oper_name, Some(js_operation.to_owned()))
    } else {
        (oper_name, None)
    }
}
