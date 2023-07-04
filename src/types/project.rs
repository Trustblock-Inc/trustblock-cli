use color_eyre::eyre::eyre;
use reqwest::{Client, StatusCode, Url};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use validator::Validate;

use crate::{
    constants::{PROJECT_SLUG_ENDPOINT, TRUSTBLOCK_API_KEY_HEADER},
    types::{Contact, Links},
    utils::apply_dotenv,
};

#[derive(Debug, Clone, Validate, Serialize, Deserialize)]
pub struct Project {
    #[validate(length(min = 1, max = 28))]
    pub name: String,
    #[validate]
    pub links: Links,
    #[validate]
    pub contact: Contact,
    #[serde(skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
}

impl Project {
    #[must_use]
    pub const fn new(
        name: String,
        twitter: Option<String>,
        telegram: Option<String>,
        github: Option<String>,
        website: Option<String>,
        email: Option<String>,
        id: Option<String>,
    ) -> Self {
        Self {
            name,
            links: Links {
                twitter,
                telegram,
                github,
                website,
            },
            contact: Contact { email },
            id,
        }
    }

    pub async fn fetch_project_id(self, api_key: &str) -> eyre::Result<Option<String>> {
        let client = Client::new();

        let url = Url::parse(&self.links.website.unwrap_or_default())?;

        apply_dotenv()?;

        let project_slug_endpoint = std::env::var("PROJECT_SLUG_ENDPOINT")
            .unwrap_or_else(|_| PROJECT_SLUG_ENDPOINT.to_string());

        let slug = url.domain().unwrap_or_default().replace('.', "-");

        let response = client
            .get(&format!("{project_slug_endpoint}{slug}"))
            .header(TRUSTBLOCK_API_KEY_HEADER, api_key)
            .send()
            .await?;

        match response.status() {
            StatusCode::NOT_FOUND => Ok(None),
            StatusCode::OK => {
                let project_response_data = response.json::<Value>().await?;
                let project_id = project_response_data["id"].to_string().replace('\"', "");

                Ok(Some(project_id))
            }
            _ => Err(eyre!(
                "Error occurred while fetching a project id: {}",
                response.text().await?
            )),
        }
    }
}
