use crate::executor::Triple;
use crate::json::parse_to_triples_json;
use datafusion::error::{DataFusionError, Result};

#[derive(Debug, serde::Deserialize)]
struct TriplesDocument {
    triples: Vec<Triple>,
}

pub fn parse_twoflow_triples(input: &[u8]) -> Result<Vec<Triple>> {
    let json = parse_to_triples_json(input)?;
    let doc: TriplesDocument = serde_json::from_str(&json).map_err(|err| {
        DataFusionError::Execution(format!(
            "failed to decode Zig canonical triples JSON: {err}"
        ))
    })?;
    Ok(doc.triples)
}
