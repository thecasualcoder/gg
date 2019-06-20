use walkdir::WalkDir;
use std::path::PathBuf;

fn main() {
    let mut git_dirs: Vec<_> = vec![];
    for entry in WalkDir::new("/Users/ninanjohn/Trials")
        .follow_links(true)
        .contents_first(true)
        .same_file_system(true)
        {
            let entry = entry.unwrap();
            if entry.file_name().eq(".git") {
                git_dirs.push(entry);
                continue;
            }
        }
    println!("{:#?}", git_dirs);
}
