use std::cmp::Reverse;

use fuzzy_matcher::{skim::SkimMatcherV2, FuzzyMatcher};

use crate::database::Task;

pub fn search(term: &str, tasks: Vec<Task>) -> Vec<Task> {
    let matcher = SkimMatcherV2::default();

    let mut scored_tasks = tasks
        .into_iter()
        .filter_map(|task| {
            matcher
                .fuzzy_match(&task.description, term)
                .map(|score| (task, score))
        })
        .collect::<Vec<(Task, i64)>>();

    scored_tasks.sort_by_key(|scored_task| Reverse(scored_task.1));

    scored_tasks
        .into_iter()
        .map(|scored_task| scored_task.0)
        .collect()
}

// TODO: Add more sorting/filtering methods
/*
pub fn alphabetical(tasks: Vec<Task>) -> Vec<Task> {
    let mut tasks = tasks;
    tasks.sort_by_key(|task| task.description.clone());

    tasks
}

pub struct CompletionFilterResult {
    incomplete: Vec<Task>,
    complete: Vec<Task>,
}

pub fn completion_filter(tasks: Vec<Task>) -> CompletionFilterResult {
    let mut tasks = tasks;
    let mut incomplete_tasks: Vec<Task> = Vec::default();

    tasks.retain(|task| {
        if !task.complete {
            incomplete_tasks.push(task.clone());
        }
        task.complete
    });

    CompletionFilterResult {
        complete: tasks,
        incomplete: incomplete_tasks,
    }
}
*/
