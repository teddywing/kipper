Kipper
======

A server that updates GitHub pull request commit statuses from Jenkins builds.


## Prerequisites

A Jenkins server configured with a "branches" project. This project will run a
build for all new commits pushed to a branch. The project's name should match
the GitHub repository name appended by "-branches". For example:

	kipper-branches

Each build name will have the following format:

	branch-name-5ac92

The branch name, followed by a hyphen "-", followed by the first five characters
of the commit SHA.


## Setup

Kipper runs a web server that listens for a GitHub webhook. Configure your
GitHub project with the following settings:

	Payload URL: http://example.com/github/pull_request_event
	Content type: application/json
	Events: Pull request
	Active: true

Since GitHub will be sending webhook requests to Kipper, it must be publicly
accessible.

To run Kipper, several configuration parameters must be passed in via command
line arguments in order for it to communicate with Jenkins and update commit
statuses on GitHub:

	./kipper \
		--jenkins-url 'http://jenkins.example.com' \
		--jenkins-user-id 'username' \
		--jenkins-token 'jenkins-token' \
		--github-token 'github-token'

By default, Kipper will run on port 8000.


## Install

A binary built for Mac OS X is available on the [releases][1] page. Download the
binary and put it in your `PATH`.

To compile from source:

	$ cargo install --git https://github.com/teddywing/kipper.git --root /usr/local


## Uninstall

	$ cargo uninstall --root /usr/local kipper


## License
Copyright © 2017 Teddy Wing. Licensed under the GNU GPLv3+ (see the included
COPYING file).


[1]: https://github.com/teddywing/kipper/releases
