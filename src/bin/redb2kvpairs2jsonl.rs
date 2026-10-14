use std::io;
use std::process::ExitCode;

use rs_redb_dump_kvpairs2jsonl::Config;

fn io_envkey2str(key: &'static str) -> impl Fn() -> String {
    move || std::env::var(key).unwrap_or_default()
}

fn io_redb_path() -> impl Fn() -> String {
    io_envkey2str("ENV_REDB_PATH")
}

fn io_table_name() -> impl Fn() -> String {
    io_envkey2str("ENV_TABLE_NAME")
}

fn io_config() -> impl Fn() -> Config {
    || {
        let redb_path: String = io_redb_path()();
        let table_name: String = io_table_name()();
        Config {
            redb_path,
            table_name,
        }
    }
}

fn io_main() -> impl FnOnce() -> Result<(), io::Error> {
    io_config()().into_redb2kvpairs2stdout()
}

fn sub() -> Result<(), io::Error> {
    io_main()()
}

fn main() -> ExitCode {
    sub().map(|_| ExitCode::SUCCESS).unwrap_or_else(|e| {
        eprintln!("{e}");

        let cfg: Config = io_config()();
        let Config {
            redb_path,
            table_name,
        } = cfg;

        eprintln!("ENV_REDB_PATH: {redb_path}");
        eprintln!("ENV_TABLE_NAME: {table_name}");

        ExitCode::FAILURE
    })
}
