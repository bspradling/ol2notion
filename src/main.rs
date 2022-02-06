use crate::arguments::Arguments;
use anyhow::Result;
use notion::models::search::{DatabaseQuery, NotionSearch};
use notion::models::{Object, Page};
use notion::NotionApi;
use open_library::models::account::ReadingLogEntry;
use open_library::{OpenLibraryAuthClient, OpenLibraryClient};
use std::collections::HashMap;
use structopt::StructOpt;

mod arguments;

#[tokio::main]
async fn main() -> Result<()> {
    //TODO change the auth client to builder pattern
    let auth_client = OpenLibraryAuthClient::new(None).unwrap();

    let arguments = Arguments::from_args();

    //TODO can you take in a reference to username and password
    let session = auth_client
        .login(
            arguments.open_library_username().clone(),
            arguments.open_library_password(),
        )
        .await?;
    let client = OpenLibraryClient::builder()
        .with_session(&session)
        .build()?;

    //TODO: add appropriate logging framework
    println!("Making a call to retrieve already read books");

    let want_to_read = client
        .account
        .get_want_to_read(arguments.open_library_username())
        .await?;

    println!("{:?}", want_to_read);

    let notion = NotionApi::new(arguments.notion_token())?;

    let search_results = notion
        .search(NotionSearch::Query(arguments.notion_database()))
        .await?;

    if search_results.results.len() != 1 {
        panic!("Expecting there to only be one result");
    }

    let option = search_results.results.get(0).unwrap();
    let database = match option {
        Object::Database { database } => database,
        _ => panic!("The supplied name for a Notion Database, wasn't a database"),
    };

    let query_response = notion
        .query_database(database.clone(), DatabaseQuery::default())
        .await?;

    let books = query_response.results;
    let cache: HashMap<String, &Page> = books
        .iter()
        .filter(|x| x.title().is_some())
        .map(|b| (b.title().unwrap(), b))
        .collect();

    let new_books: Vec<&ReadingLogEntry> = want_to_read
        .iter()
        .filter(|x| !cache.contains_key(&x.work.title))
        .collect();

    //TODO Add Notion ability to create a page within a Database

    Ok(())
}
