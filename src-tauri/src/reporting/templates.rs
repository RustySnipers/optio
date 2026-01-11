//! Report Templates
//!
//! Pre-defined templates for different report types with
//! customizable sections and content.

use super::models::*;

/// Get all available report templates
pub fn get_report_templates() -> Vec<ReportTemplate> {
    vec![
        get_executive_summary_template(),
        get_technical_assessment_template(),
        get_compliance_report_template(),
        get_network_assessment_template(),
        get_cloud_readiness_template(),
        get_security_findings_template(),
        get_full_engagement_template(),
    ]
}

/// Get a specific template by report type
pub fn get_template_for_type(report_type: ReportType) -> ReportTemplate {
    match report_type {
        ReportType::ExecutiveSummary => get_executive_summary_template(),
        ReportType::TechnicalAssessment => get_technical_assessment_template(),
        ReportType::ComplianceReport => get_compliance_report_template(),
        ReportType::NetworkAssessment => get_network_assessment_template(),
        ReportType::CloudReadiness => get_cloud_readiness_template(),
        ReportType::SecurityFindings => get_security_findings_template(),
        ReportType::FullEngagement => get_full_engagement_template(),
    }
}

fn get_executive_summary_template() -> ReportTemplate {
    ReportTemplate {
        id: "exec-summary-v1".to_string(),
        name: "Executive Summary".to_string(),
        description: "High-level overview for executives with key findings and strategic recommendations".to_string(),
        report_type: ReportType::ExecutiveSummary,
        sections: vec![
            TemplateSectionDef {
                id: "exec-overview".to_string(),
                title: "Executive Overview".to_string(),
                description: "High-level summary of the engagement".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "key-findings".to_string(),
                title: "Key Findings".to_string(),
                description: "Critical findings and risk summary".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "risk-summary".to_string(),
                title: "Risk Summary".to_string(),
                description: "Overall risk assessment by domain".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "recommendations".to_string(),
                title: "Strategic Recommendations".to_string(),
                description: "Top priority recommendations".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "next-steps".to_string(),
                title: "Next Steps".to_string(),
                description: "Proposed action items and timeline".to_string(),
                required: false,
                default_included: true,
            },
        ],
        default_config: ReportConfig {
            report_type: ReportType::ExecutiveSummary,
            title: "Executive Summary Report".to_string(),
            include_toc: false,
            include_executive_summary: false,
            include_appendices: false,
            include_charts: true,
            ..Default::default()
        },
    }
}

fn get_technical_assessment_template() -> ReportTemplate {
    ReportTemplate {
        id: "tech-assessment-v1".to_string(),
        name: "Technical Assessment".to_string(),
        description: "Detailed technical findings with vulnerability analysis and remediation guidance".to_string(),
        report_type: ReportType::TechnicalAssessment,
        sections: vec![
            TemplateSectionDef {
                id: "tech-overview".to_string(),
                title: "Assessment Overview".to_string(),
                description: "Scope, methodology, and objectives".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "methodology".to_string(),
                title: "Methodology".to_string(),
                description: "Testing methodology and tools used".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "findings-summary".to_string(),
                title: "Findings Summary".to_string(),
                description: "Overview of all findings by severity".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "detailed-findings".to_string(),
                title: "Detailed Findings".to_string(),
                description: "Individual findings with technical details".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "remediation".to_string(),
                title: "Remediation Roadmap".to_string(),
                description: "Prioritized remediation plan".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "appendix-tools".to_string(),
                title: "Appendix: Tools & Techniques".to_string(),
                description: "Tools and commands used during testing".to_string(),
                required: false,
                default_included: true,
            },
            TemplateSectionDef {
                id: "appendix-evidence".to_string(),
                title: "Appendix: Evidence".to_string(),
                description: "Supporting evidence and screenshots".to_string(),
                required: false,
                default_included: true,
            },
        ],
        default_config: ReportConfig {
            report_type: ReportType::TechnicalAssessment,
            title: "Technical Security Assessment".to_string(),
            include_toc: true,
            include_executive_summary: true,
            include_appendices: true,
            include_charts: true,
            ..Default::default()
        },
    }
}

