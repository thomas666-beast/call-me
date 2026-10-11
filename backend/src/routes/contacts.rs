use crate::auth::rbac::{CanReadContacts, CanWriteContacts};
use crate::auth::middleware::AuthUser;
use crate::models::contact::{
    normalize_phone, Contact, ContactCallDetail, ContactCallRow, ContactInput,
    ContactQuery, ContactWithCalls,
};
use crate::AppState;
use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    routing::get,
    Extension, Json, Router,
};
use uuid::Uuid;

pub fn router() -> Router<AppState> {
    Router::new()
        .route("/contacts", get(list_contacts).post(create_contact))
        .route(
            "/contacts/:id",
            get(get_contact).put(update_contact).delete(delete_contact),
        )
        .route("/contacts/lookup/:number", get(lookup_by_number))
}

async fn list_contacts(
    State(state): State<AppState>,
    _perm: CanReadContacts,
    Query(q): Query<ContactQuery>,
) -> Result<Json<Vec<Contact>>, StatusCode> {
    let limit = q.limit.clamp(1, 500);
    let offset = q.offset.max(0);

    let rows: Vec<Contact> = sqlx::query_as(
        "SELECT * FROM contacts
         WHERE ($1::text IS NULL
                OR display_name ILIKE '%' || $1 || '%'
                OR company      ILIKE '%' || $1 || '%'
                OR notes        ILIKE '%' || $1 || '%'
                OR phone_number ILIKE '%' || $1 || '%'
                OR (
                    regexp_replace($1, '[^0-9]', '', 'g') <> ''
                    AND regexp_replace(phone_number, '[^0-9]', '', 'g')
                        LIKE '%' || regexp_replace($1, '[^0-9]', '', 'g') || '%'
                ))
           AND ($2::text IS NULL OR $2 = ANY(tags))
         ORDER BY display_name ASC
         LIMIT $3 OFFSET $4",
    )
    .bind(q.search.as_deref())
    .bind(q.tag.as_deref())
    .bind(limit)
    .bind(offset)
    .fetch_all(&state.pool)
    .await
    .map_err(|e| {
        tracing::error!("contacts list failed: {:#}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(rows))
}

async fn get_contact(
    State(state): State<AppState>,
    _perm: CanReadContacts,
    Path(id): Path<Uuid>,
) -> Result<Json<ContactWithCalls>, StatusCode> {
    let contact: Contact = sqlx::query_as("SELECT * FROM contacts WHERE id = $1")
        .bind(id)
        .fetch_optional(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?
        .ok_or(StatusCode::NOT_FOUND)?;

    let calls: Vec<ContactCallRow> = sqlx::query_as(
        "SELECT id, calldate, src, dst, duration, billsec, disposition,
                uniqueid, linkedid
         FROM cdr
         WHERE src = $1 OR dst = $1
         ORDER BY calldate DESC
         LIMIT 100",
    )
    .bind(&contact.phone_number)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    let calls = enrich_calls_with_notes(&state, calls).await?;
    Ok(Json(ContactWithCalls { contact, calls }))
}

async fn lookup_by_number(
    State(state): State<AppState>,
    _perm: CanReadContacts,
    Path(number): Path<String>,
) -> Result<Json<Option<Contact>>, StatusCode> {
    let normalized = normalize_phone(&number);
    let row: Option<Contact> =
        sqlx::query_as("SELECT * FROM contacts WHERE phone_number = $1")
            .bind(&normalized)
            .fetch_optional(&state.pool)
            .await
            .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(Json(row))
}

async fn create_contact(
    State(state): State<AppState>,
    _perm: CanWriteContacts,
    Extension(AuthUser(claims)): Extension<AuthUser>,
    Json(input): Json<ContactInput>,
) -> Result<Json<Contact>, StatusCode> {
    validate(&input)?;
    let normalized = normalize_phone(&input.phone_number);
    let created_by: Option<Uuid> = claims.sub.parse().ok();

    let row: Contact = sqlx::query_as(
        "INSERT INTO contacts
            (id, phone_number, display_name, email, company, notes, tags, created_by)
         VALUES ($1, $2, $3, $4, $5, $6, $7, $8)
         RETURNING *",
    )
    .bind(Uuid::new_v4())
    .bind(&normalized)
    .bind(&input.display_name)
    .bind(&input.email)
    .bind(&input.company)
    .bind(&input.notes)
    .bind(&input.tags)
    .bind(created_by)
    .fetch_one(&state.pool)
    .await
    .map_err(map_sqlx_error)?;

    Ok(Json(row))
}

async fn update_contact(
    State(state): State<AppState>,
    _perm: CanWriteContacts,
    Path(id): Path<Uuid>,
    Json(input): Json<ContactInput>,
) -> Result<Json<Contact>, StatusCode> {
    validate(&input)?;
    let normalized = normalize_phone(&input.phone_number);

    let row: Contact = sqlx::query_as(
        "UPDATE contacts SET
            phone_number = $2,
            display_name = $3,
            email = $4,
            company = $5,
            notes = $6,
            tags = $7,
            updated_at = now()
         WHERE id = $1
         RETURNING *",
    )
    .bind(id)
    .bind(&normalized)
    .bind(&input.display_name)
    .bind(&input.email)
    .bind(&input.company)
    .bind(&input.notes)
    .bind(&input.tags)
    .fetch_optional(&state.pool)
    .await
    .map_err(map_sqlx_error)?
    .ok_or(StatusCode::NOT_FOUND)?;

    Ok(Json(row))
}

async fn delete_contact(
    State(state): State<AppState>,
    _perm: CanWriteContacts,
    Path(id): Path<Uuid>,
) -> Result<StatusCode, StatusCode> {
    let result = sqlx::query("DELETE FROM contacts WHERE id = $1")
        .bind(id)
        .execute(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    if result.rows_affected() == 0 {
        return Err(StatusCode::NOT_FOUND);
    }
    Ok(StatusCode::NO_CONTENT)
}

fn validate(input: &ContactInput) -> Result<(), StatusCode> {
    let normalized = normalize_phone(&input.phone_number);
    if normalized.is_empty() || normalized.len() > 20 {
        return Err(StatusCode::BAD_REQUEST);
    }
    if input.display_name.trim().is_empty() {
        return Err(StatusCode::BAD_REQUEST);
    }
    if let Some(email) = &input.email {
        if !email.is_empty() && !email.contains('@') {
            return Err(StatusCode::BAD_REQUEST);
        }
    }
    Ok(())
}

fn map_sqlx_error(e: sqlx::Error) -> StatusCode {
    if let sqlx::Error::Database(db) = &e {
        if db.is_unique_violation() {
            return StatusCode::CONFLICT;
        }
    }
    StatusCode::INTERNAL_SERVER_ERROR
}
/// Load notes for a set of calls in one query, then attach to each call.
pub async fn enrich_calls_with_notes(
    state: &AppState,
    calls: Vec<ContactCallRow>,
) -> Result<Vec<ContactCallDetail>, StatusCode> {
    use crate::models::call_note::NoteRow;
    use std::collections::{HashMap, HashSet};

    if calls.is_empty() {
        return Ok(Vec::new());
    }

    let uniqueids: Vec<String> = calls.iter().map(|c| c.uniqueid.clone()).collect();
    let linkedids: Vec<String> = calls.iter().map(|c| c.linkedid.clone()).collect();

    let note_rows: Vec<NoteRow> = sqlx::query_as(
        "SELECT n.id, n.call_uniqueid, n.call_linkedid, n.author_id,
                COALESCE(u.username, '(deleted)') AS author_username,
                n.note, n.created_at, n.updated_at
         FROM call_notes n
         LEFT JOIN users u ON u.id = n.author_id
         WHERE n.call_uniqueid = ANY($1) OR n.call_linkedid = ANY($2)
         ORDER BY n.created_at ASC",
    )
    .bind(&uniqueids)
    .bind(&linkedids)
    .fetch_all(&state.pool)
    .await
    .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Index notes by both uniqueid and linkedid.
    let mut by_key: HashMap<String, Vec<crate::models::call_note::PublicCallNote>> = HashMap::new();
    for row in note_rows {
        let public = row.clone().into_public();
        by_key
            .entry(row.call_uniqueid.clone())
            .or_default()
            .push(public.clone());
        if !row.call_linkedid.is_empty() && row.call_linkedid != row.call_uniqueid {
            by_key
                .entry(row.call_linkedid.clone())
                .or_default()
                .push(public);
        }
    }

    let result = calls
        .into_iter()
        .map(|c| {
            let mut notes: Vec<crate::models::call_note::PublicCallNote> = Vec::new();
            let mut seen: HashSet<Uuid> = HashSet::new();
            if let Some(v) = by_key.get(&c.uniqueid) {
                for n in v {
                    if seen.insert(n.id) {
                        notes.push(n.clone());
                    }
                }
            }
            if let Some(v) = by_key.get(&c.linkedid) {
                for n in v {
                    if seen.insert(n.id) {
                        notes.push(n.clone());
                    }
                }
            }
            ContactCallDetail {
                id: c.id,
                calldate: c.calldate,
                src: c.src,
                dst: c.dst,
                duration: c.duration,
                billsec: c.billsec,
                disposition: c.disposition,
                uniqueid: c.uniqueid,
                linkedid: c.linkedid,
                notes,
            }
        })
        .collect();

    Ok(result)
}