mod syntax;

use nom::{
    branch::alt,
    bytes::complete::{escaped_transform, tag, take_while, take_while1},
    character::complete::{multispace0, multispace1, none_of},
    combinator::{opt, value},
    multi::{many0, separated_list0},
    sequence::{delimited, preceded, separated_pair},
    IResult,
};
use std::io::{self, Read};
use std::{env, fs};

use syntax::{Attribute, TextElement};
use tidier::{FormatOptions, Indent};

fn parse_tag_name(input: &str) -> IResult<&str, &str> {
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}

fn parse_class(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag(".")(input)?;
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}
fn parse_classes(input: &str) -> IResult<&str, Vec<String>> {
    many0(parse_class)(input).map(|(input, classes)| {
        (
            input,
            classes
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<String>>(),
        )
    })
}
fn parse_id(input: &str) -> IResult<&str, &str> {
    let (input, _) = tag("#")(input)?;
    take_while1(|c: char| c.is_alphanumeric() || c == '_')(input)
}
fn parse_doc_type(input: &str) -> IResult<&str, String> {
    let (input, _) = alt((tag("!doctype"), tag("!DOCTYPE")))(input)?;
    let (input, _) = multispace1(input)?;
    let (input, _) = tag("{")(input)?;
    let (input, _) = multispace0(input)?;
    let (input, doctype) = parse_quoted_string(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("}")(input)?;
    let doc_type = format!("<!DOCTYPE {}>\n", doctype);
    Ok((input, doc_type))
}

fn parse_custom_tag(input: &str) -> IResult<&str, TextElement> {
    let (input, _) = multispace0(input)?;
    let (input, name) = parse_tag_name(input)?;
    let (input, id) = opt(parse_id)(input)?;
    let (input, classes) = opt(parse_classes)(input)?;
    let (input, _) = multispace0(input)?;
    let (input, attributes) = opt(parse_attributes)(input)?;
    let (input, _) = preceded(multispace0, tag("{"))(input)?;
    let (input, content) = parse_text(input)?;
    let (input, _) = multispace0(input)?;
    let (input, _) = tag("}")(input)?;

    Ok((
        input,
        TextElement::CustomTag {
            name: name.to_string(),
            id: id.map(|s| s.to_string()),
            classes,
            content,
            attributes,
        },
    ))
}
fn parse_text_element(input: &str) -> IResult<&str, TextElement> {
    let (input, _) = multispace0(input)?;
    let (input, content) = alt((
        parse_custom_tag,
        parse_quoted_string_element, // Adjust this as needed to ensure it doesn't consume the start of tags.
    ))(input)?;
    Ok((input, content))
}

fn parse_text(input: &str) -> IResult<&str, Vec<TextElement>> {
    many0(preceded(multispace0, parse_text_element))(input)
}
fn parse_attribute(input: &str) -> IResult<&str, Attribute> {
    let (input, _) = multispace0(input)?;
    let (input, (name, value)) = separated_pair(
        take_while(|c: char| c.is_alphanumeric() || c == '_' || c == '-'),
        tag("="),
        parse_quoted_string,
    )(input)?;

    Ok((input, Attribute(name.to_string(), value.to_string())))
}
fn parse_attributes(input: &str) -> IResult<&str, Vec<Attribute>> {
    delimited(
        tag("["),
        separated_list0(multispace1, parse_attribute),
        tag("]"),
    )(input)
}
fn parse_escapes(input: &str) -> IResult<&str, String> {
    escaped_transform(
        none_of("\"\\"), // Characters allowed without escaping
        '\\',            // Escape character
        alt((
            value("\\", tag("\\")), // Escaped backslash
            value("\"", tag("\"")), // Escaped double-quote
            value("\n", tag("n")),  // Escaped newline
        )),
    )(input)
}
fn parse_text_body(input: &str) -> IResult<&str, String> {
    parse_escapes(input)
}
fn parse_quoted_string(input: &str) -> IResult<&str, String> {
    let (input, content) = delimited(tag("\""), opt(parse_text_body), tag("\""))(input)?;
    let content = match content {
        Some(content) => content,
        None => "".to_string(),
    };
    Ok((input, content))
}
fn parse_quoted_string_element(input: &str) -> IResult<&str, TextElement> {
    let (input, content) = delimited(tag("\""), parse_escapes, tag("\""))(input)?;
    Ok((input, TextElement::Plain(content.to_string())))
}
fn to_html(text_element: &TextElement) -> String {
    match text_element {
        TextElement::Plain(s) => s.to_string(),
        TextElement::CustomTag {
            name,
            id,
            classes,
            attributes,
            content,
        } => {
            let attributes_html = match attributes {
                Some(attributes) => attributes
                    .iter()
                    .map(|Attribute(name, value)| format!("{}=\"{}\"", name, value))
                    .collect::<Vec<String>>()
                    .join(" "),
                None => "".to_string(),
            };
            let inner_html = content.iter().map(to_html).collect::<String>();

            let classes_html = match classes {
                Some(classes) => {
                    if classes.len() > 0 {
                        format!("class=\"{}\"", classes.join(" "))
                    } else {
                        "".to_string()
                    }
                }
                None => "".to_string(),
            };
            let id_html = match id {
                Some(id) => format!("id=\"{}\"", id),
                None => "".to_string(),
            };
            if name == "br" {
                return "<br>".to_string();
            }
            if name == "doctype" {
                return format!("<!DOCTYPE {}>", inner_html);
            }
            format!(
                "<{} {} {} {}>{}</{}>",
                name, id_html, classes_html, attributes_html, inner_html, name
            )
        }
    }
}

fn cleanup_html(html: &str) -> String {
    let opts = FormatOptions {
        strip_comments: false,
        indent: Indent {
            tabs: false,
            size: 2,
            attributes: false,
            cdata: false,
        },
        ..FormatOptions::DEFAULT
    };
    match tidier::format(html, false, &opts) {
        Ok(formatted) => formatted,
        Err(e) => {
            println!("Error formatting HTML: {:?}", e);
            html.to_string()
        }
    }
}

fn main() {
    let mut input = String::new();

    let args: Vec<String> = env::args().collect();
    if args.len() > 1 {
        // If a file path is provided, read from the file
        let file_path = &args[1];
        match fs::read_to_string(file_path) {
            Ok(file_content) => {
                input = file_content;
            }
            Err(e) => {
                println!("Error reading file '{}': {}", file_path, e);
                return;
            }
        }
    } else {
        // If no file path is provided, read from stdin
        match io::stdin().read_to_string(&mut input) {
            Ok(_) => (),
            Err(e) => {
                println!("Error reading from stdin: {}", e);
                return;
            }
        }
    }

    // Doctype needs to be parsed separately
    // because the main content assumes all content is in one root tag
    let (input, doctype) = match parse_doc_type(input.as_str()) {
        Ok(result) => result,
        Err(e) => {
            println!("Error parsing document type: {:?}", e);
            return;
        }
    };

    match parse_text_element(input) {
        Ok((_, parsed)) => {
            println!("{}{}", doctype, cleanup_html(&to_html(&parsed)));
        }
        Err(e) => println!("Parsing error: {:?}", e),
    }
}
