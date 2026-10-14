use std::io;

use io::BufWriter;
use io::Write;

use redb::ReadOnlyDatabase;
use redb::ReadableDatabase;

use redb::ReadOnlyTable;
use redb::ReadableTable;

use redb::ReadTransaction;

use redb::TableDefinition;

pub struct Db(pub ReadOnlyDatabase);

impl Db {
    pub fn kvpairs2sink<S>(&self, sink: &mut S, table_name: &str) -> Result<(), io::Error>
    where
        S: FnMut(&str, &str) -> Result<(), io::Error>,
    {
        let tdef: TableDefinition<&str, &str> = TableDefinition::new(table_name);
        let tx: ReadTransaction = self.0.begin_read().map_err(io::Error::other)?;
        let tab: ReadOnlyTable<&str, &str> = tx.open_table(tdef).map_err(io::Error::other)?;
        let rng = tab.iter().map_err(io::Error::other)?;
        for rpair in rng {
            let pair = rpair.map_err(io::Error::other)?;
            let (key, val) = pair;
            let k: &str = key.value();
            let v: &str = val.value();
            sink(k, v)?;
        }
        Ok(())
    }
}

impl Db {
    pub fn path2sink<S>(
        path2redb: String,
        mut sink: S,
        table_name: String,
    ) -> impl FnOnce() -> Result<(), io::Error>
    where
        S: FnMut(&str, &str) -> Result<(), io::Error>,
    {
        move || {
            let rod = ReadOnlyDatabase::open(path2redb).map_err(io::Error::other)?;
            Self(rod).kvpairs2sink(&mut sink, &table_name)
        }
    }
}

pub fn wtr2sink_simple<W>(mut wtr: W) -> impl FnMut(&str, &str) -> Result<(), io::Error>
where
    W: Write,
{
    move |key: &str, val: &str| {
        let v = &[key, val];
        serde_json::to_writer(&mut wtr, v)?;
        writeln!(&mut wtr)
    }
}

pub fn redb2kvpairs2wtr<W>(
    redb_path: String,
    mut wtr: W,
    table_name: String,
) -> Result<(), io::Error>
where
    W: Write,
{
    Db::path2sink(redb_path, wtr2sink_simple(&mut wtr), table_name)()?;
    wtr.flush()
}

pub fn redb2kvpairs2stdout(
    redb_path: String,
    table_name: String,
) -> impl FnOnce() -> Result<(), io::Error> {
    move || {
        let o = io::stdout();
        let mut ol = o.lock();
        redb2kvpairs2wtr(redb_path, BufWriter::new(&mut ol), table_name)?;
        ol.flush()
    }
}

#[derive(Debug)]
pub struct Config {
    pub redb_path: String,
    pub table_name: String,
}

impl Config {
    pub fn into_redb2kvpairs2stdout(self) -> impl FnOnce() -> Result<(), io::Error> {
        redb2kvpairs2stdout(self.redb_path, self.table_name)
    }
}
