use std::path::PathBuf;
use serde::{Serialize, Deserialize, de::DeserializeOwned};


#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct GcpAuthToken {
    access_token: String,
    expires_in: String,
    token_type: String,
}

impl yup_oauth2::GetToken for GcpAuthToken {
    fn api_key(&mut self) -> Option<String> {
        None
    }

    fn token<'b, I, T>(&mut self, scopes: I) -> Result<yup_oauth2::Token, Box<dyn std::error::Error>>
    where
        T: AsRef<str> + Ord + 'b,
        I: IntoIterator<Item = &'b T>, 
    {
        let x = yup_oauth2::Token {
            access_token: self.access_token.clone(),
            refresh_token: String::new(),
            token_type: String::from("Bearer"),
            expires_in: None,
            expires_in_timestamp: None,
        };
        Ok(x)
    }
}

impl GcpAuthToken {
    pub fn new() -> Result<Self, String> {
        let url = format!(
            "http://metadata.google.internal/computeMetadata/v1/instance/service-accounts/default/token?scopes={scopes}",
            scopes="https://www.googleapis.com/auth/cloud-platform",
        );
        let result = reqwest::Client::builder()
            .build()
            .map_err(|e| format!("http client error: {:?}", e))
            .map(|x| x.get(&url))
            .map(|x| x.header("Metadata-Flavor", "Google"))
            .and_then(|x| {
                x
                    .send()
                    .map_err(|e| format!("http client error: {:?}", e))
            })
            .and_then(|mut x| {
                x   .json::<GcpAuthToken>()
                    .map_err(|e| format!("http client error: {:?}", e))
            });
        result
    }
}