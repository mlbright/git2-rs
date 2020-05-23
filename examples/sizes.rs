#![deny(warnings)]

use git2::Error;
use git2::{Commit, Repository, Time};
use git2::{ObjectType, TreeWalkMode, TreeWalkResult};
use std::str;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Args {
    #[structopt(name = "dir", long = "git-dir")]
    /// alternative git directory to use
    flag_git_dir: Option<String>,
}

fn run(args: &Args) -> Result<(), Error> {
    let path = args.flag_git_dir.as_ref().map(|s| &s[..]).unwrap_or(".");
    let repo = Repository::open(path)?;
    let mut revwalk = repo.revwalk()?;
    revwalk.set_sorting(git2::Sort::TIME)?;
    revwalk.push_head()?;
    for commit in revwalk {
        let commit_id = match commit {
            Ok(c) => c,
            Err(e) => return Err(e),
        };
        let commit_object = repo.find_commit(commit_id).unwrap();
        print_commit(&commit_object);
        let tid = commit_object.tree_id();
        let tree = repo.find_tree(tid).unwrap();
        tree.walk(TreeWalkMode::PreOrder, |_, entry| {
            match entry.name() {
                Some(s) => println!("{}", s),
                None => {}
            }
            match entry.kind() {
                Some(k) => match k {
                    ObjectType::Blob => {
                        let id = entry.id();
                        let blob = repo.find_blob(id).unwrap();
                        println!("{}", blob.size());
                        if blob.size() > 3244 {
                            return TreeWalkResult::Ok;
                        }
                    }
                    _ => {}
                },
                None => {}
            }
            TreeWalkResult::Ok
        })
        .unwrap();
    }

    Ok(())
}

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

impl Args {}

fn main() {
    let args = Args::from_args();
    match run(&args) {
        Ok(()) => {}
        Err(e) => println!("error: {}", e),
    }
}
