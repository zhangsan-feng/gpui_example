use log::{error, info};
use reqwest::multipart;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct RestResponse<T> {
    #[serde(rename = "code")]
    pub code: String,
    #[serde(rename = "data")]
    pub(crate) data: T,
    #[serde(rename = "msg")]
    pub msg: String,
}

trait ResponseHandler {
    async fn handle(self) -> Result<RestResponse<serde_json::Value>, anyhow::Error>;
}

impl ResponseHandler for reqwest::Response {
    async fn handle(self) -> Result<RestResponse<serde_json::Value>, anyhow::Error> {
        let status = self.status();
        let bytes = self.bytes().await.unwrap_or_default();
        let body_str = String::from_utf8_lossy(&bytes);

        if status.is_success() {
            match serde_json::from_slice(&bytes) {
                Ok(data) => Ok(data),
                Err(err) => {
                    info!("序列化失败: {}, 响应内容: {}", err, body_str);
                    Err(anyhow::anyhow!("序列化失败: {}, 响应内容: {}", err, body_str))
                }
            }
        } else {
            info!("请求失败, 状态码: {}, 响应: {}", status, body_str);
            Err(anyhow::anyhow!("请求失败, 状态码: {}, 响应: {}", status, body_str))
        }
    }
}


pub struct HttpClient {
    client: reqwest::Client,
}

impl HttpClient {
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
        }
    }

    pub async fn get(&self, url: String) -> Result<RestResponse<serde_json::Value>, anyhow::Error> {
        let response = match self.client.get(&url).send().await {
            Ok(r) => r,
            Err(e) => {
                info!("GET请求失败 [{}]: {}", url, e);
                return Err(anyhow::anyhow!("GET请求失败: {}", e));
            }
        };

        response.handle().await
    }

    pub async fn post(&self, url: String, body: serde_json::Value) -> Result<RestResponse<serde_json::Value>, anyhow::Error> {
        let response = match self.client.post(&url).json(&body).send().await {
            Ok(r) => r,
            Err(e) => {
                info!("POST请求失败 [{}]: {}", url, e);
                return Err(anyhow::anyhow!("POST请求失败: {}", e));
            }
        };

        response.handle().await
    }

    pub async fn post_form(&self, url: String, form: multipart::Form) -> Result<RestResponse<serde_json::Value>, anyhow::Error> {
        let response = match self.client.post(&url).multipart(form).send().await {
            Ok(r) => r,
            Err(e) => {
                info!("POST表单请求失败 [{}]: {}", url, e);
                return Err(anyhow::anyhow!("POST表单请求失败: {}", e));
            }
        };
        response.handle().await
    }
}