//! Report Generator
//!
//! Generates reports in various formats including PDF, HTML, and Markdown.
//! Uses structured content blocks to build professional reports.

use super::models::*;
use uuid::Uuid;

/// Report generator for creating structured reports
pub struct ReportGenerator {
    config: ReportConfig,
}

impl ReportGenerator {
    pub fn new(config: ReportConfig) -> Self {
        Self { config }
    }

    /// Generate a complete report
    pub fn generate(&self) -> Result<Report, String> {
        let now = chrono::Utc::now().to_rfc3339();

        let content = self.build_content()?;

        Ok(Report {
            id: Uuid::new_v4().to_string(),
            client_id: self.config.client_id.clone(),
            config: self.config.clone(),
            status: ReportStatus::Ready,
            content: Some(content),
            file_path: None,
            file_size: None,
            created_at: now.clone(),
            updated_at: now,
            error: None,
        })
    }

    fn build_content(&self) -> Result<ReportContent, String> {
        let sections = match self.config.report_type {
            ReportType::ExecutiveSummary => self.build_executive_summary(),
            ReportType::TechnicalAssessment => self.build_technical_assessment(),
            ReportType::ComplianceReport => self.build_compliance_report(),
            ReportType::NetworkAssessment => self.build_network_assessment(),
            ReportType::CloudReadiness => self.build_cloud_readiness(),
            ReportType::SecurityFindings => self.build_security_findings(),
            ReportType::FullEngagement => self.build_full_engagement(),
        };

        let metadata = ReportMetadata {
            title: self.config.title.clone(),
            subtitle: self.config.subtitle.clone(),
            author: self.config.author.clone(),
            organization: self.config.organization.clone(),
            client_name: self.config.client_name.clone(),
            report_date: chrono::Utc::now().format("%B %d, %Y").to_string(),
            classification: self.config.classification.clone(),
            version: "1.0".to_string(),
            page_count: None,
        };

        Ok(ReportContent { sections, metadata })
    }

