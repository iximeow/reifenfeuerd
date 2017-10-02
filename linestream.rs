use std;
use futures::stream::Stream;
use futures::{Poll, Async};

pub struct LineStream<S, E> where S: Stream<Item=u8, Error=E> {
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

