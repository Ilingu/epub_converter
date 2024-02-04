use std::{fs, thread, time::Duration};

use scraper::{Html, Selector};

use crate::utils::clean_html;

pub fn build_nhad() -> epub_builder::Result<()> {
    fn fetch_chapters_urls() -> Vec<String> {
        let html_body =
            reqwest::blocking::get("https://perpetualdaydreams.com/novel/nhad/nhad-tl/")
                .expect("Couldn't get")
                .text()
                .expect("Couldn't get text");

        // extract text from html
        let document = Html::parse_document(&html_body);
        let selector =
            Selector::parse("#post-2089 > div.post-content > p:nth-child(28) > a").unwrap();

        let mut chatpers_urls = vec![];
        for element in document.select(&selector) {
            chatpers_urls.push(element.attr("href").expect("Must have an url").to_string());
        }

        chatpers_urls
    }

    fn fetch_chapters_datas() -> Vec<String> {
        let chatpers_urls = fetch_chapters_urls();

        let chapter_selector = Selector::parse(".post-content").unwrap();
        let header_selector = Selector::parse(".post-header").unwrap();
        let chapter_image_selector = Selector::parse(".post-content img").unwrap();

        chatpers_urls
            .into_iter()
            .enumerate()
            .map(|(id, url)| {
                println!("->> extracting chapter: {} at {url}", id + 1);
                // extract raw datas
                let html_body = reqwest::blocking::get(url)
                    .expect("Couldn't get")
                    .text()
                    .expect("Couldn't get text");

                // extract text from html
                let document = Html::parse_document(&html_body);

                let header_html = clean_html(
                    document
                        .select(&header_selector)
                        .next()
                        .expect("Chapter header should exist")
                        .inner_html(),
                );

                let mut chapter_html = clean_html(
                    document
                        .select(&chapter_selector)
                        .next()
                        .expect("Chapter should exist")
                        .inner_html(),
                );

                // remove non supported elems
                for img in document.select(&chapter_image_selector) {
                    chapter_html = chapter_html.replace(&clean_html(img.html()), "");
                }

                let xhtml_page = format!(
                    r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE html>
<html xmlns="http://www.w3.org/1999/xhtml" xml:lang="en-US">
<head>
<title>Chapter {}</title>
</head>
<body>
{}
<hr />
{}
</body>
</html>"#,
                    id + 1,
                    header_html,
                    chapter_html
                );

                thread::sleep(Duration::from_millis(100));
                xhtml_page
            })
            .collect::<Vec<_>>()
    }

    let image = fs::read("./assets/nhad/cover.jpg")?;
    let css = fs::read_to_string("./assets/nhad/stylesheet.css")?;
    let cover = fs::read_to_string("./assets/nhad/cover.xhtml")?;
    let title = fs::read_to_string("./assets/nhad/title.xhtml")?;

    // Create a new EpubBuilder using the zip library
    let mut epub_builder = epub_builder::EpubBuilder::new(epub_builder::ZipLibrary::new()?)?;
    // Set some metadata
    epub_builder
        .metadata("title", "Nurturing the Hero to Avoid Death")?
        .metadata("author", "mugwort")?;
    // Set epub version to 3.0
    epub_builder.epub_version(epub_builder::EpubVersion::V30);
    // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
    epub_builder.stylesheet(css.as_bytes())?;
    // Add a image cover file
    epub_builder.add_cover_image("cover.jpg", &image[..], "image/jpg")?;
    // Add a cover page
    epub_builder
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
    epub_builder.add_content(
        epub_builder::EpubContent::new("chapter_1.xhtml", chapters_xhtml[0].as_bytes())
            .title("Chapter 1")
            .reftype(epub_builder::ReferenceType::Text),
    )?;

    #[allow(clippy::needless_range_loop)]
    for id in 1..chapters_xhtml.len() {
        epub_builder.add_content(
            epub_builder::EpubContent::new(
                format!("chapter_{}.xhtml", id + 1),
                chapters_xhtml[id].as_bytes(),
            )
            .title(format!("Chapter {}", id + 1))
            .reftype(epub_builder::ReferenceType::Text),
        )?;
    }

    // generate toc
    epub_builder.inline_toc();

    // open new file
    let mut out_file = fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .append(false)
        .create(true)
        .open("./out/nhad.epub")?;

    // build epub
    epub_builder.generate(&mut out_file)?;

    Ok(())
}
