use std::{borrow::Borrow, collections::HashMap, ffi::OsStr, hash::Hash};

pub type Result = std::result::Result<String, InterpError>;

#[derive(Debug, PartialEq, Eq, thiserror::Error)]
pub enum InterpError {
    #[error("Undefined variable '{0}' at {} (fmt={2})", .1+1)]
    UndefinedVar(String, usize, String),

    #[error("Unclosed brace (found open brace at {}, fmt={1})", .0+1)]
    UnclosedBrace(usize, String),
}

pub fn interp<K, V>(fmt: &str, variables: &HashMap<K, V>) -> Result
where
    K: Borrow<str> + Hash + Eq,
    V: AsRef<OsStr>,
{
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    enum State {
        Normal,
        HashMark,
        InsideBrace,
    }
    use State::*;

    let mut state = Normal;
    let mut pos_open_brace = 0;
    let mut res = String::with_capacity(fmt.len() * 3);
    let mut var_name = String::with_capacity(32);

    for (i, c) in fmt.chars().enumerate() {
        match (c, state) {
            ('#', Normal) => {
                state = HashMark;
                res.push(c);
            }
            ('#', HashMark) => {
                state = Normal;
            }
            ('{', HashMark) => {
                state = InsideBrace;
                pos_open_brace = i;
                var_name.clear();
                res.pop(); // remove '#'
            }
            ('}', InsideBrace) => {
                state = Normal;
                let Some(value) = variables.get(&var_name) else {
                    return Err(InterpError::UndefinedVar(var_name, pos_open_brace + 1, fmt.to_owned()))
                };
                res += value.as_ref().to_string_lossy().as_ref();
            }
            (_, InsideBrace) => {
                var_name.push(c);
            }
            _ => {
                state = Normal;
                res.push(c);
            }
        }
    }

    if state == InsideBrace {
        Err(InterpError::UnclosedBrace(pos_open_brace, fmt.to_owned()))
    } else {
        res.shrink_to_fit();
        Ok(res)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn interp_ok() {
        let vars = {
            let mut m = HashMap::new();
            m.insert("firstName", "Liz");
            m.insert("lastName", "Smith");
            m.insert("age", "999");
            m.insert("_#%!?", "wooo");
            m
        };

        assert_eq!(interp("hello", &vars).unwrap(), "hello");
        assert_eq!(interp("#{firstName}", &vars).unwrap(), vars["firstName"]);
        assert_eq!(interp("#{age}", &vars).unwrap(), vars["age"]);
        assert_eq!(interp("#{_#%!?}", &vars).unwrap(), vars["_#%!?"]);
        assert_eq!(
            interp("#{firstName}#{lastName}", &vars).unwrap(),
            format!("{}{}", vars["firstName"], vars["lastName"])
        );
        assert_eq!(
            interp("firstName=#{firstName}, lastName=#{lastName}", &vars).unwrap(),
            format!(
                "firstName={}, lastName={}",
                vars["firstName"], vars["lastName"]
            )
        );
        assert_eq!(
            interp("abc #{age} xyz", &vars).unwrap(),
            format!("abc {} xyz", vars["age"])
        );
        assert_eq!(interp("abc {age} xyz", &vars).unwrap(), "abc {age} xyz");
        assert_eq!(interp("abc # {age} xyz", &vars).unwrap(), "abc # {age} xyz");
        assert_eq!(interp("abc #age xyz", &vars).unwrap(), "abc #age xyz");
        assert_eq!(interp("abc ##{age} xyz", &vars).unwrap(), "abc #{age} xyz");
        assert_eq!(interp("abc ## xyz", &vars).unwrap(), "abc # xyz");
        assert_eq!(interp("#", &vars).unwrap(), "#");
        assert_eq!(interp("##", &vars).unwrap(), "#");
        assert_eq!(interp("###", &vars).unwrap(), "##");
    }

    #[test]
    fn interp_ng() {
        let vars = {
            let mut m = HashMap::new();
            m.insert("age", "999");
            m
        };
        let fmt = "#{firstName} #{lastName}";
        assert_eq!(
            interp(fmt, &vars).unwrap_err(),
            InterpError::UndefinedVar("firstName".to_owned(), 2, fmt.to_owned())
        );
        let fmt = "#{age} #{hello";
        assert_eq!(
            interp(fmt, &vars).unwrap_err(),
            InterpError::UnclosedBrace(8, fmt.to_owned()),
        );
    }
}
