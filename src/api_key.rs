use std::path::PathBuf;


// RELATIVE TO THE HOME DIRECTORY
pub static DEV_API_KEY_PATH: &'static str = ".config/gcloud-api-keys/dev.json";

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
            let key_file_path: PathBuf = lookup_api_key_file_path()?;
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


#[derive(Debug, Clone, PartialEq)]
pub struct ApiKey {
    pub(crate) file_path: PathBuf,
    pub(crate) project_id: String,
}

impl ApiKey {
    pub fn lookup() -> Option<Self> {
        let via_std_file_path = || {
            let file_path = lookup_api_key_file_path()?;
            let project_id = lookup_project_id(None)?;
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
        let project_id = lookup_project_id(Some(file_path.clone()))?;
        Some(ApiKey {file_path, project_id})
    }
}


