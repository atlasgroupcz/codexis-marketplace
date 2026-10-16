use std::fmt;
use std::time::Duration;

use reqwest::StatusCode;
use reqwest::blocking::{Client, RequestBuilder, Response, multipart};
use reqwest::header::{AUTHORIZATION, HeaderMap, HeaderValue};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::model::{Alert, Document, File, Me, Page, Reference, WatchSet, WatchSetInput};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("invalid API URL or API key: {0}")]
    Config(String),
    #[error("request failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("{0}")]
    Problem(Problem),
}

#[derive(Debug, Default, Deserialize)]
pub struct Problem {
    #[serde(skip)]
    pub status: u16,
    pub title: Option<String>,
    pub detail: Option<String>,
    #[serde(default)]
    pub errors: Vec<FieldError>,
}

#[derive(Debug, Deserialize)]
pub struct FieldError {
    pub field: String,
    pub message: String,
}

impl fmt::Display for Problem {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "HTTP {}", self.status)?;
        if let Some(text) = self.detail.as_ref().or(self.title.as_ref()) {
            write!(formatter, ": {text}")?;
        }
        for error in &self.errors {
            write!(formatter, "\n  {}: {}", error.field, error.message)?;
        }
        if self.status == StatusCode::UNAUTHORIZED.as_u16() {
            write!(
                formatter,
                "\n  hint: the Turboleal API key is missing, revoked or invalid. Create one in Turboleal \
                 (API klíče) and save it with `turboleal-cli auth login`.\n  \
                 nápověda: API klíč Turbolealu chybí nebo byl zneplatněn. Vytvořte nový v Turbolealu \
                 (API klíče) a uložte ho přes `turboleal-cli auth login`."
            )?;
        }
        Ok(())
    }
}

pub type ApiResult<T> = Result<T, ApiError>;

/// A file to upload with the folder it belongs to inside the watch set.
pub struct Upload {
    pub name: String,
    pub folder: String,
    pub content_type: &'static str,
    pub bytes: Vec<u8>,
}

#[derive(Serialize)]
struct PageQuery {
    #[serde(skip_serializing_if = "Option::is_none")]
    after: Option<Uuid>,
    #[serde(skip_serializing_if = "Option::is_none")]
    limit: Option<u16>,
}

pub struct ApiClient {
    http: Client,
    base: String,
}

impl ApiClient {
    pub fn new(api_url: &str, api_key: &str) -> ApiResult<Self> {
        let mut headers = HeaderMap::new();
        let api_key = api_key.trim();
        let api_key = api_key
            .strip_prefix("Bearer ")
            .or_else(|| api_key.strip_prefix("bearer "))
            .unwrap_or(api_key);
        let mut authorization = HeaderValue::from_str(&format!("Bearer {api_key}"))
            .map_err(|error| ApiError::Config(error.to_string()))?;
        authorization.set_sensitive(true);
        headers.insert(AUTHORIZATION, authorization);
        let http = cdx_common::reqwest_client::client_builder()
            .user_agent(concat!("turboleal-cli/", env!("CARGO_PKG_VERSION")))
            .default_headers(headers)
            .timeout(Duration::from_secs(120))
            .build()?;
        Ok(Self {
            http,
            base: format!("{}/api/v1", api_url.trim_end_matches('/')),
        })
    }

    pub fn me(&self) -> ApiResult<Me> {
        self.send(self.http.get(self.url("/me")))
    }

    pub fn watch_sets(&self, after: Option<Uuid>, limit: Option<u16>) -> ApiResult<Page<WatchSet>> {
        self.send(
            self.http
                .get(self.url("/watch-sets"))
                .query(&PageQuery { after, limit }),
        )
    }

    pub fn watch_set(&self, id: Uuid) -> ApiResult<WatchSet> {
        self.send(self.http.get(self.url(&format!("/watch-sets/{id}"))))
    }

