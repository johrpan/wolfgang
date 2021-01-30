use crate::error::ServerError;
use actix_web::{get, web, HttpResponse};
use anyhow::{anyhow, Result};
use lazy_static::lazy_static;
use rand::seq::SliceRandom;
use serde::Serialize;
use std::collections::HashMap;
use std::sync::Mutex;

// TODO/INFO: These hardcoded questions are a placeholder for a future mechanism to autogenerate
// questions from the database. This will require a easily accissible web interface for Musicus.
// There may also be another, better solution. However, the current framework of question-answer
// pairs with randomly generated identifiers will most likely stay in place.

/// A question to identify users as human.
#[derive(Clone, Debug)]
struct Question {
    /// The question that will be sent to the client.
    pub question: &'static str,

    /// The answer that the client has to provide.
    pub answer: &'static str,
}

lazy_static! {
    /// All available captcha questions.
    static ref QUESTIONS: Vec<Question> = vec![
        Question {
            question: "In welchem Jahr wurde Johannes Brahms geboren?",
            answer: "1833",
        },
        Question {
            question: "In welchem Jahr ist Johannes Brahms gestorben?",
            answer: "1897",
        },
        Question {
            question: "In welchem Jahr wurde Ludwig van Beethoven geboren?",
            answer: "1770",
        },
        Question {
            question: "In welchem Jahr ist Ludwig van Beethoven gestorben?",
            answer: "1827",
        },
        Question {
            question: "In welchem Jahr wurde Claude Debussy geboren?",
            answer: "1862",
        },
        Question {
            question: "In welchem Jahr ist Claude Debussy gestorben?",
            answer: "1918",
        },
        Question {
            question: "In welchem Jahr wurde Sergei Rachmaninow geboren?",
            answer: "1873",
        },
        Question {
            question: "In welchem Jahr ist Sergei Rachmaninow gestorben?",
            answer: "1943",
        },
    ];
}

/// Response body data for captcha requests.
#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Captcha {
    pub id: String,
    pub question: String,
}

/// A generator and manager for captchas. This will keep track of the captchas that where created
/// for clients and delete them, once the client has tried to solve them.
pub struct CaptchaManager {
    captchas: Mutex<HashMap<String, &'static Question>>,
}

impl CaptchaManager {
    /// Create a new captcha manager.
    pub fn new() -> Self {
        Self {
            captchas: Mutex::new(HashMap::new()),
        }
    }

    /// Create a new captcha with a random ID.
    pub fn generate_captcha(&self) -> Result<Captcha> {
        let mut buffer = uuid::Uuid::encode_buffer();
        let id = uuid::Uuid::new_v4().to_simple().encode_lower(&mut buffer).to_owned();

        let question = QUESTIONS.choose(&mut rand::thread_rng())
            .ok_or_else(|| anyhow!("Failed to get random question!"))?;

        let captchas = &mut self.captchas.lock()
            .or_else(|_| Err(anyhow!("Failed to aquire lock!")))?;

        captchas.insert(id.clone(), question);

        let captcha = Captcha {
            id,
            question: question.question.to_owned(),
        };

        Ok(captcha)
    }

    /// Check whether the provided answer is correct and delete the captcha eitherway.
    pub fn check_captcha(&self, id: &str, answer: &str) -> Result<bool> {
        let captchas = &mut self.captchas.lock()
            .or_else(|_| Err(anyhow!("Failed to aquire lock!")))?;

        let question = captchas.get(id);

        let result = if let Some(question) = question {
            let result = answer == question.answer;
            captchas.remove(id);
            result
        } else {
            false
        };

        Ok(result)
    }
}

/// Request a new captcha.
#[get("/captcha")]
pub async fn get_captcha(manager: web::Data<CaptchaManager>) -> Result<HttpResponse, ServerError> {
    let manager = manager.into_inner();
    let captcha = manager.generate_captcha()?;

    Ok(HttpResponse::Ok().json(captcha))
}

