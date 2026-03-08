//! This is the arxiv module, which defines the data structures for arXiv papers.

#[derive(Debug)]
/// This struct represents the arXiv paper entry in search results.
pub struct ArxivPaperEntry {
    pub id: String,
    pub title: String,
    pub authors: Vec<String>,
    pub abstract_text: String,
}

#[derive(Debug)]
/// This struct represents the detailed information of an arXiv paper.
pub struct ArxivPaper {
    pub entry: ArxivPaperEntry,
    pub abstract_text: String,
    pub commited_time: String,
    /// The path to the cached PDF file, if it exists.
    pub pdf_cache_path: Option<String>,
    /// The path to the cached source file, if it exists.
    pub src_cache_path: Option<String>,
}

impl ArxivPaperEntry {
    /// Create a new ArxivPaperEntry.
    pub fn new(id: String, title: String, authors: Vec<String>, abstract_text: String) -> Self {
        Self {
            id,
            title,
            authors,
            abstract_text,
        }
    }

    pub fn get_arxiv_url(&self) -> String {
        format!("https://arxiv.org/abs/{}", self.id)
    }

    pub fn get_pdf_url(&self) -> String {
        format!("https://arxiv.org/pdf/{}.pdf", self.id)
    }

    pub fn get_src_url(&self) -> String {
        format!("https://arxiv.org/src/{}", self.id)
    }
}

impl ArxivPaper {
    /// Create a new ArxivPaper.
    pub fn new(entry: ArxivPaperEntry, abstract_text: String, commited_time: String) -> Self {
        Self {
            entry,
            abstract_text,
            commited_time,
            pdf_cache_path: None,
            src_cache_path: None,
        }
    }
}
