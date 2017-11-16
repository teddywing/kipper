// Copyright © 2017 Teddy Wing
//
// This file is part of Kipper.
//
// Kipper is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// Kipper is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with Kipper. If not, see <http://www.gnu.org/licenses/>.

extern crate getopts;
#[macro_use]
extern crate log;
extern crate stderrlog;
#[macro_use]
extern crate rouille;

extern crate kipper;

use std::env;
use std::thread;
use std::io::Read;
use std::time::Duration;

use getopts::Options;

use kipper::jenkins;
use kipper::pull_request::CommitRef;

const DEFAULT_PORT: u16 = 8000;

fn print_usage(opts: Options) {
    let brief = "Usage: kipper --jenkins-url 'https://jenkins.example.com' --jenkins-user-id username --jenkins-token a72a57d448694703b2c3fd19e666ecc5 --github-token 1dc41fad0516460b870014b25b11847d";
    print!("{}", opts.usage(&brief));
}

fn internal_server_error() -> rouille::Response {
    rouille::Response::text("500 Internal Server Error")
        .with_status_code(500)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut opts = Options::new();
    opts.optopt("", "jenkins-url", "Jenkins URL (required)", "https://jenkins.example.com");
    opts.optopt("", "jenkins-user-id", "Jenkins user ID (required)", "USER_ID");
    opts.optopt("", "jenkins-token", "Jenkins API token (required)", "TOKEN");
    opts.optopt(
        "",
        "github-token",
        "GitHub API token with \"repo:status\" permission (required)",
        "TOKEN"
    );
    opts.optopt("p", "port", "set port number", "PORT");
    opts.optflag("h", "help", "print this help menu");

    let opt_matches = match opts.parse(&args[1..]) {
        Ok(m) => m,
        Err(e) => panic!(e.to_string()),
    };

    if opt_matches.opt_present("h") {
        print_usage(opts);
        return;
    }

    let jenkins_url = match opt_matches.opt_str("jenkins-url") {
        Some(url) => url,
        None => {
            print_usage(opts);
            return;
        },
    };

    let jenkins_user_id = match opt_matches.opt_str("jenkins-user-id") {
        Some(user_id) => user_id,
        None => {
            print_usage(opts);
            return;
        },
    };

    let jenkins_token = match opt_matches.opt_str("jenkins-token") {
        Some(token) => token,
        None => {
            print_usage(opts);
            return;
        },
    };

    let github_token = match opt_matches.opt_str("github-token") {
        Some(token) => token,
        None => {
            print_usage(opts);
            return;
        },
    };

    let port = match opt_matches.opt_str("p") {
        Some(p) => p.parse().expect("Unable to parse specified port"),
        None => DEFAULT_PORT,
    };

    // Logging
    stderrlog::new()
        .module(module_path!())
        .timestamp(stderrlog::Timestamp::Second)
        .verbosity(4) // LogLevel::Trace
        .init()
        .expect("Logger failed to initialise");

    println!("Server listening on 127.0.0.1:{}", port);

    rouille::start_server(format!("127.0.0.1:{}", port), move |request| {
        router!(request,
            (POST) (/github/pull_request_event) => {
                let mut body = String::new();

                match request.data() {
                    None => rouille::Response::text("400 Bad Request")
                        .with_status_code(400),
                    Some(mut data) => {
                        try_or_400!(data.read_to_string(&mut body));

                        let commit_ref = match CommitRef::new(body.as_ref()) {
                            Ok(cr) => cr,
                            Err(e) => {
                                error!("{}", e.to_string());

                                return internal_server_error()
                            },
                        };

                        // Clone so we can use these values in the thread
                        // closure. Since both closures are required to be
                        // 'static, we can't use references to these values.
                        let jenkins_url = jenkins_url.clone();
                        let jenkins_user_id = jenkins_user_id.clone();
                        let jenkins_token = jenkins_token.clone();
                        let github_token = github_token.clone();

                        thread::spawn(move || {
                            thread::sleep(Duration::from_secs(30));

                            match jenkins::find_and_track_build_and_update_status(
                                commit_ref,
                                jenkins_url,
                                &jenkins_user_id,
                                &jenkins_token,
                                github_token,
                            ) {
                                Ok(_) => {},
                                Err(e) => error!("{}", e.to_string()),
                            };
                        });

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
