There's only a few weeks remaining until the end of my NEA, here's my plan for
completion

## A focus on documentation

- My documentation efforts to this point have been alright, but preliminary. It
  is important to me to have quality documentation (including better technical
  documentation of how the code works). This needs to be written *as code is
  being developed*
- In-TUI documentation will be required
- Documenting more complex parts of the code (i.e. the following recursively
  inner-joining SQL statement) is particularly desirable *even if it is
  relatively obvious what it does*. This is an unfortunate effect of the NEA
  format

    ```sql
    WITH RECURSIVE subtask_tree AS (
        SELECT *
        FROM tasks
        WHERE id = ?
    UNION ALL
        SELECT subtasks.*
        FROM tasks subtasks
    INNER JOIN subtask_tree ON subtask_tree.id = subtasks.parent
    )
    SELECT * FROM subtask_tree
    ```

- Code flow should be documented, particularly in diagram form to make it easier
  to digest
- In documentation I need to focus on making it easier for examiners rather
  users (another unfortunate effect of the NEA format)

## A focus on testing

- My SQL is tested when I compile my code
- My `database.rs` file has some additional tests for SQL (i.e. check that
  cascade delete is correctly set up)
- It is a challenge to test TUI (and possibly something that I will be doing
  manually and documenting), however this burden can be eased by writing more
  automated tests (i.e. for my search function)
- Exposing my functions via the CLI will open up opportunities for integration
  tests rather than just unit tests
- These must be documented for the sake of the NEA

## A focus on CLI

- I have focused largely on TUI usage, as it's the main place where my client
  wanted me to focus. I would still like to work on a CLI wrapper to my
  functions, particularly as it would allow integration with tools such as
  `cron` and `notify-send` to add notifications (and similar extendability)

## A focus on my client

- Yesterday, I had an NEA call with my client (we call regularly but do not
  always discuss the NEA)
- They were generally pleased with my work, and was happy with how the
  application looks so far, they gave me a few general pointers of how to
  continue

### My client's pointers

- My client thinks it is important to have a task window to view all details -
  in particular a way to properly view subtasks (rather than just in search as
  it is currently)
- My client would like a way (perhaps tab?) to view subtasks in a tree view
- My client would like a way (similar to Vim's modeline) to see both the status
  of tasks (how many there are etc.) as well as the mode we are in the program
  (searching, expanded, list etc.). This fits well with the command palette
  feature (also inspired by vim)
- My client is happy with progress so far and would like to meet again in a week
  to see progress. It is understood that this is likely to be less than on some
  previous weeks as I have mock exams

