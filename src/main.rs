#[macro_use]
extern crate rocket;

use rocket::{tokio::sync::broadcast::{channel, Sender, error::RecvError}, serde::{Serialize, Deserialize}, State, Shutdown, response::stream::{EventStream, Event}, fs::FileServer };
use rocket::form::Form;
use rocket::tokio::select;
use rocket::fs::relative;

#[get("/world")]
fn world() -> &'static str {
    "Hello World"
}

#[derive(Debug, Clone, FromForm, Serialize, Deserialize)]
#[serde(crate = "rocket::serde")]

struct Message {
    #[field(validate = len(..30))]
    pub room: String,
    #[field(validate = len(..20))]
    pub username: String,
    pub message: String,
}
// method to handle form post requests to the server
#[post("/message", data = "<form>")]
fn post(form: Form<Message>, queue: &State<Sender<Message>>) {
    let _res = queue.send(form.into_inner());
}
// method to handle new messages and show it
#[get("/events")]
async fn events(queue: &State<Sender<Message>>, mut end: Shutdown) -> EventStream![] {
    let mut rx = queue.subscribe();
    EventStream! {
        loop {
            let msg = select! {
                msg = rx.recv() => match msg {
                    Ok(msg) => msg,
                    Err(RecvError::Closed) => break,
                    Err(RecvError::Lagged(_)) => continue,
                },
                _ = &mut end => break,
            };
            yield Event::json(&msg);
        } 
    }
}
// rocket cretes and runs the server
#[launch]
fn rocket() -> _ {
    rocket::build()
        .manage(channel::<Message>(1024).0)
        .mount("/hello", routes![world, post, events]) // expose routes throug /hello
        .mount("/", FileServer::from(relative!("static"))) // mount static files including frontend
}