// Uses
use std::{env::args, fs::File, io::Read};

use anyhow::{Context, Result as AnyhowResult, anyhow};
use chrono::{DateTime, FixedOffset, NaiveDateTime};
use html_parser::{Dom, Element};

#[derive(Debug)]
struct VideoEntry<'a> {
	url:       &'a str,
	title:     &'a str,
	timestamp: DateTime<FixedOffset>,
}

fn main() -> AnyhowResult<()> {
	let path = args()
		.nth(1)
		.expect("one argument is required - the path to the file");

	let mut file_contents = String::new();
	let mut file = File::options()
		.read(true)
		.write(true)
		.open(path.as_str())
		.with_context(|| "unable to open file")?;
	file.read_to_string(&mut file_contents)
		.with_context(|| "unable to read file")?;

	let mut video_entries = Vec::new();

	println!("Parsing...");
	let dom = Dom::parse(file_contents.as_str())
		.with_context(|| "unable to parse file contents as valid HTML")?;

	let html = dom
		.children
		.iter()
		.find_map(|node| {
			if let Some(element) = node.element()
				&& element.name == "html"
			{
				Some(element)
			} else {
				None
			}
		})
		.ok_or_else(|| anyhow!("no html element could be found"))?;

	let body = html
		.children
		.iter()
		.find_map(|node| {
			if let Some(element) = node.element()
				&& element.name == "body"
			{
				Some(element)
			} else {
				None
			}
		})
		.ok_or_else(|| anyhow!("no body element could be found"))?;

	let main_div = body
		.children
		.first()
		.and_then(|node| node.element())
		.ok_or_else(|| anyhow!("no main div could be found"))?;

	println!("Collecting video entries...");
	for child_node in &main_div.children {
		let outer_cell = child_node
			.element()
			.some_if_has_class("outer-cell")
			.ok_or_else(|| anyhow!("outer-cell does not exist"))?;

		let mdl_grid = outer_cell
			.children
			.first()
			.and_then(|node| node.element())
			.some_if_has_class("mdl-grid")
			.ok_or_else(|| anyhow!("mdl-grid does not exist"))?;

		if mdl_grid.children.len() < 2 {
			return Err(anyhow!("mdl-grid is missing children"));
		}
		let header_cell = mdl_grid.children[0]
			.element()
			.some_if_has_class("header-cell")
			.ok_or_else(|| anyhow!("header-cell is missing"))?;
		let content_cell = mdl_grid.children[1]
			.element()
			.some_if_has_class("content-cell")
			.ok_or_else(|| anyhow!("content-cell is missing"))?;

		let item_category = header_cell
			.children
			.first()
			.and_then(|title_node| title_node.element())
			.and_then(|title| title.children.first())
			.and_then(|title_text_node| title_text_node.text())
			.ok_or_else(|| anyhow!("item category is missing"))?;

		let video_href = content_cell
			.children
			.iter()
			.find(|node| {
				if let Some(element) = node.element()
					&& element.name == "a"
				{
					true
				} else {
					false
				}
			})
			.and_then(|video_href_node| video_href_node.element())
			.ok_or_else(|| anyhow!("video_href_node is missing"))?;

		let video_url = video_href
			.attributes
			.get("href")
			.and_then(|option| option.as_ref())
			.ok_or_else(|| anyhow!("video URL is missing"))?
			.as_str();

		let video_title = video_href
			.children
			.first()
			.and_then(|video_title_node| video_title_node.text())
			.ok_or_else(|| anyhow!("video title is missing"))?;

		let timestamp_str = content_cell
			.children
			.get(content_cell.children.len() - 2)
			.and_then(|node| node.text())
			.ok_or_else(|| anyhow!("timestamp is missing"))?;

		let timestamp_str_no_timezone = timestamp_str
			.strip_suffix(" EDT")
			.ok_or_else(|| anyhow!("timestamp is missing the expected timezone suffix"))?
			.trim();
		let naive_timestamp =
			NaiveDateTime::parse_from_str(timestamp_str_no_timezone, "%b %e, %Y, %l:%M:%S %p")
				.with_context(|| "failed to parse the timestamp")?;

		let timestamp = naive_timestamp
			.and_local_timezone(FixedOffset::east_opt(-4 * 3600).expect("timezone offset is valid"))
			.earliest()
			.ok_or_else(|| {
				anyhow!("unable to convert the naive timestamp into a real timestamp")
			})?;

		println!("{item_category}: [{video_title}]({video_url}) @ {timestamp}");

		if item_category != "YouTube" {
			continue;
		}

		video_entries.push(VideoEntry {
			url: video_url,
			title: video_title,
			timestamp,
		});

		dbg!(&video_entries);
	}

	Ok(())
}

trait HasClass {
	fn some_if_has_class(&self, class: &str) -> Self;
}

impl HasClass for Option<&Element> {
	fn some_if_has_class(&self, class: &str) -> Self {
		self.and_then(|element| {
			element
				.classes
				.iter()
				.any(|class_name| class_name == class)
				.then_some(element)
		})
	}
}
