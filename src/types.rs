use chrono::NaiveDateTime;
use serde::Deserialize;
use derive_more::{From, Deref, DerefMut};
use std::fmt;
use crate::{parse::*, urls::*};

/// The errors that may occur when communicating with web-learning.
///
/// There is no essential difference between the sources of these two variants,
/// the only difference is whether `reqwest` reports the error or my program does.
#[derive(Debug, From)]
pub enum Error {
  /// `reqwest` reports this error.
  Network(reqwest::Error),
  /// Subsequent handling reports this error.
  Message(&'static str),
}

impl fmt::Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
    match self {
      Error::Network(e) => write!(f, "network error: {}", e),
      Error::Message(m) => write!(f, "error: {}", m),
    }
  }
}

impl std::error::Error for Error {}

/// A `Result` alias where the `Err` case is `crate::Error`.
pub type Result<T> = std::result::Result<T, Error>;

/// The owned `Id` type.
pub type Id = String;
/// The borrowed `Id` type.
pub type IdRef<'a> = &'a str;

/// Constant id for fall semester. Please refer to `LearnHelper::semester_id_list`.
pub const SEMESTER_FALL: u32 = 1;
/// Constant id for spring semester. Please refer to `LearnHelper::semester_id_list`.
pub const SEMESTER_SPRING: u32 = 2;
/// Constant id for summer semester. Please refer to `LearnHelper::semester_id_list`.
pub const SEMESTER_SUMMER: u32 = 3;

/// Define the information of a course returned by web-learning.
#[derive(Debug, Deserialize)]
pub struct Course {
  /// Used in parameters of `LearnHelper`, referred to as `course: IdRef`.
  #[serde(rename = "wlkcid")] pub id: Id,
  /// The chinese name of this course, for example, "编译原理".
  #[serde(rename = "kcm")] pub name: String,
  /// The english name of this course, for example, "Principles and Practice of Compiler Construction".
  #[serde(rename = "ywkcm")] pub english_name: String,
  /// The name of the teacher of the course.
  #[serde(rename = "jsm")] pub teacher_name: String,
  /// `teacher_number` and `course_number` are normally string representation of an integer, but there are a few cases that they are not.
  #[serde(rename = "jsh")] pub teacher_number: String,
  /// Normally referred to as "课程号".
  #[serde(rename = "kch")] pub course_number: String,
  /// Normally referred to as "课序号".
  #[serde(rename = "kxh")] pub course_index: u32,
  /// The time and location that the course is held.
  /// All courses have at least one time and location, and some may have two or more.
  #[serde(skip)] pub time_location: Vec<String>,
}

impl Course {
  /// The homepage url of the course that you see in the browser.
  pub fn url(&self) -> String { COURSE_URL(&self.id) }
}

/// Define the information of a notification returned by web-learning.
#[derive(Debug, Deserialize)]
pub struct Notification {
  /// Used in parameters of `LearnHelper`, referred to as `course: IdRef`.
  #[serde(rename = "wlkcid")] pub course_id: Id,
  /// Used in parameters of `LearnHelper`, referred to as `notification: IdRef`.
  #[serde(rename = "ggid")] pub id: Id,
  /// The title of the notification.
  #[serde(rename = "bt")] pub title: String,
  /// The content of the notification. It is a html string.
  #[serde(rename = "ggnr", deserialize_with = "base64_string")] pub content: String,
  /// Is this notification already read?
  #[serde(rename = "sfyd", deserialize_with = "str_to_bool1")] pub read: bool,
  /// Is this notification marked important by teacher?
  #[serde(rename = "sfqd", deserialize_with = "str_to_bool2")] pub important: bool,
  /// The publish time of the notification.
  #[serde(rename = "fbsjStr", deserialize_with = "date_time")] pub publish_time: NaiveDateTime,
  /// The publisher's name of the notification.
  #[serde(rename = "fbrxm")] pub publisher: String,
  /// When exists, it is the name of the attachment in the notification.
  #[serde(rename = "fjmc")] pub attachment_name: Option<String>,
  /// When exists, it is the url of the attachment in the notification.
  /// Its existence should be the same as `attachment_name`, but I can't guarantee that.
  #[serde(skip)] pub attachment_url: Option<String>,
}

