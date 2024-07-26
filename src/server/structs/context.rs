use std::{collections::HashMap, net::TcpStream, sync::{Arc, RwLock}};

pub struct Context {
    pub subscriptions: Arc<RwLock<HashMap<String, HashMap<u128, TcpStream>>>>,
}