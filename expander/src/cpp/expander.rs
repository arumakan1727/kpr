use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

use anyhow::Context;
use lazy_regex::{lazy_regex, Regex};
use serdable::GlobPattern;

use crate::cpp::assets::BITS_STDCPP_H_SORTED_HEADERS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HeaderSearchMode {
    /// Representation for angle-bracket include (#include <...>)
    OnlyIncludePath,

    /// Representation for double-quote include (#include "...")
    CurrentDirFirst,
}

static RE_INCLUDE_ANGLE_BRA: lazy_regex::Lazy<Regex> = lazy_regex!(r#"^\s*#\s*include\s*<(.+)>"#);
static RE_INCLUDE_DBL_QUOTE: lazy_regex::Lazy<Regex> = lazy_regex!(r#"^\s*#\s*include\s*"(.+)""#);
static RE_PRAGMA_ONCE: lazy_regex::Lazy<Regex> = lazy_regex!(r#"\s*#\s*pragma\s+once"#);

fn extract_include_argument(line: &str) -> Option<(String, HeaderSearchMode)> {
    RE_INCLUDE_ANGLE_BRA
        .captures(line)
        .and_then(|cap| Some((cap[1].trim().to_owned(), HeaderSearchMode::OnlyIncludePath)))
        .or_else(|| {
            RE_INCLUDE_DBL_QUOTE
                .captures(line)
                .map(|cap| (cap[1].trim().to_owned(), HeaderSearchMode::CurrentDirFirst))
        })
}

#[derive(Debug, Clone, Default)]
pub struct Expander<'a> {
    header_serch_dirs: &'a [PathBuf],
    expansion_targets: &'a [GlobPattern],
    black_list: &'a [GlobPattern],

    // [(literal_header_path, mode, header_full_path)]
    include_directive_occurrences: Vec<(String, HeaderSearchMode, PathBuf)>,
    expanded_header_abs_paths: HashSet<PathBuf>,
    found_bits_stdcpp_h: bool,
    generated_code: String,
}

impl<'a> Expander<'a> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn header_serch_dirs(mut self, v: &'a [PathBuf]) -> Self {
        self.header_serch_dirs = v;
        self
    }
    pub fn expansion_targets(mut self, v: &'a [GlobPattern]) -> Self {
        self.expansion_targets = v;
        self
    }
    pub fn black_list(mut self, v: &'a [GlobPattern]) -> Self {
        self.black_list = v;
        self
    }

    pub fn expand(
        mut self,
        source_code_dir: impl AsRef<Path>,
        source_code: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        let dir = fsutil::canonicalize_path(&source_code_dir).context("cpp expander:")?;
        self.emit(dir, source_code);
        Ok(self.get_generated_code())
    }

    pub fn get_generated_code(&self) -> String {
        let mut s = String::with_capacity(256 + self.generated_code.len());

        let mut included_header_abs_paths: HashSet<&Path> =
            HashSet::with_capacity(self.include_directive_occurrences.len());

        if self.found_bits_stdcpp_h {
            s += "#include <bits/stdc++.h>\n";
            included_header_abs_paths.extend(BITS_STDCPP_H_SORTED_HEADERS.iter().map(Path::new));
        }

        for (header, mode, header_full_path) in &self.include_directive_occurrences {
            if self.expanded_header_abs_paths.contains(header_full_path) {
                continue;
            }
            if !included_header_abs_paths.insert(&header_full_path) {
                continue;
            }

            use HeaderSearchMode::*;
            s += &match mode {
                OnlyIncludePath => format!("#include <{}>\n", header),
                CurrentDirFirst => format!("#include \"{}\"\n", header),
            };
        }

        s.push('\n');
        s += &self.generated_code;
        s
    }

    pub fn may_expand(&self, literal_header_path: impl AsRef<str>, mode: HeaderSearchMode) -> bool {
        let literal_header_path = literal_header_path.as_ref();

        if mode == HeaderSearchMode::CurrentDirFirst {
            return true;
        }

        let is_expansion_target = self
            .expansion_targets
            .iter()
            .any(|pat| pat.matches(literal_header_path));

        if !is_expansion_target {
            return false;
        }

        return !self
            .black_list
            .iter()
            .any(|pat| pat.matches(literal_header_path));
    }

    pub fn emit(&mut self, abs_cwd: impl AsRef<Path>, source_code: impl AsRef<str>) -> () {
        let source_code = source_code.as_ref();
        let abs_cwd = abs_cwd.as_ref();

        'line_loop: for line in source_code.lines() {
            if RE_PRAGMA_ONCE.is_match(line) {
                continue;
            }

            let Some((literal_header_path, mode)) = self::extract_include_argument(line) else {
                self.generated_code += line;
                self.generated_code.push('\n');
                continue;
            };

            self.include_directive_occurrences.push((
                literal_header_path.clone().into(),
                mode,
                literal_header_path.clone().into(),
            ));

            if literal_header_path == "bits/stdc++.h" {
                self.found_bits_stdcpp_h = true;
                continue;
            }
            if !self.may_expand(&literal_header_path, mode) {
                continue;
            }

            use ExpansionStatus::*;

            if mode == HeaderSearchMode::CurrentDirFirst {
                let (status, normalized_header_path) =
                    self.check_expansion(abs_cwd, &literal_header_path);

                match status {
                    AlreadyExpanded => continue,
                    MustBeExpanded(content) => {
                        self.expanded_header_abs_paths
                            .insert(normalized_header_path.clone());

                        self.include_directive_occurrences.last_mut().unwrap().2 =
                            normalized_header_path.clone();

                        self.emit(normalized_header_path.parent().unwrap(), content);
                        continue;
                    }
                    _ => (),
                }
            }

            for dir in self.header_serch_dirs {
                let (status, normalized_header_path) =
                    self.check_expansion(dir, &literal_header_path);

                match status {
                    AlreadyExpanded => continue 'line_loop,
                    MustBeExpanded(content) => {
                        self.expanded_header_abs_paths
                            .insert(normalized_header_path.clone());

                        self.include_directive_occurrences.last_mut().unwrap().2 =
                            normalized_header_path.clone();

                        self.emit(normalized_header_path.parent().unwrap(), content);
                        continue 'line_loop;
                    }
                    NoSuchHeaderFile => (),
                }
            }
        }
    }

    /// Returns (expansion_status, normalized_header_path)
    fn check_expansion(
        &self,
        search_dir: impl AsRef<Path>,
        literal_header_path: impl AsRef<Path>,
    ) -> (ExpansionStatus, PathBuf) {
        let written_header_path = literal_header_path.as_ref();
        let normalized_header_path =
            fsutil::normalize_path(search_dir.as_ref().join(written_header_path));

        if self
            .expanded_header_abs_paths
            .contains(&normalized_header_path)
        {
            return (ExpansionStatus::AlreadyExpanded, normalized_header_path);
        }

        if let Ok(content) = fsutil::read_to_string(&normalized_header_path) {
            (
                ExpansionStatus::MustBeExpanded(content),
                normalized_header_path,
            )
        } else {
            (ExpansionStatus::NoSuchHeaderFile, normalized_header_path)
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
enum ExpansionStatus {
    AlreadyExpanded,

    /// .0 = header_content
    MustBeExpanded(String),

    NoSuchHeaderFile,
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn should_be_ok_with_no_config_with_bits_stdcpp_h() {
        let generated = Expander::default()
            .expand(
                std::env::current_dir().unwrap(),
                r#"#include <iostream>
#include <cstdio>
#include <cstdio>
#include <cstdio>
#include <vector>
#include "foo/bar.hpp"
# include<hello/world>   
#include <bits/stdc++.h>
#include <bits/stdc++.h>
#include <bits/stdc++.h>
#include <algorithm>
# include"nyan"
# include"nyan"
# include"nyan"
# include"nyan"
#include "chrono"
#include "chrono"
#include "chrono"
#include "chrono"
#include <hoge>
using namespace std;

int main() {
    cout << "Hello world!" << endl;
}
"#,
            )
            .unwrap();

        println!("{}", generated);

        assert_eq!(
            generated,
            r#"#include <bits/stdc++.h>
#include "foo/bar.hpp"
#include <hello/world>
#include "nyan"
#include <hoge>

using namespace std;

int main() {
    cout << "Hello world!" << endl;
}
"#
        );
    }

    #[test]
    fn should_be_ok_with_no_config_without_bits_stdcpp_h() {
        let generated = Expander::default()
            .expand(
                std::env::current_dir().unwrap(),
                r#"#include <iostream>
#include <cstdio>
#include <cstdio>
#include <cstdio>
#include <cstdio>
#include <vector>
#include "foo/bar.hpp"
# include<hello/world>   
#include <algorithm>
# include"nyan"
# include"nyan"
# include"nyan"
# include"nyan"
#include "chrono"
#include "chrono"
#include "chrono"
#include "chrono"
#include "vector"
#include <hoge>
using namespace std;

int main() {
    cout << "Hello world!" << endl;
}
"#,
            )
            .unwrap();

        println!("{}", generated);

        assert_eq!(
            generated,
            r#"#include <iostream>
#include <cstdio>
#include <vector>
#include "foo/bar.hpp"
#include <hello/world>
#include <algorithm>
#include "nyan"
#include "chrono"
#include <hoge>

using namespace std;

int main() {
    cout << "Hello world!" << endl;
}
"#
        );
    }
}
