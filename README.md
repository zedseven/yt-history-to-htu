# `yt-history-to-htu`

[![License: MIT/Apache-2.0](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue.svg)](LICENSE-MIT)

A small program for converting YouTube watch history into TSV format for importing into [History Trends Unlimited](https://chromewebstore.google.com/detail/history-trends-unlimited/pnmchffiealhkdloeffcdnbgdnedheme?pli=1).

To download your YouTube watch history in a format that is usable by this program, visit [Google Takeout](https://takeout.google.com/) and select the *YouTube and YouTube Music* section. Under that section, click the button that says *Multiple formats*, scroll down, and for the *history* option, change the format to *JSON*.

The HTML format that Google selects by default for the YouTube watch history is stupid - it's nearly 3x larger in file size, significantly more difficult to parse, and it actually discards information. For example, the timestamp is written in a natural-language format that uses your local timezone, not UTC, and it discards sub-second information.

This tool originally started with parsing the HTML (I didn't know the JSON format was available at the time), and just the parsing step (using an off-the-shelf HTML parser) took over 16 hours to parse the 70 MB HTML file that Google gave me. Parsing the 24 MB JSON file with Serde does it in under a second.

## Usage

```bash
yt-history-to-htu watch-history.json > watch-history.tsv
```

Note that HTU may end up with duplicate entries for videos that appear in your browser history and your watch history. I can't do anything about that, since the browser extension is closed-source, and I don't know how it detects duplicates.

The titles for the history entries are a slightly custom format, to include the video title, channel name, and platform (YouTube or YouTube Music). I opted to go this way to include as much useful information as possible from the watch history, and because the real page titles when visiting the YouTube website contain some numbers at the beginning, so it wasn't possible to match the real format exactly anyway.

## Project License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or
  http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in `yt-history-to-htu` by you, as defined in the Apache-2.0 license,
shall be dual licensed as above, without any additional terms or conditions.
