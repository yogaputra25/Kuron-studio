use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BubbleResult {
    pub original: String,
    pub reading: String,
    pub translated: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FullBubble {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
    pub original: String,
    pub reading: String,
    pub translated: String,
}

/// Strip triple-backtick fences (incl ```json tag).
pub fn strip_markdown(s: &str) -> &str {
    let t = s.trim();
    if t.starts_with("```") {
        let after_open = match t.find('\n') {
            Some(i) => &t[i + 1..],
            None => return t,
        };
        let end = after_open.rfind("```").map(|i| &after_open[..i]).unwrap_or(after_open);
        return end.trim();
    }
    t
}

fn bubble_result(v: &serde_json::Value) -> BubbleResult {
    BubbleResult {
        original: v.get("original").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        reading: v.get("reading").and_then(|x| x.as_str()).unwrap_or("").to_string(),
        translated: v.get("translated").and_then(|x| x.as_str()).unwrap_or("").to_string(),
    }
}

/// Object with 1-based numeric keys; also accepts {"bubbles":[...]} wrapper.
pub fn parse_mosaic_json(s: &str) -> Result<Vec<(usize, BubbleResult)>, String> {
    let t = strip_markdown(s);
    if t.trim().is_empty() {
        return Err("empty response".to_string());
    }
    let v: serde_json::Value =
        serde_json::from_str(t).map_err(|e| format!("invalid json: {e}"))?;
    if let Some(arr) = v.get("bubbles").and_then(|a| a.as_array()) {
        return Ok(arr
            .iter()
            .enumerate()
            .map(|(i, b)| (i + 1, bubble_result(b)))
            .collect());
    }
    let obj = v.as_object().ok_or_else(|| "expected json object".to_string())?;
    let mut out: Vec<(usize, BubbleResult)> = Vec::new();
    for (k, val) in obj {
        let n: usize = k.parse().map_err(|_| format!("non-numeric key: {k}"))?;
        if n == 0 {
            return Err("keys are 1-based".to_string());
        }
        out.push((n, bubble_result(val)));
    }
    out.sort_by_key(|(n, _)| *n);
    Ok(out)
}

pub fn parse_full_image_json(s: &str) -> Result<Vec<FullBubble>, String> {
    let t = strip_markdown(s);
    if t.trim().is_empty() {
        return Err("empty response".to_string());
    }
    let v: Vec<FullBubble> =
        serde_json::from_str(t).map_err(|e| format!("invalid json: {e}"))?;
    Ok(v)
}

/// First 200 chars for logs (never include keys/b64 here by callers).
pub fn preview_200(s: &str) -> String {
    s.chars().take(200).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn valid_object() {
        let r = parse_mosaic_json(r#"{"2":{"original":"b","reading":"","translated":"B"},"1":{"original":"a","reading":"","translated":"A"}}"#).unwrap();
        assert_eq!(r.len(), 2);
        assert_eq!(r[0].0, 1);
        assert_eq!(r[1].1.translated, "B");
    }

    #[test]
    fn wrapper_array() {
        let r = parse_mosaic_json(r#"{"bubbles":[{"original":"a","reading":"","translated":"A"}]}"#).unwrap();
        assert_eq!(r, vec![(1, BubbleResult { original: "a".into(), reading: "".into(), translated: "A".into() })]);
    }

    #[test]
    fn markdown_wrapped() {
        let r = parse_mosaic_json("```json\n{\"1\":{\"original\":\"a\",\"reading\":\"\",\"translated\":\"A\"}}\n```").unwrap();
        assert_eq!(r.len(), 1);
    }

    #[test]
    fn invalid_and_empty() {
        assert!(parse_mosaic_json("not json").is_err());
        assert!(parse_mosaic_json("  ").is_err());
        assert!(parse_full_image_json("[]").unwrap().is_empty());
        assert!(parse_full_image_json("x").is_err());
    }
}
