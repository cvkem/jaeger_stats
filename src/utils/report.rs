// use crate::report::Chapter;
use std::{fs::File, io::Write, sync::Mutex};

#[derive(Copy, Clone)]
#[repr(u8)]
pub enum Chapter {
    Summary = 0,
    Issues,
    Ingest,
    Analysis,
    Details,
}

static CHAPTER_NAMES: [&str; 5] = ["Summary", "Issues", "Ingest", "Analysis", "Details"];

impl Chapter {
    fn discriminant(&self) -> usize {
        // first map to u8, otherwise you might get high or negative numbers
        unsafe { *(self as *const Self as *const u8) as usize }
    }
}

static STORE: Mutex<Vec<Vec<String>>> = Mutex::new(Vec::new());

pub fn report(chapter: Chapter, msg: String) {
    let idx = chapter.discriminant();
    if idx == Chapter::Summary as usize {
        println!("{msg}");
    }

    {
        let mut guard = STORE.lock().unwrap();
        while guard.len() <= idx {
            guard.push(Vec::new());
        }
        guard[idx].push(msg);
    }
}

pub fn write_report(path: &str) {
    let mut guard = STORE.lock().unwrap();
    let contents = (0..guard.len())
        .map(|idx| format!("{}\n{}\n\n", CHAPTER_NAMES[idx], guard[idx].join("\n")))
        .collect::<Vec<_>>()
        .join("\n");
    let mut f = File::create(path).expect("Failed to create report-file");
    f.write_all(contents.as_bytes())
        .expect("Failed to write to report.");

    // wipe the contents
    *guard = Vec::new();
}
