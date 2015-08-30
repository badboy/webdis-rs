extern crate iron;
extern crate redis;

use std::sync::Mutex;

use iron::prelude::*;
use iron::status;
use iron::headers::ContentType;
use iron::mime::{Mime, TopLevel, SubLevel};

fn redis_connection() -> redis::Connection {
    let client = redis::Client::open("redis://127.0.0.1/").unwrap();
    let con = client.get_connection().unwrap();

    con
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
    let conn = redis_connection();
    let pool = Mutex::new(conn);

    println!("Listening on http://localhost:3000");
    Iron::new(move |req: &mut Request| {
        let mut path_iter = req.url.path.iter();

        let res : redis::RedisResult<redis::Value> ;
        {
            let conn = pool.lock().unwrap();

            let command = path_iter.next().unwrap();
            let mut cmd = redis::cmd(&command);

            for p in path_iter {
                cmd.arg(p.clone());
            }

            res = cmd.query(&*conn);
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
