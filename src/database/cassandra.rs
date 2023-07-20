use cdrs::authenticators::StaticPasswordAuthenticator;
use cdrs::cluster::session::{new as new_session, Session};
use cdrs::cluster::{ClusterTcpConfig, NodeTcpConfigBuilder, TcpConnectionPool};
use cdrs::load_balancing::RoundRobin;
use once_cell::sync::OnceCell;
use cdrs::query::*;
use std::vec;

type CurrentSession = Session<RoundRobin<TcpConnectionPool<StaticPasswordAuthenticator>>>;
static SESSION: OnceCell<CurrentSession> = OnceCell::new();

const DEFAULT_VALUE: &str = "";

/// Init cassandra session
pub fn init() {
    let _ = SESSION.set(new_session(
        &ClusterTcpConfig(vec![
            NodeTcpConfigBuilder::new(
                dotenv::var("CASSANDRA_HOST").unwrap_or_else(|_| "127.0.0.1:9042".to_string()).as_str(),
                StaticPasswordAuthenticator::new(
                    &dotenv::var("CASSANDRA_USER").unwrap_or_else(|_| "cassandra".to_string()).as_str(),
                    &dotenv::var("CASSANDRA_PASSWORD").unwrap_or_else(|_| "cassandra".to_string()).as_str()
                )
            ).build()]
        ), RoundRobin::new()).expect("session should be created"));
}

/// Create tables in cassandra keyspace if not exists
pub fn create_tables() {
    //SESSION.get().unwrap().query("CREATE KEYSPACE IF NOT EXISTS accounts WITH REPLICATION = { 'class' : 'SimpleStrategy', 'replication_factor' : 1 };").expect("Keyspace create error");
    SESSION.get().unwrap().query("CREATE TABLE IF NOT EXISTS accounts.users ( vanity TEXT, email TEXT, username TEXT, avatar TEXT, banner TEXT, bio TEXT, verified BOOLEAN, flags INT, phone TEXT, password TEXT, birthdate TEXT, deleted BOOLEAN, mfa_code TEXT, expire_at TIMESTAMP, PRIMARY KEY (vanity) )").expect("accounts.users create error");
    SESSION.get().unwrap().query("CREATE TABLE IF NOT EXISTS accounts.bots ( id TEXT, user_id TEXT, client_secret TEXT, username TEXT, avatar TEXT, bio TEXT, redirect_url SET<TEXT>, flags INT, deleted BOOLEAN, PRIMARY KEY (id) )").expect("accounts.bots create error");
    SESSION.get().unwrap().query("CREATE TABLE IF NOT EXISTS accounts.oauth ( id TEXT, user_id TEXT, bot_id TEXT, ip TEXT, scope SET<TEXT>, deleted BOOLEAN, PRIMARY KEY (id) )").expect("accounts.oauth create error");
    SESSION.get().unwrap().query("CREATE TABLE IF NOT EXISTS accounts.tokens ( id TEXT, user_id TEXT, ip TEXT, date TIMESTAMP, expire_at TIMESTAMP, deleted BOOLEAN, PRIMARY KEY (id) )").expect("accounts.tokens create error");
    SESSION.get().unwrap().query("CREATE TABLE IF NOT EXISTS accounts.salts ( id TEXT, salt TEXT, PRIMARY KEY (id) )").expect("accounts.salts create error");
    SESSION.get().unwrap().query("CREATE INDEX IF NOT EXISTS ON accounts.users ( email );").expect("second index (email) create error");
    SESSION.get().unwrap().query("CREATE INDEX IF NOT EXISTS ON accounts.users ( expire_at );").expect("second index (expire_at) create error");
    SESSION.get().unwrap().query("CREATE INDEX IF NOT EXISTS ON accounts.oauth ( user_id );").expect("second index (user_id) create error");
    SESSION.get().unwrap().query("CREATE INDEX IF NOT EXISTS ON accounts.tokens ( user_id );").expect("second index (user_id) create error");
}

/// Make a query to cassandra
pub fn query<Q: ToString>(query: Q, params: Vec<String>) -> Result<cdrs::frame::Frame, cdrs::error::Error> {
    SESSION.get().unwrap().query_with_values(query, params)
}

/// Create a user for account delegation
/// ```rust
/// create_user(&"realhinome".to_string(), "email@mail.com".to_string(), "Hinome".to_string(), "SuperPassword_123456".to_string(), None, None)
/// ```
pub fn create_user(vanity: &String, email: String, username: String, password: String, phone: Option<String>, birthdate: Option<String>) -> Result<(), cdrs::error::Error> {
    let mut user: Vec<String> = vec![vanity.to_string(), email, username, password];
    if let Some(x) = phone { user.push(x); } else { user.push("".to_string()); }
    if let Some(y) = birthdate { user.push(y); } else { user.push("".to_string()); }

    SESSION.get().unwrap().query_with_values(format!("INSERT INTO accounts.users (vanity, email, username, password, phone, birthdate, flags, deleted, verified) VALUES (?, ?, ?, ?, ?, ?, {}, {}, {})", 0, false, false), user)?;

    Ok(())
}

/// Create a bot
pub fn _create_bot(vanity: String, client_secret: String, username: String) -> Result<(), cdrs::error::Error> {
    SESSION.get().unwrap().query_with_values(format!("INSERT INTO accounts.bots (id, client_secret, username, flags, deleted) VALUES (?, ?, ?, {}, {})", 0, false), vec![vanity, client_secret, username])?;

    Ok(())
}

/// Create a OAuth2 code
pub fn create_oauth(id: String, vanity: String, bot_id: String, scope: Vec<String>) {
    let _ = SESSION.get().unwrap().query_with_values("INSERT INTO accounts.oauth (id, user_id, bot_id, scope, deleted) VALUES (?, ?, ?, ?, ?)", cdrs::query_values!(id, vanity, bot_id, scope, false));
}

/// Create a new salt to split it and secure it
pub fn create_salt(salt: String) -> String {
    let id = uuid::Uuid::new_v4().to_string();

    let _ = SESSION.get().unwrap().query_with_values("INSERT INTO accounts.oauth (id, salt) VALUES (?, ?)", cdrs::query_values!(id.clone(), salt));

    id
}

/// Update a user in cassandra database
pub fn update_user(
    username: String,
    avatar: Option<String>,
    bio: Option<String>,
    birthdate: Option<String>,
    phone: Option<String>,
    email: String,
    vanity: String,
) -> Result<(), cdrs::error::Error> {
    SESSION.get().unwrap().query_with_values("UPDATE accounts.users SET username = ?, avatar = ?, bio = ?, birthdate = ?, phone = ?, email = ? WHERE vanity = ?",
    vec![
            username,
            avatar.unwrap_or_else(|| DEFAULT_VALUE.to_string()),
            bio.unwrap_or_else(|| DEFAULT_VALUE.to_string()),
            birthdate.unwrap_or_else(|| DEFAULT_VALUE.to_string()),
            phone.unwrap_or_else(|| DEFAULT_VALUE.to_string()),
            email,
            vanity
        ]
    )?;

    Ok(())
}
