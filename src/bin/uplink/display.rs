use comms::publication::Publication;
use owo_colors::{OwoColorize, Stream, Style};
use ulid::Ulid;

const CONTENT_PREVIEW_CHARS: usize = 72;

pub fn print_publications(publications: &[Publication]) {
    if publications.is_empty() {
        println!("No publications found.");
        return;
    }
    let last = publications.len() - 1;
    for (i, publication) in publications.iter().enumerate() {
        print_header(publication);
        println!("  {}", content_preview(publication.content()));
        if i < last {
            println!();
        }
    }
}

pub fn print_publication(publication: &Publication) {
    print_header(publication);
    for line in publication.content().lines() {
        println!("  {line}");
    }
    println!();
    let mastodon = publication.mastodon_url().unwrap_or("(not posted)");
    let bluesky = publication.bluesky_url().unwrap_or("(not posted)");
    println!(
        "{} {mastodon}",
        "Mastodon:".if_supports_color(Stream::Stdout, |t| t.dimmed())
    );
    println!(
        "{} {bluesky}",
        "Bluesky: ".if_supports_color(Stream::Stdout, |t| t.dimmed())
    );
}

pub fn print_publish_success(publication: &Publication) {
    let header = format!("Published · {}", format_date(publication));
    let success_style = Style::new().green().bold();
    println!(
        "{}",
        header.if_supports_color(Stream::Stdout, |t| t.style(success_style))
    );
    for line in publication.content().lines() {
        println!("  {line}");
    }
    let id = publication.id().to_string();
    println!(
        "  ID: {}",
        id.if_supports_color(Stream::Stdout, |t| t.bold())
    );
}

pub fn print_delete_success(id: &Ulid) {
    let header = format!("Deleted · {id}");
    let success_style = Style::new().green().bold();
    println!(
        "{}",
        header.if_supports_color(Stream::Stdout, |t| t.style(success_style))
    );
}

fn print_header(publication: &Publication) {
    let id = publication.id().to_string();
    let sep_date = format!(" · {}", format_date(publication));
    println!(
        "{}{}",
        id.if_supports_color(Stream::Stdout, |t| t.bold()),
        sep_date.if_supports_color(Stream::Stdout, |t| t.dimmed()),
    );
}

fn format_date(publication: &Publication) -> String {
    format!(
        "{}",
        publication.pub_date().strftime("%Y/%m/%d - %H:%M (%Z)")
    )
}

fn content_preview(content: &str) -> String {
    let first_line = content.lines().next().unwrap_or("");
    let mut chars = first_line.chars();
    let preview: String = chars.by_ref().take(CONTENT_PREVIEW_CHARS).collect();
    if chars.next().is_some() {
        format!("{preview}…")
    } else {
        preview
    }
}
