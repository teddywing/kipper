#[macro_use]
extern crate rouille;


fn main() {
    rouille::start_server("localhost:8000", move |request| {
        router!(request,
            (GET) (/) => {
                rouille::Response::text("hello world")
            },

            _ => rouille::Response::text("404 Not Found")
                    .with_status_code(404)
        )
    });
}
