## Project brief

### Summary

**TeaL** is a task manager for your terminal, it contains both a CLI and TUI
interface, with various features centered around keeping track of, organizing,
and staying up-to-date with your tasks.

### The problem

<!-- spell-checker:words Taskwarrior -->

There are currently no good and easy-to-use long-term task managers for
terminals. As power-users spend a great deal of time in a terminal but will not
necessarily want existing solutions (such as Taskwarrior) due to a lack of
ease-of-use, bugs and bloat. This project will develop a text-based task manager
with both a TUI ([text-based user
interface](https://en.wikipedia.org/wiki/Text-based_user_interface)) for easier
first-time usage, as well as a CLI (command-line interface) for scripting or
quick additions where speed may be preferable to look & feel.

### Research about the problem

#### Interviews

<!-- spell-checker:words dealbreakers -->

For my initial research, I spoke to all of my users and asked them what features
were most important to them and what features would be dealbreakers if they were
missing or present. Of the users I polled they fell into a few categories, which
I've named by the most prominent user in each.

##### The users

<!-- spell-checker:words dogfooding -->

**SpyHoodle** is a power user. She likes simple pieces of software that do one
thing and do it well. She has a specific way of organizing tasks that she likes
("Getting Things Done") and needs a task manager to allow her to use this
workflow. She wants a task manager that is fast, native to her devices and
private.

**PineappleFan** is a user, and while they try to use GUIs where possible they
always have a terminal near them. They don't want to spend time learning a new
tool, so ease-of-first-time-use and quality of the TUI matters a lot to them.

**RubyRed** is a user with similar complexity desires to PineappleFan. She finds
it difficult to get into routines, so wants a task manager that is easy to use
and lets her integrate it into her existing workflows. She knows Rust and would
like to contribute code provided the codebase is easily understandable and
extendable.

#### Existing solutions

##### Taskwarrior

<!-- spell-checker:words antifeature -->

- Taskwarrior is a good program
- However, it has a lot of features that are not all particularly easily
  exposed, for example it's [not clear when filters will or won't be applied to
  the point where I'm not sure if this is a
  bug](https://github.com/GothenburgBitFactory/taskwarrior/issues/2917), an
  antifeature for **PineappleFan** and **RubyRed**
- Taskwarrior also exposes some unnecessary features, which can make it less
  easy to find what you want- e.g. it [lets you use it as a
  calculator](https://taskwarrior.org/docs/commands/calc/), an antifeature for
  **SpyHoodle**
- Taskwarrior reimplements some features like aliases, an antifeature for
  **SpyHoodle**
- Sync between devices is complex

##### pomofocus.io

- Pomofocus.io is a good website
- However, it does not do long-term task management, which means it does not
  meet this brief
- It's also not a native app (an antifeature for **SpyHoodle**) and isn't
  terminal-based (an antifeature for **RubyRed**)

##### stackToDo

- I love stackToDo
- However, it once again doesn't do long-term task management
- While I might add a stack feature into TeaL in the long-term (or possibly make
  it as a plugin), stackToDo is not a viable alternative

##### Google Calendar

- Google calendar is alright
- It synchronizes online, which is both a feature (for **PineappleFan**) and an
  antifeature (for **SpyHoodle**) as this means that Google has access to your
  calendar
- It can be shared as an `.iCal`, meaning that it would be possible to export
  into terminal-based clients
- It requires internet, an antifeature for **RubyRed**
- It has cross-platform support, a feature for **SpyHoodle**

##### dtask

- dtask is built to be similar to taskwarrior but without some of the issues
  that taskwarrior has
- There's not really a TUI, so one still has to learn various commands in order
  to use it (an antifeature for **PineappleFan**)

##### tasklite

- TaskLite is simpler and faster than taskwarrior, a feature for **SpyHoodle**
- It doesn't contain a TUI, an antifeature for **RubyRed** and
  **PineappleFan**

##### todo.txt

- Incredibly simple, a feature for **SpyHoodle**
- Very portable, a feature for **SpyHoodle**
- Doesn't allow organization of tasks into subtasks, an antifeature for
  **RubyRed**

### Keeping it simple

From past projects I've completed, especially the EPQ, I know that it is
important for me to make a simple product first which is submittable and then
expand later if there is more time and features which I want to add.
Additionally, **SpyHoodle** cares about keeping to core features, which aligns
well with this goal.

### Stated goals

<!-- spell-checker:words Taskwarrior's -->

1. Write a task list with both a command-line interface and text-based interface
   1. Store basic information about tasks[^1] [^2] [^3]
      1. Use a SQL database to store task information
      2. Store the database on a users' computer rather than on a cloud-based
         service[^1]
   2. Allow retrieval of tasks[^1] [^2] [^3]
   3. Store task progress and completion[^1] [^2] [^3]
   4. Allow retrieval of task progress and completion[^1] [^2] [^3]
   5. Allow cleanup of old tasks[^1] [^2] [^3]
2. Allow subtasks for better organization of tasks, as well as task
   decomposition[^1] [^2] [^3]
3. Allow sorting and filtering tasks to quickly manage a long list[^1] [^2] [^3]
   1. Allow filtering based on due date
   2. Allow filtering based on title
   3. Allow filtering based on description
   4. Allow filtering based on completion status
   5. Allow filtering based on parent task
4. Be open-source, accepting helpful contributions throughout[^1] [^3]
5. Keep it intuitive. The TUI should be easy for new users to pick up[^2]
6. Keep it documented. The CLI and TUI should both have documentation teaching
   you about how to use them[^1] [^2]
7. Keep it simple. Do not add bloat features (like taskwarrior's calculator). If
   there is time for bloat features, instead develop a plugin system to allow
   users to add their own bloat features[^1]

[^1]: Feature for **SpyHoodle**
[^2]: Feature for **PineappleFan**
[^3]: Feature for **RubyRed**

## Attribution

<!-- spell-checker:words Maddie,SpyHoodle,jimmybilly -->

- Maddie (@SpyHoodle): My primary client, [#1 (spelling
  fix)](https://github.com/Minion3665/TeaL/pull/1)
- /u/jimmybilly100: [Provided me with a copy of Stanley easter
  egg](https://www.reddit.com/r/ProgrammerHumor/comments/xkfv92/comment/ipdta72/)
- [ytop](https://github.com/cjbassi/ytop): Gave me [an example of text input in
  tui-rs](https://github.com/cjbassi/ytop/blob/master/src/main.rs#L186)
- [tui-rs](https://github.com/fdehau/tui-rs): My TUI library, used both for its
  abilities and high-quality documentation and examples
