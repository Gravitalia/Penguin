use warp::{Filter, reject::Reject};
mod router;
mod helpers;
mod database;

#[derive(Debug)]
struct InvalidQuery;
impl Reject for InvalidQuery {}

async fn middleware(token: Option<String>, fallback: String) -> String {
    if token.is_some() && fallback == *"@me" {
        match helpers::get_jwt(token.unwrap()).await {
            Ok(data) => {
                data.claims.sub
            },
            Err(_) => "Invalid".to_string()
        }
    } else if fallback == *"@me" {
        "Invalid".to_string()
    } else {
        fallback
    }
}

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();

    let routes = warp::path("create").and(warp::post()).and(warp::body::json()).and(warp::header("sec")).and_then(|body: router::model::Create, finger: String| async {
        if true {
            Ok(router::create::create(body, finger).await)
        } else {
            Err(warp::reject::not_found())
        }
    })
    .or(warp::path!("users" / String).and(warp::header::optional::<String>("authorization")).and_then(|id: String, token: Option<String>| async {
        let middelware_res: String = middleware(token, id).await;
        if middelware_res != *"Invalid" {
            Ok(router::users::get(middelware_res.to_lowercase()).await)
        } else {
            Err(warp::reject::custom(InvalidQuery))
        }
    }));

    database::cassandra::init().await;
    database::cassandra::tables().await;
    helpers::init();

    warp::serve(routes)
    .run((
        [127, 0, 0, 1],
        dotenv::var("PORT").expect("Missing env `PORT`").parse::<u16>().unwrap(),
    ))
    .await;
}