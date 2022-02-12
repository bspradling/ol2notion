use open_library::models::OpenLibraryResource;

#[derive(Debug, Clone)]
pub struct Entity {
    pub id: OpenLibraryResource,
    pub title: String,
    pub authors: Vec<String>,
    pub tags: Vec<String>,
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub enum DatabaseProperty {
    Author,
    Name,
    Status,
    Tags,
    Url,
}

impl DatabaseProperty {
    pub fn name(&self) -> String {
        match self {
            DatabaseProperty::Author => "Author".to_string(),
            DatabaseProperty::Name => "Name".to_string(),
            DatabaseProperty::Status => "Status".to_string(),
            DatabaseProperty::Tags => "Tags".to_string(),
            DatabaseProperty::Url => "URL".to_string(),
        }
    }
}
