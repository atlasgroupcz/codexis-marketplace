use clap::{Args, Subcommand};

use crate::core::error::CliError;

const LAW_REF_HELP: &str = "Czech law reference in NUM/YEAR form, e.g. 89/2012";

#[derive(Subcommand, Debug, Clone)]
pub(crate) enum CzLawCommand {
    #[command(about = "Fetch Czech law metadata", arg_required_else_help = true)]
    Meta(CzLawBaseArgs),

    #[command(about = "Fetch Czech law full text", arg_required_else_help = true)]
    Text(CzLawTextArgs),

    #[command(
        about = "Fetch Czech law table of contents",
        arg_required_else_help = true
    )]
    Toc(CzLawBaseArgs),

    #[command(about = "Fetch Czech law versions", arg_required_else_help = true)]
    Versions(CzLawBaseArgs),

    #[command(
        about = "Fetch related documents for a Czech law",
        arg_required_else_help = true
    )]
    Related(CzLawRelatedArgs),

    #[command(
        name = "related-counts",
        about = "Fetch relation counts for a Czech law",
        arg_required_else_help = true
    )]
    RelatedCounts(CzLawRelatedCountsArgs),
}

#[derive(Args, Debug, Clone)]
pub(crate) struct CzLawBaseArgs {
    #[arg(value_name = "LAW_REF", help = LAW_REF_HELP)]
    pub(crate) law_ref: String,

    #[arg(long, help = "Print the translated curl command and exit")]
    pub(crate) dry_run: bool,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct CzLawTextArgs {
    #[command(flatten)]
    pub(crate) base: CzLawBaseArgs,

    #[arg(
        long = "part",
        help = "Part element id (repeatable)",
        value_name = "PART"
    )]
    pub(crate) parts: Vec<String>,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct CzLawRelatedArgs {
    #[command(flatten)]
    pub(crate) base: CzLawBaseArgs,

    #[arg(long = "type", help = "Relation type filter", value_name = "TYPE")]
    pub(crate) relation_type: Option<String>,

    #[arg(long = "part", help = "Part element id", value_name = "PART")]
    pub(crate) part: Option<String>,

    #[arg(long, help = "Pagination offset", value_name = "OFFSET")]
    pub(crate) offset: Option<u64>,

    #[arg(long, help = "Result limit", value_name = "LIMIT")]
    pub(crate) limit: Option<u64>,

    #[arg(long, help = "Sort field", value_name = "SORT")]
    pub(crate) sort: Option<String>,

    #[arg(long, help = "Sort order", value_name = "ORDER")]
    pub(crate) order: Option<String>,
}

#[derive(Args, Debug, Clone)]
pub(crate) struct CzLawRelatedCountsArgs {
    #[command(flatten)]
    pub(crate) base: CzLawBaseArgs,

    #[arg(long = "part", help = "Part element id", value_name = "PART")]
    pub(crate) part: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CzLawRef {
    number: String,
    year: String,
}

impl CzLawCommand {
    pub(crate) fn dry_run(&self) -> bool {
        match self {
            Self::Meta(args) | Self::Toc(args) | Self::Versions(args) => args.dry_run,
            Self::Text(args) => args.base.dry_run,
            Self::Related(args) => args.base.dry_run,
            Self::RelatedCounts(args) => args.base.dry_run,
        }
    }

