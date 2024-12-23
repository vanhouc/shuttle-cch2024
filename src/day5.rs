use axum::{
    http::{HeaderMap, StatusCode},
    response::IntoResponse,
};
use cargo_manifest::Manifest;
use thiserror::Error;
use toml::Table;
use tracing::{error, instrument};

#[derive(Error, Debug, PartialEq)]
pub enum ManifestError {
    #[error("toml was not valid")]
    InvalidToml,
    #[error("json was not valid")]
    InvalidJson,
    #[error("yaml was not valid")]
    InvalidYaml,
    #[error("unsupported manifest format")]
    Unsupported,
    #[error("toml was not a valid cargo manifest")]
    InvalidCargoToml,
    #[error("no christmas keyword found")]
    NotChristmas,
    #[error("no valid orders found")]
    NoOrders,
}

impl IntoResponse for ManifestError {
    fn into_response(self) -> axum::response::Response {
        match self {
            ManifestError::InvalidToml => StatusCode::NO_CONTENT.into_response(),
            ManifestError::InvalidJson => StatusCode::NO_CONTENT.into_response(),
            ManifestError::InvalidYaml => StatusCode::NO_CONTENT.into_response(),
            ManifestError::Unsupported => StatusCode::UNSUPPORTED_MEDIA_TYPE.into_response(),
            ManifestError::InvalidCargoToml => {
                (StatusCode::BAD_REQUEST, "Invalid manifest").into_response()
            }
            ManifestError::NotChristmas => {
                (StatusCode::BAD_REQUEST, "Magic keyword not provided").into_response()
            }
            ManifestError::NoOrders => StatusCode::NO_CONTENT.into_response(),
        }
    }
}

fn manifest_json(json: String) -> Result<String, ManifestError> {
    let mut deserializer = serde_json::Deserializer::from_str(&json);
    let mut toml_string = String::new();
    let serializer = toml::Serializer::new(&mut toml_string);
    serde_transcode::transcode(&mut deserializer, serializer)
        .inspect_err(|error| error!(%error, "json was not valid"))
        .map_err(|_| ManifestError::InvalidJson)?;

    Ok(toml_string)
}

fn manifest_yaml(yaml: String) -> Result<String, ManifestError> {
    let deserializer = serde_yaml::Deserializer::from_str(&yaml);
    let mut toml_string = String::new();
    let serializer = toml::Serializer::new(&mut toml_string);
    serde_transcode::transcode(deserializer, serializer)
        .inspect_err(|error| error!(%error, "yaml was not valid"))
        .map_err(|_| ManifestError::InvalidYaml)?;

    Ok(toml_string)
}

#[instrument(ret, skip_all)]
pub async fn manifest(headers: HeaderMap, body: String) -> Result<String, ManifestError> {
    let toml = match headers.get("Content-Type") {
        Some(content_type) if content_type == "application/json" => manifest_json(body),
        Some(content_type) if content_type == "application/yaml" => manifest_yaml(body),
        Some(content_type) if content_type == "application/toml" => Ok(body),
        _ => Err(ManifestError::Unsupported),
    }?;
    let manifest: Table = toml
        .parse()
        .inspect_err(|error| error!(%error, "toml was not valid"))
        .map_err(|_| ManifestError::InvalidToml)?;

    if Manifest::from_slice(toml.as_bytes()).is_err() {
        error!("toml was not a valid Cargo.toml");
        return Err(ManifestError::InvalidCargoToml);
    }

    if manifest
        .get("package")
        .and_then(|package| package.get("keywords"))
        .and_then(|keywords| keywords.as_array())
        .and_then(|keywords| {
            keywords
                .iter()
                .find(|keyword| keyword.as_str() == Some("Christmas 2024"))
        })
        .is_none()
    {
        error!("christmas keyword not supplied");
        return Err(ManifestError::NotChristmas);
    }

    let orders = manifest
        .get("package")
        .and_then(|value| value.get("metadata"))
        .and_then(|value| value.get("orders"))
        .and_then(|value| value.as_array())
        .ok_or(ManifestError::NoOrders)
        .inspect_err(|_| error!("package.metadata.orders not present"))?;

    let valid_orders: Vec<String> = orders
        .iter()
        .filter_map(|order| {
            let item = order.get("item").and_then(|value| value.as_str())?;
            let quantity = order.get("quantity").and_then(|value| value.as_integer())?;
            Some(format!("{}: {}", item, quantity))
        })
        .collect();

    if valid_orders.is_empty() {
        error!("manifest contained no valid orders");
        return Err(ManifestError::NoOrders);
    }
    let answer = valid_orders.join("\n");
    Ok(answer)
}

#[cfg(test)]
mod test {
    use axum::http::HeaderMap;
    use toml::toml;

    #[tokio::test]
    async fn test_manifest() {
        let toml = r#"
            [package]
            name = "not-a-gift-order"
            authors = ["Not Santa"]
            keywords = ["Christmas 2024"]

            [[package.metadata.orders]]
            item = "Toy car"
            quantity = 2

            [[package.metadata.orders]]
            item = "Lego brick"
            quantity = 230
        "#;
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/toml".parse().unwrap());
        let actual = super::manifest(headers, toml.to_string()).await.unwrap();
        assert_eq!(actual, "Toy car: 2\nLego brick: 230");
    }

    #[test]
    fn toml_table() {
        let toml = toml! {
            [package]
            name = "not-a-gift-order"
            authors = ["Not Santa"]
            keywords = ["Christmas 2024"]

            [[package.metadata.orders]]
            item = "Toy car"
            quantity = 2

            [[package.metadata.orders]]
            item = "Lego brick"
            quantity = 230
        };
        println!("{toml:?}");
        let actual = toml["package"]["metadata"]["orders"][0]["item"]
            .as_str()
            .unwrap();
        assert_eq!(actual, "Toy car");
    }

    #[tokio::test]
    async fn test_keyword_validation() {
        let toml = r#"
            [package]
            name = "not-a-gift-order"
            authors = ["Not Santa"]
            keywords = ["Fartmas 2069"]
        "#;
        let mut headers = HeaderMap::new();
        headers.insert("Content-Type", "application/toml".parse().unwrap());
        let actual = super::manifest(headers, toml.to_string()).await;
        assert_eq!(actual, Err(super::ManifestError::NotChristmas));
    }
}
