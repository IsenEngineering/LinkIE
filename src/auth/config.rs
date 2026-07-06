use openidconnect::{AdditionalProviderMetadata, ClientId, ClientSecret, IssuerUrl, ProviderMetadata, RedirectUrl, RevocationErrorResponseType, RevocationUrl, core::{CoreClaimName, CoreClaimType, CoreClient, CoreClientAuthMethod, CoreGrantType, CoreJweKeyManagementAlgorithm, CoreResponseMode, CoreResponseType, CoreSubjectIdentifierType}};

use openidconnect::{
    EmptyAdditionalClaims, EmptyExtraTokenFields, EndpointMaybeSet, EndpointNotSet, EndpointSet,
    IdTokenFields, StandardErrorResponse, StandardTokenIntrospectionResponse,
    StandardTokenResponse,
};
use openidconnect::core::{
    CoreAuthDisplay, CoreAuthPrompt, CoreErrorResponseType, CoreGenderClaim, CoreJsonWebKey,
    CoreJweContentEncryptionAlgorithm, CoreJwsSigningAlgorithm, CoreRevocableToken, CoreTokenType
};
use serde::{Deserialize, Serialize};

pub type OidcClient = openidconnect::Client<
    EmptyAdditionalClaims,
    CoreAuthDisplay,
    CoreGenderClaim,
    CoreJweContentEncryptionAlgorithm,
    CoreJsonWebKey,
    CoreAuthPrompt,
    StandardErrorResponse<CoreErrorResponseType>,
    StandardTokenResponse<
        IdTokenFields<
            EmptyAdditionalClaims,
            EmptyExtraTokenFields,
            CoreGenderClaim,
            CoreJweContentEncryptionAlgorithm,
            CoreJwsSigningAlgorithm,
        >,
        CoreTokenType,
    >,
    StandardTokenIntrospectionResponse<EmptyExtraTokenFields, CoreTokenType>,
    CoreRevocableToken,
    StandardErrorResponse<RevocationErrorResponseType>,
    EndpointSet,
    EndpointNotSet,
    EndpointNotSet,
    EndpointSet,
    EndpointMaybeSet,
    EndpointMaybeSet,
>;

pub type OidcToken = StandardTokenResponse<
    IdTokenFields<
    EmptyAdditionalClaims, 
    EmptyExtraTokenFields, 
    CoreGenderClaim, 
    CoreJweContentEncryptionAlgorithm, 
    CoreJwsSigningAlgorithm>, 
    CoreTokenType
>;

#[derive(Clone, Debug, Deserialize, Serialize)]
struct RevocationEndpointProviderMetadata {
    revocation_endpoint: String,
}
impl AdditionalProviderMetadata for RevocationEndpointProviderMetadata {}
type GoogleProviderMetadata = ProviderMetadata<
    RevocationEndpointProviderMetadata,
    CoreAuthDisplay,
    CoreClientAuthMethod,
    CoreClaimName,
    CoreClaimType,
    CoreGrantType,
    CoreJweContentEncryptionAlgorithm,
    CoreJweKeyManagementAlgorithm,
    CoreJsonWebKey,
    CoreResponseMode,
    CoreResponseType,
    CoreSubjectIdentifierType,
>;

pub(super) struct OidcConfig {
    pub client_id: ClientId,
    pub secret: ClientSecret,
    pub issuer_url: IssuerUrl
}

impl OidcConfig {
    // instantiate a config
    pub fn new() -> Self {
        let client_id = std::env::var("GOOGLE_CLIENT_ID")
            .expect("GOOGLE_CLIENT_ID should be supplied to use google oidc");

        let secret = std::env::var("GOOGLE_CLIENT_SECRET")
            .expect("GOOGLE_CLIENT_SECRET should be supplied to use google oidc");

        let issuer_url = IssuerUrl::new("https://accounts.google.com".to_string())
            .expect("Invalid issuer URL");

        Self {
            client_id: ClientId::new(client_id),
            secret: ClientSecret::new(secret),
            issuer_url
        }
    }

    // instantiate a client from a config
    pub async fn client(self) -> OidcClient {
        let http_client = reqwest::Client::builder()
            // prevents SSRF
            .redirect(reqwest::redirect::Policy::none())
            .build().unwrap();

        // let's search for metadata at .../.well-known/openid-configuration
        let provider_metadata = GoogleProviderMetadata::discover_async(
            self.issuer_url, &http_client).await.unwrap();

        let revocation_endpoint = provider_metadata
            .additional_metadata()
            .revocation_endpoint
            .clone();

        CoreClient::from_provider_metadata(
            provider_metadata,
            self.client_id,
            Some(self.secret),
        ).set_redirect_uri(
            RedirectUrl::new("http://localhost/auth/validate".to_string()).unwrap()
        ).set_revocation_url(RevocationUrl::new(revocation_endpoint).unwrap())
    }
}