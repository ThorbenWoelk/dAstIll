use super::{
    create_writing_flag, delete_writing_flag, list_recent_writing_flags, list_video_writing_flags,
    writing_flag_id_of_key,
};
use crate::db::Store;

#[test]
fn writing_flag_id_of_key_reads_the_file_name() {
    assert_eq!(
        writing_flag_id_of_key("writing-flags/user-1/17000000000001234.json"),
        Some(17000000000001234)
    );
    assert_eq!(
        writing_flag_id_of_key("writing-flags/user-1/notes.txt"),
        None
    );
}

#[tokio::test]
async fn writing_flags_are_listed_per_reader_and_video() {
    let store = Store::for_test().await;
    let mine = create_writing_flag(&store, "u1", "v1", "Bad line", "Vague", "", "")
        .await
        .unwrap();
    create_writing_flag(&store, "u1", "v2", "Other line", "Hype", "", "")
        .await
        .unwrap();
    create_writing_flag(&store, "u2", "v1", "Their line", "Wordy", "", "")
        .await
        .unwrap();

    let flags = list_video_writing_flags(&store, "u1", "v1").await.unwrap();
    assert_eq!(flags.len(), 1);
    assert_eq!(flags[0].id, mine.id);
    assert_eq!(flags[0].comment, "Vague");
}

#[tokio::test]
async fn only_the_reader_who_flagged_can_delete_it() {
    let store = Store::for_test().await;
    let flag = create_writing_flag(&store, "u1", "v1", "Bad line", "Vague", "", "")
        .await
        .unwrap();

    assert!(!delete_writing_flag(&store, "u2", flag.id).await.unwrap());
    assert!(delete_writing_flag(&store, "u1", flag.id).await.unwrap());
    assert!(
        list_video_writing_flags(&store, "u1", "v1")
            .await
            .unwrap()
            .is_empty()
    );
}

#[tokio::test]
async fn recent_writing_flags_come_from_every_reader_newest_first() {
    let store = Store::for_test().await;
    for (user, text) in [("u1", "first"), ("u2", "second"), ("u1", "third")] {
        create_writing_flag(&store, user, "v1", text, "note", "", "")
            .await
            .unwrap();
        tokio::time::sleep(std::time::Duration::from_millis(2)).await;
    }

    let recent = list_recent_writing_flags(&store, 2).await.unwrap();
    let texts = recent
        .iter()
        .map(|flag| flag.text.as_str())
        .collect::<Vec<_>>();
    assert_eq!(texts, ["third", "second"]);
}
