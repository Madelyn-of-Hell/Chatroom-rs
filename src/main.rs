use std::alloc::handle_alloc_error;
use std::fmt::format;
use std::io::{Write, BufReader, BufRead, Read};
use std::net::{TcpListener, TcpStream};
use std::process::Command;
use std::time::{Duration,SystemTime};
use std::thread;
use serde_json;
use regex::Regex;
use std::fs;
use serde_json::{from_str, to_string, Value};
use log::debug;

const ENV_PATH:&str = "/Users/daddyslime/RustroverProjects/my_dumb_webserver";

struct HttpResponse {
    status: String,
    filename: String,
    content_type: String,
    content_disposition: String,
    has_php: bool
}
fn main() {
    let ip:String = String::from("0.0.0.0:80");
    let listener:TcpListener = TcpListener::bind(&ip)
        .expect(&format!("Failed at binding to port {}", &ip));
    println!("Listening to {}", &ip);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {thread::spawn(|| handle_client(stream));}
            Err(e) => {eprintln!("Failed to establish a connection: {}", e);}
        }
    }
}
// fn
fn handle_client(mut stream: TcpStream) {
    let mut buffer = BufReader::new(&mut stream);
    let mut request: Vec<String> = vec![];
    loop {
        let mut line = String::new();

        let _ = buffer.read_line(&mut line);

        // The final line is just /r/n
        if line.len() == 2 {
            break
        }
        request.push(line.trim().to_string());

    }
    println!("Packet received: {:#?}", request);

    if request[0].contains("GET") {
        println!("Getting Files");
        let trimmed_request = request[0].split_at(4).1.chars().rev().collect::<String>().split_at(9).1.chars().rev().collect::<String>();
        let db: Value = serde_json::from_str(&fs::read_to_string(format!("{ENV_PATH}/src/Site Files/pages.json")).unwrap()).unwrap();
        let response: HttpResponse = HttpResponse {
            status:             db["pages"][&trimmed_request][0].as_str() .unwrap_or_else(|| "HTTP/1.1 404 NOT FOUND").to_string(),
            filename:           db["pages"][&trimmed_request][1].as_str() .unwrap_or_else(|| "Site Files/funnel.html").to_string(),
            content_type:       db["pages"][&trimmed_request][2].as_str() .unwrap_or_else(|| "text/html").to_string(),
            content_disposition:db["pages"][&trimmed_request][3].as_str() .unwrap_or_else(|| "inline").to_string(),
            has_php:            db["pages"][&trimmed_request][4].as_bool().unwrap_or_else(|| false)
        };
        assemble_response(&stream, response);
    } else if request[0].contains("POST") {
        println!("Receiving data");
        let mut submission_data: Vec<[u8;1]> = vec![];
        while !submission_data.ends_with(&[[b'%'], [b'7'], [b'C']]) {

            let mut bytes:[u8;1] = [0];

            let _ = buffer.read_exact(&mut bytes);
            // The final line is just /r/n
            submission_data.push(bytes);
        }
        let submission_data = format!("{}",std::str::from_utf8(submission_data.clone().iter().copied().flatten().collect::<Vec<u8>>().as_slice()).unwrap());
        let msg_regex = Regex::new(r"author=(.+)&message=(.+)&").unwrap();
        let Some(message) = msg_regex.captures(&submission_data) else { return };
        println!("{}",submission_data);

        write_message((&message[1], &message[2]));
        let db:Value = from_str(&fs::read_to_string(format!("{ENV_PATH}/src/Site Files/pages.json")).unwrap()).unwrap();
        let response: HttpResponse = HttpResponse {
            status:             db["pages"]["/"][0].as_str() .unwrap_or_else(|| "HTTP/1.1 404 NOT FOUND").to_string(),
            filename:           db["pages"]["/"][1].as_str() .unwrap_or_else(|| "Site Files/funnel.html").to_string(),
            content_type:       db["pages"]["/"][2].as_str() .unwrap_or_else(|| "text/html").to_string(),
            content_disposition:db["pages"]["/"][3].as_str() .unwrap_or_else(|| "inline").to_string(),
            has_php:            db["pages"]["/"][4].as_bool().unwrap_or_else(|| false)
        };
        assemble_response(&stream, response)
    }
}
fn write_message(message: (&str, &str)) -> () {
    let mut db:Value = serde_json::from_str(&fs::read_to_string(format!("{ENV_PATH}/src/Site Files/message_log.json")).unwrap()).unwrap();

    db["messages"].as_array_mut().unwrap().push(serde_json::json!({"author": message.0, "message":message.1}));

    fs::write(format!("{ENV_PATH}/src/Site Files/message_log.json"), to_string(&db).unwrap()).unwrap();
    println!("Wrote to file");
}

fn assemble_response(mut stream: &TcpStream, http_response: HttpResponse) {
    let file = match http_response.has_php {
        true => Command::new("php").arg(format!("{ENV_PATH}/src/{}", http_response.filename)).output().unwrap().stdout,
        _ => fs::read(format!("{ENV_PATH}/src/{}",http_response.filename)).unwrap()
    };
    println!("Response body: [\n{}{}\n]", &std::str::from_utf8(file.as_slice()).unwrap_or_default()[0..50], "... Response Truncated");

    let response_length = &file.len();
    let mut response:Vec<u8> = format!("{}\r\nContent-Length: {response_length}\r\nContent-Type: {}\r\nContent-Disposition:{}\r\n\r\n",http_response.status,http_response.content_type,http_response.content_disposition).bytes().collect::<Vec<_>>();
    println!("Response header: [\n{}\n]", std::str::from_utf8(response.as_slice()).unwrap());
    response.append(&mut file.to_vec());
    send_response(&stream, &response.as_slice())
}
fn send_response(mut stream: &TcpStream, response: &[u8]) {
    stream.write(response).expect("could not write");
}
