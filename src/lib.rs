#![allow(non_snake_case)]
#![feature(async_closure)]
#![feature(external_doc)]
#![deny(missing_docs)]
#![doc(include = "../readme.md")]

mod parse;
mod urls;
/// Defines data structures of the information fetched from web-learning.
pub mod types;
/// Blocking version api, need `features = ["blocking"]` to enable.
#[cfg(feature = "blocking")]
pub mod blocking;

use reqwest::{Client, ClientBuilder, multipart::{Form, Part}};
use futures::future::{try_join3, try_join_all};
use crate::{parse::*, urls::*, types::*};

#[macro_use]
mod macros {
  // it cannot be a function, because the `Form` type in async api and blocking api of `reqwest` is different
  // it can be used in both submitting homework, or replying to discussion
  macro_rules! form_file {
    ($form: expr, $file: expr) => {
      if let Some((name, data)) = $file {
        $form.part("fileupload", Part::bytes(data).file_name(name.to_owned()))
      } else { $form.text("fileupload", "undefined") }
    };
  }

  // `a` for `async`, `b` for `blocking`
  macro_rules! check_success {
    (a, $req: expr, $msg: expr) => { if $req.send().await?.text().await?.contains("success") { Ok(()) } else { Err($msg.into()) } };
    (b, $req: expr, $msg: expr) => { if $req.send()?.text()?.contains("success") { Ok(()) } else { Err($msg.into()) } };
  }
}

/// The core struct type, representing a login session to web-learning.
///
/// It is only a simple wrapper of `reqwest::Client`, and it is also a public field,
/// because I don't care about user modifying it, or create a `LearnHelper` instance through `LearnHelper(...)`.
/// After all they will have to pay a price (getting `Err` result) if their action is not proper.
pub struct LearnHelper(pub Client);

// compiler requires type annotation in async closure, so extract them here
const OK: Result<()> = Ok(());

impl LearnHelper {
  /// Do login with the given `username` and `password`.
  ///
  /// If you want to create a `LearnHelper` instance with other configuration, you can simply use `LearnHelper(...)` to construct one.
  ///
  /// If the `username` or `password` is wrong, it will generally result in an `Err`.
  pub async fn login(username: &str, password: &str) -> Result<Self> {
    let client = ClientBuilder::new().cookie_store(true).user_agent(USER_AGENT).build()?;
    let params = [("i_user", username), ("i_pass", password), ("atOnce", "true")];
    let res = client.post(LOGIN).form(&params).send().await?.text().await?;
    let ticket_start = res.find("ticket=").ok_or("failed to login")? + 7; // 7 == "ticket=".len()
    let ticket_len = res[ticket_start..].find("\"").ok_or("failed to login")?;
    client.post(&AUTH_ROAM(&res[ticket_start..ticket_start + ticket_len])).send().await?;
    Ok(Self(client))
  }

  /// Logout from web-learning, and end the login session, consuming `self`.
  ///
  /// You may logout if you wish, and it is not necessary.
  pub async fn logout(self) -> Result<()> {
    self.0.post(LOGOUT).send().await?;
    Ok(())
  }

  /// Return a list of semester ids of this student. These ids will later be referred to as `semester: IdRef`.
  ///
  /// A semester id has the form of "year1-year2-[1/2/3]", where `1` means fall, `2` means spring, `3` means summer.
  /// This is define by constants `SEMESTER_FALL`, `SEMESTER_SPRING`, `SEMESTER_SUMMER`.
  pub async fn semester_id_list(&self) -> Result<Vec<Id>> {
    let res = self.0.get(SEMESTER_LIST).send().await?.json::<Vec<Option<String>>>().await?;
    Ok(res.into_iter().filter_map(|x| x).collect()) // there is `null` in response
  }

  /// Return a list of courses of a given semester. Parameter `semester` refers to the return value of `semester_id_list`.
  pub async fn course_list(&self, semester: IdRef<'_>) -> Result<Vec<Course>> {
    let mut res = self.0.get(&COURSE_LIST(semester)).send().await?.json::<JsonWrapper1<Course>>().await?.resultList;
    try_join_all(res.iter_mut().map(async move |x| {
      x.time_location = self.0.get(&COURSE_TIME_LOCATION(&x.id)).send().await?.json().await?;
      OK
    })).await?;
    Ok(res)
  }

  /// Return a list of discussions of a given course. Parameter `course` refers to `Course::id`.
  pub async fn notification_list(&self, course: IdRef<'_>) -> Result<Vec<Notification>> {
    let mut res = self.0.get(&NOTIFICATION_LIST(course)).send().await?.json::<JsonWrapper2<JsonWrapper20<Notification>>>().await?.object.aaData;
    try_join_all(res.iter_mut().map(async move |x| {
      x.attachment_url = if x.attachment_name.is_some() {
        const MSG: &str = "invalid notification attachment format";
        let res = self.0.get(&NOTIFICATION_DETAIL(&x.id, course)).send().await?.text().await?;
        let href_end = res.find("\" class=\"ml-10\"").ok_or(MSG)?;
        let href_start = res[..href_end].rfind("a href=\"").ok_or(MSG)? + 8;
        Some(PREFIX.to_owned() + &res[href_start..href_end])
      } else { None };
      OK
    })).await?;
    Ok(res)
  }

