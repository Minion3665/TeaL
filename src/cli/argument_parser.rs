use eyre::Result;
use std::collections::HashMap;
use std::collections::HashSet;
use std::collections::VecDeque;
use unicode_segmentation::UnicodeSegmentation;
pub struct Arguments<'a> {
    pub args: HashMap<&'a str, Vec<&'a str>>,
    pub flags: HashSet<&'a str>,
    pub subcommand: Vec<&'a str>,
    pub command: &'a str,
}

pub fn parse_args<'a>(args: Vec<&'a str>) -> Result<Arguments<'a>> {
    if args.is_empty() {
        return Err(eyre::Error::msg("Empty arguments vectors are unparseable as the first item in the vector must always be the command name"));
    }

    let mut args_queue = VecDeque::from(args);

    let mut removed_arguments: HashSet<&str> = HashSet::new();
    let mut arguments: HashMap<&str, Vec<&str>> = HashMap::new();
    let mut subcommand: Vec<&str> = Vec::new();
    let command = args_queue.pop_front().unwrap();

    let mut processing_flag: Option<&str> = None;
    let mut processing_flag_values: Vec<&str> = vec![];

    while !args_queue.is_empty() {
        let arg = args_queue.pop_front().unwrap();
        if arg.starts_with("--") {
            if let Some(flag_name) = processing_flag {
                arguments.insert(flag_name, processing_flag_values.clone());
            }

            let flag_name = arg.strip_prefix("--").unwrap();
            processing_flag = Some(flag_name);

            if arg.starts_with("--no-") {
                removed_arguments.insert(arg.strip_prefix("--no-").unwrap());
                processing_flag = None;
                continue;
            }

            processing_flag_values = if let Some(flag_values) = arguments.get(flag_name) {
                flag_values.to_owned()
            } else {
                Vec::new()
            };
        } else if arg.starts_with("-") {
            let flag_letters = arg.strip_prefix("-").unwrap();

            for letter in flag_letters.graphemes(true) {
                // ^ The true flag means that extended grapheme clusters are used
                // In practice this adds some newer characters to what is considered a grapheme
                if let Some(flag_name) = processing_flag {
                    arguments.insert(flag_name, processing_flag_values);
                }

                processing_flag = Some(letter);
                processing_flag_values = if let Some(flag_values) = arguments.get(letter) {
                    flag_values.to_owned()
                } else {
                    Vec::new()
                };
            }
        } else {
            if processing_flag.is_some() {
                processing_flag_values.push(arg);
            } else {
                subcommand.push(arg);
            }
        }
    }

    if let Some(flag_name) = processing_flag {
        arguments.insert(flag_name, processing_flag_values);
    }

    let mut flags = HashSet::new();

    let args = HashMap::from_iter(arguments.into_iter().filter(|(key, value)| {
        if removed_arguments.contains(key) {
            return false;
        } else if value.is_empty() {
            flags.insert(key.clone());
            return false;
        } else {
            return true;
        }
    }));

    Ok(Arguments {
        args,
        flags,
        subcommand,
        command,
    })
}

pub fn parse_ids(ids: Option<&Vec<&str>>) -> Result<Vec<i64>, String> {
    if let Some(task_ids) = ids {
        let mut parsed_task_ids: Vec<i64> = vec![];
        for id in task_ids {
            let last_id_bit = id.split(".").last().unwrap_or_default();
            if let Ok(id) = last_id_bit.parse::<i64>() {
                parsed_task_ids.push(id);
            } else {
                return Err(format!("Invalid task id: {}", id));
            }
        }
        Ok(parsed_task_ids)
    } else {
        Err("No task ids provided".to_owned())
    }
}
