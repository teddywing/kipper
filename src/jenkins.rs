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

struct Job {
    display_name: String,
    result: JobStatus,
}

impl Job {
    fn new(payload: String) -> Job {
        let mut job = json::parse(payload.as_ref()).unwrap();

        Job {
            display_name: job["displayName"].take_string().unwrap(),
            result: result_from_job(job["result"].take_string()),
        }
    }
}

pub fn update_commit_status(commit_ref) {
    let jobs = get_jobs();

    for job_url in jobs {
        let payload = request_job(job_url);

        // Does `displayName` match
        if job_for_commit(payload, commit_ref) {
            // spawn thread
            let status = result_from_job(payload);
        }
    }
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

// Does the `commit_ref` correspond to the job in the `payload`?
pub fn job_for_commit(payload: String, commit_ref: CommitRef) -> bool {
}

pub fn result_from_job(status: Option<String>) -> JobStatus {
    match status {
        None => JobStatus::Pending,
        Some(s) => {
            match s.as_ref() {
                "SUCCESS" => JobStatus::Success,
                "FAILURE" => JobStatus::Failure,
                _ => JobStatus::Unknown,
            }
        }
    }
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_new_creates_a_job_from_payload() {
        let payload = r#"{
            "displayName": "3296-fix-typo-700d0",
            "result": "SUCCESS"
        }"#.to_string();

        let job = Job::new(payload);

        assert_eq!(job.display_name, "3296-fix-typo-700d0");
        assert_eq!(job.result, JobStatus::Success);
    }

    #[test]
    fn get_jobs_queries_jobs_from_jenkins_api() {
        get_jobs("changes".to_string());
    }

    #[test]
    fn result_from_job_is_success() {
        assert_eq!(
            result_from_job(Some("SUCCESS".to_string())),
            JobStatus::Success
        );
    }

    #[test]
    fn result_from_job_is_failure() {
        assert_eq!(
            result_from_job(Some("FAILURE".to_string())),
            JobStatus::Failure
        );
    }

    #[test]
    fn result_from_job_is_pending() {
        assert_eq!(
            result_from_job(None),
            JobStatus::Pending
        );
    }
}