impl Notification {
  /// The detail page url of the notification that you see in the browser.
  pub fn url(&self) -> String { NOTIFICATION_DETAIL(&self.id, &self.course_id) }
}

/// Define the information of a file returned by web-learning.
#[derive(Debug, Deserialize)]
pub struct File {
  /// Used in parameters of `LearnHelper`, referred to as `file: IdRef`.
  #[serde(rename = "wjid")] pub id: Id,
  /// The title (or you may prefer to call it "name") of the file.
  #[serde(rename = "bt")] pub title: String,
  /// The description of the file. It is a html string.
  #[serde(rename = "ms")] pub description: String,
  /// Size in bytes.
  #[serde(rename = "wjdx")] pub raw_size: u32,
  /// Size description, for example, "1M".
  #[serde(rename = "fileSize")] pub size: String,
  /// The time that the teacher uploaded this file.
  #[serde(rename = "scsj", deserialize_with = "date_time")] pub upload_time: NaiveDateTime,
  /// Is this file **not** already read?
  #[serde(rename = "isNew", deserialize_with = "int_to_bool")] pub new: bool,
  /// Is this file marked important by teacher?
  #[serde(rename = "sfqd", deserialize_with = "int_to_bool")] pub important: bool,
  /// The number of the students that have visited this file.
  #[serde(rename = "llcs")] pub visit_count: u32,
  /// The number of the students that have downloaded this file.
  #[serde(rename = "xzcs")] pub download_cunt: u32,
  /// Suffix name of the file, for example, "zip", "ppt".
  #[serde(rename = "wjlx")] pub file_type: String,
}

impl File {
  /// The url that starts download. You can feed it to `reqwest::Client` to download the file.
  pub fn download_url(&self) -> String { FILE_DOWNLOAD(&self.id) }
}

/// Define the information of a homework assignment returned by web-learning.
#[derive(Debug, Deserialize, Deref, DerefMut)]
pub struct Homework {
  /// Used in parameters of `LearnHelper`, referred to as `course: IdRef`.
  #[serde(rename = "wlkcid")] pub course_id: Id,
  /// Used in parameters of `LearnHelper`, referred to as `homework: IdRef`.
  #[serde(rename = "zyid")] pub id: Id,
  /// Used in parameters of `LearnHelper`, referred to as `student_homework: IdRef`.
  #[serde(rename = "xszyid")] pub student_homework_id: Id,
  /// The title (or you may prefer to call it "name") of the homework.
  #[serde(rename = "bt")] pub title: String,
  /// The time that the teacher published the homework.
  #[serde(rename = "kssjStr", deserialize_with = "date_time")] pub assign_time: NaiveDateTime,
  /// The time that the homework is due.
  #[serde(rename = "jzsjStr", deserialize_with = "date_time")] pub deadline: NaiveDateTime,
  /// When exists (when the student has submitted the homework), it is the time that the student submitted the homework.
  #[serde(rename = "scsjStr", deserialize_with = "option_date_time")] pub submit_time: Option<NaiveDateTime>,
  /// When exists (when the student has submitted the homework), it is the content of the submitted homework.
  /// It is a html string.
  #[serde(rename = "zynrStr", deserialize_with = "nonempty_string")] pub submit_content: Option<String>,
  /// When exists (when the teacher has graded the homework), it is the grade that the student received.
  #[serde(rename = "cj")] pub grade: Option<f32>,
  /// When exists (when the teacher has graded the homework), it is the time that the teacher graded the homework.
  #[serde(rename = "pysjStr", deserialize_with = "option_date_time")] pub grade_time: Option<NaiveDateTime>,
  /// When exists (when the teacher has graded the homework), it is the name of the teacher that graded the homework.
  #[serde(rename = "jsm", deserialize_with = "nonempty_string")] pub grader_name: Option<String>,
  /// When exists (when the teacher has graded the homework), it is comment by the teacher in the grade.
  #[serde(rename = "pynr", deserialize_with = "nonempty_string")] pub grade_content: Option<String>,
  /// Some extra fields of the homework.
  #[serde(skip)]
  #[deref]
  #[deref_mut]
  pub detail: HomeworkDetail,
}

