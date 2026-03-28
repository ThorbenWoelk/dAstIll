use serde::Deserialize;

use crate::db;
use crate::models::{
    Channel, Highlight, HighlightChannelGroup, HighlightVideoGroup, Summary, Transcript, Video,
};
use crate::services::search::SearchSourceKind;


include!("frag_01.rs");
include!("frag_02.rs");
include!("frag_03.rs");
