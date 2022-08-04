const CONTAINER_ID: &str = "note";

fn docker_engine_api_get_inner(path: &str) -> Vec<u8> {
    let mut handle = curl::easy::Easy::new();
    handle.unix_socket("/var/run/docker.sock").unwrap();
    handle.url(&format!("http://localhost{path}")).unwrap();
    let mut output = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer
            .write_function(|data| {
                output.extend_from_slice(data);
                Ok(data.len())
            })
            .unwrap();
        transfer.perform().unwrap();
    }
    let status_code = handle.response_code().unwrap();
    dbg!(status_code);
    output
}

// curl --unix-socket /var/run/docker.sock http://localhost/version
// reqwest not support unix socket
// hyper Client::from(steam) tokio UnixStream(tokio::net::UnixStream::connect)
fn docker_engine_api_get(path: &str) {
    let output = docker_engine_api_get_inner(path);
    let output = serde_json::from_slice::<serde_json::Value>(&output).unwrap();
    println!("GET {path}");
    println!("{}", serde_json::to_string_pretty(&output).unwrap());
}

// https://docs.docker.com/engine/api/v1.41
// docker API use HTTP status code indicate request success or error
#[test]
fn docker_version() {
    docker_engine_api_get("/version");
}

#[test]
fn docker_info() {
    docker_engine_api_get("/info");
}

/// docker ps return (container)fields is same as docker inspect
#[test]
fn docker_ps() {
    docker_engine_api_get("/containers/json");
}

#[test]
fn docker_inspect_container() {
    docker_engine_api_get(&format!("/containers/{CONTAINER_ID}/json"));
}

#[test]
fn docker_logs() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();
    rt.block_on(async {
        let stream = tokio::net::UnixStream::connect("/var/run/docker.sock")
            .await
            .unwrap();
        let (mut sender, conn) = hyper::client::conn::handshake(stream).await.unwrap();
        tokio::task::spawn(async move {
            if let Err(err) = conn.await {
                panic!("Connection failed: {:?}", err);
            }
        });

        let req = hyper::Request::builder()
            .uri(&format!(
                "http://localhost/containers/{CONTAINER_ID}/logs?stdout=true&stderr=true"
            ))
            .header(
                hyper::header::HOST,
                hyper::header::HeaderValue::from_static("localhost"),
            )
            .body(hyper::Body::empty())
            .unwrap();
        let mut res = sender.send_request(req).await.unwrap();
        println!("Response: {}", res.status());
        println!("Headers: {:#?}\n", res.headers());

        let mut body = Vec::new();
        while let Some(next) = hyper::body::HttpBody::data(&mut res).await {
            let chunk = next.unwrap();
            body.extend(chunk.to_vec());
        }
        println!("{}", String::from_utf8_lossy(&body));
    });
}

// current change to the base image?
#[test]
fn changes() {
    docker_engine_api_get(&format!("/containers/{CONTAINER_ID}/changes"));
}

// Export a container's filesystem as a tar archive
#[test]
fn docker_export() {
    let output = docker_engine_api_get_inner(&format!("/containers/{CONTAINER_ID}/export"));
    std::fs::write("target/output.tar", output).unwrap();
}