    pub fn create_watch_set(&self, input: &WatchSetInput) -> ApiResult<WatchSet> {
        self.send(self.http.post(self.url("/watch-sets")).json(input))
    }

    pub fn update_watch_set(&self, id: Uuid, input: &WatchSetInput) -> ApiResult<WatchSet> {
        self.send(
            self.http
                .patch(self.url(&format!("/watch-sets/{id}")))
                .json(input),
        )
    }

    pub fn delete_watch_set(&self, id: Uuid) -> ApiResult<()> {
        self.send_empty(self.http.delete(self.url(&format!("/watch-sets/{id}"))))
    }

    pub fn files(
        &self,
        watch_set: Uuid,
        after: Option<Uuid>,
        limit: Option<u16>,
    ) -> ApiResult<Page<File>> {
        self.send(
            self.http
                .get(self.url(&format!("/watch-sets/{watch_set}/files")))
                .query(&PageQuery { after, limit }),
        )
    }

    pub fn upload_files(&self, watch_set: Uuid, uploads: Vec<Upload>) -> ApiResult<Vec<File>> {
        let mut form = multipart::Form::new();
        for upload in uploads {
            let part = multipart::Part::bytes(upload.bytes)
                .file_name(upload.name)
                .mime_str(upload.content_type)?;
            form = form.part("files", part).text("paths", upload.folder);
        }
        self.send(
            self.http
                .post(self.url(&format!("/watch-sets/{watch_set}/files")))
                .multipart(form),
        )
    }

    pub fn delete_file(&self, watch_set: Uuid, file: Uuid) -> ApiResult<()> {
        self.send_empty(
            self.http
                .delete(self.url(&format!("/watch-sets/{watch_set}/files/{file}"))),
        )
    }

    pub fn documents(&self, watch_set: Uuid) -> ApiResult<Vec<Document>> {
        self.send(
            self.http
                .get(self.url(&format!("/watch-sets/{watch_set}/documents"))),
        )
    }

    pub fn references(
        &self,
        watch_set: Uuid,
        linked: Option<bool>,
        after: Option<Uuid>,
        limit: Option<u16>,
    ) -> ApiResult<Page<Reference>> {
        let request = self
            .http
            .get(self.url(&format!("/watch-sets/{watch_set}/references")))
            .query(&PageQuery { after, limit });
        self.send(match linked {
            Some(linked) => request.query(&[("linked", linked)]),
            None => request,
        })
    }

    pub fn alerts(
        &self,
        watch_set: Option<Uuid>,
        acknowledged: Option<bool>,
        after: Option<Uuid>,
        limit: Option<u16>,
    ) -> ApiResult<Page<Alert>> {
        let mut request = self
            .http
            .get(self.url("/alerts"))
            .query(&PageQuery { after, limit });
        if let Some(watch_set) = watch_set {
            request = request.query(&[("watchSetId", watch_set)]);
        }
        if let Some(acknowledged) = acknowledged {
            request = request.query(&[("acknowledged", acknowledged)]);
        }
        self.send(request)
    }

    pub fn alert(&self, id: Uuid) -> ApiResult<Alert> {
        self.send(self.http.get(self.url(&format!("/alerts/{id}"))))
    }

    pub fn acknowledge_alert(&self, id: Uuid) -> ApiResult<Alert> {
        self.send(
            self.http
                .put(self.url(&format!("/alerts/{id}/acknowledgement"))),
        )
    }

    fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    fn send<T: DeserializeOwned>(&self, request: RequestBuilder) -> ApiResult<T> {
        Ok(Self::check(request.send()?)?.json()?)
    }

    fn send_empty(&self, request: RequestBuilder) -> ApiResult<()> {
        Self::check(request.send()?).map(|_| ())
    }

    fn check(response: Response) -> ApiResult<Response> {
        let status = response.status();
        if status.is_success() {
            return Ok(response);
        }
        let mut problem: Problem = response.json().unwrap_or_default();
        problem.status = status.as_u16();
        Err(ApiError::Problem(problem))
    }
}
