use crate::ffi::{parse_json_with, twoflow_parse_grouped_json, twoflow_parse_triples_json};
use datafusion::error::Result;

pub fn parse_to_grouped_json(input: &[u8]) -> Result<String> {
    parse_json_with(input, twoflow_parse_grouped_json)
}

pub fn parse_to_triples_json(input: &[u8]) -> Result<String> {
    parse_json_with(input, twoflow_parse_triples_json)
}
