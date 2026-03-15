use authora::AuthoraClient;

#[tokio::main]
async fn main() {
    let api_key = std::env::var("AUTHORA_API_KEY").expect("AUTHORA_API_KEY required");
    let client = AuthoraClient::new(&api_key);

    let agent = client.agents().create("my-agent", "ws_...").await.unwrap();
    println!("Created agent: {}", agent.id);
}
