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

#[cfg(test)]
mod tests {
    // fn 
}
