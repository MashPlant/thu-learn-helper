use reqwest::{blocking::{Client, ClientBuilder, multipart::{Form, Part}}};
use crate::{DELETE_DR_TIMEOUT, check_delete_dr_success};
use crate::{parse::*, urls::*, types::*};

/// Same as `crate::LearnHelper`, except that it is a blocking api.
pub struct LearnHelper(pub Client);

impl LearnHelper {
  /// Same as `crate::LearnHelper::login`, except that it is a blocking api.
  pub fn login(username: &str, password: &str) -> Result<Self> {
    let client = ClientBuilder::new().cookie_store(true).user_agent(USER_AGENT).build()?;
    let params = [("i_user", username), ("i_pass", password), ("atOnce", "true")];
    let res = client.post(LOGIN).form(&params).send()?.text()?;
    let ticket_start = res.find("ticket=").ok_or("failed to login")? + 7;
    let ticket_len = res[ticket_start..].find("\"").ok_or("failed to login")?;
    client.post(&AUTH_ROAM(&res[ticket_start..ticket_start + ticket_len])).send()?;
    Ok(Self(client))
  }

  /// Same as `crate::LearnHelper::logout`, except that it is a blocking api.
  pub fn logout(self) -> Result<()> {
    self.0.post(LOGOUT).send()?;
    Ok(())
  }

  /// Same as `crate::LearnHelper::semester_id_list`, except that it is a blocking api.
  pub fn semester_id_list(&self) -> Result<Vec<Id>> {
    let res = self.0.get(SEMESTER_LIST).send()?.json::<Vec<Option<String>>>()?;
    Ok(res.into_iter().filter_map(|x| x).collect())
  }

  /// Same as `crate::LearnHelper::course_list`, except that it is a blocking api.
  pub fn course_list(&self, semester: IdRef) -> Result<Vec<Course>> {
    let mut res = self.0.get(&COURSE_LIST(semester)).send()?.json::<JsonWrapper1<Course>>()?.resultList;
    for x in &mut res {
      x.time_location = self.0.get(&COURSE_TIME_LOCATION(&x.id)).send()?.json()?;
    }
    Ok(res)
  }

  /// Same as `crate::LearnHelper::notification_list`, except that it is a blocking api.
  pub fn notification_list(&self, course: IdRef) -> Result<Vec<Notification>> {
    let mut res = self.0.get(&NOTIFICATION_LIST(course)).send()?.json::<JsonWrapper2<JsonWrapper20<Notification>>>()?.object.aaData;
    for x in &mut res {
      x.attachment_url = if x.attachment_name.is_some() {
        const MSG: &str = "invalid notification attachment format";
        let res = self.0.get(&NOTIFICATION_DETAIL(&x.id, course)).send()?.text()?;
        let href_end = res.find("\" class=\"ml-10\"").ok_or(MSG)?;
        let href_start = res[..href_end].rfind("a href=\"").ok_or(MSG)? + 8;
        Some(PREFIX.to_owned() + &res[href_start..href_end])
      } else { None };
    }
    Ok(res)
  }

  /// Same as `crate::LearnHelper::file_list`, except that it is a blocking api.
  pub fn file_list(&self, course: IdRef) -> Result<Vec<File>> {
    Ok(self.0.get(&FILE_LIST(course)).send()?.json::<JsonWrapper2<Vec<File>>>()?.object)
  }

  /// Same as `crate::LearnHelper::homework_list`, except that it is a blocking api.
  pub fn homework_list(&self, course: IdRef) -> Result<Vec<Homework>> {
    let mut ret = Vec::new();
    for f in &HOMEWORK_LIST_ALL {
      let mut res = self.0.get(&f(course)).send()?.json::<JsonWrapper2<JsonWrapper20<Homework>>>()?.object.aaData;
      for x in &mut res {
        let res = self.0.get(&x.url()).send()?.text()?;
        x.detail = parse_homework_detail(&res).ok_or("invalid homework detail format")?;
      }
      ret.append(&mut res);
    }
    Ok(ret)
  }

  /// Same as `crate::LearnHelper::submit_homework`, except that it is a blocking api.
  pub fn submit_homework(&self, student_homework: IdRef, content: String, file: Option<(&str, Vec<u8>)>) -> Result<()> {
    let form = Form::new().text("zynr", content).text("xszyid", student_homework.to_owned()).text("isDeleted", "0");
    let form = form_file!(form, file);
    check_success!(b,  self.0.post(HOMEWORK_SUBMIT).multipart(form), "failed to submit homework")
  }

  /// Same as `crate::LearnHelper::discussion_list`, except that it is a blocking api.
  pub fn discussion_list(&self, course: IdRef) -> Result<Vec<Discussion>> {
    Ok(self.0.get(&DISCUSSION_LIST(course)).send()?.json::<JsonWrapper2<JsonWrapper21<_>>>()?.object.resultsList)
  }

  /// Same as `crate::LearnHelper::discussion_replies`, except that it is a blocking api.
  pub fn discussion_replies(&self, course: IdRef, discussion: IdRef, discussion_board: IdRef) -> Result<Vec<DiscussionReply>> {
    let res = self.0.get(&DISCUSSION_REPLIES(course, discussion, discussion_board)).send()?.text()?;
    parse_discussion_replies(&res).ok_or("invalid discussion replies format".into())
  }

  /// Same as `crate::LearnHelper::reply_discussion`, except that it is a blocking api.
  pub fn reply_discussion(&self, course: IdRef, discussion: IdRef, content: String, respondent_reply: Option<IdRef<'_>>, file: Option<(&str, Vec<u8>)>) -> Result<()> {
    let form = Form::new().text("wlkcid", course.to_owned()).text("tltid", discussion.to_owned()).text("nr", content.to_owned());
    let form = form_file!(form, file);
    let form = if let Some(x) = respondent_reply { form.text("fhhid", x.to_owned()).text("_fhhid", x.to_owned()) } else { form };
    check_success!(b, self.0.post(REPLY_DISCUSSION).multipart(form), "failed to reply discussion")
  }

  /// Same as `crate::LearnHelper::delete_discussion_reply`, except that it is a blocking api.
  pub fn delete_discussion_reply(&self, course: IdRef, reply: IdRef) -> Result<()> {
    check_delete_dr_success(
      self.0.post(&DELETE_DISCUSSION_REPLY(course, reply)).timeout(DELETE_DR_TIMEOUT).send().and_then(|r| r.text()))
  }
}