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

use pull_request::CommitRef;

pub fn job_name(commit_ref: &CommitRef) -> String {
    let (sha, _) = commit_ref.sha.split_at(5);

    format!("{}-{}", commit_ref.branch, sha)
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn job_name_is_branch_appended_by_commit_sha_prefix() {
        let commit_ref = CommitRef {
            owner: "sybil".to_owned(),
            repo: "sybil-system".to_owned(),
            sha: "159f8769b897ed7774700d0b2777def8ac838b8f".to_owned(),
            branch: "5912-make-logo-bigger".to_owned(),
        };

        assert_eq!(
            job_name(&commit_ref),
            "5912-make-logo-bigger-159f8"
        );
    }
}