  /// Return a list of files of a given course. Parameter `course` refers to `Course::id`.
  pub async fn file_list(&self, course: IdRef<'_>) -> Result<Vec<File>> {
    Ok(self.0.get(&FILE_LIST(course)).send().await?.json::<JsonWrapper2<Vec<File>>>().await?.object)
  }

  /// Return a list of homework assignments of a given course. Parameter `course` refers to `Course::id`.
  pub async fn homework_list(&self, course: IdRef<'_>) -> Result<Vec<Homework>> {
    let f = async move |f: fn(&str) -> String| {
      let mut res = self.0.get(&f(course)).send().await?.json::<JsonWrapper2<JsonWrapper20<Homework>>>().await?.object.aaData;
      try_join_all(res.iter_mut().map(async move |x| {
        let res = self.0.get(&x.url()).send().await?.text().await?;
        x.detail = parse_homework_detail(&res).ok_or("invalid homework detail format")?;
        OK
      })).await?;
      Ok::<_, Error>(res)
    };
    let (mut res, mut h1, mut h2) = try_join3(f(HOMEWORK_LIST_ALL[0]), f(HOMEWORK_LIST_ALL[1]), f(HOMEWORK_LIST_ALL[2])).await?;
    res.reserve(h1.len() + h2.len());
    res.append(&mut h1);
    res.append(&mut h2);
    Ok(res)
  }

  /// Submitting homework to a given homework assignment.
  /// - Parameter `student_homework` refers to `Homework::student_homework_id`.
  /// - Parameter `content` is the content of your submission.
  /// - Parameter `file` is `(file name, file content)` when it exists. File name is only used in
  ///   web-learning, this function won't perform file reading.
  pub async fn submit_homework(&self, student_homework: IdRef<'_>, content: String, file: Option<(&str, Vec<u8>)>) -> Result<()> {
    // the performance loss caused by defining parameters as IdRef instead of impl Into<Cow<'static, str>> is negligible
    // however giving them type IdRef makes the api much clearer
    let form = Form::new().text("zynr", content).text("xszyid", student_homework.to_owned()).text("isDeleted", "0");
    let form = form_file!(form, file);
    check_success!(a, self.0.post(HOMEWORK_SUBMIT).multipart(form), "failed to submit homework")
  }

  /// Return a list of discussions of a given course. Parameter `course` refers to `Course::id`.
  pub async fn discussion_list(&self, course: IdRef<'_>) -> Result<Vec<Discussion>> {
    Ok(self.0.get(&DISCUSSION_LIST(course)).send().await?.json::<JsonWrapper2<JsonWrapper21<_>>>().await?.object.resultsList)
  }

  /// Return a list of discussion replies of a given discussion.
  /// - Parameter `course` refers to `Course::id`.
  /// - Parameter `discussion` refers to `Discussion::id`.
  /// - Parameter `discussion_board` refers to `Discussion::board_id`.
  pub async fn discussion_replies(&self, course: IdRef<'_>, discussion: IdRef<'_>, discussion_board: IdRef<'_>) -> Result<Vec<DiscussionReply>> {
    let res = self.0.get(&DISCUSSION_REPLIES(course, discussion, discussion_board)).send().await?.text().await?;
    parse_discussion_replies(&res).ok_or("invalid discussion replies format".into())
  }

  /// Reply to a given discussion.
  /// - Parameter `course` refers to `Course::id`.
  /// - Parameter `discussion` refers to `Discussion::id`.
  /// - Parameter `content` is the content of your reply.
  /// - Parameter `respondent_reply`: when exists, it refers to `DiscussionReply0::id`, meaning that you are replying to this reply.
  /// When doesn't exist, it means append a reply to the discussion.
  /// - Parameter `file`: has the same semantics as the parameter `file` in `submit_homework`.
  pub async fn reply_discussion(&self, course: IdRef<'_>, discussion: IdRef<'_>, content: String, respondent_reply: Option<IdRef<'_>>, file: Option<(&str, Vec<u8>)>) -> Result<()> {
    let form = Form::new().text("wlkcid", course.to_owned()).text("tltid", discussion.to_owned()).text("nr", content.to_owned());
    let form = form_file!(form, file);
    let form = if let Some(x) = respondent_reply { form.text("fhhid", x.to_owned()).text("_fhhid", x.to_owned()) } else { form };
    check_success!(a, self.0.post(REPLY_DISCUSSION).multipart(form), "failed to reply discussion")
  }

  /// Deleting a given discussion reply.
  /// - Parameter `course` refers to `Course::id`.
  /// - Parameter `reply` refers to `DiscussionReply0::id`.
  /// 
  /// Trying to delete a reply not published by yourself will generally result in an `Err`.
  pub async fn delete_discussion_reply(&self, course: IdRef<'_>, reply: IdRef<'_>) -> Result<()> {
    check_success!(a, self.0.post(&DELETE_DISCUSSION_REPLY(course, reply)), "failed to delete discussion reply")
  }
}