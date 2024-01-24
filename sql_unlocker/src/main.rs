use colour::{green_ln, red_ln};
use oracle_lib::OracleClient;

fn run(client: OracleClient) -> oracle_lib::Result<()> {
    let con = client.get_con()?;
    let count_ws: usize = con
        .query(
            "SELECT count(*) FROM WMSYS.WM$WORKSPACES_TABLE$ WHERE workspace != 'LIVE'",
            &[],
        )?
        .nth(0)
        .unwrap()?
        .get(0)?;

    green_ln!(
        "{}",
        match count_ws {
            0 => "No workspaces found".to_string(),
            _ => {
                con.execute(
                    "begin for workspaces in (SELECT workspace FROM WMSYS.WM$WORKSPACES_TABLE$ WHERE workspace != 'LIVE') loop wmsys.lt.mergeworkspace(workspaces.workspace, TRUE, TRUE); end loop; end;",
                    &[]
                );

                let after_merge_count_ws: usize = con
                    .query(
                        "SELECT count(*) FROM WMSYS.WM$WORKSPACES_TABLE$ WHERE workspace != 'LIVE'",
                        &[],
                    )?
                    .nth(0)
                    .unwrap()?
                    .get(0)?;
                format!("Deleted {} workspaces", count_ws - after_merge_count_ws)
            }
        }
    );

    con.commit()?;
    con.close()?;
    Ok(())
}

fn main() {
    println!(
        "{} {} by {}",
        env!("CARGO_PKG_NAME"),
        env!("CARGO_PKG_VERSION"),
        env!("CARGO_PKG_AUTHORS").replace(':', ", ")
    );

    let mut client = OracleClient::new(
        "".to_string(),
        1521,
        "".to_string(),
        "".to_string(),
        "".to_string(),
    );

    client.extract_oic();

    match run(client) {
        Ok(_) => (),
        Err(e) => {
            red_ln!(
                "\n{}\n{}",
                e.to_string(),
                std::error::Error::source(&e)
                    .map(|t| t.to_string())
                    .unwrap_or(String::new())
            );
        }
    };
}
