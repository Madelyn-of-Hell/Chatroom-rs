use std::fmt::format;
use std::io::{Read, Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration};
use std::process::Command;
use std::thread;
use serde_json;
use std::fs;
use std::ptr::read;
use serde_json::Value;

const ENV_PATH:&str = "/Users/daddyslime/RustroverProjects/my_dumb_webserver";


fn main() {
    let ip:String = String::from("0.0.0.0:80");
    let listener:TcpListener = TcpListener::bind(&ip)
        .expect(&format!("Failed at binding to port {}", &ip));
    println!("Listening to {}", &ip);
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {thread::spawn(|| hande_client(stream));}
            Err(e) => {eprintln!("Failed to establish a connection: {}", e);}
        }
    }
}

fn hande_client(mut stream: TcpStream) {
    let buffer = BufReader::new(&mut stream);
    let request:Vec<_> = buffer
        .lines()
        .map(|result| result.unwrap())
        .take_while(|line| !line.is_empty())
        .collect();

    // let (response_status, filename, content_type, content_disposition) = match request[0].as_str() {
    //     "GET /10.177.8.110:3000/Site Files/Rick%20Astley%20-%20Never%20Gonna%20Give%20You%20Up%20(Official%20Music%20Video)%20[dQw4w9WgXcQ].webm HTTP/1.1" => ("HTTP/1.1 200 OK", "Site Files/Rick Astley - Never Gonna Give You Up (Official Music Video) [dQw4w9WgXcQ].webm", "video/webm", "inline"),
    //     "GET /404.html HTTP/1.1" => ("HTTP/1.1 200 OK", "404.html", "text/html", "inline"),
    //     "GET /rickroll.html HTTP/1.1" => ("HTTP/1.1 200 OK", "rickroll.html", "text/html", "inline"),
    //     "GET /download HTTP/1.1" => ("HTTP/1.1 200 OK", "open_me.txt", "text/plain", "attachment; filename=\"open_me.txt\""),
    //
    //     "POST /send_message.js HTTP/1.1" => ("HTTP/1.1 200 OK", "send_message.js", "text/javascript", "inline"),
    //
    //     _ => ("HTTP/1.1 404 NOT FOUND", "funnel.html", "text/plain", "inline"),
    // };
    let mut response_status:String = String::new();
    let mut filename:String = String::new();
    let mut content_type:String = String::new();
    let mut content_disposition:String = String::new();
    if request[0].contains("GET") {
        let trimmed_request = request[0].split_at(4).1.chars().rev().collect::<String>().split_at(9).1.chars().rev().collect::<String>();
        let db: Value = serde_json::from_str(&fs::read_to_string("/Users/daddyslime/RustroverProjects/my_dumb_webserver/src/Site Files/pages.json").unwrap()).unwrap();
        response_status = db["pages"][&trimmed_request][0].as_str().unwrap_or_else(||"HTTP/1.1 404 NOT FOUND").to_string();
        filename = db["pages"][&trimmed_request][1].as_str().unwrap_or_else(||"Site Files/funnel.html").to_string();
        content_type = db["pages"][&trimmed_request][2].as_str().unwrap_or_else(||"text/html").to_string();
        content_disposition = db["pages"][&trimmed_request][3].as_str().unwrap_or_else(||"inline").to_string();
    }
    else {
        let response_status = "";
        let filename = "";
        let content_type = "";
        let content_disposition = "";
    }

    println!("Packet received: {:#?}", request);
    let mut file = match filename.contains(".html") {
        true => Command::new("php").arg(format!("{ENV_PATH}/src/{filename}")).output().unwrap().stdout,
        _ => fs::read(format!("{ENV_PATH}/src/{filename}")).unwrap()
    };
    println!("Response body: [\n{}\n]", std::str::from_utf8(file.as_slice()).unwrap_or_default());

    let response_length = &file.len();
    let mut response:Vec<u8> = format!("{response_status}\r\nContent-Length: {response_length}\r\nContent-Type: {content_type}\r\nContent-Disposition:{content_disposition}\r\n\r\n").bytes().collect::<Vec<_>>();
    println!("Response header: [\n{}\n]", std::str::from_utf8(response.as_slice()).unwrap());
    response.append(&mut file.to_vec());
    send_response(&stream, &response.as_slice())
}
fn send_response(mut stream: &TcpStream, response: &[u8]) {
    stream.write(response).expect("could not write");
}
// fn