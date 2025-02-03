use crate::config::solution::{Solution, Lang};
use ocj_config::{self as config, solution::Verdict, tests::Config};
use tokio::{fs::{self, File}, io::AsyncWriteExt, process::Command, task::JoinHandle};

pub const DIR: &str = "solutions";

pub async fn judge(solution: Solution) -> Result<Verdict, ()> {
    _ = fs::create_dir(format!("{DIR}/{}", solution.id)).await;
    let file_path = format!("{DIR}/{}/solution.{}", solution.id, solution.lang.file_ext());

    let file = File::create(&file_path).await;
    let mut file = if let Ok(file) = file {
        file
    } else {
        log::error!("can't create file");
        return Err(());
    };

    file.write(solution.code.as_bytes()).await.unwrap();

    let mut command = match solution.lang {
        Lang::Cpp => {
            let mut c = Command::new("g++");
            c.args(&[
                format!("{file_path}"),
                "-o".to_string(), format!("{DIR}/{}/solution", solution.id),
            ]);
            c
        }
    };

    let tests_dir = format!("{}/{}/", config::file::TESTS, solution.problem_number);

    let problem_config = if let Ok(conf) = serde_json::from_str::<Config>(
        if let Ok(s) = tokio::fs::read_to_string(format!("{tests_dir}{}.json", crate::config::file::PROBLEM_TEST_CONFIG)).await {
            s
        } else {
            log::error!("cannot find config file for {} problem", solution.problem_number);
            return Err(());
        }.as_str()
    ) {
        conf
    } else {
        log::error!("cannot parse config file for {} problem", solution.problem_number);
        return Err(());
    };

    

    let status = command.status().await;
    let status = if let Ok(status) = status {
        status
    } else {
        return Ok(Verdict::Ce);
    };

    if !status.success() {
        return Ok(Verdict::Ce);
    }

    let mut tasks = Vec::<JoinHandle<Verdict>>::new();

    for test_number in 1..=(problem_config.test_count) {
        tasks.push(tokio::spawn(async move {
            let mut command = Command::new(format!("{DIR}/{}/solution", solution.id));
            command.stdin(std::fs::File::open(format!("{}/{}/{test_number}.in", config::file::TESTS, solution.problem_number)).unwrap());
            command.stdout(std::fs::File::create(format!("{DIR}/{}/{test_number}.out", solution.id)).unwrap());

            let status = command.status().await;
            if let Ok(status) = status {
                if !status.success() {
                    return Verdict::Re;
                }
            } else {
                return Verdict::Re;
            };

            command = Command::new(format!("{}/{}/checker", config::file::TESTS, solution.problem_number));
            command.arg(format!("{}/{}/{test_number}.out", config::file::TESTS, solution.problem_number));
            command.arg(format!("{DIR}/{}/{test_number}.out", solution.id));

            let output = command.output().await;
            if let Ok(output) = output {
                let v = if let Ok(v) = String::from_utf8(output.stdout) {
                    v
                } else {
                    log::error!("incorrect checker verdict");
                    return Verdict::Pe;
                };

                match v.as_str() {
                    "Ok" => {
                        Verdict::Ok
                    },

                    "Wa" => {
                        Verdict::Wa
                    },

                    "Pe" => {
                        Verdict::Pe
                    },

                    _ => {
                        log::error!("incorrect checker verdict");
                        Verdict::Pe
                    },
                }
            } else {
                log::error!("checker output incorrect");
                Verdict::Pe
            }
        }));
    }

    let mut res_verdict = Verdict::Ok;

    for (test_number, handle) in tasks.into_iter().enumerate().map(|(n, h)| (n + 1, h)) {
        let verdict = handle.await.unwrap();
        tokio::fs::remove_file(format!("{DIR}/{}/{test_number}.out", solution.id)).await.unwrap();
        if Verdict::Ok != verdict && res_verdict == Verdict::Ok {
            res_verdict = verdict;
        }
    }

    tokio::fs::remove_dir_all(format!("{DIR}/{}", solution.id)).await.unwrap();
    log::debug!("{res_verdict:?}");
    Ok(res_verdict)
}