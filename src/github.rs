extern crate mockito;

use pull_request::CommitRef;

#[cfg(not(test))]
const API_URL: &'static str = "https://api.github.com";

#[cfg(test)]
const API_URL: &'static str = mockito::SERVER_URL;

enum CommitStatus {
    Error,
    Failure,
    Pending,
    Success,
}

fn update_commit_status(
    commit_ref: CommitRef,
    state: CommitStatus,
    target_url: String,
    description: Option<String>,
    context: String,
) {
    let client = reqwest::Client::new();

    let mut params = HashMap::new();
    params.insert("state", state);
    params.insert("target_url", target_url);
    params.insert("description", description);
    params.insert("context", context);

    let response = client.post(
        format!("{}/repos/{}/{}/statuses/{}", API_URL, commit_ref.repo, commit_ref.sha)
    );
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
            repo: "Hello-World".to_string(),
            sha: "6dcb09b5b57875f334f61aebed695e2e4193db5e".to_string(),
            branch: "not-used".to_string(),
        };

        update_commit_status(
            commit_ref,
            CommitStatus::Success,
            "https://jenkins.example.com/job/octocat/3",
            None,
            "continuous-integration/jenkins"
        );

        mock.assert();
    }
}
