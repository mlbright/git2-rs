#![deny(warnings)]

use git2::Error;
use git2::{Commit, Repository, Time};
use git2::{ObjectType, TreeWalkMode, TreeWalkResult};
use std::str;
use structopt::StructOpt;

const MAX_OBJECT_SIZE: usize = 18672;
static ZERO_COMMIT: &str = "0000000000000000000000000000000000000000";

#[derive(StructOpt)]
struct Args {
    #[structopt(name = "dir", long = "git-dir")]
    /// alternative git directory to use
    flag_git_dir: Option<String>,
    #[structopt(name = "oldrev")]
    oldrev: String,
    #[structopt(name = "newrev")]
    newrev: String,
    #[structopt(name = "refname")]
    refname: String,
}

fn run(args: &Args) -> Result<(), Error> {
    if args.newrev == ZERO_COMMIT {
        return Ok(());
    }

    let path = args.flag_git_dir.as_ref().map(|s| &s[..]).unwrap_or(".");
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;

    let revspec = repo.revparse(&format!("{}..{}", &args.newrev, &args.oldrev))?;
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
            Err(e) => return Err(e),
        };
        let commit_object = repo.find_commit(commit_id).unwrap();
        // print_commit(&commit_object);
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
            println!("{} {}", size, name);
            if size > MAX_OBJECT_SIZE {
                println!(
                    "{} in {} has size {}, bigger than {}",
                    name, args.refname, size, MAX_OBJECT_SIZE
                );
            }
            TreeWalkResult::Ok
        })
        .unwrap();
    }

    Ok(())
}

#[allow(dead_code)]
fn print_commit(commit: &Commit) {
    println!("commit {}", commit.id());

    if commit.parents().len() > 1 {
        print!("Merge:");
        for id in commit.parent_ids() {
            print!(" {:.8}", id);
        }
        println!();
    }

    let author = commit.author();
    println!("Author: {}", author);
    print_time(&author.when(), "Date:   ");
    println!();

    for line in String::from_utf8_lossy(commit.message_bytes()).lines() {
        println!("    {}", line);
    }
    println!();
}

fn print_time(time: &Time, prefix: &str) {
    let (offset, sign) = match time.offset_minutes() {
        n if n < 0 => (-n, '-'),
        n => (n, '+'),
    };
    let (hours, minutes) = (offset / 60, offset % 60);
    let ts = time::Timespec::new(time.seconds() + (time.offset_minutes() as i64) * 60, 0);
    let time = time::at(ts);

    println!(
        "{}{} {}{:02}{:02}",
        prefix,
        time.strftime("%a %b %e %T %Y").unwrap(),
        sign,
        hours,
        minutes
    );
}

fn main() {
    let args = Args::from_args();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}
