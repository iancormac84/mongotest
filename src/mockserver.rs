use mongodb::{Client, ThreadedClient};
use mongotest::{graphiql, handle_graphql, AppState};
use tide::App;

fn main() {
    let client = Client::connect("localhost", 27017).expect("Failed to initialize client.");
    let mut app = App::with_state(AppState::new(client, "foos_and_things"));
    app.at("/graphql").get(handle_graphql).post(handle_graphql);
    app.at("/graphiql").get(graphiql).post(graphiql);
    app.run("127.0.0.1:8000").unwrap();
}
