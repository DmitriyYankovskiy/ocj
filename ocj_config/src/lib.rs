pub mod port {
    pub const HTTP_FOR_CLIENT: u16 = 5504;
    pub const WS_FOR_MACHINE: u16 = 5505;
    pub const HTTP_FOR_ADMIN: u16 = 5506;
}

pub mod auth {
    pub const SECURE_TOKEN_HTTP_HEADER: &'static str = "Access-Token";

    use std::str::FromStr;

    use serde::{Serialize, Deserialize};
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
    pub struct Token(pub u128);
    impl FromStr for Token {
        type Err = <u128 as FromStr>::Err;
    
        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(Self(u128::from_str(s)?))
        }
    }

    impl ToString for Token {
        fn to_string(&self) -> String {
            self.0.to_string()
        }
    }
}

pub mod file {
    pub const STATEMENTS: &str = "statements"; 
    pub const TESTS: &str = "tests";
    pub const PROBLEM_TEST_CONFIG: &str = "config";
}

pub mod contest {
    pub use serde::{Serialize, Deserialize};
    #[derive(Debug, Serialize, Deserialize)]
    pub struct Time {
        start: std::time::SystemTime,
        duration: Option<std::time::Duration>,
    }

    pub type File = [u8];
}

pub mod msg {
    use std::time;

    use serde::{Serialize, Deserialize};
    use crate::{solution, contest};

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum ServerToMachine {
        JudgeSolution(solution::Solution),
        UpdateTests(Box<contest::File>),
        InitFailed,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum MachineToServer {
        Init,
        JudgeResult(solution::JudgeResult),
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum UpdateDuration {
        Add(time::Duration),
        Sub(time::Duration),
        Set(time::Duration),
    }

    pub mod admin_to_server {
        use super::contest::File;
        pub mod contest {
            use super::*;
            pub mod tests {
                use super::*;
                pub type Update = Box<File>;
            }
            pub mod time {
                use super::*;
                pub type Update = Box<File>;
            }
        }
        pub mod tokens {
            pub type Get = Box<str>;
        }

    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum ServerToAdmin<T> {
        Ok(T),
        Err(Box<str>),
    }
    // #[derive(Debug, Serialize, Deserialize)]
    // pub enum AdminToServer {
    //     UpdateContest {
    //         contest: Vec<u8>,
    //     },
    //     BeingReady {
    //         time: contest::Time,
    //     },
    //     StartNow {
    //         time: contest::Time,
    //     },
    //     FinishNow,
    //     UpdateStartTime(time::SystemTime),
    //     UpdateDuration(UpdateDuration),
    // }
}

pub mod solution {
    use serde::{Serialize, Deserialize};

    pub type Id = u128;
    pub type ProblemNum = u16;

    #[derive(Clone, Serialize, Deserialize, Eq, PartialEq, Debug)]
    pub enum Verdict {
        Ok, Wa, Re, Tl, Ml, Pe, Ce,
    }

    #[derive(Clone, Serialize, Deserialize, Debug)]
    pub struct JudgeResult {
        pub solution_id: Id,
        pub verdict: Verdict,
        pub score: u8,
        pub problem_number: ProblemNum,
    }

    #[derive(Clone, Debug, Serialize, Deserialize)]
    pub enum Lang {
        Cpp,
    }

    impl Lang {
        pub fn file_ext(&self) -> &'static str {
            match self {
                Self::Cpp => "cpp",
            }
        }
    }

    #[derive(Debug, Serialize, Deserialize, Clone)]
    pub struct Solution {
        pub code: String,
        pub lang: Lang,

        pub problem_number: ProblemNum,
        pub id: Id,
    }

}

pub mod tests {
    use serde::Deserialize;

    #[derive(Debug, Deserialize)]
    pub struct Config {
        pub test_count: u16,
    }
}