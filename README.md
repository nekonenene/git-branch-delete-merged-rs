# git-branch-delete-merged [Rust Edition]

This CLI app deletes local branches that have been merged **also "[Squash and merge](https://docs.github.com/en/pull-requests/collaborating-with-pull-requests/incorporating-changes-from-a-pull-request/about-pull-request-merges#squash-and-merge-your-commits)"**.

It is based on https://github.com/nekonenene/git-branch-delete-merged written in the Go language, and rewritten in the Rust language.


## Installation

### Using Homebrew:

macOS or Linux can install through [Homebrew](https://brew.sh).

```sh
# Install
brew install nekonenene/tap/git-branch-delete-merged

# Update
brew update && brew upgrade git-branch-delete-merged

# Uninstall
brew remove git-branch-delete-merged
```

### Using binary:

macOS or Linux can do the following.

```sh
# Install / Update
curl -fL https://github.com/nekonenene/git-branch-delete-merged-rs/releases/latest/download/git-branch-delete-merged_$(uname -s)_$(uname -m) -o git-branch-delete-merged && chmod +x git-branch-delete-merged
mv git-branch-delete-merged /usr/local/bin

# Uninstall
rm /usr/local/bin/git-branch-delete-merged
```

### Using cargo:

After [installing cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html),

```sh
# Install
cargo install --git https://github.com/nekonenene/git-branch-delete-merged-rs

# Update
cargo install --force --git https://github.com/nekonenene/git-branch-delete-merged-rs

# Uninstall
cargo uninstall git-branch-delete-merged
```


## Usage

### General usage

If you want to delete a branch that has merged into the `main` branch:

```sh
git-branch-delete-merged main
```

And if the branch to delete exists, you will get a prompt like this:

```
Target branches: [dev1]

Are you sure to delete 'dev1' branch? [y|n|l|d|q|help]:
```

Please type one and press enter.

* `y`: Yes, delete the branch
* `n`: No, skip deleting
* `l`: Show git logs of the branch
* `d`: Show the latest commit of the branch and its diff
* `q`: Quit immediately
* `h`: Show help

### Skip prompt

If you want to delete all merged branches without confirmation, `--yes` option will be useful.

```sh
git-branch-delete-merged main --yes
```


## Thank you

[not-an-aardvark/git-delete-squashed](https://github.com/not-an-aardvark/git-delete-squashed) is the reference code that helps finding branches which has squashed and merged.
