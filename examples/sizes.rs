#![deny(warnings)]

// use git2::Error;
use git2::Repository;
use git2::{ObjectType, TreeWalkMode, TreeWalkResult};
use std::env;
use std::str;
use structopt::StructOpt;

const DEFAULT_MAX_OBJECT_SIZE: usize = 26214400;
static MAX_OBJECT_SIZE_VAR: &str = "MAX_OBJECT_SIZE";
static ZERO_COMMIT: &str = "0000000000000000000000000000000000000000";

#[derive(StructOpt)]
struct Args {
    /// alternative git directory to use
    #[structopt(name = "dir", long = "git-dir")]
    flag_git_dir: Option<String>,
    #[structopt(name = "oldrev")]
    oldrev: String,
    #[structopt(name = "newrev")]
    newrev: String,
    #[structopt(name = "refname")]
    refname: String,
}

fn validate(args: &Args) -> Result<(), Box<dyn std::error::Error>> {
    let max_object_size = match env::var(MAX_OBJECT_SIZE_VAR) {
        Ok(val) => val.parse::<usize>()?,
        Err(_) => DEFAULT_MAX_OBJECT_SIZE,
    };

    if args.newrev == ZERO_COMMIT {
        return Ok(());
    }

    let path = args.flag_git_dir.as_ref().map(|s| &s[..]).unwrap_or(".");
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;

    let revspec = if args.oldrev == ZERO_COMMIT {
        repo.revparse(&args.newrev)?
    } else {
        repo.revparse(&format!("{}..{}", &args.newrev, &args.oldrev))?
    };

    if revspec.mode().contains(git2::RevparseMode::SINGLE) {
        revwalk.push(revspec.from().unwrap().id())?;
    } else {
        let from = revspec.from().unwrap().id();
        let to = revspec.to().unwrap().id();
        revwalk.push_range(&format!("{}..{}", from, to))?;
    }

    for commit in revwalk {
        let commit_id = match commit {
            Ok(c) => c,
            Err(e) => return Err(Box::new(e)),
        };
        let commit_object = repo.find_commit(commit_id).unwrap();
        let tid = commit_object.tree_id();
        let tree = repo.find_tree(tid).unwrap();
        tree.walk(TreeWalkMode::PreOrder, |_, entry| {
            let name = match entry.name() {
                Some(s) => s,
                None => "n/a",
            };
            let size = match entry.kind() {
                Some(k) => match k {
                    ObjectType::Blob => {
                        let id = entry.id();
                        let blob = repo.find_blob(id).unwrap();
                        blob.size()
                    }
                    _ => 0,
                },
                None => 0,
            };
            if size > max_object_size {
                println!(
                    "{} in {} has size {}, bigger than {}",
                    name, args.refname, size, max_object_size
                );
                std::process::exit(1);
            }
            TreeWalkResult::Ok
        })
        .unwrap();
    }

    Ok(())
}

fn main() {
    let args = Args::from_args();
    match validate(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}