fn get_compliance_report_template() -> ReportTemplate {
    ReportTemplate {
        id: "compliance-v1".to_string(),
        name: "Compliance Report".to_string(),
        description: "Framework compliance status, control assessments, and gap analysis".to_string(),
        report_type: ReportType::ComplianceReport,
        sections: vec![
            TemplateSectionDef {
                id: "compliance-overview".to_string(),
                title: "Compliance Overview".to_string(),
                description: "Assessment scope and framework summary".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "framework-status".to_string(),
                title: "Framework Compliance Status".to_string(),
                description: "Compliance scores by framework".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "control-matrix".to_string(),
                title: "Control Assessment Matrix".to_string(),
                description: "Detailed control-by-control assessment".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "gap-analysis".to_string(),
                title: "Gap Analysis".to_string(),
                description: "Identified gaps and deficiencies".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "remediation-plan".to_string(),
                title: "Remediation Plan".to_string(),
                description: "Plan to address compliance gaps".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "evidence-summary".to_string(),
                title: "Evidence Summary".to_string(),
                description: "Summary of collected evidence".to_string(),
                required: false,
                default_included: true,
            },
        ],
        default_config: ReportConfig {
            report_type: ReportType::ComplianceReport,
            title: "Compliance Assessment Report".to_string(),
            include_toc: true,
            include_executive_summary: true,
            include_appendices: true,
            include_charts: true,
            ..Default::default()
        },
    }
}

fn get_network_assessment_template() -> ReportTemplate {
    ReportTemplate {
        id: "network-v1".to_string(),
        name: "Network Assessment".to_string(),
        description: "Network topology, asset inventory, and infrastructure analysis".to_string(),
        report_type: ReportType::NetworkAssessment,
        sections: vec![
            TemplateSectionDef {
                id: "network-overview".to_string(),
                title: "Network Overview".to_string(),
                description: "Scope and discovery methodology".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "topology".to_string(),
                title: "Network Topology".to_string(),
                description: "Network architecture and segments".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "asset-inventory".to_string(),
                title: "Asset Inventory".to_string(),
                description: "Discovered assets by category".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "service-analysis".to_string(),
                title: "Service Analysis".to_string(),
                description: "Running services and protocols".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "security-posture".to_string(),
                title: "Security Posture".to_string(),
                description: "Network security observations".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "recommendations".to_string(),
                title: "Recommendations".to_string(),
                description: "Network hardening recommendations".to_string(),
                required: true,
                default_included: true,
            },
        ],
        default_config: ReportConfig {
            report_type: ReportType::NetworkAssessment,
            title: "Network Assessment Report".to_string(),
            include_toc: true,
            include_executive_summary: true,
            include_appendices: true,
            include_charts: true,
            ..Default::default()
        },
    }
}

fn get_cloud_readiness_template() -> ReportTemplate {
    ReportTemplate {
        id: "cloud-readiness-v1".to_string(),
        name: "Cloud Readiness".to_string(),
        description: "Cloud migration readiness assessment with cost projections".to_string(),
        report_type: ReportType::CloudReadiness,
        sections: vec![
            TemplateSectionDef {
                id: "cloud-overview".to_string(),
                title: "Assessment Overview".to_string(),
                description: "Migration goals and scope".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "readiness-score".to_string(),
                title: "Readiness Assessment".to_string(),
                description: "Overall readiness score and breakdown".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "workload-analysis".to_string(),
                title: "Workload Analysis".to_string(),
                description: "Application and workload assessment".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "migration-strategy".to_string(),
                title: "Migration Strategy".to_string(),
                description: "Recommended migration approach (6 Rs)".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "cost-analysis".to_string(),
                title: "Cost Analysis".to_string(),
                description: "TCO comparison and cost projections".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "roadmap".to_string(),
                title: "Migration Roadmap".to_string(),
                description: "Phased migration plan".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "risks".to_string(),
                title: "Risks & Mitigations".to_string(),
                description: "Migration risks and mitigation strategies".to_string(),
                required: false,
                default_included: true,
            },
        ],
        default_config: ReportConfig {
            report_type: ReportType::CloudReadiness,
            title: "Cloud Readiness Assessment".to_string(),
            include_toc: true,
            include_executive_summary: true,
            include_appendices: false,
            include_charts: true,
            ..Default::default()
        },
    }
}

