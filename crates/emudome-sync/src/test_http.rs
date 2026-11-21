//! A scripted HTTP/1.1 server on `127.0.0.1:0` for the tests of the request shapes (token
//! exchange, refresh, Drive calls, resumable uploads): it records every request and answers
//! with what the test's responder returns. Never a fixed port, never a call to Google.

use std::sync::Arc;

use parking_lot::Mutex;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::{TcpListener, TcpStream};

/// A request as the server received it.
#[derive(Clone, Debug)]
pub struct Recorded {
    pub method: String,
    /// The path with its query, as sent.
    pub target: String,
    /// Names lowercased.
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Recorded {
    pub fn header(&self, name: &str) -> Option<&str> {
        self.headers
            .iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v.as_str())
    }

    pub fn path(&self) -> &str {
        self.target.split('?').next().unwrap_or("")
    }

    pub fn form(&self) -> Vec<(String, String)> {
        url::form_urlencoded::parse(&self.body)
            .into_owned()
            .collect()
    }

    pub fn form_value(&self, name: &str) -> Option<String> {
        self.form()
            .into_iter()
            .find(|(n, _)| n == name)
            .map(|(_, v)| v)
    }
}

/// An answer.
#[derive(Clone, Debug)]
pub struct Reply {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Vec<u8>,
}

impl Reply {
    pub fn json(status: u16, value: serde_json::Value) -> Self {
        Self {
            status,
            headers: vec![("content-type".into(), "application/json".into())],
            body: value.to_string().into_bytes(),
        }
    }

    pub fn bytes(status: u16, body: &[u8]) -> Self {
        Self {
            status,
            headers: Vec::new(),
            body: body.to_vec(),
        }
    }
}

type Responder = dyn Fn(&Recorded) -> Reply + Send + Sync;

pub struct MockServer {
    pub base: String,
    requests: Arc<Mutex<Vec<Recorded>>>,
    task: tokio::task::JoinHandle<()>,
}

impl MockServer {
    pub async fn start(responder: impl Fn(&Recorded) -> Reply + Send + Sync + 'static) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").await.unwrap();
        let base = format!("http://{}", listener.local_addr().unwrap());
        let requests = Arc::new(Mutex::new(Vec::new()));
        let responder: Arc<Responder> = Arc::new(responder);
        let seen = requests.clone();
        let task = tokio::spawn(async move {
            while let Ok((stream, _)) = listener.accept().await {
                tokio::spawn(serve(stream, seen.clone(), responder.clone()));
            }
        });
        Self {
            base,
            requests,
            task,
        }
    }

    pub fn url(&self, path: &str) -> String {
        format!("{}{path}", self.base)
    }

    pub fn requests(&self) -> Vec<Recorded> {
        self.requests.lock().clone()
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        self.task.abort();
    }
}

async fn serve(mut stream: TcpStream, seen: Arc<Mutex<Vec<Recorded>>>, responder: Arc<Responder>) {
    let mut buf = Vec::new();
    loop {
        let Some(request) = read_request(&mut stream, &mut buf).await else {
            return;
        };
        let reply = responder(&request);
        seen.lock().push(request);
        let mut head = format!("HTTP/1.1 {} X\r\n", reply.status);
        for (n, v) in &reply.headers {
            head.push_str(&format!("{n}: {v}\r\n"));
        }
        head.push_str(&format!("content-length: {}\r\n\r\n", reply.body.len()));
        if stream.write_all(head.as_bytes()).await.is_err()
            || stream.write_all(&reply.body).await.is_err()
        {
            return;
        }
    }
}

async fn read_request(stream: &mut TcpStream, buf: &mut Vec<u8>) -> Option<Recorded> {
    let head_end = loop {
        if let Some(i) = buf.windows(4).position(|w| w == b"\r\n\r\n") {
            break i;
        }
        let mut chunk = [0u8; 8192];
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
    };
    let head = String::from_utf8_lossy(&buf[..head_end]).into_owned();
    let mut lines = head.split("\r\n");
    let mut first = lines.next()?.split(' ');
    let method = first.next()?.to_owned();
    let target = first.next()?.to_owned();
    let headers: Vec<(String, String)> = lines
        .filter_map(|l| l.split_once(':'))
        .map(|(n, v)| (n.trim().to_ascii_lowercase(), v.trim().to_owned()))
        .collect();
    let length: usize = headers
        .iter()
        .find(|(n, _)| n == "content-length")
        .and_then(|(_, v)| v.parse().ok())
        .unwrap_or(0);
    let body_start = head_end + 4;
    while buf.len() < body_start + length {
        let mut chunk = [0u8; 65536];
        let n = stream.read(&mut chunk).await.ok()?;
        if n == 0 {
            return None;
        }
        buf.extend_from_slice(&chunk[..n]);
    }
    let body = buf[body_start..body_start + length].to_vec();
    buf.drain(..body_start + length);
    Some(Recorded {
        method,
        target,
        headers,
        body,
    })
}
