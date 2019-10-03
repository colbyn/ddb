use std::rc::Rc;
use std::cell::{Cell, RefCell, RefMut};
use std::path::PathBuf;
use serde::{Serialize, Deserialize, de::DeserializeOwned};

///////////////////////////////////////////////////////////////////////////////
// MISC
///////////////////////////////////////////////////////////////////////////////

pub type YupOAuth = yup_oauth2::ServiceAccountAccess<hyper::Client>;


///////////////////////////////////////////////////////////////////////////////
// API-KEY DATA TYPES
///////////////////////////////////////////////////////////////////////////////

/// RELATIVE TO THE HOME DIRECTORY
pub static DEV_API_KEY_PATH: &'static str = ".config/gcloud-api-keys/dev.json";


#[derive(Debug, Clone, PartialEq)]
pub(crate) struct ApiKey {
    pub(crate) file_path: PathBuf,
    pub(crate) project_id: String,
}

impl ApiKey {
    pub fn lookup_api_key_file_path() -> Option<PathBuf> {
        let mut output_path = dirs::home_dir()?;
        output_path.push(DEV_API_KEY_PATH);
        if output_path.exists() {
            Some(output_path)
        } else {
            eprintln!("missing api keys at: ~/{}", DEV_API_KEY_PATH);
            None
        }
    }

    pub fn lookup_project_id(from_file: Option<PathBuf>) -> Option<String> {
        match from_file {
            None => {
                let key_file_path: PathBuf = ApiKey::lookup_api_key_file_path()?;
                let value = std::fs::read(&key_file_path)
                    .ok()
                    .and_then(|x| serde_json::from_slice::<serde_json::Value>(&x).ok())?;
                value
                    .as_object()
                    .and_then(|x| x.get("project_id"))
                    .and_then(|x| x.as_str().map(|x| x.to_owned()))
            }
            Some(key_file_path) => {
                let value = std::fs::read(&key_file_path)
                    .ok()
                    .and_then(|x| serde_json::from_slice::<serde_json::Value>(&x).ok())?;
                value
                    .as_object()
                    .and_then(|x| x.get("project_id"))
                    .and_then(|x| x.as_str().map(|x| x.to_owned()))
            }
        }
    }

    pub fn lookup() -> Option<Self> {
        let via_std_file_path = || {
            let file_path = ApiKey::lookup_api_key_file_path()?;
            let project_id = ApiKey::lookup_project_id(None)?;
            Some(ApiKey {file_path, project_id})            
        };
        let via_env_var = || {
            std::env::var("GOOGLE_APPLICATION_CREDENTIALS")
                .ok()
                .and_then(|auth_file_path| {
                    ApiKey::from_file(&PathBuf::from(auth_file_path))
                })
        };
        via_std_file_path()
            .or(via_env_var())
    }
    pub fn from_file(file_path: &PathBuf) -> Option<Self> {
        let file_path = file_path.clone();
        let project_id = ApiKey::lookup_project_id(Some(file_path.clone()))?;
        Some(ApiKey {file_path, project_id})
    }
}



///////////////////////////////////////////////////////////////////////////////
// EXTERNAL AUTH API
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Clone)]
pub struct Auth {
    pub(crate) project_id: String,
    pub(crate) interface: AuthInterface,
}


#[derive(Debug, Clone)]
pub(crate) enum AuthInterface {
    ApiKey(YupOAuthInterface),
    Gcp(GcpAuthToken),
}


impl Auth {
    /// Currently checks the following for a valid credentials file:
    /// * `~/.config/gcloud-api-keys/dev.json`
    /// * `GOOGLE_APPLICATION_CREDENTIALS` environment variable
    /// 
    /// Expects the JSON file to contain a `project_id` field.
    pub fn new_via_api_key() -> Result<Self, String> {
        ApiKey::lookup()
            .map(|auth| {
                let project_id = auth.project_id.clone();
                let key_file = auth.file_path
                    .to_str()
                    .expect("auth.file_path.to_str() failed");
                let client_secret = yup_oauth2::service_account_key_from_file(&key_file.to_owned())
                    .expect("yup_oauth2::service_account_key_from_file failed");
                let client = hyper::Client::with_connector(
                    hyper::net::HttpsConnector::new(hyper_rustls::TlsClient::new())
                );
                let access = yup_oauth2::ServiceAccountAccess::new(client_secret, client);
                Auth {
                    project_id: project_id,
                    interface: AuthInterface::ApiKey(YupOAuthInterface(
                        Rc::new(RefCell::new(access))
                    )),
                }
            })
            .ok_or(String::from("api key lookup failed"))
    }
    
    /// For instances running in google cloud platform.
    /// 
    /// Does not support refreshing. Should be fine if running from Google cloud run.
    /// Currently supports checking the metadata server for access tokens.
    pub fn new_via_gcp() -> Result<Self, String> {
        let project_id = GcpAuthToken::lookup_project_id()?;
        let access = GcpAuthToken::new()?;
        let result = Auth {
            project_id: project_id,
            interface: AuthInterface::Gcp(access),
        };
        Ok(result)
    }

    /// Automatically find auth credentials.
    /// 
    /// See `Auth::new_via_api_key` and `Auth::new_via_gcp`
    /// for interface specific details.
    pub fn new() -> Result<Self, String> {
        Auth::new_via_api_key()
            .or(Auth::new_via_gcp())
    }
}


impl yup_oauth2::GetToken for Auth {
    fn api_key(&mut self) -> Option<String> {
        match &mut self.interface {
            AuthInterface::Gcp(x) => x.api_key(),
            AuthInterface::ApiKey(x) => x.0
                .try_borrow_mut()
                .map_err(|e| eprintln!("[warning] Auth (value already borrowed with a mutable reference!): {:?}", e))
                .ok()?
                .api_key(),
        }
    }

    fn token<'b, I, T>(&mut self, scopes: I) -> Result<yup_oauth2::Token, Box<dyn std::error::Error>>
    where
        T: AsRef<str> + Ord + 'b,
        I: IntoIterator<Item = &'b T>, 
    {
        match &mut self.interface {
            AuthInterface::Gcp(x) => x.token(scopes),
            AuthInterface::ApiKey(x) => x.0
                .try_borrow_mut()
                .map_err(|e| -> Box<dyn std::error::Error> {
                    eprintln!("[warning] Auth (value already borrowed with a mutable reference!): {:?}", e);
                    Box::new(e)
                })
                .and_then(|mut x| x.token(scopes))
        }
    }
}



///////////////////////////////////////////////////////////////////////////////
// AUTH VIA API-KEY
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone)]
pub(crate) struct YupOAuthInterface(Rc<RefCell<YupOAuth>>);

impl std::fmt::Debug for YupOAuthInterface {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "YupOAuthInterface(...)")
    }
}



///////////////////////////////////////////////////////////////////////////////
// AUTH VIA GCP-METADATA-SERVER
///////////////////////////////////////////////////////////////////////////////

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub(crate) struct GcpAuthToken {
    access_token: String,
    token_type: String,
    expires_in: u32,
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
    pub fn lookup_project_id() -> Result<String, String> {
        let url = "http://metadata.google.internal/computeMetadata/v1/project/project-id";
        reqwest::Client::builder()
            .build()
            .map_err(|e| format!("http client error: {:?}", e))
            .map(|x| x.get(url))
            .map(|x| x.header("Metadata-Flavor", "Google"))
            .and_then(|x| {
                x
                    .send()
                    .map_err(|e| format!("http client error: {:?}", e))
            })
            .and_then(|mut x| {
                x   .text()
                    .map_err(|e| format!("http client error: {:?}", e))
            })
    }
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

