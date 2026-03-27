//! Upwork passive capture: `/api/v3/search/jobs` and GraphQL responses → normalized payloads.

use rusvel_core::domain::BrowserEvent;

/// URL patterns from PASTE-36: job search API and GraphQL.
pub fn matches_capture_url(url: &str) -> bool {
    let u = url.to_lowercase();
    u.contains("/api/v3/search/jobs")
        || u.contains("/api/graphql")
        || (u.contains("upwork.com") && u.contains("/graphql"))
}

/// Extract [`BrowserEvent::DataCaptured`] entries from a parsed JSON body.
pub fn events_from_response(
    url: &str,
    body: &serde_json::Value,
    tab_id: &str,
) -> Vec<BrowserEvent> {
    let mut events = Vec::new();
    if url.to_lowercase().contains("/api/graphql") {
        if let Some(ev) = parse_graphql_client_or_jobs(body, tab_id) {
            events.push(ev);
        }
        return events;
    }
    for job in extract_job_objects(body) {
        if let Some(norm) = normalize_job(&job) {
            events.push(BrowserEvent::DataCaptured {
                platform: "upwork".into(),
                kind: "job_listing".into(),
                data: norm,
                tab_id: tab_id.to_string(),
            });
        }
    }
    events
}

fn parse_graphql_client_or_jobs(body: &serde_json::Value, tab_id: &str) -> Option<BrowserEvent> {
    let data = body.get("data")?;
    if let Some(search) = data.pointer("/search") {
        if let Some(jobs) = search.get("jobs").or_else(|| search.get("results")) {
            let mut collected = Vec::new();
            if let Some(arr) = jobs.as_array() {
                for j in arr {
                    if let Some(norm) = normalize_job(j) {
                        collected.push(norm);
                    } else if let Some(inner) = j.get("job") {
                        if let Some(norm) = normalize_job(inner) {
                            collected.push(norm);
                        }
                    }
                }
            }
            if !collected.is_empty() {
                return Some(BrowserEvent::DataCaptured {
                    platform: "upwork".into(),
                    kind: "job_listing".into(),
                    data: serde_json::json!({ "jobs": collected }),
                    tab_id: tab_id.to_string(),
                });
            }
        }
    }
    if let Some(client) = data.get("client") {
        if let Some(norm) = normalize_client(client) {
            return Some(BrowserEvent::DataCaptured {
                platform: "upwork".into(),
                kind: "client_profile".into(),
                data: norm,
                tab_id: tab_id.to_string(),
            });
        }
    }
    if let Some(v) = data.pointer("/talentJobSearch/jobs") {
        let fake = serde_json::json!({ "jobs": v });
        return parse_graphql_client_or_jobs(
            &serde_json::json!({ "data": { "search": fake } }),
            tab_id,
        );
    }
    None
}

fn extract_job_objects(body: &serde_json::Value) -> Vec<serde_json::Value> {
    let mut out = Vec::new();
    if let Some(jobs) = body.get("jobs").and_then(|v| v.as_array()) {
        for j in jobs {
            if let Some(inner) = j.get("job") {
                out.push(inner.clone());
            } else {
                out.push(j.clone());
            }
        }
    }
    if let Some(results) = body.get("results").and_then(|v| v.as_array()) {
        for j in results {
            if let Some(inner) = j.get("job") {
                out.push(inner.clone());
            } else {
                out.push(j.clone());
            }
        }
    }
    out
}

fn normalize_job(job: &serde_json::Value) -> Option<serde_json::Value> {
    let title = job
        .get("title")
        .or_else(|| job.pointer("/info/title"))
        .and_then(|v| v.as_str())?;
    let ciphertext = job
        .get("ciphertext")
        .or_else(|| job.get("uid"))
        .or_else(|| job.get("id"))
        .and_then(|v| v.as_str());
    let url = ciphertext.map(|c| format!("https://www.upwork.com/jobs/~{c}"));
    let description = job
        .get("description")
        .or_else(|| job.pointer("/info/description"))
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    let budget = extract_budget(job);
    let skills = extract_skills(job);
    let posted_at = job
        .get("createdOn")
        .or_else(|| job.get("postedOn"))
        .or_else(|| job.get("publishTime"))
        .and_then(|v| v.as_str())
        .map(String::from);
    Some(serde_json::json!({
        "title": title,
        "url": url,
        "description": description,
        "budget": budget,
        "skills": skills,
        "posted_at": posted_at,
        "raw_ref": ciphertext,
    }))
}

fn normalize_client(client: &serde_json::Value) -> Option<serde_json::Value> {
    let name = client
        .get("name")
        .or_else(|| client.get("companyName"))
        .and_then(|v| v.as_str())?;
    let cid = client
        .get("ciphertext")
        .or_else(|| client.get("id"))
        .and_then(|v| v.as_str());
    let profile_url = cid.map(|c| format!("https://www.upwork.com/ag/offeror/{c}/"));
    Some(serde_json::json!({
        "name": name,
        "company": client.get("company").and_then(|v| v.as_str()),
        "total_spent": client.get("totalSpent").or_else(|| client.get("total_spent")),
        "rating": client.get("rating").or_else(|| client.get("feedbackScore")),
        "country": client.get("country").or_else(|| client.pointer("/location/country")),
        "profile_url": profile_url,
        "metadata": client,
    }))
}

fn extract_budget(job: &serde_json::Value) -> Option<String> {
    if let Some(b) = job.get("budget") {
        if let Some(amt) = b.get("amount").and_then(|v| v.as_f64()) {
            return Some(format!("${amt:.0}"));
        }
        if let Some(s) = b.as_str() {
            return Some(s.to_string());
        }
    }
    job.get("amount")
        .and_then(|v| v.as_f64())
        .map(|a| format!("${a:.0}"))
}

fn extract_skills(job: &serde_json::Value) -> Vec<String> {
    let mut skills = Vec::new();
    if let Some(attrs) = job.get("attrs").and_then(|v| v.as_array()) {
        for a in attrs {
            if let Some(s) = a.get("skill").and_then(|v| v.as_str()) {
                skills.push(s.to_string());
            }
        }
    }
    if skills.is_empty()
        && let Some(arr) = job.get("skills").and_then(|v| v.as_array())
    {
        for s in arr {
            if let Some(t) = s.as_str() {
                skills.push(t.to_string());
            } else if let Some(t) = s.get("name").and_then(|v| v.as_str()) {
                skills.push(t.to_string());
            }
        }
    }
    skills
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_v3_search_shape() {
        let body = serde_json::json!({
            "jobs": [
                {
                    "job": {
                        "title": "Rust API",
                        "ciphertext": "abc123",
                        "description": "Need Axum expert",
                        "budget": { "amount": 5000.0 },
                        "attrs": [{ "skill": "rust" }]
                    }
                }
            ]
        });
        let evs = events_from_response("https://www.upwork.com/api/v3/search/jobs", &body, "tab1");
        assert_eq!(evs.len(), 1);
        match &evs[0] {
            BrowserEvent::DataCaptured { kind, data, .. } => {
                assert_eq!(kind, "job_listing");
                assert_eq!(data["title"], "Rust API");
            }
            _ => panic!("expected DataCaptured"),
        }
    }
}
