extern crate acestream;

#[test]
fn http_url_check() {
    let engine = acestream::Engine::new();
    assert_eq!(engine.engine_url.as_str(), "http://127.0.0.1:6878/");
}
