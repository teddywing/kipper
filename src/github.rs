extern crate mockito;
extern crate reqwest;

use std::collections::HashMap;
use std::fmt;

use self::reqwest::header::{Accept, qitem};

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
    commit_ref: &CommitRef,
    state: &CommitStatus,
    target_url: String,
    description: Option<String>,
    context: String,
) {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("state", state.to_string());
    params.insert("target_url", target_url);
    params.insert("context", context);

    if let Some(d) = description {
        params.insert("description", d);
    }

    client.post(
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
                vec![qitem("application/vnd.github.v3+json".parse().unwrap())]
            )
        )
        .json(&params)
        .send()
        .unwrap();
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
            owner: "octocat".to_string(),
            repo: "Hello-World".to_string(),
            sha: "6dcb09b5b57875f334f61aebed695e2e4193db5e".to_string(),
            branch: "not-used".to_string(),
        };

        update_commit_status(
            &commit_ref,
            &CommitStatus::Success,
            "https://jenkins.example.com/job/octocat/3".to_string(),
            None,
            "continuous-integration/jenkins".to_string()
        );

        mock.assert();
    }
}
