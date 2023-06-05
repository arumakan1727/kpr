use scraper::{ElementRef, Html};

use crate::{error::*, model::SampleTestcase, util};

fn extract_testcase(pre: ElementRef) -> String {
    let node = pre.first_child().unwrap().value();
    let mut s = node.as_text().unwrap().trim().to_owned();
    s.push('\n');
    s
}

pub fn scrape_testcases(doc: &Html) -> Result<Vec<SampleTestcase>> {
    let sel_parts_modern_ver = util::selector_must_parsed("#task-statement .lang-ja > .part");
    let sel_parts_old_ver = util::selector_must_parsed("#task-statement > .part");
    let sel_h3 = util::selector_must_parsed("h3");
    let sel_pre = util::selector_must_parsed("pre");

    let mut in_cases = Vec::with_capacity(5);
    let mut out_cases = Vec::with_capacity(5);

    for node in doc
        .select(&sel_parts_modern_ver)
        .chain(doc.select(&sel_parts_old_ver))
    {
        let h3 = node.select(&sel_h3).next().unwrap();
        let title = h3.text().next().unwrap().trim().to_lowercase();
        if title.starts_with("入力例") || title.starts_with("sample input") {
            let pre = node.select(&sel_pre).next().unwrap();
            in_cases.push(extract_testcase(pre));
        } else if title.starts_with("出力例") || title.starts_with("sample output") {
            let pre = node.select(&sel_pre).next().unwrap();
            out_cases.push(extract_testcase(pre));
        }
    }
    let cases: Vec<_> = in_cases
        .into_iter()
        .zip(out_cases)
        .enumerate()
        .map(|(i, (input, output))| SampleTestcase {
            ord: (i + 1) as u32,
            input,
            output,
        })
        .collect();
    Ok(cases)
}
