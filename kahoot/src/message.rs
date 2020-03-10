use serde::Deserialize;
use std::collections::HashMap;

#[derive(Debug)]
enum MessageType {
    GetReady,
    StartQuestion,
    GameOver,
    TimeUp,
    PlayAgain,
    AnswerSelected,
    AnswerResponse,
    RevealAnswer,
    StartQuiz,
    ResetController,
    SubmitFeedback,
    Feedback,
    RevealRanking,
    UsernameAccepted,
    UsernameRejected,

    GameBlockStart,
    GameBlockEnd,
    GameBlockAnswer,
}

impl MessageType {
    fn from_u64(n: u64) -> Option<Self> {
        match n {
            1 => Some(Self::GetReady),
            2 => Some(Self::StartQuestion),
            3 => Some(Self::GameOver),
            4 => Some(Self::TimeUp),
            5 => Some(Self::PlayAgain),
            6 => Some(Self::AnswerSelected),
            7 => Some(Self::AnswerResponse),
            8 => Some(Self::RevealAnswer),
            9 => Some(Self::StartQuiz),
            10 => Some(Self::ResetController),
            11 => Some(Self::SubmitFeedback),
            12 => Some(Self::Feedback),
            13 => Some(Self::RevealRanking),
            14 => Some(Self::UsernameAccepted),
            15 => Some(Self::UsernameRejected),

            43 => Some(Self::GameBlockStart),
            44 => Some(Self::GameBlockEnd),
            45 => Some(Self::GameBlockAnswer),
            _ => None,
        }
    }
}

#[derive(Debug)]
pub enum Message {
    GetReady {
        msg: GetReadyMessage,
    },
    StartQuestion {
        msg: StartQuestionMessage,
    },
    GameOver {
        msg: Box<GameOverMessage>,
        cid: String,
    },
    TimeUp {
        msg: TimeUpMessage,
    },
    PlayAgain {
        msg: PlayAgainMessage,
    },
    RevealAnswer {
        msg: Box<RevealAnswerMessage>,
    },
    StartQuiz {
        msg: StartQuizMessage,
    },
    Feedback {
        msg: FeedbackMessage,
    },
    RevealRanking {
        msg: RevealRankingMessage,
        cid: String,
    },
    UsernameAccepted {
        msg: UsernameAcceptedMessage,
        cid: String,
    },
    Unknown(serde_json::Value),
}

impl Message {
    pub fn from_value(value: serde_json::Value) -> Self {
        match value.get("type").and_then(|s| s.as_str()) {
            Some("message") => {}
            _ => {
                return Self::Unknown(value);
            }
        };

        let message_type = match value
            .get("id")
            .and_then(|v| MessageType::from_u64(v.as_u64()?))
        {
            Some(t) => t,
            None => {
                return Self::Unknown(value);
            }
        };

        let content = match value.get("content").and_then(|s| s.as_str()) {
            Some(s) => s,
            None => {
                return Self::Unknown(value);
            }
        };

        match message_type {
            MessageType::GetReady => {
                let msg = match serde_json::from_str::<GetReadyMessage>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };

                Self::GetReady { msg }
            }
            MessageType::StartQuestion => {
                let msg = match serde_json::from_str::<StartQuestionMessage>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };

                Self::StartQuestion { msg }
            }
            MessageType::GameOver => {
                let msg = match serde_json::from_str::<Box<GameOverMessage>>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };

                let cid = match value.get("cid").and_then(|s| s.as_str()) {
                    Some(s) => s.to_string(),
                    None => {
                        return Self::Unknown(value);
                    }
                };

                Self::GameOver { msg, cid }
            }
            MessageType::PlayAgain => {
                let msg = match serde_json::from_str::<PlayAgainMessage>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };

                Self::PlayAgain { msg }
            }
            MessageType::TimeUp => {
                let msg = match serde_json::from_str::<TimeUpMessage>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };

                Self::TimeUp { msg }
            }
            MessageType::RevealAnswer => {
                let msg = match serde_json::from_str::<Box<RevealAnswerMessage>>(content) {
                    Ok(c) => c,
                    Err(_e) => {
                        return Self::Unknown(value);
                    }
                };

                Self::RevealAnswer { msg }
            }
            MessageType::StartQuiz => {
                let msg = match serde_json::from_str::<StartQuizMessage>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };
                Self::StartQuiz { msg }
            }
            MessageType::Feedback => {
                let msg = match serde_json::from_str::<FeedbackMessage>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };
                Self::Feedback { msg }
            }
            MessageType::RevealRanking => {
                let msg = match serde_json::from_str::<RevealRankingMessage>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };

                let cid = match value.get("cid").and_then(|s| s.as_str()) {
                    Some(s) => s.to_string(),
                    None => {
                        return Self::Unknown(value);
                    }
                };

                Self::RevealRanking { msg, cid }
            }
            MessageType::UsernameAccepted => {
                let msg = match serde_json::from_str::<UsernameAcceptedMessage>(content) {
                    Ok(c) => c,
                    Err(_) => {
                        return Self::Unknown(value);
                    }
                };

                let cid = match value.get("cid").and_then(|s| s.as_str()) {
                    Some(s) => s.to_string(),
                    None => {
                        return Self::Unknown(value);
                    }
                };

                Self::UsernameAccepted { msg, cid }
            }
            _ => Self::Unknown(value),
        }
    }
}

