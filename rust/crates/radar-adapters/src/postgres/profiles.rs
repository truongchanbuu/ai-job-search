use super::*;
use radar_application::profiles::*;

impl PgRepository {
    pub async fn current_profile(&self, fallback: &Profile) -> Result<Profile> {
        let row=sqlx::query("SELECT p.snapshot FROM owner_profile o LEFT JOIN profile_revisions p ON p.revision=o.active_revision WHERE o.singleton=true").fetch_one(&self.pool).await?;
        Ok(row
            .try_get::<Option<Json<Profile>>, _>("snapshot")?
            .map_or_else(|| fallback.clone(), |p| p.0))
    }
    pub async fn profile_version(&self, revision: &str) -> Result<Profile> {
        let row = sqlx::query("SELECT snapshot FROM profile_revisions WHERE revision=$1")
            .bind(revision)
            .fetch_optional(&self.pool)
            .await?
            .ok_or(StoreError::NotFound)?;
        Ok(row.try_get::<Json<Profile>, _>("snapshot")?.0)
    }
    pub async fn profile_view(&self, fallback: &Profile) -> Result<ProfileView> {
        let state=sqlx::query("SELECT o.revision,o.active_revision,p.snapshot FROM owner_profile o LEFT JOIN profile_revisions p ON p.revision=o.active_revision WHERE o.singleton=true").fetch_one(&self.pool).await?;
        let active: Option<String> = state.try_get("active_revision")?;
        let profile = state
            .try_get::<Option<Json<Profile>>, _>("snapshot")?
            .map_or_else(|| fallback.clone(), |p| p.0);
        let config = self.config().await?;
        let evidence =
            radar_application::ai::evidence(&profile, config.effective.settings().weekly_hours)
                .map_err(|_| StoreError::Invalid("invalid profile evidence".into()))?;
        let rows=sqlx::query("SELECT revision,created_at FROM profile_revisions ORDER BY created_at DESC,revision LIMIT 30").fetch_all(&self.pool).await?;
        let versions = rows
            .iter()
            .map(|r| {
                Ok(ProfileVersion {
                    revision: r.try_get("revision")?,
                    created_at: r.try_get("created_at")?,
                })
            })
            .collect::<Result<Vec<_>>>()?;
        Ok(ProfileView {
            revision: state.try_get("revision")?,
            source: if active.is_some() {
                "db-approved"
            } else {
                "deployment-markdown"
            }
            .into(),
            profile,
            evidence,
            versions,
        })
    }
    pub async fn save_profile(
        &self,
        fallback: &Profile,
        request: ProfileUpdate,
    ) -> Result<ProfileView> {
        if !request.confirmed_facts {
            return Err(StoreError::Invalid(
                "explicit confirmation of profile facts and sharing permissions required".into(),
            ));
        }
        let profile=crate::profile_from_documents(request.documents).map_err(|_|StoreError::Invalid("invalid profile modules or evidence; check front matter, skills, rates and portfolio references; values redacted".into()))?;
        let mut tx = self.begin("save_profile").await?;
        let revision: i64 = sqlx::query_scalar(
            "SELECT revision FROM owner_profile WHERE singleton=true FOR UPDATE",
        )
        .fetch_one(&mut *tx)
        .await?;
        if revision != request.expected_revision {
            return Err(StoreError::Conflict);
        }
        // Retain the original deployment profile for explicit recovery without writing Markdown files.
        for snapshot in [fallback, &profile] {
            sqlx::query("INSERT INTO profile_revisions(revision,snapshot) VALUES($1,$2) ON CONFLICT DO NOTHING").bind(&snapshot.revision).bind(Json(snapshot)).execute(&mut *tx).await?;
        }
        sqlx::query(
            "UPDATE owner_profile SET revision=revision+1,active_revision=$1 WHERE singleton=true",
        )
        .bind(&profile.revision)
        .execute(&mut *tx)
        .await?;
        audit(
            &mut tx,
            "owner",
            "profile_saved",
            "profile",
            json!({"profile_revision":profile.revision,"revision":revision+1}),
        )
        .await?;
        crate::database::commit(tx, &self.pool, "save_profile").await?;
        self.profile_view(fallback).await
    }
}
