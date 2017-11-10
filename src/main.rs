#[macro_use]
extern crate rouille;

extern crate kipper;

use std::io::Read;

use kipper::jenkins;
use kipper::pull_request::CommitRef;

fn main() {
    rouille::start_server("localhost:8000", move |request| {
        router!(request,
            (POST) (/github/pull_request_event) => {
                let mut body = String::new();

                match request.data() {
                    None => rouille::Response::text("400 Bad Request")
                        .with_status_code(400),
                    Some(mut data) => {
                        try_or_400!(data.read_to_string(&mut body));

                        let commit_ref = CommitRef::new(body.as_ref());

                        jenkins::find_and_track_build_and_update_status(commit_ref);

                        rouille::Response::text("202 Accepted")
                            .with_status_code(202)
                    }
                }
            },

            _ => rouille::Response::text("404 Not Found")
                    .with_status_code(404)
        )
    });
}
