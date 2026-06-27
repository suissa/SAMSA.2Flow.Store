use twoflow_query::query_2flow_sql;

#[tokio::main]
async fn main() -> datafusion::error::Result<()> {
    let input = include_bytes!("basic.2flow");
    let batches = query_2flow_sql(
        input,
        "SELECT object FROM graph WHERE subject = 'Torre_Eiffel' AND predicate = 'altura'",
    )
    .await?;
    for batch in batches {
        println!("{batch:?}");
    }
    Ok(())
}
