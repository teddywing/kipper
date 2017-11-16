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
extern crate url;

use std::error::Error;
use std::thread::sleep;
use std::time::{Duration, Instant};

use self::reqwest::header;
use self::url::Url;

use af83;
use github;
use pull_request::CommitRef;

#[derive(Debug, PartialEq, Eq)]
pub enum JobStatus {
    Success,
    Failure,
    Pending,
    Unknown,
}

impl JobStatus {
    fn commit_status(&self) -> github::CommitStatus {
        match *self {
            JobStatus::Success => github::CommitStatus::Success,
            JobStatus::Failure => github::CommitStatus::Failure,
            JobStatus::Pending => github::CommitStatus::Pending,
            JobStatus::Unknown => github::CommitStatus::Error,
        }
    }
}

pub struct Job {
    display_name: String,
    result: JobStatus,
}

impl Job {
    fn new(payload: String) -> Result<Job, Box<Error>> {
        let mut job = json::parse(payload.as_ref())?;

        Ok(
            Job {
                display_name: job["displayName"].take_string().unwrap_or_default(),
                result: result_from_job(job["result"].take_string()),
            }
        )
    }
}

pub fn find_and_track_build_and_update_status(
    commit_ref: CommitRef,
    jenkins_url: String,
    jenkins_user_id: &String,
    jenkins_token: &String,
    github_token: String,
) -> Result<(), Box<Error>> {
    let jenkins_client = jenkins_request_client(
        &jenkins_user_id,
        &jenkins_token
    )?;

    let jobs = get_jobs(
        &jenkins_url,
        &jenkins_client,
        commit_ref.repo.as_ref()
    )?;
    let t20_minutes = 60 * 20;

    for job_url in jobs {
        debug!("Looking for job: {}", job_url);

        let mut job = request_job(
            &jenkins_url,
            &jenkins_client,
            job_url.as_ref()
        )?;

        // Does `displayName` match
        if job_for_commit(&job, &commit_ref) {
            debug!("Job found: {}", job_url);

            // Start timer
            let now = Instant::now();

            let commit_status = job.result.commit_status();

            github::update_commit_status(
                &github_token,
                &commit_ref,
                &commit_status,
                job_url.clone(),
                None,
                "continuous-integration/jenkins".to_owned()
            ).expect(
                format!(
                    "GitHub pending status update failed for {}/{} {}.",
                    commit_ref.owner,
                    commit_ref.repo,
                    commit_ref.sha
                ).as_ref()
            );

            while job.result == JobStatus::Pending {
                // loop
                // if timer > 20 minutes
                //   call github::update_commit_status with timeout error
                //   return
                // wait 30 seconds
                // call request_job again
                // if the status is different
                //   call github::update_commit_status
                //   stop

                debug!("Waiting for job to finish");

                if now.elapsed().as_secs() == t20_minutes {
                    github::update_commit_status(
                        &github_token,
                        &commit_ref,
                        &github::CommitStatus::Error,
                        job_url.clone(),
                        Some("The status checker timed out.".to_owned()),
                        "continuous-integration/jenkins".to_owned()
                    ).expect(
                        format!(
                            "GitHub timeout error status update failed for {}/{} {}.",
                            commit_ref.owner,
                            commit_ref.repo,
                            commit_ref.sha
                        ).as_ref()
                    );

                    return Ok(())
                }

                sleep(Duration::from_secs(30));

                let updated_job = request_job(
                    &jenkins_url,
                    &jenkins_client,
                    job_url.as_ref()
                ).expect(
                    format!("Failed to request job '{}'.", job_url).as_ref()
                );

                if job.result != updated_job.result {
                    github::update_commit_status(
                        &github_token,
                        &commit_ref,
                        &updated_job.result.commit_status(),
                        job_url.clone(),
                        None,
                        "continuous-integration/jenkins".to_owned()
                    ).expect(
                        format!(
                            "GitHub status update failed for {}/{} {}.",
                            commit_ref.owner,
                            commit_ref.repo,
                            commit_ref.sha
                        ).as_ref()
                    );

                    return Ok(())
                }

                job = updated_job;
            }

            return Ok(())
        }
    }

    Ok(())
}

pub fn auth_credentials(user_id: String, token: String) -> header::Basic {
    header::Basic {
        username: user_id,
        password: Some(token),
    }
}