fn get_security_findings_template() -> ReportTemplate {
    ReportTemplate {
        id: "security-findings-v1".to_string(),
        name: "Security Findings".to_string(),
        description: "Security vulnerabilities, risk ratings, and prioritized remediation".to_string(),
        report_type: ReportType::SecurityFindings,
        sections: vec![
            TemplateSectionDef {
                id: "findings-overview".to_string(),
                title: "Findings Overview".to_string(),
                description: "Summary of security findings".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "critical-findings".to_string(),
                title: "Critical Findings".to_string(),
                description: "Critical severity findings".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "high-findings".to_string(),
                title: "High Findings".to_string(),
                description: "High severity findings".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "medium-findings".to_string(),
                title: "Medium Findings".to_string(),
                description: "Medium severity findings".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "low-findings".to_string(),
                title: "Low & Informational".to_string(),
                description: "Low severity and informational findings".to_string(),
                required: false,
                default_included: true,
            },
            TemplateSectionDef {
                id: "remediation-priority".to_string(),
                title: "Remediation Priority".to_string(),
                description: "Prioritized remediation plan".to_string(),
                required: true,
                default_included: true,
            },
        ],
        default_config: ReportConfig {
            report_type: ReportType::SecurityFindings,
            title: "Security Findings Report".to_string(),
            include_toc: true,
            include_executive_summary: true,
            include_appendices: true,
            include_charts: true,
            ..Default::default()
        },
    }
}

fn get_full_engagement_template() -> ReportTemplate {
    ReportTemplate {
        id: "full-engagement-v1".to_string(),
        name: "Full Engagement Report".to_string(),
        description: "Comprehensive report combining all assessment modules".to_string(),
        report_type: ReportType::FullEngagement,
        sections: vec![
            TemplateSectionDef {
                id: "engagement-overview".to_string(),
                title: "Engagement Overview".to_string(),
                description: "Full scope and objectives".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "executive-summary".to_string(),
                title: "Executive Summary".to_string(),
                description: "High-level summary for executives".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "compliance-assessment".to_string(),
                title: "Compliance Assessment".to_string(),
                description: "Framework compliance analysis".to_string(),
                required: false,
                default_included: true,
            },
            TemplateSectionDef {
                id: "network-assessment".to_string(),
                title: "Network Assessment".to_string(),
                description: "Network and asset analysis".to_string(),
                required: false,
                default_included: true,
            },
            TemplateSectionDef {
                id: "security-findings".to_string(),
                title: "Security Findings".to_string(),
                description: "Detailed security findings".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "cloud-readiness".to_string(),
                title: "Cloud Readiness".to_string(),
                description: "Cloud migration assessment".to_string(),
                required: false,
                default_included: false,
            },
            TemplateSectionDef {
                id: "remediation-roadmap".to_string(),
                title: "Remediation Roadmap".to_string(),
                description: "Comprehensive remediation plan".to_string(),
                required: true,
                default_included: true,
            },
            TemplateSectionDef {
                id: "appendices".to_string(),
                title: "Appendices".to_string(),
                description: "Supporting documentation".to_string(),
                required: false,
                default_included: true,
            },
        ],
        default_config: ReportConfig {
            report_type: ReportType::FullEngagement,
            title: "Security Engagement Report".to_string(),
            include_toc: true,
            include_executive_summary: true,
            include_appendices: true,
            include_charts: true,
            ..Default::default()
        },
    }
}

