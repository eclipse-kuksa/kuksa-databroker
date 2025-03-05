use cucumber::{cli, gherkin::Step, given, then, when, writer, World as _};
use reqwest::Client;
use serde_json::Value;

#[derive(cucumber::World, Debug)]
#[world(init = Self::new)]
struct VissWorld {
    client: Client,
    response: Option<reqwest::Response>,
    authorization_token: Option<String>,
}

#[given("the VISSv2 server is reachable")]
async fn the_viss_server_is_reachable(world: &mut VissWorld) {
    // Hier könnte eine Überprüfung der Servererreichbarkeit erfolgen
    // Zum Beispiel: Ein einfacher GET-Request an die Basis-URL
}

#[given("I have a valid authorization token")]
async fn i_have_a_valid_authorization_token(world: &mut VissWorld) {
    world.authorization_token = Some("<valid_token>".to_string()); // Ersetze dies durch ein gültiges Token
}

#[when(expr = "I send a GET request to {path} with path {data_path}")]
async fn i_send_a_get_request(world: &mut VissWorld, path: String, data_path: String) {
    let mut request_builder = world.client.get(format!("https://<server>{}", path));

    if let Some(token) = &world.authorization_token {
        request_builder = request_builder.header("Authorization", token);
    }

    world.response = Some(request_builder.query(&[("path", data_path)]).send().await.unwrap());
}

#[then(expr = "the response status code should be {status_code}")]
async fn the_response_status_code_should_be(world: &mut VissWorld, status_code: usize) {
    let response = world.response.as_ref().unwrap();
    assert_eq!(response.status().as_u16() as usize, status_code);
}

#[then(expr = "the response should contain the data point for {data_path}")]
async fn the_response_should_contain_the_data_point(world: &mut VissWorld, data_path: String) {
    let response = world.response.as_ref().unwrap();
    let json: Value = response.json().await.unwrap();
    assert!(json["data"].is_object());
    assert_eq!(json["data"]["path"], data_path);
    assert!(json["data"]["data_point"].is_object());
}

#[then(expr = "the response should indicate that authorization is required")]
async fn the_response_should_indicate_authorization_required(world: &mut VissWorld) {
    let response = world.response.as_ref().unwrap();
    let json: Value = response.json().await.unwrap();
    assert_eq!(response.status().as_u16(), 401);
    assert_eq!(json["error"]["code"], 401);
    assert_eq!(json["error"]["message"], "Authorization required");
}

#[then(expr = "the response should indicate that the path is not found")]
async fn the_response_should_indicate_path_not_found(world: &mut VissWorld) {
    let response = world.response.as_ref().unwrap();
    let json: Value = response.json().await.unwrap();
    assert_eq!(response.status().as_u16(), 404);
    assert_eq!(json["error"]["code"], 404);
    assert_eq!(json["error"]["message"], "Path not found");
}

#[tokio::main]
async fn main() {

VissWorld::cucumber()
    .run("features/viss_read.feature")
    .await;

    // let cucumber = cucumber::Cucumber::<VissWorld>::new()
    //     .given("the VISSv2 server is reachable", the_viss_server_is_reachable)
    //     .given("I have a valid authorization token", i_have_a_valid_authorization_token)
    //     .when("I send a GET request to {path} with path {data_path}", i_send_a_get_request)
    //     .then("the response status code should be {status_code}", the_response_status_code_should_be)
    //     .then("the response should contain the data point for {data_path}", the_response_should_contain_the_data_point)
    //     .then("the response should indicate that authorization is required", the_response_should_indicate_authorization_required)
    //     .then("the response should indicate that the path is not found", the_response_should_indicate_path_not_found);

    // cucumber.run().await;
}
