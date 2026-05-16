// Uses
use std::{borrow::Cow, env::args, fs::File, io::Read};

use anyhow::{Context, Result as AnyhowResult, anyhow};
use chrono::{DateTime, Utc};
use reflection::Reflection;
use reflection_derive::Reflection;
use serde::{Deserialize, Serialize};
use serde_json::from_str as parse_from_json_str;
use tsv::{Config as TsvConfig, to_string as to_tsv_str};

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[allow(dead_code)]
struct HistoryEntry<'a> {
	header:            Cow<'a, str>,
	title:             Cow<'a, str>,
	title_url:         Cow<'a, str>,
	#[serde(default)]
	subtitles:         Vec<SubtitleEntry<'a>>,
	time:              DateTime<Utc>,
	products:          Vec<Cow<'a, str>>,
	#[serde(default)]
	details:           Vec<DetailEntry<'a>>,
	activity_controls: Vec<Cow<'a, str>>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[allow(dead_code)]
struct SubtitleEntry<'a> {
	name: Cow<'a, str>,
	url:  Cow<'a, str>,
}

#[derive(Deserialize, Debug)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
#[allow(dead_code)]
struct DetailEntry<'a> {
	name: Cow<'a, str>,
}

#[derive(Serialize, Reflection, Debug)]
struct OutputEntry<'a> {
	url:              &'a str,
	timestamp_millis: String,
	transition_type:  &'static str,
	title:            String,
}

fn main() -> AnyhowResult<()> {
	let path = args()
		.nth(1)
		.ok_or_else(|| anyhow!("one argument is required - the path to the file"))?;

	let mut file_contents = String::new();
	let mut file = File::options()
		.read(true)
		.write(true)
		.open(path.as_str())
		.with_context(|| "unable to open file")?;
	file.read_to_string(&mut file_contents)
		.with_context(|| "unable to read file")?;

	let history_entries: Vec<HistoryEntry> = parse_from_json_str(file_contents.as_str())
		.with_context(|| "deserialising from JSON failed")?;

	let mut output_entries = Vec::with_capacity(history_entries.len());
	for history_entry in &history_entries {
		if history_entry.header.is_empty() {
			return Err(anyhow!("history entry header is empty"));
		}

		let video_title = history_entry
			.title
			.strip_prefix("Watched ")
			.or_else(|| history_entry.title.strip_prefix("Viewed "))
			.ok_or_else(|| anyhow!("history entry title does not start with an expected prefix"))?
			.trim();

		let channel_name = history_entry
			.subtitles
			.first()
			.map(|subtitle_entry| subtitle_entry.name.trim());

		let constructed_title = if let Some(channel_name) = channel_name {
			format!(
				"{video_title} - {channel_name} - {} (from watch history)",
				history_entry.header
			)
		} else {
			format!(
				"{video_title} - {} (from watch history)",
				history_entry.header
			)
		};

		let output_entry = OutputEntry {
			url:              history_entry.title_url.as_ref(),
			timestamp_millis: format!("U{}", history_entry.time.timestamp_millis()),
			transition_type:  "link",
			title:            constructed_title,
		};

		output_entries.push(output_entry);
	}

	let output_tsv = to_tsv_str(
		&output_entries,
		TsvConfig::make_config(false, "()".to_owned(), "1".to_owned(), "0".to_owned())
			.with_context(|| "TSV config is invalid")?,
	)
	.with_context(|| "serialising to TSV failed")?;

	println!("{}", output_tsv);

	Ok(())
}
