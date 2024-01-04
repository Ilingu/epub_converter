use std::{fs, thread, time::Duration};

use scraper::{Html, Selector};

use crate::utils::{clean_html, decrypt_open_sans_jumbld};

pub fn build_ietclh() -> epub_builder::Result<()> {
    fn fetch_chapters_urls() -> Vec<String> {
        let html_body = reqwest::blocking::get("https://chrysanthemumgarden.com/novel-tl/ietclh/")
            .expect("Couldn't get")
            .text()
            .expect("Couldn't get text");

        // extract text from html
        let document = Html::parse_document(&html_body);
        let selector = Selector::parse(".chapter-item > a").unwrap();

        let mut chatpers_urls = vec![];
        for element in document.select(&selector) {
            chatpers_urls.push(element.attr("href").expect("Must have an url").to_string());
        }

        chatpers_urls
    }

    fn fetch_chapters_datas() -> Vec<String> {
        let chatpers_urls = fetch_chapters_urls();

        let chapter_selector = Selector::parse("#novel-content").unwrap();
        let header_selector = Selector::parse("header.entry-header").unwrap();
        let chapter_jummed_text_selector = Selector::parse("#novel-content .jum").unwrap();
        let chapter_annoying_elems_selector = Selector::parse(
            "#novel-content [style='height:1px;width:0;overflow:hidden;display:inline-block']",
        )
        .unwrap();
        let chapter_image_selector = Selector::parse("#novel-content img").unwrap();

        chatpers_urls
            .into_iter()
            .enumerate()
            .map(|(id, url)| {
                println!("->> extracting chapter: {}", id + 1);
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

                // decrypt jummed text (font ceasar encryption)
                for jummed_element in document.select(&chapter_jummed_text_selector) {
                    let jummed_text = clean_html(jummed_element.inner_html());
                    let dejummed_text = decrypt_open_sans_jumbld(&jummed_text);
                    chapter_html = chapter_html.replace(&jummed_text, &dejummed_text);
                }

                // remove ammoying or non supported elems
                for annoying_elem in document.select(&chapter_annoying_elems_selector) {
                    chapter_html = chapter_html.replace(&clean_html(annoying_elem.html()), "");
                }
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

    let image = fs::read("./assets/ietclh/cover.jpg")?;
    let css = fs::read_to_string("./assets/ietclh/stylesheet.css")?;
    let cover = fs::read_to_string("./assets/ietclh/cover.xhtml")?;
    let title = fs::read_to_string("./assets/ietclh/title.xhtml")?;

    // Create a new EpubBuilder using the zip library
    let mut idwtr_epub = epub_builder::EpubBuilder::new(epub_builder::ZipLibrary::new()?)?;
    // Set some metadata
    idwtr_epub
        .metadata("title", "It’s Easy to Take Care of a Live-in Hero!")?
        .metadata("author", "Goldfish")?;
    // Set epub version to 3.0
    idwtr_epub.epub_version(epub_builder::EpubVersion::V30);
    // Set the stylesheet (create a "stylesheet.css" file in EPUB that is used by some generated files)
    idwtr_epub.stylesheet(css.as_bytes())?;
    // Add a image cover file
    idwtr_epub.add_cover_image("cover.jpg", &image[..], "image/jpg")?;
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
    for id in 1..chapters_xhtml.len() {
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
        .open("./out/its_easy_to_take_care_of_a_live_in_hero.epub")?;

    // build epub
    idwtr_epub.generate(&mut out_file)?;

    Ok(())
}
