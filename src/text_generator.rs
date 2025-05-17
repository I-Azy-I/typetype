
use serde::{Deserialize, Serialize};
use std::fs::File;
use std::io::{BufReader, Read};
use std::error::Error;
use rand::prelude::*;


const PATH_LANGUAGES: &str = "languages/";
const PATH_TEXTS: &str = "texts/";

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
struct WordList {
    name: String,
    words: Vec<String>,
}


pub fn get_language(name: &str) -> Result<WordList, Box<dyn Error>> {
    let file = File::open(format!("{PATH_LANGUAGES}{name}"))?;
    let reader = BufReader::new(file);
    let word_list = serde_json::from_reader(reader)?;
    Ok(word_list)
}



pub fn get_text(name: &str) -> Result<String, Box<dyn Error>> {
    let file = File::open(format!("{PATH_TEXTS}{name}"))?;
    let mut reader = BufReader::new(file);
    let mut contents = String::new();
    reader.read_to_string(&mut contents);
    Ok(format_text(&contents))
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
            },
            
            // Case 2: Current character is punctuation
            c if punctuation_marks.contains(&c) => {
                // If next character is not a space, add one
                if let Some(&next) = iter.peek() {
                    if next != ' ' {
                        result.push(' ');
                    }
                }
            },
            
            // Case 3: Any other character - do nothing special
            _ => {}
        }
    }
    
    result
}



pub struct TextGenerator {
    word_list: WordList,
    rng: StdRng,
}
impl TextGenerator {
    pub fn from_language(name: &str, seed: Option<u64>) -> Self {
        let word_list = get_language(name).unwrap();
        let rng = if let Some(seed) = seed {
            StdRng::seed_from_u64(seed)
        } else {
            StdRng::from_os_rng()
        };
        TextGenerator {word_list, rng}
    }
    pub fn into_iter(self) -> TextGeneratorIntoIter {
        TextGeneratorIntoIter {
            word_list: self.word_list,
            rng: self.rng,
        }
    }
}

#[derive(Debug)]
pub struct TextGeneratorIntoIter {
    word_list: WordList,
    rng: StdRng,
}
impl TextGeneratorIntoIter {
    pub fn skip_n(mut self, n: usize) -> Self {
        for _ in 0..n {
            self.next();
        }
        self
    }
}
impl Iterator for TextGeneratorIntoIter {
    type Item = String;
    
    fn next(&mut self) -> Option<Self::Item> {
        self.word_list.words.choose(&mut self.rng).cloned()
    }
}