/// Get report type info for UI display
pub fn get_report_type_info() -> Vec<ReportTypeInfo> {
    vec![
        ReportTypeInfo {
            report_type: ReportType::ExecutiveSummary,
            name: ReportType::ExecutiveSummary.display_name().to_string(),
            description: ReportType::ExecutiveSummary.description().to_string(),
            estimated_pages: ReportType::ExecutiveSummary.estimated_pages().to_string(),
            icon: "file-text".to_string(),
        },
        ReportTypeInfo {
            report_type: ReportType::TechnicalAssessment,
            name: ReportType::TechnicalAssessment.display_name().to_string(),
            description: ReportType::TechnicalAssessment.description().to_string(),
            estimated_pages: ReportType::TechnicalAssessment.estimated_pages().to_string(),
            icon: "code".to_string(),
        },
        ReportTypeInfo {
            report_type: ReportType::ComplianceReport,
            name: ReportType::ComplianceReport.display_name().to_string(),
            description: ReportType::ComplianceReport.description().to_string(),
            estimated_pages: ReportType::ComplianceReport.estimated_pages().to_string(),
            icon: "shield-check".to_string(),
        },
        ReportTypeInfo {
            report_type: ReportType::NetworkAssessment,
            name: ReportType::NetworkAssessment.display_name().to_string(),
            description: ReportType::NetworkAssessment.description().to_string(),
            estimated_pages: ReportType::NetworkAssessment.estimated_pages().to_string(),
            icon: "network".to_string(),
        },
        ReportTypeInfo {
            report_type: ReportType::CloudReadiness,
            name: ReportType::CloudReadiness.display_name().to_string(),
            description: ReportType::CloudReadiness.description().to_string(),
            estimated_pages: ReportType::CloudReadiness.estimated_pages().to_string(),
            icon: "cloud".to_string(),
        },
        ReportTypeInfo {
            report_type: ReportType::SecurityFindings,
            name: ReportType::SecurityFindings.display_name().to_string(),
            description: ReportType::SecurityFindings.description().to_string(),
            estimated_pages: ReportType::SecurityFindings.estimated_pages().to_string(),
            icon: "alert-triangle".to_string(),
        },
        ReportTypeInfo {
            report_type: ReportType::FullEngagement,
            name: ReportType::FullEngagement.display_name().to_string(),
            description: ReportType::FullEngagement.description().to_string(),
            estimated_pages: ReportType::FullEngagement.estimated_pages().to_string(),
            icon: "file-stack".to_string(),
        },
    ]
}

/// Report type info for frontend display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportTypeInfo {
    pub report_type: ReportType,
    pub name: String,
    pub description: String,
    pub estimated_pages: String,
    pub icon: String,
}

/// Get export format options
pub fn get_export_formats() -> Vec<ExportFormatInfo> {
    vec![
        ExportFormatInfo {
            format: ExportFormat::Pdf,
            name: "PDF".to_string(),
            description: "Professional PDF document".to_string(),
            icon: "file-pdf".to_string(),
        },
        ExportFormatInfo {
            format: ExportFormat::Html,
            name: "HTML".to_string(),
            description: "Web-viewable HTML document".to_string(),
            icon: "file-code".to_string(),
        },
        ExportFormatInfo {
            format: ExportFormat::Markdown,
            name: "Markdown".to_string(),
            description: "Plain text Markdown format".to_string(),
            icon: "file-text".to_string(),
        },
        ExportFormatInfo {
            format: ExportFormat::Json,
            name: "JSON".to_string(),
            description: "Structured JSON data".to_string(),
            icon: "file-json".to_string(),
        },
    ]
}

/// Export format info for frontend display
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ExportFormatInfo {
    pub format: ExportFormat,
    pub name: String,
    pub description: String,
    pub icon: String,
}
