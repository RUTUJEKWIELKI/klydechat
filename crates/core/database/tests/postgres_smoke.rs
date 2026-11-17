#![cfg(feature = "postgres")]

use revolt_database::{Account, DatabaseInfo, Member, MemberCompositeKey, PartialMember, PartialUser, RelationshipStatus, Server, Session, User};
use revolt_result::Result;
use serde_json::json;

/// Runs a smoke test against a live PostgreSQL instance.
///
/// Skipped unless `POSTGRES_SMOKE_URI` is set, e.g.
/// `POSTGRES_SMOKE_URI=postgres://postgres:postgres@localhost/revolt cargo test -p revolt-database --test postgres_smoke`
#[tokio::test]
async fn postgres_driver_smoke() -> Result<()> {
    let Ok(uri) = std::env::var("POSTGRES_SMOKE_URI") else {
        eprintln!("Skipping: POSTGRES_SMOKE_URI is not set.");
        return Ok(());
    };

    let db = DatabaseInfo::Postgres { uri }
        .connect()
        .await
        .expect("Failed to connect to PostgreSQL.");

    db.migrate_database().await.ok();

    let user_a = User::create(&db, "smokeuser_a".to_string(), None::<String>, None::<PartialUser>).await?;
    let user_b = User::create(&db, "smokeuser_b".to_string(), None::<String>, None::<PartialUser>).await?;

    assert_eq!(user_b.discriminator.len(), 4);

    let fetched = db.fetch_user(&user_a.id).await?;
    assert_eq!(fetched.username, "smokeuser_a");

    let by_username = db
        .fetch_user_by_username("SMOKEUSER_A", &user_a.discriminator)
        .await?;
    assert_eq!(by_username.id, user_a.id);

    let discriminators = db.fetch_discriminators_in_use("smokeuser_a").await?;
    assert_eq!(discriminators.len(), 1);

    let users = db
        .fetch_users(&[user_a.id.clone(), user_b.id.clone()])
        .await?;
    assert_eq!(users.len(), 2);

    db.set_relationship(&user_a.id, &user_b.id, &RelationshipStatus::Friend)
        .await?;
    let fetched = db.fetch_user(&user_a.id).await?;
    assert_eq!(
        fetched.relationship_with(&user_b.id),
        RelationshipStatus::Friend
    );

    db.pull_relationship(&user_a.id, &user_b.id).await?;
    let fetched = db.fetch_user(&user_a.id).await?;
    assert_eq!(
        fetched.relationship_with(&user_b.id),
        RelationshipStatus::None
    );

    db.set_relationship(&user_a.id, &user_b.id, &RelationshipStatus::Friend)
        .await?;
    db.clear_user_relationships(&user_b.id, vec![user_a.id.clone()])
        .await?;
    let fetched = db.fetch_user(&user_a.id).await?;
    assert_eq!(
        fetched.relationship_with(&user_b.id),
        RelationshipStatus::None
    );

    let account: Account = serde_json::from_value(json!({
        "_id": user_a.id,
        "email": "Smoke.User@Example.COM",
        "email_normalised": "smoke.user@example.com",
        "password": "argon2-hash",
        "verification": { "status": "Verified" },
        "mfa": {}
    }))
    .expect("Failed to build account fixture.");

    db.save_account(&account).await?;

    let fetched = db
        .fetch_account_by_normalised_email("SMOKE.USER@example.com")
        .await?
        .expect("account should exist");
    assert_eq!(fetched.id, user_a.id);

    let bumped = db.bump_lockout_count(&user_a.id).await?;
    assert_eq!(bumped.lockout.as_ref().map(|l| l.attempts), Some(1));

    let session = Session {
        id: "session_smoke".to_string(),
        user_id: user_a.id.clone(),
        token: "smoke-token".to_string(),
        name: "Smoke".to_string(),
        last_seen: revolt_database::iso8601_timestamp::Timestamp::now_utc(),
        origin: None,
        subscription: None,
    };

    db.save_session(&session).await?;
    let fetched = db.fetch_session_by_token("smoke-token").await?;
    assert_eq!(fetched.id, "session_smoke");
    db.delete_session("session_smoke").await?;
    assert!(db.fetch_session_by_token("smoke-token").await.is_err());

    let server: Server = serde_json::from_value(json!({
        "_id": "server_smoke",
        "owner": user_a.id,
        "name": "Smoke Server",
        "channels": [],
        "default_permissions": 0
    }))
    .expect("Failed to build server fixture.");

    db.insert_server(&server).await?;
    let fetched = db.fetch_server("server_smoke").await?;
    assert_eq!(fetched.name, "Smoke Server");
    assert!(db.fetch_owned_servers(&user_a.id).await?.len() == 1);

    let member = Member {
        id: MemberCompositeKey {
            server: "server_smoke".to_string(),
            user: user_b.id.clone(),
        },
        joined_at: revolt_database::iso8601_timestamp::Timestamp::now_utc(),
        nickname: None,
        pronouns: None,
        avatar: None,
        roles: vec!["role_1".to_string()],
        timeout: None,
        can_publish: true,
        can_receive: true,
    };

    assert!(db.insert_or_merge_member(&member).await?.is_none());

    let merged = {
        db.update_member(
            &member.id,
            &PartialMember {
                timeout: Some(
                    revolt_database::iso8601_timestamp::Timestamp::now_utc()
                        + std::time::Duration::from_secs(3600),
                ),
                ..Default::default()
            },
            vec![],
        )
        .await?;
        db.soft_delete_member(&member.id).await?;
        db.insert_or_merge_member(&member).await?
    };
    assert!(merged.is_some(), "rejoining a soft-deleted member should merge");
    assert!(merged.as_ref().unwrap().nickname.is_none());

    assert_eq!(db.fetch_member_count("server_smoke").await?, 1);
    assert_eq!(db.fetch_server_count(&user_b.id).await?, 1);

    let member_with_role = Member {
        id: MemberCompositeKey {
            server: "server_smoke".to_string(),
            user: user_a.id.clone(),
        },
        joined_at: revolt_database::iso8601_timestamp::Timestamp::now_utc(),
        nickname: None,
        pronouns: None,
        avatar: None,
        roles: vec!["role_1".to_string()],
        timeout: None,
        can_publish: true,
        can_receive: true,
    };
    db.insert_or_merge_member(&member_with_role).await?;

    let mut generator = db
        .fetch_all_members_with_roles_chunked("server_smoke", &["role_1".to_string()])
        .await?;
    let mut seen = 0;
    while let Some(batch_member) = generator.next().await {
        assert_eq!(batch_member.roles, vec!["role_1".to_string()]);
        seen += 1;
    }
    assert_eq!(seen, 1, "member generator should yield the role holder");

    db.acknowledge_channels(&user_b.id, &["channel_smoke".to_string()])
        .await?;
    let unread = db
        .fetch_unread(&user_b.id, "channel_smoke")
        .await?
        .expect("unread should exist after acknowledgement");
    assert_eq!(unread.id.channel, "channel_smoke");

    db.add_mention_to_unread("channel_smoke", &user_b.id, &["msg_1".to_string()])
        .await?;
    let mentions = db.fetch_unread_mentions(&user_b.id).await?;
    assert_eq!(mentions.len(), 1);
    assert_eq!(mentions[0].mentions.as_deref(), Some(&["msg_1".to_string()][..]));

    db.delete_user(&user_b.id).await?;
    assert!(db.fetch_user(&user_b.id).await.is_err());

    Ok(())
}
