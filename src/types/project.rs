use crate::{ constants::{ PROJECT_ENDPOINT }, types::{ Contact, Links }, utils::apply_dotenv };

use serde::{ Deserialize, Serialize };

use reqwest::{ Client };

use validator::Validate;

use color_eyre::eyre::eyre;

use serde_json::Value;

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
        github: Option<String>,
        website: Option<String>,
        email: Option<String>,
        id: Option<String>
    ) -> Self {
        Self {
            name,
            links: Links {
                twitter,
                github,
                website,
            },
            contact: Contact { email },
            id,
        }
    }

    pub async fn fetch_project_id(self) -> eyre::Result<Option<String>> {
        let client = Client::new();

        apply_dotenv()?;

        let project_endpoint = std::env
            ::var("PROJECT_ENDPOINT")
            .unwrap_or_else(|_| PROJECT_ENDPOINT.to_string());

        let response = client
            .get(&project_endpoint)
            .query(&[("query", self.name.as_str())])
            .send().await?;

        if !response.status().is_success() {
            return Err(
                eyre!(
                    "Could not get project.  Response: {:?}\n Body: {:#?}",
                    response.status(),
                    response.json::<Value>().await?
                )
            );
        }

        let project_response_data = response.json::<Value>().await?;

        let projects_found = &project_response_data["projectsFound"];

        match projects_found {
            Value::Null => { Ok(None) }

            Value::Array(val) => {
                let project_id = val
                    .iter()
                    .find(|project| project["name"].eq(&self.name))
                    .and_then(|project| project["value"].as_str())
                    .unwrap_or_default()
                    .trim()
                    .to_string()
                    .replace('\"', "");

                if project_id.is_empty() {
                    return Ok(None);
                }

                Ok(Some(project_id))
            }

            _ => Err(eyre!("Error occurred while fetching a project")),
        }
    }
}