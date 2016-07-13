extern crate iron;
extern crate r2d2;
extern crate r2d2_redis;
extern crate redis;

use std::sync::Arc;
use std::ops::Deref;
use std::time::Duration;
use r2d2_redis::RedisConnectionManager;

use iron::prelude::*;
use iron::status;
use iron::headers::ContentType;
use iron::mime::{Mime, TopLevel, SubLevel};

macro_rules! unwrap_or_error_response {
    ($expr:expr, $or:expr) => (
        match $expr {
            Ok(x) => x,
            Err(_) => return Ok(Response::with(($or)))
        }
    )
}

fn to_json(val: redis::Value) -> String {
    let mut json = String::from("");

    match val {
        redis::Value::Data(data) => {
            let d = String::from_utf8(data).unwrap();
            json.push_str("\"");
            json.push_str(&d);
            json.push_str("\"");
        }
        redis::Value::Okay => {
            json.push_str("\"OK\"");
        }
        redis::Value::Nil => {
            json.push_str("null");
        }
        redis::Value::Int(i) => {
            json.push_str(&i.to_string());
        }
        redis::Value::Bulk(bulk) => {
           json.push_str("[");
           let all = bulk.into_iter().map(|val| {
               to_json(val)
           }).collect::<Vec<_>>().join(",");
           json.push_str(&all);
           json.push_str("]");
        }
        redis::Value::Status(status) => {
           json.push_str("\"");
           json.push_str(&status);
           json.push_str("\"");
        }
    }

    json
}

fn main() {
    let config = r2d2::Config::builder()
        .connection_timeout(Duration::from_millis(2*1000))
        .pool_size(3)
        .build();
    let manager = RedisConnectionManager::new("redis://localhost").unwrap();
    let pool = Arc::new(r2d2::Pool::new(config, manager).unwrap());


    println!("Listening on http://localhost:3000");
    Iron::new(move |req: &mut Request| {
        let pool = pool.clone();
        let mut path_iter = req.url.path().into_iter();

        let res : redis::RedisResult<redis::Value> ;
        {
            let conn = unwrap_or_error_response!(pool.get(), status::InternalServerError);

            let command = path_iter.next().unwrap();
            let mut cmd = redis::cmd(&command);

            for p in path_iter {
                cmd.arg(p.clone());
            }

            res = cmd.query(conn.deref());
        }

        match res {
            Ok(res) => {
                let mut json = String::from("{\"data\":");

                json.push_str(&to_json(res));
                json.push_str("}");

                let mut resp = Response::with((status::Ok, json));
                resp.headers.set(ContentType(
                        Mime(TopLevel::Application, SubLevel::Json, vec![])));

                Ok(resp)
            }
            Err(err) => {
                let mut json = String::from("{\"error\":\"");

                json.push_str(&format!("{}", err));
                json.push_str("\"}");

                let mut resp = Response::with((status::BadRequest, json));
                resp.headers.set(ContentType(
                        Mime(TopLevel::Application, SubLevel::Json, vec![])));

                Ok(resp)
            }
        }
    }).http("localhost:3000").unwrap();
}
