use std::{
    collections::HashSet,
    path::{Path, PathBuf},
};

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

#[derive(Debug, Clone)]
pub struct Expander {
    header_serch_dirs: Vec<PathBuf>,
    expansion_targets: Vec<Pattern>,
    black_list: Vec<Pattern>,

    // [(literal_header_path, mode, header_full_path)]
    include_directive_occurrences: Vec<(String, HeaderSearchMode, PathBuf)>,
    expanded_header_abs_paths: HashSet<PathBuf>,
    includes_bits_stdcpp_h: bool,
    generated_code: String,
}

#[derive(Debug, Clone, Default)]
pub struct ExpanderBuilder {
    header_serch_dirs: Vec<PathBuf>,
    expansion_targets: Vec<Pattern>,
    black_list: Vec<Pattern>,
}

impl ExpanderBuilder {
    pub fn header_serch_dirs(mut self, v: Vec<PathBuf>) -> Self {
        self.header_serch_dirs = v;
        self
    }
    pub fn expansion_targets(mut self, v: Vec<Pattern>) -> Self {
        self.expansion_targets = v;
        self
    }
    pub fn black_list(mut self, v: Vec<Pattern>) -> Self {
        self.black_list = v;
        self
    }
    pub fn expand(
        self,
        source_code_dir: impl AsRef<Path>,
        source_code: impl AsRef<str>,
    ) -> anyhow::Result<String> {
        let dir = fsutil::canonicalize_path(source_code_dir)?;
        let e = Expander {
            header_serch_dirs: self.header_serch_dirs,
            expansion_targets: self.expansion_targets,
            black_list: self.black_list,

            includes_bits_stdcpp_h: false,
            include_directive_occurrences: Vec::with_capacity(128),
            expanded_header_abs_paths: HashSet::with_capacity(16),
            generated_code: String::with_capacity(source_code.as_ref().len()),
        };
        Ok(e.expand(dir, source_code).get_generated_code())
    }
}

impl Expander {
    pub fn builder() -> ExpanderBuilder {
        ExpanderBuilder::default()
    }

    pub fn get_generated_code(&self) -> String {
        let mut s = String::with_capacity(256 + self.generated_code.len());

        let mut included_header_abs_paths: HashSet<&Path> =
            HashSet::with_capacity(self.include_directive_occurrences.len());

        if self.includes_bits_stdcpp_h {
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

    pub fn expand(mut self, abs_cwd: impl AsRef<Path>, source_code: impl AsRef<str>) -> Self {
        let source_code = source_code.as_ref();
        let abs_cwd = abs_cwd.as_ref();

        'line_loop: for line in source_code.lines() {
            if RE_PRAGMA_ONCE.is_match(line) {
                continue;
            }

            let Some((literal_header_path, mode)) = Self::extract_include_argument(line) else {
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
                self.includes_bits_stdcpp_h = true;
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

                        self = self.expand(normalized_header_path.parent().unwrap(), content);
                        continue;
                    }
                    _ => (),
                }
            }

            for dir in &self.header_serch_dirs {
                let (status, normalized_header_path) =
                    self.check_expansion(dir, &literal_header_path);

                match status {
                    AlreadyExpanded => continue 'line_loop,
                    MustBeExpanded(content) => {
                        self.expanded_header_abs_paths
                            .insert(normalized_header_path.clone());

                        self.include_directive_occurrences.last_mut().unwrap().2 =
                            normalized_header_path.clone();

                        self = self.expand(normalized_header_path.parent().unwrap(), content);
                        continue 'line_loop;
                    }
                    NoSuchHeaderFile => (),
                }
            }
        }
        self
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
        let generated = Expander::builder()
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
        let generated = Expander::builder()
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