impl Homework {
  /// The detail page url of the homework that you see in the browser.
  pub fn url(&self) -> String { HOMEWORK_DETAIL(&self.course_id, &self.id, &self.student_homework_id) }

  /// The page that you click "submit homework" in browser.
  pub fn submit_page(&self) -> String { HOMEWORK_SUBMIT_PAGE(&self.course_id, &self.student_homework_id) }
}

/// It is always part of `Homework`, splitting it as a struct is only for convenience.
#[derive(Debug, Default)]
pub struct HomeworkDetail {
  /// The description of the homework. It is a html string.
  pub description: String,
  /// When exists, it is the `(name, url)` of the attachment of the homework.
  pub attachment_name_url: Option<(String, String)>,
  /// When exists, it is the `(name, url)` of the attachment of the submission of the homework.
  pub submit_attachment_name_url: Option<(String, String)>,
  /// When exists, it is the `(name, url)` of the attachment of the grade of the homework.
  pub grade_attachment_name_url: Option<(String, String)>,
}

/// Define the information of a discussion returned by web-learning.
#[derive(Debug, Deserialize)]
pub struct Discussion {
  /// Used in parameters of `LearnHelper`, referred to as `discussion: IdRef`.
  #[serde(rename = "id")] pub id: Id,
  /// Used in parameters of `LearnHelper`, referred to as `discussion_board: IdRef`.
  #[serde(rename = "bqid")] pub board_id: String,
  /// The title of the discussion.
  #[serde(rename = "bt")] pub title: String,
  /// The name of the people that published the discussion.
  /// The content he published is regarded as the first reply to this discussion.
  #[serde(rename = "fbrxm")] pub publisher_name: String,
  /// The publish time of the discussion.
  #[serde(rename = "fbsj", deserialize_with = "date_time1")] pub publish_time: NaiveDateTime,
  /// The name of the last replier to this discussion.
  #[serde(rename = "zhhfrxm", deserialize_with = "nonempty_string")] pub last_replier_name: Option<String>,
  /// The time that the last reply to this discussion was published.
  #[serde(rename = "zhhfsj", deserialize_with = "option_date_time1")] pub last_reply_time: Option<NaiveDateTime>,
  /// The number of the people that have visited this discussion.
  #[serde(rename = "djs")] pub visit_count: u32,
  /// The number of the people that have replied to this discussion.
  #[serde(rename = "hfcs")] pub reply_count: u32,
}

/// Define the prototype of a discussion reply. Parameter `R` means the type of sub-replies.
#[derive(Debug)]
pub struct DiscussionReply0<R> {
  /// When exists, it is used in parameters of `LearnHelper`, referred to as `reply: IdRef`.
  /// The first reply is publisher's content, and cannot be further replied, so it doesn't have an `id`.
  pub id: Option<String>,
  /// The author name of the reply.
  pub author: String,
  /// The publish time of the reply.
  pub publish_time: NaiveDateTime,
  /// The content of the reply. It is a html string.
  pub content: String,
  /// Sub-replies, `R` is `Vec<...>` if there is any, `()` if there is none
  pub replies: R,
}

/// The real discussion reply type in web-learning.
pub type DiscussionReply = DiscussionReply0<Vec<DiscussionReply0<()>>>;