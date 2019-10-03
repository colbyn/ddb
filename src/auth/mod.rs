pub mod gcp;
pub mod api_key;

pub use api_key::ApiKey;

pub type YupOAuth = yup_oauth2::ServiceAccountAccess<hyper::Client>;

pub enum AuthTokenProxy {
    YupOAuth(YupOAuth),
    Gcp(crate::auth::gcp::GcpAuthToken),
}

impl yup_oauth2::GetToken for AuthTokenProxy {
    fn api_key(&mut self) -> Option<String> {
        match self {
            AuthTokenProxy::Gcp(x) => x.api_key(),
            AuthTokenProxy::YupOAuth(x) => x.api_key(),
        }
    }

    fn token<'b, I, T>(&mut self, scopes: I) -> Result<yup_oauth2::Token, Box<dyn std::error::Error>>
    where
        T: AsRef<str> + Ord + 'b,
        I: IntoIterator<Item = &'b T>, 
    {
        match self {
            AuthTokenProxy::Gcp(x) => x.token(scopes),
            AuthTokenProxy::YupOAuth(x) => x.token(scopes),
        }
    }
}
