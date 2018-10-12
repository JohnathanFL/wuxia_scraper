extern crate reqwest;
extern crate scraper;

use std::process::{Command, ExitStatus};
use std::io::{self, Write, stdin, stdout, BufRead};
use std::fs::File;
use std::thread::sleep;
use std::time::Duration;


use scraper::{
    Html, Selector, ElementRef, Node,
};

fn prompt(promp: &str) -> String {
    print!("{}: ", promp);
    stdout().flush();

    let mut res = String::new();
    let stdin = stdin();
    return stdin.lock().lines().next()
        .expect("Failed to read console input!")
        .expect("Failed to read console input!");
}

const BASE_URL: &str = "https://www.wuxiaworld.com";

const HTML_HEAD: &str = r#"<!doctype html>
<html>
<head>
    <meta charset="utf-8">
</head>
<body>
"#;

const HTML_TAIL: &str = "</body></html>";

macro_rules! flush {
    () => {stdout().flush();};
}

fn main() {
    let chapLinkSelector = Selector::parse(".chapter-item a").unwrap();
    let chapTextSelector = Selector::parse(".p-15 .fr-view").unwrap();

    println!("Welcome to the WuxiaScraper!");
    let url = prompt("Please enter the section of the url AFTER '/novel/'");

    let storyUrl = format!("{}/novel/{}", BASE_URL, url);

    println!("Getting index...");
    flush!();
    let indexPageBody = reqwest::get(&storyUrl)
        .expect("Failed to get story page!")
        .text()
        .expect("Failed to get index text!");

    println!("Parsing index...");
    flush!();
    let indexPage = Html::parse_document(&indexPageBody);

    println!("Building a chapter list...");
    flush!();
    let mut i = 0;
    // Title to URL
    let mut chapURLList: Vec<(String, String)> = Vec::new();
    for chapLink in indexPage.select(&chapLinkSelector) {
        let chapTitle = {
            let temp: String = chapLink.text().collect();
            temp.trim().to_string()
        };
        let chapURL = chapLink.value().attr("href").unwrap().trim().to_string();

        chapURLList.push((chapTitle, chapURL));

        println!("Found chapter URL: {:?}", chapURLList[i]);
        flush!();
        i += 1;
    }

    println!("Found {} chapters!", chapURLList.len());
    flush!();

    let start: usize = prompt("Enter a starting chapter (or press enter for chap1)").parse()
        .unwrap_or(1) - 1;
    let end: usize = prompt(
                            format!("Enter an ending chapter or press enter for chap{})",
                                    chapURLList.len()).as_str())
        .parse()
        .unwrap_or(chapURLList.len()) - 1;

    println!("Downloading from chap {} to chap {}...", start + 1, end + 1);
    flush!();

    let tocFileName = format!("{}.html", url);

    let mut tocFile = File::create(tocFileName.clone()).unwrap();
    tocFile.write_all(HTML_HEAD.as_bytes());
    tocFile.write_all("<ol>\n".as_bytes());

    for i in start..=end {
        sleep(Duration::from_millis(500));

        let fileName = format!("{}.html", chapURLList[i].0);
        tocFile.write_all(format!("<li><a href=\"{}\">{}</a></li>", fileName, chapURLList[i].0)
            .as_bytes());

        let mut chapFile = File::create(fileName)
            .expect("Could not open file for chapter!");

        let chapUrl = format!("{}{}", BASE_URL, chapURLList[i].1);
        println!("Attempting to download from URL {}", chapUrl);
        flush!();

        let chapBody = reqwest::get(chapUrl.as_str())
            .expect("Failed to get chapter!")
            .text()
            .expect("Failed to get chapter text!");

        let chapDom = Html::parse_document(&chapBody);
        let text = chapDom.select(&chapTextSelector).next().unwrap().html();

        chapFile.write_all(HTML_HEAD.as_bytes());
        chapFile.write_all(text.as_bytes());
        chapFile.write_all(HTML_TAIL.as_bytes());

        chapFile.flush();
    }

    tocFile.write_all(r#"</ol>"#.as_bytes());
    tocFile.write_all(HTML_TAIL.as_bytes());
    tocFile.flush();
    drop(tocFile);

    println!("Generating epub...");
    flush!();
    let ebookOutput = Command::new("ebook-convert")
        .arg(format!(r#"{}"#, tocFileName))
        .arg(format!(r#"{}.epub"#, url))
        .args(&["--max-levels", "1"])
        .args(&["--max-toc-links", "0"])
        .args(&["--level1-toc", "/html/body/ol/li/a"])
        .output()
        .expect("Failed to create epub!")
        .status;

    if !ebookOutput.success() {
        println!("Failed to create epub!");
    }

    //println!("{:?}", ebookOutput);
}
