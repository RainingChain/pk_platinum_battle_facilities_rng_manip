
#[derive(Debug,PartialEq, PartialOrd,Clone, Copy)]
pub struct Date {
  pub jd: i32,
}
impl Date {
  const fn new(jd:i32) -> Date {
    Date {
      jd,
    }
  }
  pub const fn new3(year:i32, month:i32, day:i32) -> Date {
    let a = if month < 3 { 1 } else { 0 };
    let y = year + 4800 - a;
    let m = month + 12 * a - 3;
    let jd = day + ((153 * m + 2) / 5) - 32045 + 365 * y + (y / 4) - (y / 100) + (y / 400);
    Date::new(jd)
  }
  pub fn getParts(&self) -> (i32, i32, i32) {
    let a = self.jd + 32044;
    let b = (4 * a + 3) / 146097;
    let c = a - (146097 * b) / 4;

    let d = (4 * c + 3) / 1461;
    let e = c - (1461 * d) / 4;
    let m = (5 * e + 2) / 153;

    let year = 100 * b + d - 4800 + (m / 10);
    let month = m + 3 - 12 * (m / 10);
    let day = e - ((153 * m + 2) / 5) + 1;

    ( year, month, day )
  }
  pub fn addDays(&mut self, days:i32){
    self.jd += days;
  }

  pub fn toInputString(&self) -> String {
    let (year,month,day) = self.getParts();
    format!("{}-{:02}-{:02}", year,month,day)
  }
  pub fn fromInputString(str:String) -> Date {
    let sp:Vec<&str> = str.split("-").collect();
    let to_int = |i:usize| String::from(sp[i]).parse::<i32>().unwrap();
    Date::new3(to_int(0), to_int(1), to_int(2))
  }

}
