use std::fs::File;
use std::path::Path;

use anyhow::{bail, Context, Result};
use globset::{Glob, GlobSet, GlobSetBuilder};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuleSet {
    /// Query params to remove (exact match)
    pub remove_params: Vec<String>,
    /// Query params to remove (glob patterns, e.g., "utm_*")
    pub remove_param_globs: Vec<String>,
    /// Params to keep even if matched by remove rules
    pub keep_params: Vec<String>,
    /// Host-specific rules
    pub host_rules: Vec<HostRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct HostRule {
    /// Host glob(s) this rule applies to (e.g., "*.google.com")
    pub hosts: Vec<String>,
    /// If present, unwrap this param as the real target URL (e.g., "url", "u", "q").
    pub unwrap_params: Vec<String>,
    /// Additional params to remove
    pub remove_params: Vec<String>,
    /// Additional param globs to remove
    pub remove_param_globs: Vec<String>,
    /// If true, drop all params except those in keep_params
    pub strip_all_params: Option<bool>,
    /// Params to keep for this host
    pub keep_params: Vec<String>,
}

impl RuleSet {
    pub fn builtin() -> Self {
        // Curated baseline list of common trackers
        let mut s = RuleSet::default();
        s.remove_params = vec![
            "gclid".into(),
            "gbraid".into(),
            "wbraid".into(),
            "fbclid".into(),
            "igshid".into(),
            "twclid".into(),
            "mc_eid".into(),
            "msclkid".into(),
            "dclid".into(),
            "icid".into(),
            "mkt_tok".into(),
            "vero_id".into(),
            "vero_conv".into(),
            "spm".into(),
            "ncid".into(),
            "epik".into(),
            "si".into(),
        ];
        s.remove_param_globs = vec![
            "utm_*".into(),
            "pk_*".into(), // Matomo/Piwik campaign params
            "mtm_*".into(),
            "oly_*".into(),
            "s_cid*".into(),
            "aff*".into(),
            "ref*".into(),
        ];

        // Known redirect wrappers with embedded target URL
        s.host_rules = vec![
            HostRule {
                hosts: vec!["*.google.com".into()],
                unwrap_params: vec!["url".into(), "q".into(), "u".into()],
                remove_params: vec![],
                remove_param_globs: vec![],
                strip_all_params: None,
                keep_params: vec![],
            },
            HostRule {
                hosts: vec!["*.facebook.com".into(), "*.lm.facebook.com".into()],
                unwrap_params: vec!["u".into()],
                ..Default::default()
            },
            HostRule {
                hosts: vec!["out.reddit.com".into()],
                unwrap_params: vec!["url".into()],
                ..Default::default()
            },
            HostRule {
                hosts: vec!["*.youtube.com".into(), "youtu.be".into()],
                unwrap_params: vec!["q".into()],
                ..Default::default()
            },
        ];

        s
    }

    pub fn merge(&mut self, other: RuleSet) {
        self.remove_params.extend(other.remove_params);
        self.remove_param_globs.extend(other.remove_param_globs);
        self.keep_params.extend(other.keep_params);
        self.host_rules.extend(other.host_rules);
    }

    pub fn from_path(path: &Path) -> Result<Self> {
        let file = File::open(path)
            .with_context(|| format!("Failed to open rules file {}", path.display()))?;
        let ext = path.extension().and_then(|s| s.to_str()).unwrap_or("");
        let rules = match ext.to_ascii_lowercase().as_str() {
            "yaml" | "yml" => serde_yaml::from_reader(file)
                .with_context(|| "Failed to parse YAML rules")?,
            "json" => serde_json::from_reader(file)
                .with_context(|| "Failed to parse JSON rules")?,
            _ => bail!("Unsupported rules file extension: {}", ext),
        };
        Ok(rules)
    }

    pub(crate) fn matcher_for(&self, host: &str) -> Result<CompiledHostRules> {
        let mut matched: Vec<&HostRule> = Vec::new();
        for hr in &self.host_rules {
            for pat in &hr.hosts {
                let glob = Glob::new(pat)
                    .with_context(|| format!("Invalid host glob: {}", pat))?;
                let gs = glob.compile_matcher();
                if gs.is_match(host) {
                    matched.push(hr);
                    break;
                }
            }
        }
        Ok(CompiledHostRules::new(matched))
    }

    pub(crate) fn compile_param_globs(&self) -> Result<GlobSet> {
        let mut builder = GlobSetBuilder::new();
        for pat in &self.remove_param_globs {
            builder.add(Glob::new(pat)
                .with_context(|| format!("Invalid param glob: {}", pat))?);
        }
        let gs = builder.build()?;
        Ok(gs)
    }
}

pub struct CompiledHostRules {
    pub unwrap_params: Vec<String>,
    pub remove_params: Vec<String>,
    pub remove_param_globs: GlobSet,
    pub strip_all_params: bool,
    pub keep_params: Vec<String>,
}

impl CompiledHostRules {
    fn new(rules: Vec<&HostRule>) -> Self {
        let mut unwrap_params = Vec::new();
        let mut remove_params = Vec::new();
        let mut keep_params = Vec::new();
        let mut strip_all = false;
        let mut builder = GlobSetBuilder::new();

        for r in rules {
            unwrap_params.extend(r.unwrap_params.iter().cloned());
            remove_params.extend(r.remove_params.iter().cloned());
            keep_params.extend(r.keep_params.iter().cloned());
            if r.strip_all_params.unwrap_or(false) {
                strip_all = true;
            }
            for g in &r.remove_param_globs {
                if let Ok(glob) = Glob::new(g) {
                    builder.add(glob);
                }
            }
        }
        let remove_param_globs = builder.build().unwrap_or_else(|_| GlobSetBuilder::new().build().unwrap());
        Self { unwrap_params, remove_params, remove_param_globs, strip_all_params: strip_all, keep_params }
    }
}

