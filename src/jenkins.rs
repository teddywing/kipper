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
extern crate mockito;
extern crate reqwest;

use self::reqwest::header::{Authorization, Basic};

use af83;
use pull_request::CommitRef;

#[cfg(not(test))]
const API_URL: &'static str = "http://jenkins.example.com";

#[cfg(test)]
const API_URL: &'static str = mockito::SERVER_URL;

#[derive(Debug, PartialEq, Eq)]
pub enum JobStatus {
    Success,
    Failure,
    Pending,
    Unknown,
}

pub struct Job {
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

pub fn get_jobs(repo_name: String) -> Vec<String> {
    let client = reqwest::Client::new();

    let credentials = auth_credentials();

    let mut response = client.get(&format!("{}/job/{}-branches/api/json", API_URL, repo_name))
        .header(Authorization(credentials))
        .send()
        .unwrap();

    let body = response.text().unwrap();

    let jobs = json::parse(body.as_ref()).unwrap();

    jobs["builds"].members()
        .map(|job| {
            job["url"].clone().take_string().unwrap()
        })
        .collect::<Vec<String>>()
}

pub fn request_job(url: String) -> Job {
    let client = reqwest::Client::new();

    let credentials = auth_credentials();

    let mut response = client.get(&format!("{}/api/json", url))
        .header(Authorization(credentials))
        .send()
        .unwrap();

    let body = response.text().unwrap();

    let mut job = json::parse(body.as_ref()).unwrap();

    Job {
        display_name: job["displayName"].take_string().unwrap(),
        result: result_from_job(job["result"].take_string()),
    }
}

// Does the `commit_ref` correspond to the job?
pub fn job_for_commit(job: Job, commit_ref: CommitRef) -> bool {
    job.display_name == af83::job_name(commit_ref)
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
    use self::mockito::mock;

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
        let mock = mock("GET", "/job/changes-branches/api/json")
            .with_status(200)
            .with_header("content-type", "application/json;charset=utf-8")
            .with_body(r#"
                {
                  "displayName": "changes-branches",
                  "builds": [
                    {
                      "_class": "hudson.model.FreeStyleBuild",
                      "number": 18,
                      "url": "http://jenkins.example.com/job/changes-branches/18/"
                    },
                    {
                      "_class": "hudson.model.FreeStyleBuild",
                      "number": 17,
                      "url": "http://jenkins.example.com/job/changes-branches/17/"
                    }
                  ]
                }
            "#)
            .create();

        let jobs = get_jobs("changes".to_string());

        assert_eq!(
            jobs,
            [
                "http://jenkins.example.com/job/changes-branches/18/",
                "http://jenkins.example.com/job/changes-branches/17/"
            ]
        );
    }

    #[test]
    fn request_job_queries_a_job_from_the_jenkins_api() {
        let mock = mock("GET", "/job/changes-branches/15/api/json")
            .with_status(200)
            .with_header("content-type", "application/json;charset=utf-8")
            .with_body(r#"
                {
                  "displayName": "2388-delete-the-codes-391af",
                  "result": "SUCCESS"
                }
            "#)
            .create();

        let job = request_job("http://jenkins.example.com/job/changes-branches/17".to_string());

        let expected = Job {
            display_name: "2388-delete-the-codes-391af".to_string(),
            result: JobStatus::Success,
        };

        assert_eq!(job.display_name, expected.display_name);
        assert_eq!(job.result, expected.result);
    }

    #[test]
    fn job_for_commit_returns_true_when_commit_matches_job() {
        let job = Job {
            display_name: "1753-fix-everything-b4a28".to_string(),
            result: JobStatus::Pending,
        };

        let commit_ref = CommitRef {
            repo: "vivid-system".to_string(),
            sha: "b4a286e67d8f5eaf1f6ba5c57fc3c7d91ac0fd1c".to_string(),
            branch: "1753-fix-everything".to_string(),
        };

        assert_eq!(job_for_commit(job, commit_ref), true);
    }

    #[test]
    fn job_for_commit_returns_false_when_commit_doesnt_match_job() {
        let job = Job {
            display_name: "5234-eliminate-widgetmacallit-5a28c".to_string(),
            result: JobStatus::Success,
        };

        let commit_ref = CommitRef {
            repo: "vivid-system".to_string(),
            sha: "b4a286e67d8f5eaf1f6ba5c57fc3c7d91ac0fd1c".to_string(),
            branch: "1753-fix-everything".to_string(),
        };

        assert_eq!(job_for_commit(job, commit_ref), false);
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
