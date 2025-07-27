#![allow(dead_code)]

use std::collections::HashMap;

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct ExportDocument {
    pub chats: ExportDocumentChats,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct ExportDocumentChats {
    pub list: Vec<ExportDocumentChat>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct ExportDocumentChat {
    pub messages: Vec<Message>,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct Message {
    pub id: usize,
    pub r#type: String,
    pub date: String,
    pub date_unixtime: String,
    #[serde(flatten)]
    pub sender: Sender,
    pub text: Text,
    pub text_entities: Vec<TextEntity>,
}

impl Message {
    pub fn text(&self) -> String {
        self.text_entities
            .iter()
            .map(|e| e.text.clone())
            .collect::<Vec<_>>()
            .join(" ")
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case", untagged)]
enum Sender {
    From { from: String, from_id: String },
    Actor { actor: String, actor_id: String },
}

impl Sender {
    fn name(&self) -> String {
        match self {
            Sender::From { from, from_id: _ } => from.clone(),
            Sender::Actor { actor, actor_id: _ } => actor.clone(),
        }
    }

    fn id(&self) -> String {
        match self {
            Sender::From { from: _, from_id } => from_id.clone(),
            Sender::Actor { actor: _, actor_id } => actor_id.clone(),
        }
    }
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(untagged)]
enum Text {
    Plain(String),
    Rich(Vec<TextComponent>),
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(untagged)]
enum TextComponent {
    Text(String),
    Entity(TextEntity),
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
struct TextEntity {
    r#type: TextEntityType,
    text: String,
}

#[derive(Clone, Debug, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
enum TextEntityType {
    Plain,
    Link,
    Mention,
    BotCommand,
    Bold,
    Italic,
    Strikethrough,
}

pub struct ImportedDrink {
    pub user_tg_id: String,
    pub user_tg_nick: String,
    pub timestamp: usize,
}

pub fn parse(
    source: String,
) -> Result<Vec<ImportedDrink>, serde_path_to_error::Error<serde_json::Error>> {
    let jd = &mut serde_json::Deserializer::from_str(&source);

    let result: Result<ExportDocument, _> = serde_path_to_error::deserialize(jd);
    match result {
        Ok(data) => {
            let msgs = data.chats.list[0].messages.clone();

            println!("found {} messages", msgs.len());

            let mut nums = vec![None];

            let mut last_1 = 0;
            for msg in msgs {
                if msg.id == 18215 {
                    break;
                }

                let txt = msg.text();

                if txt.chars().filter(|c| *c == '\n').count() > 4 {
                    continue;
                }
                if [
                    10995, 13578, 13399, 13489, 17745, 10998, 8872, 8873, 9024, 13498, 9009, 8940,
                    8854, 8275, 8302, 8305, 10120, 10140, 10143, 10144, 8212, 8205, 8266, 8387,
                    8407, 10515, 10518, 10511, 10510, 10507, 10503, 10590, 10632, 15595, 11376,
                    13058, 13057, 14292, 15648, 15175, 15167, 17686, 8423, 9440, 9176, 9356, 9357,
                    9410, 9415, 9437, 8792, 12182, 10608, 14291, 15153, 15650, 15031, 16222, 17691,
                    16683, 15155, 8410, 8407, 8404, 8403, 8450, 8449, 8444, 8446, 8635, 8574, 9026,
                    8682, 8686, 8639, 8548, 17685, 8198, 9167, 8334, 10477, 10479, 10480, 10887,
                    8383, 8470, 8467, 8708, 8715, 8734, 8869, 9426, 9568, 9592, 9640, 11375, 13563,
                    10259, 9863, 9858, 9853, 13066, 13497, 13494, 13493, 13492, 17276, 17273,
                    12184, 10013, 10012,
                ]
                .contains(&msg.id)
                {
                    continue;
                }

                if [17683, 13955, 13961, 8776, 15159].contains(&msg.id) {
                    let num = last_1 + 1;
                    println!("MANUAL OVERRIDE {}", num);
                    nums.push(Some((msg.clone(), num, false)));
                    last_1 = num;
                    continue;
                }
                if [17269, 8575, 9969].contains(&msg.id) {
                    let num = last_1 + 2;
                    println!("MANUAL OVERRIDE 2 {}", num);
                    nums.push(Some((msg.clone(), num, false)));
                    last_1 = num;
                    continue;
                }
                if [13695].contains(&msg.id) {
                    let num = last_1 + 8;
                    println!("MANUAL OVERRIDE 8 {}", num);
                    nums.push(Some((msg.clone(), num, false)));
                    last_1 = num;
                    continue;
                }

                let txt_parse = txt.parse::<usize>();

                let skip_words = if txt_parse.is_ok() { usize::MAX - 1 } else { 0 };
                let words = txt
                    .split(' ')
                    .filter(|w| {
                        if w.is_empty() {
                            return false;
                        }
                        if (w.ends_with("l") || w.ends_with("L"))
                            && (w.contains(".") || w.contains(","))
                        {
                            println!("DISCARD LIQUID AMOUNT: {} {w}", msg.id);
                            return false;
                        }
                        if w.starts_with("-") {
                            println!("DISCARD MINUS: {} {w} {}", msg.id, txt);
                            return false;
                        }
                        if *w == "<3" {
                            println!("DISCARD LOVE: {} {w}", msg.id);
                            return false;
                        }

                        true
                    })
                    .collect::<Vec<_>>()
                    .join(" ");
                let words = words
                    .split_terminator(&[' ', ',', '&', '-', '+', '(', ')'])
                    .flat_map(|w| w.split_whitespace())
                    .filter(|w| !w.ends_with("%")) // we're not interested in percentages
                    .filter(|w| !w.ends_with("cl")) // nor centiliters
                    .filter(|w| !w.ends_with("€")) // nor currency
                    .filter(|w| !w.ends_with("$"))
                    .filter(|w| !w.contains(":")) // nor times
                    .filter(|w| !w.ends_with("pv")) // nor days
                    .filter(|w| !w.ends_with("h")) // nor hours
                    .filter(|w| !w.ends_with("m") && !w.ends_with("min")) // nor minutes
                    .filter(|w| !w.ends_with("v")) // nor ages
                    .filter(|w| !w.contains(".")) // or dates
                    .filter_map(|w| {
                        // let without_letters =
                        //     w.chars().filter(|c| c.is_numeric()).collect::<String>();
                        let without_letters = w;
                        without_letters.parse::<usize>().ok()
                    })
                    .skip(skip_words);

                let mut last_local = last_1;
                txt.parse::<usize>()
                    .iter()
                    .cloned()
                    .chain(words)
                    .map(|num| {
                        let mut relative = false;
                        let mut num = num;
                        if num < 50
                        // && (txt.contains(&format!("+{num}"))
                        //     || txt.contains(&format!("+ {num}")))
                        {
                            num += last_1;
                            relative = true;
                        }

                        (num, relative)
                    })
                    .for_each(|(num, relative)| {
                        nums.push(Some((msg.clone(), num, relative)));
                        last_local = num;
                    });
                last_1 = last_local;
            }

            nums.push(None);

            let mut rolling_average = 66.0;
            let calc_rolling = |r: f64| r + 4.5;

            let mut results = "".to_owned();
            let mut log = "".to_owned();
            let mut valid = 0;
            let mut duplicates = 0;
            let mut duplicates_allowed = 0;

            let mut msgs = vec![];

            for window in nums.windows(3) {
                let (
                    (prev_msg, prev_num, prev_relative),
                    (msg, num, relative),
                    (next_msg, next_num, _),
                ) = match (window[0].clone(), window[1].clone(), window[2].clone()) {
                    (_, None, _) => unreachable!(),
                    (None, Some(_), None) => unreachable!(),
                    (Some(p), Some(c), Some(n)) => (p, c, n),
                    (None, Some((msg, num, _relative)), Some(_))
                    | (Some(_), Some((msg, num, _relative)), None) => {
                        log.push_str(&format!("EDGE {}", msg.text()));
                        results.push_str(&format!(
                            "{},{},{},{:?}\n",
                            msg.date,
                            msg.date_unixtime,
                            num,
                            msg.sender.name()
                        ));
                        msgs.push((num + duplicates_allowed, msg.clone()));
                        continue;
                    }
                };

                if num == 0 {
                    continue;
                }

                if num == prev_num {
                    println!(
                        "DUPLICATE {} {}, {}, {} | {:?} L {} C {} N {}",
                        msg.date_unixtime,
                        prev_num,
                        num,
                        next_num,
                        msg.sender,
                        prev_msg.text(),
                        msg.text(),
                        next_msg.text()
                    );
                    duplicates += 1;
                    if !prev_relative {
                        continue;
                    }
                    assert!(!relative);
                    // drunk people are bad at arithmetic :)
                    // let's assume that if the user used a relative number and then the correct
                    // count, they wanted to add one
                    println!("ALLOW");
                    // num += 1;
                    duplicates_allowed += 1;
                }

                if prev_num < num && num < next_num {
                    //
                } else {
                    // println!(
                    //     "DISC {} {}, {}, {} | {:?} L {} C {} N {}",
                    //     msg.date_unixtime,
                    //     prev_num,
                    //     num,
                    //     next_num,
                    //     msg.sender,
                    //     prev_msg.text(),
                    //     msg.text(),
                    //     next_msg.text()
                    // );
                }

                if (num as f64) < (calc_rolling(rolling_average) - 4.0) {
                    println!(
                        "SMALL {} {}, {}, {} | {} | {:?} L {} C {} N {}",
                        msg.date_unixtime,
                        prev_num,
                        num,
                        next_num,
                        rolling_average,
                        msg.sender,
                        prev_msg.text(),
                        msg.text(),
                        next_msg.text()
                    );
                    continue;
                }

                if ((num as f64) - calc_rolling(rolling_average)).abs() < 30.0 || true {
                    results.push_str(&format!(
                        "{},{},{},{:?}\n",
                        msg.date,
                        msg.date_unixtime,
                        num + duplicates_allowed,
                        msg.sender.name()
                    ));
                    rolling_average = ((rolling_average * 4.0) + (num as f64)) / 5.0;
                    println!(" {num} {} {}", rolling_average, msg.text());
                    valid += 1;
                    msgs.push((num + duplicates_allowed, msg.clone()));
                } else {
                    println!(
                        "OFF {} {}, {}, {} | {} | {:?} L {} C {} N {}",
                        msg.date_unixtime,
                        prev_num,
                        num,
                        next_num,
                        rolling_average,
                        msg.sender,
                        prev_msg.text(),
                        msg.text(),
                        next_msg.text()
                    );
                    rolling_average = ((rolling_average * 99.0) + (num as f64)) / 100.0;
                }
            }

            let mut valid_sum: i64 = 0;
            let mut by_user = HashMap::<_, i64>::new();

            let mut last = 66;
            let mut drinks = vec![];
            for (num, msg) in msgs {
                let diff = (num as i64) - (last as i64);
                valid_sum += diff;
                if diff > 0 {
                    let key = msg.sender.name();
                    let previous = by_user.get(&key).cloned().unwrap_or_default();
                    by_user.insert(key, previous + diff);
                    for _ in 0..diff {
                        drinks.push(ImportedDrink {
                            user_tg_id: msg.sender.id(),
                            user_tg_nick: msg.sender.name(),
                            timestamp: msg.date_unixtime.parse().unwrap(),
                        });
                    }
                } else {
                    println!("{diff} {} {}", msg.id, msg.text());
                }
                last = num;
            }

            println!("extracted {} entries, totaling {}", valid, valid_sum);
            println!("{duplicates} duplicates, {duplicates_allowed} allowed");
            let mut pairs = by_user.into_iter().collect::<Vec<_>>();
            pairs.sort_by_key(|(_, s)| *s);
            for (k, v) in pairs {
                println!("\t{k}\t\t{v}");
            }

            Ok(drinks)
        }
        Err(err) => {
            println!("ERR PATH {}", err.path());
            println!("{}", err);
            Err(err)
        }
    }
}
