use serde::{Deserialize, Deserializer};

pub fn parse_pagination<'de, D>(d: D) -> Result<Option<Paginate>, D::Error>
    where
        D: Deserializer<'de>,
{
    let url = Option::<String>::deserialize(d)?;
    match url {
        Some(url) => {
            let parts = url.split("?").collect::<Vec<&str>>();
            if parts.len() < 2 {
                return Ok(Some(Paginate::default()));
            }

            let paginate: Paginate = serde_qs::from_str(parts[1])
                .map_err(serde::de::Error::custom)?;

            Ok(Some(paginate))
        }
        None => Ok(None)
    }
}

#[derive(Default, Debug, Deserialize, Clone, Copy, PartialEq)]
pub struct Paginate {
    pub offset: Option<usize>,
    pub limit: Option<usize>,
}