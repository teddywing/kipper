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

extern crate mockito;
extern crate reqwest;

use std::collections::HashMap;
use std::error::Error;
use std::fmt;

use self::reqwest::header::{Accept, Authorization, Bearer, qitem};

use pull_request::CommitRef;

#[cfg(not(test))]
const API_URL: &'static str = "https://api.github.com";

#[cfg(test)]
const API_URL: &'static str = mockito::SERVER_URL;

pub enum CommitStatus {
    Error,
    Failure,
    Pending,
    Success,
}

impl fmt::Display for CommitStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            CommitStatus::Error => write!(f, "error"),
            CommitStatus::Failure => write!(f, "failure"),
            CommitStatus::Pending => write!(f, "pending"),
            CommitStatus::Success => write!(f, "success"),
        }
    }
}

pub fn update_commit_status(
    github_token: &String,
    commit_ref: &CommitRef,
    state: &CommitStatus,
    target_url: String,
    description: Option<String>,
    context: String,
) -> Result<(), Box<Error>> {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("state", state.to_string());
    params.insert("target_url", target_url);
    params.insert("context", context);

    if let Some(d) = description {
        params.insert("description", d);
    }

    let mut response = client.post(
            &format!(
                "{}/repos/{}/{}/statuses/{}",
                API_URL,
                commit_ref.owner,
                commit_ref.repo,
                commit_ref.sha
            )
        )
        .header(
            Accept(
                vec![qitem("application/vnd.github.v3+json".parse()?)]
            )
        )
        .header(
            Authorization(
                Bearer {
                    token: github_token.to_owned()
                }
            )
        )
        .json(&params)
        .send()?;

    debug!("{}", response.url());
    debug!("{}", response.status());
    debug!("{}", response.headers());
    debug!("{}", response.text()?);

    Ok(())
}


#[cfg(test)]
mod tests {
    use self::mockito::mock;

    use super::*;

    #[test]
    fn update_commit_status_makes_a_request_to_github() {
        let mock = mock("POST", "/repos/octocat/Hello-World/statuses/6dcb09b5b57875f334f61aebed695e2e4193db5e")
            .with_status(201)
            .create();

        let commit_ref = CommitRef {
            owner: "octocat".to_owned(),
            repo: "Hello-World".to_owned(),
            sha: "6dcb09b5b57875f334f61aebed695e2e4193db5e".to_owned(),
            branch: "not-used".to_owned(),
        };

        update_commit_status(
            &"token".to_owned(),
            &commit_ref,
            &CommitStatus::Success,
            "https://jenkins.example.com/job/octocat/3".to_owned(),
            None,
            "continuous-integration/jenkins".to_owned()
        ).expect("Failed to update commit status");

        mock.assert();
    }
}
