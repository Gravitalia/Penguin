use warp::Filter;
mod router;
mod helpers;
mod database;

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
    .or(warp::path!("users" / String).and_then(|id| async {
        if true {
            Ok(router::users::get(id).await)
        } else {
            Err(warp::reject::not_found())
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