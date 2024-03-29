use crate::database::FlatTaskTreeElement;
use tabled::Table;

pub(crate) fn render_table(tasks: Vec<FlatTaskTreeElement>, raw: bool) -> String {
    let mut table = Table::new(tasks);
    let table_string = if raw {
        table.with(tabled::style::Style::empty().vertical('\t'));
        let headerless_table = table
            .to_string()
            .lines()
            .skip(1)
            .collect::<Vec<_>>()
            .join("\n");
        strip_ansi::strip_ansi(&headerless_table)
    } else {
        table.with(tabled::style::Style::sharp());
        table.to_string()
    };
    table_string
}
