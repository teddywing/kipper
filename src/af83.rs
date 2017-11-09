use pull_request::CommitRef;

pub fn job_name(commit_ref: CommitRef) -> String {
    let (sha, _) = commit_ref.sha.split_at(5);

    format!("{}-{}", commit_ref.branch, sha)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_name_is_branch_appended_by_commit_sha_prefix() {
        let commit_ref = CommitRef {
            owner: "sybil".to_string(),
            repo: "sybil-system".to_string(),
            sha: "159f8769b897ed7774700d0b2777def8ac838b8f".to_string(),
            branch: "5912-make-logo-bigger".to_string(),
        };

        assert_eq!(
            job_name(commit_ref),
            "5912-make-logo-bigger-159f8"
        );
    }
}
