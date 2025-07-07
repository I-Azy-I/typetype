use std::path::Path;

use crate::config::{PATH_LANGUAGES, PATH_TEXTS};
use log::error;
use rand::prelude::*;
use serde::{Deserialize, Serialize};
use tokio::fs::{self, File};
use tokio::io::AsyncReadExt;
use tokio::io::BufReader;

#[derive(Debug, Copy, Clone)]
enum ErrorTextGenerator {}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
struct WordList {
    name: String,
    words: Vec<String>,
}

async fn get_language(path_source: impl AsRef<Path>) -> Option<WordList> {
    let path_source = path_source.as_ref();
    match File::open(path_source).await {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut contents = Vec::new();
            if let Err(e) = reader.read_to_end(&mut contents).await {
                error!(
                    "Failed to read file '{}': {}",
                    path_source.to_string_lossy(),
                    e
                );
                return None;
            }
            match serde_json::from_slice(&contents) {
                Ok(word_list) => Some(word_list),
                Err(e) => {
                    error!(
                        "Failed to parse JSON for '{}': {}",
                        path_source.to_string_lossy(),
                        e
                    );
                    None
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to open file '{}': {}",
                path_source.to_string_lossy(),
                e
            );
            None
        }
    }
}

pub async fn get_text(path_source: impl AsRef<Path>) -> Option<String> {
    let path_source = path_source.as_ref();
    match File::open(path_source).await {
        Ok(file) => {
            let mut reader = BufReader::new(file);
            let mut contents = String::new();
            match reader.read_to_string(&mut contents).await {
                Ok(_) => Some(format_text(&contents)),
                Err(e) => {
                    error!(
                        "Failed to read file '{}': {}",
                        path_source.to_string_lossy(),
                        e
                    );
                    None
                }
            }
        }
        Err(e) => {
            error!(
                "Failed to open file '{}': {}",
                path_source.to_string_lossy(),
                e
            );
            None
        }
    }
}

fn format_text(input: &str) -> String {
    // Step 1: Replace newlines with spaces
    let without_newlines = input.replace('\n', " ");

    // Step 2 & 3: Handle space deduplication and punctuation spacing in a single pass
    let punctuation_marks = ['.', ',', '!', '?', ':', ';'];
    let mut result = String::with_capacity(without_newlines.len());
    let mut iter = without_newlines.chars().peekable();
    while let Some(c) = iter.next() {
        // Add the current character
        result.push(c);

        match c {
            // Case 1: Current character is a space
            ' ' => {
                // Skip any subsequent spaces
                while iter.peek() == Some(&' ') {
                    iter.next();
                }
            }

            // Case 2: Current character is punctuation
            c if punctuation_marks.contains(&c) => {
                // If next character is not a space, add one
                if let Some(&next) = iter.peek() {
                    if next != ' ' {
                        result.push(' ');
                    }
                }
            }

            // Case 3: Any other character - do nothing special
            _ => {}
        }
    }

    result
}

#[derive(Debug, Clone)]
pub struct TextGenerator {
    word_list: WordList,
    rng: StdRng,
    seed: u64,
}
impl TextGenerator {
    pub async fn from_language(path_source: impl AsRef<Path>, seed: Option<u64>) -> Option<Self> {
        let word_list = get_language(path_source).await?;
        let (rng, seed) = if let Some(seed) = seed {
            (StdRng::seed_from_u64(seed), seed)
        } else {
            let mut rng = rand::rng();
            let seed: u64 = rng.random();
            (StdRng::seed_from_u64(seed), seed)
        };
        Some(TextGenerator {
            word_list,
            rng,
            seed,
        })
    }
    pub fn seed(&self) -> u64 {
        self.seed
    }
    pub fn iter<'a>(&'a mut self) -> TextGeneratorIter<'a> {
        TextGeneratorIter {
            word_list: &self.word_list,
            rng: &mut self.rng,
        }
    }
}

#[derive(Debug)]
pub struct TextGeneratorIter<'a> {
    word_list: &'a WordList,
    rng: &'a mut StdRng,
}
impl<'a> TextGeneratorIter<'a> {
    pub fn skip_n(mut self, n: usize) -> Self {
        for _ in 0..n {
            self.next();
        }
        self
    }
}
impl<'a> Iterator for TextGeneratorIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        self.word_list.words.choose(&mut self.rng).cloned()
    }
}

pub async fn fetch_languages_name() -> Vec<String> {
    list_files_in_folder(PATH_LANGUAGES.to_string()).await
}

pub async fn fetch_texts_name() -> Vec<String> {
    list_files_in_folder(PATH_TEXTS.to_string()).await
}

async fn list_files_in_folder(path: String) -> Vec<String> {
    let mut files = Vec::new();

    match fs::read_dir(&path).await {
        Ok(mut dir) => {
            while let Some(entry_result) = dir.next_entry().await.unwrap_or_else(|e| {
                error!("Failed to read directory entry: {e}");
                None
            }) {
                match entry_result.file_type().await {
                    Ok(file_type) if file_type.is_file() => {
                        match entry_result.file_name().into_string() {
                            Ok(name) => files.push(name),
                            Err(os_str) => error!("Invalid UTF-8 in filename: {:?}", os_str),
                        }
                    }
                    Ok(_) => {} // skip directories or other non-files
                    Err(e) => error!("Failed to get file type: {e}"),
                }
            }
        }
        Err(e) => error!("Failed to open directory '{}': {}", path, e),
    }

    files
}
