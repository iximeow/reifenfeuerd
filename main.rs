use rustc_serialize::json::Json;
use std::str;
use std::io::BufRead;

pub trait JsonObjectStreamer: Sized {
    fn json_objects(&mut self) -> JsonObjects<Self>;
}

impl<T: BufRead> JsonObjectStreamer for T {
    fn json_objects(&mut self) -> JsonObjects<T> {
        JsonObjects { reader: self }
    }
}

pub struct JsonObjects<'a, B> where B: 'a {
    reader: &'a mut B
}

impl<'a, B> Iterator for JsonObjects<'a, B> where B: BufRead + 'a {

    type Item = Json;

    fn next(&mut self) -> Option<Json> {

        let mut buf: Vec<u8> = Vec::new();

        let _ = self.reader.read_until(b'\r', &mut buf);

        if buf.last() == Some(&b'\r') {
            buf.pop();
            let mut b: String = String::new();
            match self.reader.read_line(&mut b) {
                Ok(_)  => (),
                Err(_) => return None,
            }
        }

        let line = match str::from_utf8(&buf) {
            Ok(line) => line,
            Err(_)   => return None
        };

        Json::from_str(line).ok()

    }

}

extern crate url;
#[macro_use] extern crate hyper;
extern crate rustc_serialize;
extern crate oauthcli;
extern crate tokio_core;
extern crate futures;
extern crate hyper_tls;

use hyper::{Client, Method, Request};
use hyper::header::Headers;
use std::io::BufReader;
use tokio_core::reactor::Core;
use futures::future::Future;
use futures::Stream;
use hyper::client::FutureResponse;
use hyper_tls::HttpsConnector;
//use json_streamer::JsonObjectStreamer;

header! { (Authorization, "Authorization") => [String] }
header! { (Accept, "Accept") => [String] }
header! { (ContentType, "Content-Type") => [String] }

fn main() {

    //Change these values to your real Twitter API credentials
    let consumer_key = "0af9c1AoEi5X7IjtOKAtP60Za";
    let consumer_secret = "1fxEzRhQtQSWKus4oqDwdg5DALIjGpINg0PGjkYVwKT8EEMFCh";
    let token = "629126745-VePBD9ciKwpuVuIeEcNnxwxQFNWDXEy8KL3dGRRg";
    let token_secret = "uAAruZzJu03NvMlH6cTeGku7NqVPro1ddKN4BxORy5hWG";

    //Track words
//    let params: Vec<(String, String)> = vec![("track".to_string(), "london".to_string())];
    let params: Vec<(String, String)> = vec![];
//    let url = "https://stream.twitter.com/1.1/statuses/filter.json";
    let url = "https://userstream.twitter.com/1.1/user.json";
//    let url = "https://stream.twitter.com/1.1/statuses/sample.json";

    let header = oauthcli::authorization_header(
        "GET",
        url::Url::parse(url).unwrap(),
        None, // Realm
        consumer_key,
        consumer_secret,
        Some(token),
        Some(token_secret),
        oauthcli::SignatureMethod::HmacSha1,
        &oauthcli::timestamp(),
        &oauthcli::nonce(),
        None, // oauth_callback
        None, // oauth_verifier
        params.clone().into_iter()
    );

    let mut core = Core::new().unwrap();

    let connector = HttpsConnector::new(4, &core.handle()).unwrap();

    let client = Client::configure()
        .keep_alive(true)
        .connector(connector)
        .build(&core.handle());

    let param_string: String = params.iter().map(|p| p.0.clone() + &"=".to_string() + &p.1).collect::<Vec<String>>().join("&");

    let mut req = Request::new(Method::Get, url.parse().unwrap());
    req.set_body(param_string);

    {
        let mut headers = req.headers_mut();
        headers.set(Authorization(header.to_owned()));
        headers.set(Accept("*/*".to_owned()));
        headers.set(ContentType("application/x-www-form-urlencoded".to_owned()));
    };

    println!("requesting...");
    /*
    let work = client.request(req).and_then(|res| {
        res.body().for_each(move |body: hyper::Chunk| {
            println!("hmmm");
            println!("{}", std::str::from_utf8(&body).unwrap());
            Ok(())
        })
    });
    */

    let work = client.request(req).and_then(|res| {
        LineStream::new(res.body()
            .map(|chunk| futures::stream::iter(chunk.into_iter().map(|b| -> Result<u8, hyper::Error> { Ok(b) })))
            .flatten())
            .for_each(|s| Ok(print!("{}", str::from_utf8(&s).unwrap())))
    });

    println!("Before?");
    let resp = core.run(work).unwrap();
    println!("After?");

    /*
    for obj in BufReader::new(res).json_objects() {
        println!("{:?}", obj.as_object().unwrap().get("text").unwrap().as_string().unwrap());
    }*/

}

//extern crate futures;
//use futures::stream::Stream;
//use futures::{Future, Poll, Async};
use futures::{Poll, Async};
/*
fn main() {
  let lines = "line 1.\nline 2...\n   LINE 3  \n".as_bytes();
  let bytestream = futures::stream::iter(lines.iter().map(|byte| -> Result<_, ()> { Ok(*byte) }));
  let linestream = LineStream::new(bytestream);
  
  linestream.for_each(|line| {
    println!("Bytes: {:?}", line);
    println!("Line: {}", String::from_utf8(line).unwrap());
    Ok(())
  }).wait().unwrap()
}
*/

struct LineStream<S, E> where S: Stream<Item=u8, Error=E> {
  stream: S,
  progress: Vec<u8>
}

impl<S,E> LineStream<S, E> where S: Stream<Item=u8, Error=E> + Sized {
  pub fn new(stream: S) -> LineStream<S, E> {
    LineStream {
      stream: stream,
      progress: vec![]
    }
  }
}

impl<S, E> Stream for LineStream<S, E> where S: Stream<Item=u8, Error=E> {
  type Item = Vec<u8>;
  type Error = E;
  
  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    loop {
      match self.stream.poll() {
        Ok(Async::Ready(Some(byte))) => {
          if byte == 0x0a { 
            let mut new_vec = vec![];
            std::mem::swap(&mut self.progress, &mut new_vec);
            return Ok(Async::Ready(Some(new_vec)))
          } else {
            self.progress.push(byte)
          }
        },
        Ok(Async::Ready(None)) => return Ok(Async::Ready(None)),
        Ok(Async::NotReady) => return Ok(Async::NotReady),
        Err(e) => return Err(e)
      }
    }
  }
}

