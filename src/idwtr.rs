use std::{fs, thread, time::Duration};

use scraper::{Html, Selector};

pub fn build_i_dont_want_this_reincarnation() -> epub_builder::Result<()> {
    const NB_OF_CHAPTERS: usize = 346;
    fn fetch_chapters_datas() -> Vec<String> {
        let selector = Selector::parse("#box-content div.c-content.mt-2 > p").unwrap();

        (0..NB_OF_CHAPTERS)
            .map(|id| {
                println!("->> extracting chapter: {}", id + 1);
                // extract raw datas
                let url = format!(
                    "https://allnovelbook.com/novel/i-dont-want-this-reincarnation/chapter-{}",
                    id + 1
                );
                let html_body = reqwest::blocking::get(url)
                    .expect("Couldn't get")
                    .text()
                    .expect("Couldn't get text");

                // extract text from html
                let document = Html::parse_document(&html_body);

                let mut text = String::new();
                for element in document.select(&selector) {
                    text.push_str(&element.html());
                    text.push('\n');
                }

                let xhtml_page = format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en-US">
<head>
<title>Chapter {}</title>
</head>
<body>
<h1 style="text-align: center">Chapter {}</h1>
<hr />
{}
</body>
</html>"#,
                    id + 1,
                    id + 1,
                    text
                );

                thread::sleep(Duration::from_millis(100));
                xhtml_page
            })
            .collect::<Vec<_>>()
    }

    let image = fs::read("./assets/idwtr/I-Dont-Want-This-Reincarnation.jpg")?;
    let css = fs::read_to_string("./assets/idwtr/stylesheet.css")?;
    let cover = fs::read_to_string("./assets/idwtr/cover.xhtml")?;
    let title = fs::read_to_string("./assets/idwtr/title.xhtml")?;

    // Create a new EpubBuilder using the zip library
    let mut idwtr_epub = epub_builder::EpubBuilder::new(epub_builder::ZipLibrary::new()?)?;
    // Set some metadata
    idwtr_epub
        .metadata("title", "I Don't Want This Reincarnation")?
        .metadata("author", "Chaseon")?;
    // Set epub version to 3.0
    idwtr_epub.epub_version(epub_builder::EpubVersion::V30);
    // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
    idwtr_epub.stylesheet(css.as_bytes())?;
    // Add a image cover file
    idwtr_epub.add_cover_image(
        "I-Dont-Want-This-Reincarnation.jpg",
        &image[..],
        "image/jpg",
    )?;
    // Add a cover page
    idwtr_epub
        .add_content(
            epub_builder::EpubContent::new("cover.xhtml", cover.as_bytes())
                .title("Cover")
                .reftype(epub_builder::ReferenceType::Cover),
        )?
        // Add a title page
        .add_content(
            epub_builder::EpubContent::new("title.xhtml", title.as_bytes())
                .title("Title")
                .reftype(epub_builder::ReferenceType::TitlePage),
        )?;

    let chapters_xhtml = fetch_chapters_datas();
    // Add a chapter, mark it as beginning of the "real content"
    idwtr_epub.add_content(
        epub_builder::EpubContent::new("chapter_1.xhtml", chapters_xhtml[0].as_bytes())
            .title("Chapter 1")
            .reftype(epub_builder::ReferenceType::Text),
    )?;

    #[allow(clippy::needless_range_loop)]
    for id in 1..NB_OF_CHAPTERS {
        idwtr_epub.add_content(
            epub_builder::EpubContent::new(
                format!("chapter_{}.xhtml", id + 1),
                chapters_xhtml[id].as_bytes(),
            )
            .title(format!("Chapter {}", id + 1))
            .reftype(epub_builder::ReferenceType::Text),
        )?;
    }

    // generate toc
    idwtr_epub.inline_toc();

    // open new file
    let mut out_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .append(false)
        .create(true)
        .open("./out/i_dont_want_this_reincarnation.epub")?;

    // build epub
    idwtr_epub.generate(&mut out_file)?;

    Ok(())
}
