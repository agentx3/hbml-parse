This is a parser for [HBML](https://github.com/heyitsdoodler/hbml) (Hyper Braced Markup Language). This is just a fun little something I made while learning Rust. 

It supports arbitrary alphanumeric tags, ids, classes, and attributes.

#### Usage:

`hbml-parser index.hbml`

You can also pipe files to it

`cat index.hbml | hbml-parser`


Example input:
```yaml
!doctype { "html" }
html {
  head {
    style {"
      html {
        background-color: red !important;
      }
      .content {
        background-image: url(\"example.com\");
      }
    "}
    title { "Title here" }
  }
  body#root {
    h1.title {
      "This is some body text. Escape with \\\\ backslash."
    }
    div#main.content {
      div.content[style="color: red;"] {
        "child div"
        b{" bolded text "}
        "regular text"
        br{}
      }
    }
  }
}

```

Output:

```html
<!DOCTYPE html>
<html>
  <head>

    <style>
      html {
        background-color: red !important;
      }
      .content {
        background-image: url("example.com");
      }
    </style>
    <title>Title here</title>
  </head>
  <body id="root">
    <h1 class="title">
      This is some body text. Escape with \\ backslash.
    </h1>
    <div id="main" class="content">
      <div class="content" style="color: red;">
        child div <b>bolded text</b> regular text<br>
      </div>
    </div>
  </body>
</html>
```
