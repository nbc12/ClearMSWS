use rust_embed::RustEmbed;
use tempfile::TempDir;

// re-export rust-oracle
pub use oracle::*;

#[derive(RustEmbed)]
// Just include junk in debug mode, no need to zip the files.
// The libs are always going to be installed locally for dev mode anyway.
#[cfg_attr(debug_assertions, folder = "./src/")]
#[cfg_attr(not(debug_assertions), folder = "./instantclient_21_6/")]
// #[folder = "./instantclient_21_6/"]
struct OIC;

/// Loads the oracle client libs into a temporary directory with `Self::extract_oic` if they are not already installed.
pub struct OracleClient {
    temp_dir: Option<TempDir>,
    pub db_host: String,
    pub db_port: u16,
    pub db_svc: String,
    pub db_user: String,
    pub db_pass: String,
}

impl OracleClient {
    pub fn new(
        db_host: String,
        db_port: u16,
        db_svc: String,
        db_user: String,
        db_pass: String,
    ) -> Self {
        Self {
            temp_dir: None,
            db_host,
            db_port,
            db_svc,
            db_user,
            db_pass,
        }
    }

    /// Creates a String from the builtin config in DB_HOST, DB_PORT, and DB_SVC
    pub fn con_str(&self) -> String {
        format!("{}:{}/{}", self.db_host, self.db_port, self.db_svc)
    }

    /// Extracts the oracle client libs if they fail to load.
    pub fn extract_oic(&mut self) {
        // if we error on loading the OIC libs
        if oracle::Version::client().is_err() {
            let temp_dir = tempfile::tempdir(). unwrap();

            // extract the OIC to the temp dir
            OIC::iter().for_each(|f| {
                let file = OIC::get(&f). unwrap();
                let path = temp_dir.path().join(&*f);

                // We don't need to worry about handling the results here, because I have no way of handling not being able to create a directory.
                // Not to mention if it doesn't work, rust-oracle will give the user instructions to install the OIC libs themselves anyway.
                match std::fs::create_dir_all(path.parent(). unwrap()) {
                    Ok(_) => (),
                    Err(_) => (),
                };
                match std::fs::write(path, file.data) {
                    Ok(_) => (),
                    Err(_) => (),
                };
            });

            // add the temp dir to PATH
            let mut v: Vec<_> = std::env::split_paths(&std::env::var_os("PATH").unwrap()).collect();
            v.push(temp_dir.path().to_path_buf());
            std::env::set_var("PATH", std::env::join_paths(v).unwrap());

            self.temp_dir = Some(temp_dir);
        }
    }

    pub fn get_con(&self) -> oracle::Result<Connection> {
        let con_str = self.con_str();
        println!("Connecting to {}...", &con_str);
        oracle::Connection::connect(&self.db_user, &self.db_pass, &con_str)
    }
}

#[test]
fn test() -> oracle::Result<()> {
    fn get_polyline_ogc_geom_points(t: &SqlValue) -> oracle::Result<Vec<(f64, f64)>> {
        use oracle::sql_type::{Collection, Object};
    
        let r: Object = t.get()?;
        let t: Collection = r.get("SDO_ORDINATES")?;
        let first_idx = t.first_index()?;
        let last_idx = t.last_index()?;
        let mut idx = first_idx;
        let mut points: Vec<f64> = Vec::new();
        loop {
            if idx > last_idx {
                break;
            }
            match t.get(idx) {
                Ok(p) => points.push(p),
                Err(_) => (),
            }
            idx += 1;
            // idx = t.next_index(idx)?;
        }
        if points.len() % 2 == 1 {
            Err(Error::InternalError(
                "Non even number of coordinates!".to_string(),
            ))?
        }
        Ok(points.chunks(2).map(|t| (t[0], t[1])).collect())
    }

    let client = OracleClient::new(
        "".to_string(),
        1521,
        "".to_string(),
        "".to_string(),
        "".to_string(),
    );
    let con = client.get_con()?;
    let res = con.query("select ogc_geometry from fiberspan", &[])?;
    let p: Vec<Vec<(f64, f64)>> = res
        .into_iter()
        .map(|obj| {
            let obj = obj.unwrap();
            let t = &obj.sql_values()[0];
            get_polyline_ogc_geom_points(&t).unwrap()
        })
        .collect();
    println!("finished getting all: {}", p.len());
    let l = p.len();
    let av = p
        .into_iter()
        .map(|p| {
            let len = p.len();
            let sum = p.into_iter().reduce(|u, v| (u.0 + v.0, u.1 + v.1)).unwrap();
            (sum.0 / len as f64, sum.1 / len as f64)
        })
        .reduce(|u, v| (u.0 + v.0, u.1 + v.1))
        .unwrap();
    let r = (av.0 / l as f64, av.1 / l as f64);
    println!("{:?}", r);
    Ok(())
}
