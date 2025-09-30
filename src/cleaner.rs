use anyhow::{Context, Result};
use percent_encoding::percent_decode_str;
use url::{form_urlencoded, Url};

use crate::rules::{CompiledHostRules, RuleSet};

pub struct UrlCleaner {
    rules: RuleSet,
}

impl UrlCleaner {
    pub fn new(rules: RuleSet) -> Self { Self { rules } }

    pub fn clean(&self, raw: &str) -> Result<String> {
        // Trim whitespace and surrounding < > often used in copy/pastes
        let trimmed = raw.trim().trim_start_matches('<').trim_end_matches('>');
        let mut url = Url::parse(trimmed).with_context(|| format!("Invalid URL: {}", raw))?;

        // Some trackers put fake fragments that include params (e.g., #xtor=... or #ref=...)
        if let Some(frag) = url.fragment() {
            if frag.contains('=') {
                // Drop the fragment entirely; safer default for share links
                url.set_fragment(None);
            }
        }

        // Host-specific unwrap logic
        if let Some(host) = url.host_str() {
            let hr = self.rules.matcher_for(host)?;
            if let Some(unwrapped) = try_unwrap(&url, &hr) {
                // Recursively clean the inner URL with global rules applied as well
                return self.clean(&unwrapped);
            }
        }

        // Global param filtering
        let cleaned = self.clean_query_params(url)?;
        Ok(cleaned.into_string())
    }

    fn clean_query_params(&self, mut url: Url) -> Result<Url> {
        let host = url.host_str().unwrap_or("");
        let host_rules = self.rules.matcher_for(host)?;
        let global_globs = self.rules.compile_param_globs()?;

        let mut new_q: Vec<(String, String)> = Vec::new();
        let mut changed = false;

        let qpairs: Vec<(String, String)> = url.query_pairs().map(|(k, v)| (k.into_owned(), v.into_owned())).collect();

        for (k, v) in qpairs.iter() {
            // Keep precedence: explicit keep overrides remove
            if contains_case_insensitive(&self.rules.keep_params, k) || contains_case_insensitive(&host_rules.keep_params, k) {
                new_q.push((k.clone(), v.clone()));
                continue;
            }

            // Strip-all for this host?
            if host_rules.strip_all_params {
                changed = true;
                continue;
            }

            // Exact-name removals
            if contains_case_insensitive(&self.rules.remove_params, k) || contains_case_insensitive(&host_rules.remove_params, k) {
                changed = true;
                continue;
            }

            // Glob removals (global + host)
            let k_lower = k.to_ascii_lowercase();
            if global_globs.is_match(&k_lower) || host_rules.remove_param_globs.is_match(&k_lower) {
                changed = true;
                continue;
            }

            new_q.push((k.clone(), v.clone()));
        }

        if changed {
            if new_q.is_empty() {
                url.set_query(None);
            } else {
                let mut ser = form_urlencoded::Serializer::new(String::new());
                for (k, v) in new_q { ser.append_pair(&k, &v); }
                url.set_query(Some(&ser.finish()));
            }
        }
        Ok(url)
    }
}

fn contains_case_insensitive(list: &[String], key: &str) -> bool {
    let kl = key.to_ascii_lowercase();
    list.iter().any(|e| e.eq_ignore_ascii_case(&kl) || e.eq_ignore_ascii_case(key))
}

fn try_unwrap(url: &Url, host_rules: &CompiledHostRules) -> Option<String> {
    if host_rules.unwrap_params.is_empty() {
        return None;
    }
    let mut candidates: Vec<String> = Vec::new();
    for (k, v) in url.query_pairs() {
        if host_rules.unwrap_params.iter().any(|p| p.eq_ignore_ascii_case(&k)) {
            candidates.push(v.into_owned());
        }
    }
    // Some wrappers encode the URL multiple times; decode once.
    for c in candidates {
        let decoded = percent_decode_str(&c).decode_utf8().ok()?.to_string();
        if Url::parse(&decoded).is_ok() {
            return Some(decoded);
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_utm() {
        let c = UrlCleaner::new(RuleSet::builtin());
        let u = c.clean("https://example.com/?utm_source=a&x=1").unwrap();
        assert_eq!(u, "https://example.com/?x=1");
    }

    #[test]
    fn test_unwrap_google() {
        let c = UrlCleaner::new(RuleSet::builtin());
        let u = c.clean("https://www.google.com/url?url=https%3A%2F%2Fexample.com%2Fa%3Futm_medium%3D1").unwrap();
        assert_eq!(u, "https://example.com/a");
    }
}

