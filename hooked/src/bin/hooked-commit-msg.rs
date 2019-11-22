use shared::find_root;
use std::{
  env::args,
  error::Error,
  fs,
  process,
};
use unicode_segmentation::UnicodeSegmentation;

fn main() {
  if let Err(e) = || -> Result<(), Box<dyn Error>> {
    let path = find_root()?.join(args().last().ok_or_else(|| {
      "Expected to be passed a path to the git commit message"
    })?);

    let file = fs::read_to_string(path)?;
    let mut lines = file.lines();

    if let Some(ref first_line) = lines.next() {
      let length = first_line.graphemes(true).count();
      if length > 50 {
        return Err(
          "Your commit header is over 50 characters (i.e. graphemes) in length.\n\
           Commit messages titles should be between 10 to 50 characters".into());
      }
      if length < 10 {
        return Err(
          "Your commit header is less than 10 characters (i.e. graphemes) in length.\n\
           Commit messages titles should be between 10 to 50 characters".into());
      }
    }

    for line in lines {
      let length = line.graphemes(true).count();
      if length > 72 {
        return Err(
          "One of the lines in the body of the commit is over 72 characters (i.e. graphemes) in \n\
           length. Commit messages titles should be between 10 to 50 characters".into());
      }
    }

    Ok(())
  }() {
    eprintln!("{}", e);
    process::exit(1);
  }
}
