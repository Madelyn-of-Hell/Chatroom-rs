use std::fmt::format;
use std::io::{Read, Write, BufReader, BufRead};
use std::net::{TcpListener, TcpStream};
use std::time::{Duration};
use std::thread;
use std::fs;
use std::process::Command;
const ENV_PATH:&str = "/Users/daddyslime/RustroverProjects/non_blocking_input_test";

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

    let (response_status, filename, content_type, content_disposition) = match request[0].as_str() {
        "GET / HTTP/1.1" => ("HTTP/1.1 200 OK", "index.html", "text/html", "inline"),
        "GET /10.177.8.110:3000/media/Rick%20Astley%20-%20Never%20Gonna%20Give%20You%20Up%20(Official%20Music%20Video)%20[dQw4w9WgXcQ].webm HTTP/1.1" => ("HTTP/1.1 200 OK", "media/Rick Astley - Never Gonna Give You Up (Official Music Video) [dQw4w9WgXcQ].webm", "video/webm", "inline"),
        "GET /404.html HTTP/1.1" => ("HTTP/1.1 200 OK", "404.html", "text/html", "inline"),
        "GET /rickroll.html HTTP/1.1" => ("HTTP/1.1 200 OK", "rickroll.html", "text/html", "inline"),
        "GET /download HTTP/1.1" => ("HTTP/1.1 200 OK", "open_me.txt", "text/plain", "attachment; filename=\"open_me.txt\""),

        "POST /send_message.js HTTP/1.1" => ("HTTP/1.1 200 OK", "send_message.js", "text/javascript", "inline"),

        _ => ("HTTP/1.1 404 NOT FOUND", "funnel.html", "text/plain", "inline"),
    };

    println!("Packet received: {:#?}", request);
    let mut file = match filename.contains(".html") {
        true => Command::new("php").arg(format!("{ENV_PATH}/src/{filename}")).output().unwrap().stdout,
        _ => fs::read(format!("{ENV_PATH}/src/{filename}")).unwrap()
    };
    println!("{:?}", Command::new("php").arg(format!("{ENV_PATH}/src/{filename}")).output().unwrap());
    println!("Response body: [\n{}\n]", std::str::from_utf8(file.as_slice()).unwrap());

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