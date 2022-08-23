use anyhow::Result;
use futures::{stream, StreamExt, TryStreamExt};
use google_drive3::{
    api::Scope,
    hyper,
    hyper_rustls::{HttpsConnector, HttpsConnectorBuilder},
    oauth2::{self, ServiceAccountAuthenticator},
    DriveHub,
};
use hyper::{client::HttpConnector, Body, Client, Response};

pub struct Drive {
    hub: DriveHub<HttpsConnector<HttpConnector>>,
}

impl Drive {
    pub async fn new(service_account_bytes: &[u8]) -> Result<Self> {
        let service_account_key = oauth2::parse_service_account_key(service_account_bytes)?;

        let authenticator = ServiceAccountAuthenticator::builder(service_account_key)
            .build()
            .await?;

        let hub = DriveHub::new(
            Client::builder().build(
                HttpsConnectorBuilder::new()
                    .with_native_roots()
                    .https_only()
                    .enable_http2()
                    .build(),
            ),
            authenticator,
        );

        Ok(Self { hub })
    }

    pub async fn list_in_folder(&self, folder_name: &str) -> Result<Vec<String>> {
        let (_, list) = self
            .hub
            .files()
            .list()
            .q(&format!(
                "mimeType = 'application/vnd.google-apps.folder' and name = '{}'",
                folder_name
            ))
            .doit()
            .await?;

        let file_ids = stream::iter(list.files.unwrap_or_default())
            .filter_map(move |folder| async move { folder.id })
            .map(move |folder_id| async move {
                let (_, list) = self
                    .hub
                    .files()
                    .list()
                    .q(&format!("'{}' in parents", folder_id))
                    .doit()
                    .await?;

                anyhow::Ok(
                    list.files
                        .unwrap_or_default()
                        .into_iter()
                        .filter_map(|file| file.id),
                )
            })
            .buffer_unordered(4)
            .try_collect::<Vec<_>>()
            .await?
            .into_iter()
            .flatten()
            .collect::<Vec<_>>();

        Ok(file_ids)
    }

    pub async fn download_file(&self, file_id: &str) -> Result<Response<Body>> {
        let (response, _) = self
            .hub
            .files()
            .get(file_id)
            .param("alt", "media")
            .add_scope(Scope::Readonly)
            .doit()
            .await?;

        Ok(response)
    }
}
