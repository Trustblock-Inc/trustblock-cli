use crate::{
    constants::{ PROJECT_ENDPOINT, TRUSTBLOCK_API_KEY_HEADER },
    types::{ Contact, Links },
    utils::apply_dotenv,
};

use serde::{ Deserialize, Serialize };

use reqwest::{ Client, StatusCode };

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
}

impl Project {
    #[must_use]
    pub const fn new(
        name: String,
        twitter: Option<String>,
        github: Option<String>,
        website: Option<String>,
        email: Option<String>
    ) -> Self {
        Self {
            name,
            links: Links {
                twitter,
                github,
                website,
            },
            contact: Contact { email },
        }
    }

    pub async fn fetch_project_id(self, auth_token: &String) -> eyre::Result<String> {
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
            Value::Null => {
                let project_id = self.create_project(auth_token, client, project_endpoint).await?;

                Ok(project_id)
            }

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
                    let project_id = self.create_project(
                        auth_token,
                        client,
                        project_endpoint
                    ).await?;
                    return Ok(project_id);
                }

                Ok(project_id)
            }

            _ => Err(eyre!("Error occured while fetching a project")),
        }
    }

    pub async fn create_project(
        self,
        auth_token: &String,
        client: Client,
        project_endpoint: String
    ) -> eyre::Result<String> {
        println!("Project not found, creating new project...\n");

        let response = client
            .post(project_endpoint)
            .header(TRUSTBLOCK_API_KEY_HEADER, auth_token)
            .json(&self)
            .send().await?;

        match response.status() {
            StatusCode::UNAUTHORIZED => Err(eyre!("Unauthorized, please check your auth token.")),

            StatusCode::CREATED => {
                let project_response_data = response.json::<Value>().await?;

                let project_id = project_response_data["id"].to_string().trim().replace('\"', "");

                Ok(project_id)
            }

            _ =>
                Err(
                    eyre!(
                        "Could not create project. Response: {:?}\n Response Text: {:?}",
                        response.status(),
                        response.json::<Value>().await?
                    )
                ),
        }
    }
}