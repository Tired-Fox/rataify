struct Cache {
    pub client_id: String,
    pub client_secret: String,
    pub token: Option<String>,
    pub refresh_token: Option<String>,
}