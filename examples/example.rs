use std::io::{self, BufRead, Write};
use thu_learn_helper::LearnHelper;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
  let (mut username, mut password) = (String::new(), String::new());
  let (stdin, stdout) = (io::stdin(), io::stdout());
  let (mut stdin, mut stdout) = (stdin.lock(), stdout.lock());
  stdout.write_all("Username:".as_bytes())?;
  stdout.flush()?;
  stdin.read_line(&mut username)?;
  stdout.write_all("Password:".as_bytes())?;
  stdout.flush()?;
  stdin.read_line(&mut password)?;
  let t = LearnHelper::login(username.trim(), password.trim()).await?;
  let ss = t.semester_id_list().await?;
  let cs = t.course_list(&ss[0]).await?;
  println!("{:#?}", cs);
  t.logout().await?;
  Ok(())
}