    pub(crate) fn resource(&self) -> Result<String, CliError> {
        match self {
            Self::Meta(args) => build_resource(&args.law_ref, "meta", &[]),
            Self::Text(args) => build_resource(&args.base.law_ref, "text", &args.parts),
            Self::Toc(args) => build_resource(&args.law_ref, "toc", &[]),
            Self::Versions(args) => build_resource(&args.law_ref, "versions", &[]),
            Self::Related(args) => build_related_resource(args),
            Self::RelatedCounts(args) => build_related_counts_resource(args),
        }
    }
}

fn build_resource(law_ref: &str, suffix: &str, parts: &[String]) -> Result<String, CliError> {
    let law_ref = CzLawRef::parse(law_ref)?;
    let resource = format!(
        "cdx://cz_law/{}/{}/{}",
        law_ref.number, law_ref.year, suffix
    );

    let params = parts
        .iter()
        .map(|part| part.trim())
        .filter(|part| !part.is_empty())
        .map(|part| ("part", part.to_string()))
        .collect::<Vec<_>>();

    Ok(with_query_params(resource, params))
}

fn build_related_resource(args: &CzLawRelatedArgs) -> Result<String, CliError> {
    let law_ref = CzLawRef::parse(&args.base.law_ref)?;
    let resource = format!("cdx://cz_law/{}/{}/related", law_ref.number, law_ref.year);
    let mut params = Vec::new();

    if let Some(value) = normalize_query_value(args.relation_type.as_deref()) {
        params.push(("type", value));
    }
    if let Some(value) = normalize_query_value(args.part.as_deref()) {
        params.push(("part", value));
    }
    if let Some(value) = args.offset {
        params.push(("offset", value.to_string()));
    }
    if let Some(value) = args.limit {
        params.push(("limit", value.to_string()));
    }
    if let Some(value) = normalize_query_value(args.sort.as_deref()) {
        params.push(("sort", value));
    }
    if let Some(value) = normalize_query_value(args.order.as_deref()) {
        params.push(("order", value));
    }

    Ok(with_query_params(resource, params))
}

fn build_related_counts_resource(args: &CzLawRelatedCountsArgs) -> Result<String, CliError> {
    let law_ref = CzLawRef::parse(&args.base.law_ref)?;
    let resource = format!(
        "cdx://cz_law/{}/{}/related/counts",
        law_ref.number, law_ref.year
    );
    let params = normalize_query_value(args.part.as_deref())
        .map(|value| vec![("part", value)])
        .unwrap_or_default();

    Ok(with_query_params(resource, params))
}

fn with_query_params(resource: String, params: Vec<(&str, String)>) -> String {
    if params.is_empty() {
        return resource;
    }

    let mut resource = resource;
    resource.push('?');
    for (index, (key, value)) in params.iter().enumerate() {
        if index > 0 {
            resource.push('&');
        }
        resource.push_str(key);
        resource.push('=');
        resource.push_str(value);
    }
    resource
}

fn normalize_query_value(value: Option<&str>) -> Option<String> {
    value
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToString::to_string)
}

impl CzLawRef {
    fn parse(raw: &str) -> Result<Self, CliError> {
        let raw = raw.trim();
        let Some((number, year)) = raw.split_once('/') else {
            return Err(invalid_law_ref());
        };

        if number.is_empty()
            || year.is_empty()
            || year.contains('/')
            || !number.chars().all(|ch| ch.is_ascii_digit())
            || !year.chars().all(|ch| ch.is_ascii_digit())
        {
            return Err(invalid_law_ref());
        }

        Ok(Self {
            number: number.to_string(),
            year: year.to_string(),
        })
    }
}

fn invalid_law_ref() -> CliError {
    CliError::InvalidLawRef("LAW_REF must be in NUM/YEAR form, for example 89/2012".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_law_ref() {
        let law_ref = CzLawRef::parse("89/2012").unwrap();
        assert_eq!(law_ref.number, "89");
        assert_eq!(law_ref.year, "2012");
    }

    #[test]
    fn rejects_invalid_law_ref() {
        let error = CzLawRef::parse("89-2012").unwrap_err();
        assert!(matches!(error, CliError::InvalidLawRef(_)));
    }

    #[test]
    fn builds_cz_law_meta_resource() {
        let resource = build_resource("89/2012", "meta", &[]).unwrap();
        assert_eq!(resource, "cdx://cz_law/89/2012/meta");
    }

    #[test]
    fn builds_cz_law_text_resource_with_repeated_parts() {
        let resource = build_resource(
            "89/2012",
            "text",
            &["paragraf1".to_string(), "paragraf2".to_string()],
        )
        .unwrap();

        assert_eq!(
            resource,
            "cdx://cz_law/89/2012/text?part=paragraf1&part=paragraf2"
        );
    }

    #[test]
    fn builds_cz_law_related_resource_with_filters() {
        let resource = build_related_resource(&CzLawRelatedArgs {
            base: CzLawBaseArgs {
                law_ref: "89/2012".to_string(),
                dry_run: false,
            },
            relation_type: Some("SOUVISEJICI_JUDIKATURA".to_string()),
            part: Some("paragraf1".to_string()),
            offset: Some(20),
            limit: Some(5),
            sort: Some("date".to_string()),
            order: Some("desc".to_string()),
        })
        .unwrap();

        assert_eq!(
            resource,
            "cdx://cz_law/89/2012/related?type=SOUVISEJICI_JUDIKATURA&part=paragraf1&offset=20&limit=5&sort=date&order=desc"
        );
    }

    #[test]
    fn builds_cz_law_related_counts_resource() {
        let resource = build_related_counts_resource(&CzLawRelatedCountsArgs {
            base: CzLawBaseArgs {
                law_ref: "89/2012".to_string(),
                dry_run: false,
            },
            part: Some("paragraf1".to_string()),
        })
        .unwrap();

        assert_eq!(
            resource,
            "cdx://cz_law/89/2012/related/counts?part=paragraf1"
        );
    }
}