    fn build_executive_summary(&self) -> Vec<ReportSection> {
        vec![
            ReportSection {
                id: "exec-overview".to_string(),
                title: "Executive Overview".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Paragraph {
                        text: format!(
                            "This executive summary provides a high-level overview of the security assessment \
                            conducted for {}. The assessment evaluated the organization's security posture \
                            across multiple domains including infrastructure, compliance, and risk management.",
                            self.config.client_name
                        ),
                    },
                    ContentBlock::KeyValue {
                        items: vec![
                            KeyValueItem { key: "Assessment Period".to_string(), value: "January 2026".to_string() },
                            KeyValueItem { key: "Scope".to_string(), value: "Enterprise Infrastructure".to_string() },
                            KeyValueItem { key: "Classification".to_string(), value: self.config.classification.clone().unwrap_or_else(|| "Confidential".to_string()) },
                        ],
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "key-findings".to_string(),
                title: "Key Findings".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Callout {
                        callout_type: CalloutType::Critical,
                        title: Some("Critical Findings".to_string()),
                        text: "2 critical vulnerabilities require immediate attention".to_string(),
                    },
                    ContentBlock::Metric {
                        label: "Overall Security Score".to_string(),
                        value: "72/100".to_string(),
                        change: Some("+5".to_string()),
                        trend: Some("improving".to_string()),
                    },
                    ContentBlock::Chart {
                        chart_type: ChartType::Pie,
                        title: "Findings by Severity".to_string(),
                        data: ChartData {
                            labels: vec!["Critical".to_string(), "High".to_string(), "Medium".to_string(), "Low".to_string()],
                            datasets: vec![ChartDataset {
                                label: "Findings".to_string(),
                                data: vec![2.0, 5.0, 12.0, 8.0],
                                color: None,
                            }],
                        },
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "risk-summary".to_string(),
                title: "Risk Summary".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Table {
                        headers: vec!["Risk Area".to_string(), "Current State".to_string(), "Target State".to_string(), "Priority".to_string()],
                        rows: vec![
                            vec!["Access Control".to_string(), "Moderate".to_string(), "Strong".to_string(), "High".to_string()],
                            vec!["Data Protection".to_string(), "Weak".to_string(), "Strong".to_string(), "Critical".to_string()],
                            vec!["Network Security".to_string(), "Strong".to_string(), "Strong".to_string(), "Medium".to_string()],
                            vec!["Incident Response".to_string(), "Moderate".to_string(), "Strong".to_string(), "High".to_string()],
                        ],
                        caption: Some("Risk assessment summary by domain".to_string()),
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "recommendations".to_string(),
                title: "Strategic Recommendations".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::NumberedList {
                        items: vec![
                            "Implement multi-factor authentication across all critical systems".to_string(),
                            "Deploy endpoint detection and response (EDR) solution".to_string(),
                            "Establish formal vulnerability management program".to_string(),
                            "Develop and test incident response procedures".to_string(),
                            "Conduct security awareness training for all employees".to_string(),
                        ],
                    },
                ],
                subsections: vec![],
            },
        ]
    }

    fn build_technical_assessment(&self) -> Vec<ReportSection> {
        vec![
            ReportSection {
                id: "tech-overview".to_string(),
                title: "Technical Assessment Overview".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Paragraph {
                        text: "This technical assessment provides detailed analysis of security vulnerabilities, \
                              misconfigurations, and areas for improvement identified during the engagement.".to_string(),
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "methodology".to_string(),
                title: "Methodology".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Paragraph {
                        text: "The assessment followed industry-standard methodologies including OWASP, NIST, and PTES.".to_string(),
                    },
                    ContentBlock::BulletList {
                        items: vec![
                            "Network reconnaissance and enumeration".to_string(),
                            "Vulnerability scanning and validation".to_string(),
                            "Configuration review and hardening analysis".to_string(),
                            "Access control and authentication testing".to_string(),
                            "Security architecture review".to_string(),
                        ],
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "findings".to_string(),
                title: "Detailed Findings".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Finding {
                        id: "FIND-001".to_string(),
                        title: "Outdated SSL/TLS Configuration".to_string(),
                        severity: "High".to_string(),
                        description: "Multiple servers were found using deprecated TLS 1.0 and 1.1 protocols.".to_string(),
                        impact: "Attackers could potentially exploit known vulnerabilities in older protocols.".to_string(),
                        recommendation: "Disable TLS 1.0 and 1.1, enable TLS 1.2+ with strong cipher suites.".to_string(),
                    },
                    ContentBlock::Finding {
                        id: "FIND-002".to_string(),
                        title: "Missing Security Headers".to_string(),
                        severity: "Medium".to_string(),
                        description: "Web applications lack recommended security headers.".to_string(),
                        impact: "Applications may be vulnerable to clickjacking, XSS, and other attacks.".to_string(),
                        recommendation: "Implement CSP, X-Frame-Options, X-Content-Type-Options headers.".to_string(),
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "remediation".to_string(),
                title: "Remediation Roadmap".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Table {
                        headers: vec!["Finding".to_string(), "Priority".to_string(), "Effort".to_string(), "Timeline".to_string()],
                        rows: vec![
                            vec!["FIND-001".to_string(), "High".to_string(), "Medium".to_string(), "30 days".to_string()],
                            vec!["FIND-002".to_string(), "Medium".to_string(), "Low".to_string(), "14 days".to_string()],
                        ],
                        caption: Some("Recommended remediation timeline".to_string()),
                    },
                ],
                subsections: vec![],
            },
        ]
    }

    fn build_compliance_report(&self) -> Vec<ReportSection> {
        vec![
            ReportSection {
                id: "compliance-overview".to_string(),
                title: "Compliance Assessment Overview".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Paragraph {
                        text: "This report presents the findings from the compliance assessment against \
                              applicable regulatory frameworks and industry standards.".to_string(),
                    },
                    ContentBlock::Chart {
                        chart_type: ChartType::Gauge,
                        title: "Overall Compliance Score".to_string(),
                        data: ChartData {
                            labels: vec!["Compliance".to_string()],
                            datasets: vec![ChartDataset {
                                label: "Score".to_string(),
                                data: vec![78.5],
                                color: Some("#22C55E".to_string()),
                            }],
                        },
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "framework-status".to_string(),
                title: "Framework Compliance Status".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Table {
                        headers: vec!["Framework".to_string(), "Controls Assessed".to_string(), "Compliant".to_string(), "Score".to_string()],
                        rows: vec![
                            vec!["NIST CSF 2.0".to_string(), "106".to_string(), "84".to_string(), "79.2%".to_string()],
                            vec!["SOC 2 Type II".to_string(), "64".to_string(), "52".to_string(), "81.3%".to_string()],
                            vec!["GDPR".to_string(), "42".to_string(), "31".to_string(), "73.8%".to_string()],
                        ],
                        caption: Some("Compliance status by framework".to_string()),
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "gaps".to_string(),
                title: "Gap Analysis".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Callout {
                        callout_type: CalloutType::Warning,
                        title: Some("Key Gaps Identified".to_string()),
                        text: "8 high-priority control gaps require remediation before next audit".to_string(),
                    },
                    ContentBlock::BulletList {
                        items: vec![
                            "PR.AC-1: Access control policies need formalization".to_string(),
                            "DE.CM-1: Network monitoring coverage insufficient".to_string(),
                            "RS.RP-1: Incident response plan requires update".to_string(),
                        ],
                    },
                ],
                subsections: vec![],
            },
        ]
    }

    fn build_network_assessment(&self) -> Vec<ReportSection> {
        vec![
            ReportSection {
                id: "network-overview".to_string(),
                title: "Network Assessment Overview".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Paragraph {
                        text: "This report documents the network infrastructure assessment including \
                              asset discovery, topology mapping, and security posture analysis.".to_string(),
                    },
                    ContentBlock::KeyValue {
                        items: vec![
                            KeyValueItem { key: "Total Assets Discovered".to_string(), value: "247".to_string() },
                            KeyValueItem { key: "Network Segments".to_string(), value: "12".to_string() },
                            KeyValueItem { key: "Critical Systems".to_string(), value: "34".to_string() },
                        ],
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "asset-inventory".to_string(),
                title: "Asset Inventory Summary".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Chart {
                        chart_type: ChartType::Bar,
                        title: "Assets by Category".to_string(),
                        data: ChartData {
                            labels: vec!["Servers".to_string(), "Workstations".to_string(), "Network".to_string(), "Security".to_string(), "Other".to_string()],
                            datasets: vec![ChartDataset {
                                label: "Count".to_string(),
                                data: vec![42.0, 156.0, 28.0, 12.0, 9.0],
                                color: Some("#3B82F6".to_string()),
                            }],
                        },
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "services".to_string(),
                title: "Service Analysis".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Table {
                        headers: vec!["Service".to_string(), "Port".to_string(), "Instances".to_string(), "Risk Level".to_string()],
                        rows: vec![
                            vec!["SSH".to_string(), "22".to_string(), "45".to_string(), "Low".to_string()],
                            vec!["HTTP".to_string(), "80".to_string(), "23".to_string(), "Medium".to_string()],
                            vec!["HTTPS".to_string(), "443".to_string(), "38".to_string(), "Low".to_string()],
                            vec!["RDP".to_string(), "3389".to_string(), "12".to_string(), "High".to_string()],
                            vec!["SMB".to_string(), "445".to_string(), "67".to_string(), "Medium".to_string()],
                        ],
                        caption: Some("Top services discovered across the network".to_string()),
                    },
                ],
                subsections: vec![],
            },
        ]
    }

    fn build_cloud_readiness(&self) -> Vec<ReportSection> {
        vec![
            ReportSection {
                id: "cloud-overview".to_string(),
                title: "Cloud Readiness Assessment".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Paragraph {
                        text: "This assessment evaluates the organization's readiness for cloud migration \
                              and provides recommendations for a successful transition.".to_string(),
                    },
                    ContentBlock::Metric {
                        label: "Cloud Readiness Score".to_string(),
                        value: "68%".to_string(),
                        change: None,
                        trend: None,
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "readiness-by-area".to_string(),
                title: "Readiness by Area".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Chart {
                        chart_type: ChartType::Radar,
                        title: "Readiness Assessment".to_string(),
                        data: ChartData {
                            labels: vec![
                                "Business".to_string(), "Technical".to_string(), "Security".to_string(),
                                "Operations".to_string(), "Financial".to_string(), "People".to_string()
                            ],
                            datasets: vec![ChartDataset {
                                label: "Current".to_string(),
                                data: vec![75.0, 60.0, 70.0, 55.0, 80.0, 65.0],
                                color: Some("#3B82F6".to_string()),
                            }],
                        },
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "cost-analysis".to_string(),
                title: "Cost Analysis".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Table {
                        headers: vec!["Provider".to_string(), "Monthly Estimate".to_string(), "Annual Estimate".to_string(), "Savings".to_string()],
                        rows: vec![
                            vec!["AWS".to_string(), "$12,450".to_string(), "$149,400".to_string(), "18%".to_string()],
                            vec!["Azure".to_string(), "$11,890".to_string(), "$142,680".to_string(), "22%".to_string()],
                            vec!["GCP".to_string(), "$11,200".to_string(), "$134,400".to_string(), "26%".to_string()],
                        ],
                        caption: Some("Estimated cloud costs by provider".to_string()),
                    },
                ],
                subsections: vec![],
            },
        ]
    }

    fn build_security_findings(&self) -> Vec<ReportSection> {
        vec![
            ReportSection {
                id: "findings-overview".to_string(),
                title: "Security Findings Overview".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Paragraph {
                        text: "This report presents security findings identified during the assessment, \
                              organized by severity and including remediation recommendations.".to_string(),
                    },
                    ContentBlock::Chart {
                        chart_type: ChartType::Donut,
                        title: "Findings by Severity".to_string(),
                        data: ChartData {
                            labels: vec!["Critical".to_string(), "High".to_string(), "Medium".to_string(), "Low".to_string(), "Info".to_string()],
                            datasets: vec![ChartDataset {
                                label: "Findings".to_string(),
                                data: vec![2.0, 7.0, 15.0, 12.0, 5.0],
                                color: None,
                            }],
                        },
                    },
                ],
                subsections: vec![],
            },
            ReportSection {
                id: "critical-findings".to_string(),
                title: "Critical Findings".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Callout {
                        callout_type: CalloutType::Critical,
                        title: Some("Immediate Action Required".to_string()),
                        text: "The following findings require immediate attention due to their severity.".to_string(),
                    },
                    ContentBlock::Finding {
                        id: "SEC-001".to_string(),
                        title: "Unpatched Critical Vulnerability (CVE-2024-XXXX)".to_string(),
                        severity: "Critical".to_string(),
                        description: "Multiple systems running vulnerable software versions.".to_string(),
                        impact: "Remote code execution possible, complete system compromise.".to_string(),
                        recommendation: "Apply vendor patches immediately, isolate affected systems.".to_string(),
                    },
                ],
                subsections: vec![],
            },
        ]
    }

    fn build_full_engagement(&self) -> Vec<ReportSection> {
        let mut sections = vec![
            ReportSection {
                id: "engagement-overview".to_string(),
                title: "Engagement Overview".to_string(),
                level: 1,
                blocks: vec![
                    ContentBlock::Paragraph {
                        text: format!(
                            "This comprehensive report documents the full security engagement conducted for {}. \
                            The assessment covered multiple domains including compliance, network security, \
                            cloud readiness, and vulnerability analysis.",
                            self.config.client_name
                        ),
                    },
                ],
                subsections: vec![],
            },
        ];

        // Add sections from each report type
        sections.extend(self.build_executive_summary());
        sections.push(ReportSection {
            id: "page-break-1".to_string(),
            title: String::new(),
            level: 0,
            blocks: vec![ContentBlock::PageBreak],
            subsections: vec![],
        });
        sections.extend(self.build_compliance_report());
        sections.extend(self.build_network_assessment());
        sections.extend(self.build_security_findings());

        sections
    }
}

/// Convert report content to HTML
pub fn content_to_html(content: &ReportContent) -> String {
    let mut html = String::new();

    // Document header
    html.push_str("<!DOCTYPE html>\n<html lang=\"en\">\n<head>\n");
    html.push_str("<meta charset=\"UTF-8\">\n");
    html.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1.0\">\n");
    html.push_str(&format!("<title>{}</title>\n", content.metadata.title));
    html.push_str("<style>\n");
    html.push_str(include_str!("report_styles.css"));
    html.push_str("</style>\n</head>\n<body>\n");

    // Cover page
    html.push_str("<div class=\"cover-page\">\n");
    html.push_str(&format!("<h1 class=\"title\">{}</h1>\n", content.metadata.title));
    if let Some(ref subtitle) = content.metadata.subtitle {
        html.push_str(&format!("<h2 class=\"subtitle\">{}</h2>\n", subtitle));
    }
    html.push_str(&format!("<p class=\"client\">Prepared for: {}</p>\n", content.metadata.client_name));
    html.push_str(&format!("<p class=\"author\">Prepared by: {}</p>\n", content.metadata.author));
    html.push_str(&format!("<p class=\"date\">{}</p>\n", content.metadata.report_date));
    if let Some(ref classification) = content.metadata.classification {
        html.push_str(&format!("<p class=\"classification\">{}</p>\n", classification));
    }
    html.push_str("</div>\n");

    // Sections
    for section in &content.sections {
        html.push_str(&section_to_html(section));
    }

    html.push_str("</body>\n</html>");
    html
}

fn section_to_html(section: &ReportSection) -> String {
    let mut html = String::new();

    if !section.title.is_empty() {
        let tag = format!("h{}", section.level.min(6));
        html.push_str(&format!("<{} id=\"{}\">{}</{}>\n", tag, section.id, section.title, tag));
    }

    for block in &section.blocks {
        html.push_str(&block_to_html(block));
    }

    for subsection in &section.subsections {
        html.push_str(&section_to_html(subsection));
    }

    html
}

fn block_to_html(block: &ContentBlock) -> String {
    match block {
        ContentBlock::Paragraph { text } => format!("<p>{}</p>\n", text),

        ContentBlock::Heading { text, level } => {
            let tag = format!("h{}", level.min(&6));
            format!("<{0}>{1}</{0}>\n", tag, text)
        }

        ContentBlock::BulletList { items } => {
            let items_html: String = items.iter().map(|i| format!("<li>{}</li>", i)).collect();
            format!("<ul>{}</ul>\n", items_html)
        }

        ContentBlock::NumberedList { items } => {
            let items_html: String = items.iter().map(|i| format!("<li>{}</li>", i)).collect();
            format!("<ol>{}</ol>\n", items_html)
        }

        ContentBlock::Table { headers, rows, caption } => {
            let mut html = String::from("<table class=\"report-table\">\n");
            if let Some(cap) = caption {
                html.push_str(&format!("<caption>{}</caption>\n", cap));
            }
            html.push_str("<thead><tr>");
            for h in headers {
                html.push_str(&format!("<th>{}</th>", h));
            }
            html.push_str("</tr></thead>\n<tbody>\n");
            for row in rows {
                html.push_str("<tr>");
                for cell in row {
                    html.push_str(&format!("<td>{}</td>", cell));
                }
                html.push_str("</tr>\n");
            }
            html.push_str("</tbody></table>\n");
            html
        }

        ContentBlock::Chart { chart_type, title, data: _ } => {
            format!("<div class=\"chart-placeholder\" data-type=\"{:?}\"><p>{}</p><p>[Chart: {:?}]</p></div>\n", chart_type, title, chart_type)
        }

        ContentBlock::KeyValue { items } => {
            let mut html = String::from("<dl class=\"key-value\">\n");
            for item in items {
                html.push_str(&format!("<dt>{}</dt><dd>{}</dd>\n", item.key, item.value));
            }
            html.push_str("</dl>\n");
            html
        }

        ContentBlock::Callout { callout_type, title, text } => {
            let class = match callout_type {
                CalloutType::Info => "callout-info",
                CalloutType::Warning => "callout-warning",
                CalloutType::Critical => "callout-critical",
                CalloutType::Success => "callout-success",
                CalloutType::Note => "callout-note",
            };
            let mut html = format!("<div class=\"callout {}\">\n", class);
            if let Some(t) = title {
                html.push_str(&format!("<strong>{}</strong>\n", t));
            }
            html.push_str(&format!("<p>{}</p>\n</div>\n", text));
            html
        }

        ContentBlock::Code { language, content } => {
            let lang = language.as_deref().unwrap_or("text");
            format!("<pre><code class=\"language-{}\">{}</code></pre>\n", lang, content)
        }

        ContentBlock::Finding { id, title, severity, description, impact, recommendation } => {
            let severity_class = match severity.to_lowercase().as_str() {
                "critical" => "severity-critical",
                "high" => "severity-high",
                "medium" => "severity-medium",
                _ => "severity-low",
            };
            format!(
                "<div class=\"finding {}\">\n\
                <div class=\"finding-header\"><span class=\"finding-id\">{}</span> <span class=\"finding-title\">{}</span> <span class=\"severity-badge\">{}</span></div>\n\
                <div class=\"finding-body\">\n\
                <p><strong>Description:</strong> {}</p>\n\
                <p><strong>Impact:</strong> {}</p>\n\
                <p><strong>Recommendation:</strong> {}</p>\n\
                </div></div>\n",
                severity_class, id, title, severity, description, impact, recommendation
            )
        }

        ContentBlock::Metric { label, value, change, trend } => {
            let mut html = format!("<div class=\"metric\"><span class=\"metric-value\">{}</span><span class=\"metric-label\">{}</span>", value, label);
            if let Some(c) = change {
                let trend_class = if trend.as_deref() == Some("improving") { "trend-up" } else { "trend-down" };
                html.push_str(&format!("<span class=\"metric-change {}\">{}</span>", trend_class, c));
            }
            html.push_str("</div>\n");
            html
        }

        ContentBlock::PageBreak => "<div class=\"page-break\"></div>\n".to_string(),
    }
}

/// Convert report content to Markdown
pub fn content_to_markdown(content: &ReportContent) -> String {
    let mut md = String::new();

    // Title
    md.push_str(&format!("# {}\n\n", content.metadata.title));
    if let Some(ref subtitle) = content.metadata.subtitle {
        md.push_str(&format!("## {}\n\n", subtitle));
    }

    md.push_str(&format!("**Client:** {}\n\n", content.metadata.client_name));
    md.push_str(&format!("**Author:** {}\n\n", content.metadata.author));
    md.push_str(&format!("**Date:** {}\n\n", content.metadata.report_date));
    md.push_str("---\n\n");

    // Sections
    for section in &content.sections {
        md.push_str(&section_to_markdown(section));
    }

    md
}

fn section_to_markdown(section: &ReportSection) -> String {
    let mut md = String::new();

    if !section.title.is_empty() {
        let prefix = "#".repeat(section.level as usize);
        md.push_str(&format!("{} {}\n\n", prefix, section.title));
    }

    for block in &section.blocks {
        md.push_str(&block_to_markdown(block));
        md.push('\n');
    }

    for subsection in &section.subsections {
        md.push_str(&section_to_markdown(subsection));
    }

    md
}

fn block_to_markdown(block: &ContentBlock) -> String {
    match block {
        ContentBlock::Paragraph { text } => format!("{}\n", text),
        ContentBlock::Heading { text, level } => {
            let prefix = "#".repeat(*level as usize);
            format!("{} {}\n", prefix, text)
        }
        ContentBlock::BulletList { items } => {
            items.iter().map(|i| format!("- {}\n", i)).collect()
        }
        ContentBlock::NumberedList { items } => {
            items.iter().enumerate().map(|(n, i)| format!("{}. {}\n", n + 1, i)).collect()
        }
        ContentBlock::Table { headers, rows, caption } => {
            let mut md = String::new();
            if let Some(cap) = caption {
                md.push_str(&format!("*{}*\n\n", cap));
            }
            md.push_str(&format!("| {} |\n", headers.join(" | ")));
            md.push_str(&format!("| {} |\n", headers.iter().map(|_| "---").collect::<Vec<_>>().join(" | ")));
            for row in rows {
                md.push_str(&format!("| {} |\n", row.join(" | ")));
            }
            md
        }
        ContentBlock::Chart { title, .. } => format!("[Chart: {}]\n", title),
        ContentBlock::KeyValue { items } => {
            items.iter().map(|i| format!("- **{}:** {}\n", i.key, i.value)).collect()
        }
        ContentBlock::Callout { callout_type, title, text } => {
            let prefix = match callout_type {
                CalloutType::Critical => "> ⛔ ",
                CalloutType::Warning => "> ⚠️ ",
                CalloutType::Info => "> ℹ️ ",
                CalloutType::Success => "> ✅ ",
                CalloutType::Note => "> 📝 ",
            };
            let mut md = String::new();
            if let Some(t) = title {
                md.push_str(&format!("{}**{}**\n", prefix, t));
            }
            md.push_str(&format!("{}{}\n", prefix, text));
            md
        }
        ContentBlock::Code { language, content } => {
            format!("```{}\n{}\n```\n", language.as_deref().unwrap_or(""), content)
        }
        ContentBlock::Finding { id, title, severity, description, impact, recommendation } => {
            format!(
                "### {} - {} [{}]\n\n**Description:** {}\n\n**Impact:** {}\n\n**Recommendation:** {}\n",
                id, title, severity, description, impact, recommendation
            )
        }
        ContentBlock::Metric { label, value, .. } => format!("**{}:** {}\n", label, value),
        ContentBlock::PageBreak => "\n---\n".to_string(),
    }
}
