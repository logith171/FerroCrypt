use reqwest::header::{HeaderMap, AUTHORIZATION, USER_AGENT};
use serde_json::json;

pub struct SyncClient;

impl SyncClient {
    /// Uploads the encrypted vault data to a GitHub Gist.
    /// This allows for the "Multi-device cloud synchronization" mentioned in your doc.
    pub async fn upload_to_gist(
        token: &str,
        gist_id: &str,
        encrypted_data: &[u8],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        
        // GitHub Gists are text-based. We convert our binary encrypted data
        // into a Hex string so it can be stored safely in a JSON payload.
        let content_hex = hex::encode(encrypted_data);

        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("token {}", token).parse()?);
        headers.insert(USER_AGENT, "FerroCrypt-App".parse()?);

        let payload = json!({
            "description": "FerroCrypt Encrypted Backup",
            "public": false,
            "files": {
                "vault.enc": {
                    "content": content_hex
                }
            }
        });

        let response = client
            .patch(format!("https://api.github.com/gists/{}", gist_id))
            .headers(headers)
            .json(&payload)
            .send()
            .await?;

        if response.status().is_success() {
            Ok(())
        } else {
            let err_msg = format!("Sync failed with status: {}", response.status());
            Err(err_msg.into())
        }
    }

    /// Pulls the encrypted vault data down from GitHub.
    pub async fn download_from_gist(
        token: &str,
        gist_id: &str,
    ) -> Result<Vec<u8>, Box<dyn std::error::Error>> {
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        headers.insert(AUTHORIZATION, format!("token {}", token).parse()?);
        headers.insert(USER_AGENT, "FerroCrypt-App".parse()?);

        let response = client
            .get(format!("https://api.github.com/gists/{}", gist_id))
            .headers(headers)
            .send()
            .await?
            .json::<serde_json::Value>()
            .await?;

        // Extract the hex content from the JSON response
        let content_hex = response["files"]["vault.enc"]["content"]
            .as_str()
            .ok_or("Could not find vault.enc in Gist")?;

        // Convert Hex back to raw encrypted bytes
        let encrypted_data = hex::decode(content_hex)?;
        Ok(encrypted_data)
    }
}