#[derive(Debug, Clone)]
pub struct Attribute(pub String,pub String); // (Name, Value)

#[derive(Debug )]
pub enum TextElement {
    Plain(String),
    CustomTag {
        name: String,
        content: Vec<TextElement>,
        id: Option<String>,
        classes: Option<Vec<String>>,
        attributes: Option< Vec<Attribute> >,
    },
}
