use crate::arguments::Arguments;
use crate::models::{DatabaseProperty, Entity};
use anyhow::Result;
use notion::ids::PropertyId;
use notion::models::properties::{
    CreatePropertyValueRequest, CreatedSelectedValue, PropertyConfiguration, PropertyValue,
    SelectedValue,
};
use notion::models::search::{DatabaseQuery, NotionSearch};
use notion::models::text::{RichText, RichTextCommon};
use notion::models::{
    CreatePageRequest, CreatePropertiesRequest, Icon, Object, Page, Parent, Properties, Text,
};
use notion::NotionApi;
use open_library::models::account::ReadingLogEntry;
use open_library::models::works::Work;
use open_library::{OpenLibraryAuthClient, OpenLibraryClient};
use rand::seq::SliceRandom;
use std::collections::HashMap;
use structopt::StructOpt;

mod arguments;
mod models;

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

    println!("Querying Database Id: {:?}", database.clone().id);

    let query_response = notion
        .query_database(database.clone(), DatabaseQuery::default())
        .await?;

    println!("Query Response: {:?}", &query_response);

    let books = query_response.results;
    let cache: HashMap<String, &Page> = books
        .iter()
        .filter(|x| x.title().is_some())
        .map(|b| (b.title().unwrap(), b))
        .collect();

    let new_works: Vec<Result<Entity>> = futures::future::join_all(
        want_to_read
            .iter()
            .filter(|x| !cache.contains_key(&x.work.title))
            .map(|x| async {
                Ok(Entity {
                    id: x.work.key.clone(),
                    title: x.work.title.clone(),
                    authors: x.work.author_names.clone(),
                    tags: client.works.get(&x.clone().work.key.into()).await?.subjects,
                })
            }),
    )
    .await;

    println!("New Works: {:?}", &new_works);

    //TOD0: improved error handling around the .unwrap()
    let tag_options: Vec<String> = match database.properties.get("Tags").unwrap() {
        PropertyConfiguration::MultiSelect { id, multi_select } => multi_select
            .options
            .iter()
            .map(|x| x.name.clone())
            .collect(),
        _ => panic!("Unsupported type for Tags"),
    };

    let pages_to_create: Vec<CreatePageRequest> = new_works
        .into_iter()
        .flat_map(|x| x)
        .map(|entity| {
            let work_tags = entity
                .tags
                .iter()
                .filter(|x| tag_options.contains(x))
                .take(5)
                .map(|x| CreatedSelectedValue { name: x.clone() })
                .collect();

            CreatePageRequest {
                properties: CreatePropertiesRequest {
                    properties: HashMap::from([
                        (
                            DatabaseProperty::Name.name(),
                            CreatePropertyValueRequest::Title {
                                title: vec![RichText::Text {
                                    rich_text: RichTextCommon {
                                        plain_text: entity.title.clone(),
                                        href: None,
                                        annotations: None,
                                    },
                                    text: notion::models::text::Text {
                                        content: entity.title.clone(),
                                        link: None,
                                    },
                                }],
                            },
                        ),
                        (
                            DatabaseProperty::Author.name(),
                            CreatePropertyValueRequest::Text {
                                rich_text: vec![RichText::Text {
                                    rich_text: RichTextCommon {
                                        plain_text: entity.authors.join(","),
                                        href: None,
                                        annotations: None,
                                    },
                                    text: notion::models::text::Text {
                                        content: entity.authors.join(","),
                                        link: None,
                                    },
                                }],
                            },
                        ),
                        (
                            DatabaseProperty::Tags.name(),
                            CreatePropertyValueRequest::MultiSelect {
                                multi_select: work_tags,
                            },
                        ),
                        (
                            DatabaseProperty::Status.name(),
                            CreatePropertyValueRequest::Select {
                                select: CreatedSelectedValue {
                                    // TODO: make configurable
                                    name: "Inbox".to_string(),
                                },
                            },
                        ),
                        (
                            DatabaseProperty::Url.name(),
                            CreatePropertyValueRequest::Url {
                                url: format!("https://www.openlibrary.org{}", entity.id),
                            },
                        ),
                    ]),
                },
                parent: Parent::Database {
                    database_id: database.id.clone(),
                },
                icon: Icon::Emoji {
                    emoji: ["📕", "📗", "📘", "📙", "📔"]
                        .choose(&mut rand::thread_rng())
                        .unwrap()
                        .to_string(),
                },
                children: vec![],
            }
        })
        .collect();

    println!("Pages to Create: {:?}", pages_to_create);

    let created_pages: Vec<Result<Page>> = futures::future::join_all(
        pages_to_create
            .iter()
            .map(|x| async { Ok(notion.create_page(x.clone()).await?) }),
    )
    .await;

    println!("Created Pages {:?}", created_pages);

    Ok(())
}
