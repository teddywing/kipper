// maybe wait a few seconds to be sure a Jenkins job was created. This happens at the caller.

// make request to [branch]-branches
// if it comes back successfully with a `builds` hash
// request all URLs in `builds`
//   if its `displayName` matches [branch]-commitsha{5}
//     check `result` ('SUCCESS', 'FAILURE', nonexistent)
//     update GitHub commit status
//       if pending
//           start a thread that checks every 30 seconds for the `result` and update GitHub commit status
//                 if time spent > 20 minutes
//                    set GH commit status to error (timeout)
//                 if `result` is successful or failed, update status and stop
//   set GH status to error (no job found)

// fn update_github_status(commit_ref)
// fn get_jobs(repo_name)
// fn af83 job name from commit_ref (separate af83 module)
// fn update_github_commit_status(status, message) (lives in GitHub module)
// fn request_job(url)
// fn result_from_job(payload)

extern crate json;
extern crate reqwest;

use self::reqwest::header::{Authorization, Basic};

#[derive(Debug, PartialEq, Eq)]
pub enum JobStatus {
    Success,
    Failure,
    Pending,
    Unknown,
}

pub fn auth_credentials() -> Basic {
    Basic {
        username: "username".to_string(),
        password: Some("token".to_string()),
    }
}

pub fn get_jobs(repo_name: String) {//-> Vec<String> {
    let client = reqwest::Client::new();

    let credentials = auth_credentials();

    let mut res = client.get("http://jenkins.example.com/job/changes-branches/18/api/json")
        .header(Authorization(credentials))
        .send()
        .unwrap();

    println!("{}", res.status());
}

pub fn result_from_job(payload: String) -> JobStatus {
    let mut job = json::parse(payload.as_ref()).unwrap();

    if job["result"].is_null() {
        return JobStatus::Pending
    }

    let status = job["result"].take_string().unwrap();

    if status == "SUCCESS" {
        JobStatus::Success
    } else if status == "FAILURE" {
        JobStatus::Failure
    } else {
        JobStatus::Unknown
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn get_jobs_queries_jobs_from_jenkins_api() {
        get_jobs("changes".to_string());
    }

    #[test]
    fn result_from_job_is_success() {
        let payload = r#"{
            "result": "SUCCESS"
        }"#;

        assert_eq!(
            result_from_job(payload.to_owned()),
            JobStatus::Success
        );
    }

    #[test]
    fn result_from_job_is_failure() {
        let payload = r#"{
            "result": "FAILURE"
        }"#;

        assert_eq!(
            result_from_job(payload.to_owned()),
            JobStatus::Failure
        );
    }

    #[test]
    fn result_from_job_is_pending() {
        let payload = r#"{
        }"#;

        assert_eq!(
            result_from_job(payload.to_owned()),
            JobStatus::Pending
        );
    }
}
