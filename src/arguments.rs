use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt(
    name = "Arguments",
    about = "Synchronize Open Library Reading Log with Notion"
)]
pub struct Arguments {
    #[structopt(
        long = "open-library-username",
        help = "The Open Library username to retrieve the Reading Log from",
        env = "OPEN_LIBRARY_USERNAME"
    )]
    open_library_username: String,

    #[structopt(
        long = "open-library-password",
        help = "The Open Library password for the Open Library username supplied",
        env = "OPEN_LIBRARY_PASSWORD"
    )]
    open_library_password: String,

    #[structopt(
        long = "notion-token",
        help = "The Notion token that has permissions to read/write within the Notion Database",
        env = "NOTION_TOKEN"
    )]
    notion_token: String,

    #[structopt(
        long = "notion-database",
        help = "The Notion database that contains Book data",
        env = "NOTION_DATABASE"
    )]
    notion_database: String,
}

impl Arguments {
    pub fn open_library_username(&self) -> String {
        self.open_library_username.clone()
    }

    pub fn open_library_password(&self) -> String {
        self.open_library_password.clone()
    }

    pub fn notion_token(&self) -> String {
        self.notion_token.clone()
    }

    pub fn notion_database(&self) -> String {
        self.notion_database.clone()
    }
}