#[derive(Deserialize, Debug)]
pub struct GetReadyMessage {
    #[serde(rename = "questionIndex")]
    pub question_index: usize,

    #[serde(rename = "gameBlockType")]
    game_block_type: String,

    #[serde(rename = "gameBlockLayout")]
    game_block_layout: Option<String>,

    #[serde(rename = "quizQuestionAnswers")]
    pub quiz_question_answers: Vec<usize>,

    #[serde(rename = "timeLeft")]
    pub time_left: usize,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct StartQuestionMessage {
    #[serde(rename = "questionIndex")]
    pub question_index: usize,

    #[serde(rename = "gameBlockType")]
    game_block_type: String,

    #[serde(rename = "quizQuestionAnswers")]
    pub quiz_question_answers: Vec<usize>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct GameOverMessage {
    pub rank: u64,
    pub cid: String,
    #[serde(rename = "correctCount")]
    pub correct_count: u64,

    #[serde(rename = "incorrectCount")]
    pub incorrect_count: u64,

    #[serde(rename = "isKicked")]
    pub is_kicked: bool,

    #[serde(rename = "isGhost")]
    pub is_ghost: bool,

    #[serde(rename = "unansweredCount")]
    pub unanswered_count: u64,

    #[serde(rename = "playerCount")]
    pub player_count: u64,

    #[serde(rename = "startTime")]
    pub start_time: u64,

    #[serde(rename = "quizId")]
    pub quiz_id: String,

    pub name: String,

    #[serde(rename = "totalScore")]
    pub total_score: u64,

    #[serde(rename = "hostId")]
    pub host_id: String,

    #[serde(rename = "challengeId")]
    challenge_id: Option<serde_json::Value>,

    #[serde(rename = "isOnlyNonPointGameBlockKahoot")]
    pub is_only_non_point_game_block_kahoot: bool,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct TimeUpMessage {
    #[serde(rename = "questionNumber")]
    pub question_number: u64,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Deserialize)]
pub struct PlayAgainMessage {
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct RevealAnswerMessage {
    #[serde(rename = "type")]
    quiz_type: String,

    pub choice: usize,

    #[serde(rename = "isCorrect")]
    pub is_correct: bool,

    pub text: String,

    #[serde(rename = "receivedTime")]
    pub received_time: u64,

    #[serde(rename = "pointsQuestion")]
    pub points_question: bool,

    pub points: u64,

    #[serde(rename = "correctAnswers")]
    pub correct_answers: Vec<String>,

    #[serde(rename = "totalScore")]
    pub total_score: u64,

    #[serde(rename = "pointsData")]
    pub points_data: RevealAnswerMessagePointsData,

    pub rank: u64,

    nemesis: Option<serde_json::Value>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct RevealAnswerMessagePointsData {
    #[serde(rename = "answerStreakPoints")]
    pub answer_streak_points: serde_json::Value,

    #[serde(rename = "questionPoints")]
    pub question_points: u64,

    #[serde(rename = "totalPointsWithBonuses")]
    pub total_points_with_bonuses: u64,

    #[serde(rename = "totalPointsWithoutBonuses")]
    pub total_points_without_bonuses: u64,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct RevealAnswerMessagePointsDataAnswerStreakPoints {
    #[serde(rename = "streakLevel")]
    pub streak_level: u64,

    #[serde(rename = "streakBonus")]
    pub streak_bonus: u64,

    #[serde(rename = "totalStreakPoints")]
    pub total_streak_points: u64,

    #[serde(rename = "previousStreakLevel")]
    pub previous_streak_level: u64,

    #[serde(rename = "previousStreakBonus")]
    pub previous_streak_bonus: u64,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct StartQuizMessage {
    #[serde(rename = "quizName")]
    pub quiz_name: String,

    #[serde(rename = "quizType")]
    quiz_type: String,

    #[serde(rename = "quizQuestionAnswers")]
    pub quiz_question_answers: Vec<usize>,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct FeedbackMessage {
    #[serde(rename = "quizType")]
    quiz_type: String,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct RevealRankingMessage {
    #[serde(rename = "podiumMedalType")]
    podium_medal_type: String,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Deserialize, Debug)]
pub struct UsernameAcceptedMessage {
    #[serde(rename = "playerName")]
    pub player_name: String,

    #[serde(rename = "quizType")]
    quiz_type: String,

    #[serde(rename = "playerV2")]
    pub player_v2: bool,

    #[serde(rename = "hostPrimaryUsage")]
    host_primary_usage: String,

    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}
