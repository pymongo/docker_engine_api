const CONTAINER_ID: &str = "8b395a0ec10a";

// curl --unix-socket /var/run/docker.sock http://localhost/version
// reqwest not support unix socket
// hyper Client::from(steam) tokio UnixStream(tokio::net::UnixStream::connect)
fn docker_engine_api_get(path: &str) {
    let mut handle = curl::easy::Easy::new();
    handle.unix_socket("/var/run/docker.sock").unwrap();
    handle.url(&format!("http://localhost{path}")).unwrap();
    let mut output = Vec::new();
    {
        let mut transfer = handle.transfer();
        transfer.write_function(|data| {
            output.extend_from_slice(data);
            Ok(data.len())
        }).unwrap();
        transfer.perform().unwrap();
    }
    let status_code = handle.response_code().unwrap();
    let output = serde_json::from_slice::<serde_json::Value>(&output).unwrap();
    println!("GET {path}");
    dbg!(status_code);
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

#[test]
fn docker_ps() {
    docker_engine_api_get("/containers/json");
}

#[test]
fn docker_inspect_container() {
    docker_engine_api_get(&format!("/containers/{CONTAINER_ID}/json"));
}
