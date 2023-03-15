use tabled::Table;
use crate::database::FlatTaskTreeElement;

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
        let colorless_table = strip_ansi::strip_ansi(&headerless_table);
        colorless_table
    } else {
        table.with(tabled::style::Style::sharp());
        table.to_string()
    };
    table_string
}