pub fn get_jobs(
    jenkins_url: &String,
    client: &reqwest::Client,
    repo_name: &str
) -> Result<Vec<String>, Box<Error>> {
    let mut response = client.get(
        &format!("{}/job/{}-branches/api/json", jenkins_url, repo_name)
    ).send()?;

    let body = response.text()?;

    let jobs = json::parse(body.as_ref())?;

    Ok(
        jobs["builds"].members()
            .map(|job| {
                job["url"].clone().take_string().unwrap_or_default()
            })
            .collect::<Vec<String>>()
    )
}

pub fn request_job(
    jenkins_url: &String,
    client: &reqwest::Client,
    url: &str
) -> Result<Job, Box<Error>> {
    let url = Url::parse(url.as_ref())?;

    let mut response = client.get(
        &format!("{}{}/api/json", jenkins_url, url.path())
    ).send()?;

    let body = response.text()?;

    let job = Job::new(body)?;

    Ok(job)
}

// Does the `commit_ref` correspond to the job?
pub fn job_for_commit(job: &Job, commit_ref: &CommitRef) -> bool {
    job.display_name == af83::job_name(&commit_ref)
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


fn jenkins_request_client(user_id: &String, token: &String) -> Result<reqwest::Client, Box<Error>> {
    let credentials = auth_credentials(user_id.to_owned(), token.to_owned());

    let mut headers = header::Headers::new();
    headers.set(header::Authorization(credentials));

    let client = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    Ok(client)
}


#[cfg(test)]
mod tests {
    use self::mockito::mock;

    use super::*;

    fn test_request_client() -> reqwest::Client {
        jenkins_request_client(
            &"username".to_owned(),
            &"token".to_owned()
        ).expect("Failed to build Jenkins request client")
    }

    #[test]
    fn job_new_creates_a_job_from_payload() {
        let payload = r#"{
            "displayName": "3296-fix-typo-700d0",
            "result": "SUCCESS"
        }"#.to_owned();

        let job = Job::new(payload).expect("Failed to create job from payload");

        assert_eq!(job.display_name, "3296-fix-typo-700d0");
        assert_eq!(job.result, JobStatus::Success);
    }

    #[test]
    fn get_jobs_queries_jobs_from_jenkins_api() {
        let _mock = mock("GET", "/job/changes-branches/api/json")
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

        let jobs = get_jobs(
            &mockito::SERVER_URL.to_owned(),
            &test_request_client(),
            "changes"
        ).expect("Failed to request jobs");

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
        let _mock = mock("GET", "/job/changes-branches/15/api/json")
            .with_status(200)
            .with_header("content-type", "application/json;charset=utf-8")
            .with_body(r#"
                {
                  "displayName": "2388-delete-the-codes-391af",
                  "result": "SUCCESS"
                }
            "#)
            .create();

        let job = request_job(
            &mockito::SERVER_URL.to_owned(),
            &test_request_client(),
            "http://jenkins.example.com/job/changes-branches/15"
        ).expect("Failed to request job");

        let expected = Job {
            display_name: "2388-delete-the-codes-391af".to_owned(),
            result: JobStatus::Success,
        };

        assert_eq!(job.display_name, expected.display_name);
        assert_eq!(job.result, expected.result);
    }

    #[test]
    fn job_for_commit_returns_true_when_commit_matches_job() {
        let job = Job {
            display_name: "1753-fix-everything-b4a28".to_owned(),
            result: JobStatus::Pending,
        };

        let commit_ref = CommitRef {
            owner: "uso".to_owned(),
            repo: "vivid-system".to_owned(),
            sha: "b4a286e67d8f5eaf1f6ba5c57fc3c7d91ac0fd1c".to_owned(),
            branch: "1753-fix-everything".to_owned(),
        };

        assert_eq!(job_for_commit(&job, &commit_ref), true);
    }

    #[test]
    fn job_for_commit_returns_false_when_commit_doesnt_match_job() {
        let job = Job {
            display_name: "5234-eliminate-widgetmacallit-5a28c".to_owned(),
            result: JobStatus::Success,
        };

        let commit_ref = CommitRef {
            owner: "uso".to_owned(),
            repo: "vivid-system".to_owned(),
            sha: "b4a286e67d8f5eaf1f6ba5c57fc3c7d91ac0fd1c".to_owned(),
            branch: "1753-fix-everything".to_owned(),
        };

        assert_eq!(job_for_commit(&job, &commit_ref), false);
    }

    #[test]
    fn result_from_job_is_success() {
        assert_eq!(
            result_from_job(Some("SUCCESS".to_owned())),
            JobStatus::Success
        );
    }

    #[test]
    fn result_from_job_is_failure() {
        assert_eq!(
            result_from_job(Some("FAILURE".to_owned())),
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
