use anyhow::Result;
use std::io::Cursor;
use skim::prelude::*;

pub fn handle_skim(input: String) -> Result<Option<String>> {
    let options = SkimOptionsBuilder::default()
        .height(String::from("50%"))
        .no_multi(true)
        .build()
        .unwrap();

    let item_reader = SkimItemReader::default();
    let items = item_reader.of_bufread(Cursor::new(input));

    let output = Skim::run_with(&options, Some(items));

    match output {
        Some(out) if out.is_abort => Ok(None),
        Some(out) => {
            let selection = out
                .selected_items
                .first()
                .map(|item| item.output().to_string());
            Ok(selection)
        }
        None => Ok(None),
    }
}
