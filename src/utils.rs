pub fn clean_html(html_content: String) -> String {
    html_content
        .replace("<hr>", "<hr/>")
        .replace("<br>", "<br/>")
}

pub fn decrypt_open_sans_jumbld(jum_str: &str) -> String {
    const MAJ_ALPHABET_JUM: &str = "JKABRUDQZCTHFVLIWNEYPSXGOM";
    const MIN_ALPHABET_JUM: &str = "tonquerzlawicvfjpsyhgdmkbx";
    const ALPHABET: &str = "abcdefghijklmnopqrstuvwxyz";
    jum_str
        .chars()
        .map(|c| match c.is_ascii_alphabetic() {
            true => {
                let alphapos = ALPHABET
                    .chars()
                    .position(|alpha_c| alpha_c == c.to_ascii_lowercase())
                    .unwrap();
                match c.is_lowercase() {
                    true => MIN_ALPHABET_JUM.chars().nth(alphapos).unwrap(),
                    false => MAJ_ALPHABET_JUM.chars().nth(alphapos).unwrap(),
                }
            }
            false => c,
        })
        .collect()
}
