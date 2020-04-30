use chrono::NaiveDateTime;
use serde::{Deserialize, Deserializer, de::Error};
use select::{document::Document, node::Node, predicate::{Predicate, Class as C}};
use crate::{urls::*, types::HomeworkDetail};

#[derive(Deserialize)]
pub struct JsonWrapper1<T> { pub resultList: Vec<T> }

#[derive(Deserialize)]
pub struct JsonWrapper2<T> { pub object: T }

#[derive(Deserialize)]
pub struct JsonWrapper20<T> { pub aaData: Vec<T> }

#[derive(Deserialize)]
pub struct JsonWrapper21<T> { pub resultsList: Vec<T> }

impl HomeworkDetail {
  pub(crate) fn from_html(detail: &str) -> Option<Self> {
    let detail = Document::from(detail);
    let mut file_div = detail.find(C("list").and(C("fujian")).and(C("clearfix")));
    fn name_url(e: Option<Node>) -> Option<(String, String)> {
      for e in e?.find(C("ftitle")) {
        let e = e.children().nth(1)?;
        let name = e.children().next()?.as_text()?.to_owned();
        let href = e.attr("href")?;
        let url_start = href.find("downloadUrl=")? + 12;
        return Some((name, PREFIX.to_owned() + &href[url_start..]));
      }
      None
    }
    Some(HomeworkDetail {
      description: detail.find(C("list").and(C("calendar")).and(C("clearfix")).descendant(C("fl").and(C("right"))).descendant(C("c55")))
        .next()?.inner_html(),
      attachment_name_url: name_url(file_div.next()),
      submit_attachment_name_url: name_url(file_div.nth(1)),
      grade_attachment_name_url: name_url(file_div.next()),
    })
  }
}

pub fn date_time<'d, D>(d: D) -> Result<NaiveDateTime, D::Error> where D: Deserializer<'d> {
  NaiveDateTime::parse_from_str(<&str>::deserialize(d)?, "%Y-%m-%d %H:%M").map_err(Error::custom)
}

pub fn date_time1<'d, D>(d: D) -> Result<NaiveDateTime, D::Error> where D: Deserializer<'d> {
  NaiveDateTime::parse_from_str(<&str>::deserialize(d)?, "%Y-%m-%d %H:%M:%S").map_err(Error::custom)
}

// there is indeed some duplication, a better approach is to use the newtype pattern and define wrapper class for NaiveDateTime
// but that would involve more boilerplate code, and is harder to use
pub fn option_date_time<'d, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error> where D: Deserializer<'d> {
  match <Option<&str>>::deserialize(d)? {
    Some("") | None => Ok(None),
    Some(s) => NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M").map_err(Error::custom).map(Some)
  }
}

pub fn option_date_time1<'d, D>(d: D) -> Result<Option<NaiveDateTime>, D::Error> where D: Deserializer<'d> {
  match <Option<&str>>::deserialize(d)? {
    Some("") | None => Ok(None),
    Some(s) => NaiveDateTime::parse_from_str(s, "%Y-%m-%d %H:%M:%S").map_err(Error::custom).map(Some)
  }
}

pub fn str_to_bool1<'d, D>(d: D) -> Result<bool, D::Error> where D: Deserializer<'d> {
  Ok(<&str>::deserialize(d)? == "是")
}

pub fn str_to_bool2<'d, D>(d: D) -> Result<bool, D::Error> where D: Deserializer<'d> {
  Ok(<&str>::deserialize(d)? == "1")
}

pub fn base64_string<'d, D>(d: D) -> Result<String, D::Error> where D: Deserializer<'d> {
  let s = <Option<&str>>::deserialize(d)?.unwrap_or("");
  Ok(String::from_utf8(base64::decode(s).map_err(Error::custom)?).map_err(Error::custom)?)
}

pub fn nonempty_string<'d, D>(d: D) -> Result<Option<String>, D::Error> where D: Deserializer<'d> {
  Ok(<Option<String>>::deserialize(d)?.filter(|s| !s.is_empty()))
}

pub fn int_to_bool<'d, D>(d: D) -> Result<bool, D::Error> where D: Deserializer<'d> {
  Ok(u32::deserialize(d)? != 0)
}
