use headless_chrome::{types::PrintToPdfOptions, Browser};
use std::fs;
use std::path::PathBuf;
use std::str::FromStr;
use structopt::StructOpt;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "print-to-pdf",
    about = "A command line tool convert local HTML files to PDF",
    setting = structopt::clap::AppSettings::ColoredHelp
)]
struct Flags {
    /// Path to the HTML file
    #[structopt(long)]
    pub html_path: PathBuf,

    /// PDF filename
    #[structopt(long)]
    pub out: PathBuf,

    /// Scale percentage
    #[structopt(long)]
    pub scale: Option<f64>,

    /// Page layout
    #[structopt(long, possible_values = &["legal", "slideshow"], default_value = "legal", case_insensitive = true)]
    pub layout: PageLayout,
}

impl Flags {
    fn to_pdf_options(&self) -> PrintToPdfOptions {
        let mut base = match self.layout {
            PageLayout::Legal => default_pdf_options(),
            PageLayout::Slideshow => slideshow_pdf_options(),
        };

        if let Some(scale) = self.scale {
            base.scale = Some(scale);
        }

        base
    }
}

#[derive(Debug)]
enum PageLayout {
    Legal,
    Slideshow,
}

impl FromStr for PageLayout {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_ref() {
            "legal" => Ok(PageLayout::Legal),
            "slideshow" => Ok(PageLayout::Slideshow),
            v => Err(format!("Unknown layout: {}", v)),
        }
    }
}

fn main() -> anyhow::Result<()> {
    let flags = Flags::from_args();
    let pwd = std::env::current_dir()?;
    let path_to_html = std::fs::canonicalize(pwd.join(&flags.html_path))?;
    let file_path = format!("file://{}", path_to_html.as_os_str().to_str().unwrap());

    eprintln!("starting new browser");
    let browser = Browser::default()?;
    eprintln!("opening tab");
    let tab = browser.new_tab()?;
    eprintln!("navigating to {}", file_path);
    let temp = tab.navigate_to(&file_path)?.wait_until_navigated()?;
    eprintln!("loaded; now printing to PDF");

    let local_pdf = temp.print_to_pdf(Some(flags.to_pdf_options()))?;

    eprintln!("writing to disk");
    fs::write(flags.out, &local_pdf)?;

    eprintln!("PDF successfully created from local web page.");

    Ok(())
}

fn default_pdf_options() -> PrintToPdfOptions {
    PrintToPdfOptions {
        landscape: None,
        display_header_footer: None,
        print_background: Some(true),
        scale: Some(1.0),
        paper_width: Some(8.5),
        paper_height: Some(11.0),
        margin_top: None,
        margin_right: None,
        margin_bottom: None,
        margin_left: None,
        page_ranges: None,
        ignore_invalid_page_ranges: None,
        footer_template: None,
        header_template: None,
        prefer_css_page_size: Some(true),
        transfer_mode: None,
    }
}

fn slideshow_pdf_options() -> PrintToPdfOptions {
    let mut options = default_pdf_options();
    options.landscape = Some(true);
    options.paper_width = Some(16.0);
    options.paper_height = Some(9.0);

    options
}
