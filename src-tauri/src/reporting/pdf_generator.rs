//! PDF Generation Module
//!
//! Generates PDF reports using the printpdf library.
//! Supports executive summaries with Network Health Score, Compliance Status, and Assets.

use printpdf::*;
use std::fs::File;
use std::io::BufWriter;
use std::path::PathBuf;

use crate::grc::models::{ComplianceStatusReport, ExecutiveReportData};

/// PDF Generator for executive reports
pub struct PdfGenerator {
    /// Document title
    title: String,
    /// Primary brand color (hex)
    primary_color: String,
}

impl PdfGenerator {
    pub fn new(title: String) -> Self {
        Self {
            title,
            primary_color: "#3B82F6".to_string(),
        }
    }

    pub fn with_primary_color(mut self, color: String) -> Self {
        self.primary_color = color;
        self
    }

    /// Generate an executive report PDF
    pub fn generate_executive_report(
        &self,
        data: &ExecutiveReportData,
        output_path: &PathBuf,
    ) -> Result<u64, String> {
        // Create PDF document (Letter size: 215.9mm x 279.4mm)
        let (doc, page1, layer1) = PdfDocument::new(
            &self.title,
            Mm(215.9),
            Mm(279.4),
            "Cover",
        );

        let font = doc.add_builtin_font(BuiltinFont::HelveticaBold)
            .map_err(|e| format!("Failed to add font: {}", e))?;
        let font_regular = doc.add_builtin_font(BuiltinFont::Helvetica)
            .map_err(|e| format!("Failed to add font: {}", e))?;

        // Cover page
        let cover_layer = doc.get_page(page1).get_layer(layer1);
        self.draw_cover_page(&cover_layer, &font, &font_regular, data)?;

        // Executive Summary page
        let (page2, layer2) = doc.add_page(Mm(215.9), Mm(279.4), "Executive Summary");
        let summary_layer = doc.get_page(page2).get_layer(layer2);
        self.draw_executive_summary(&summary_layer, &font, &font_regular, data)?;

        // Compliance Status page
        let (page3, layer3) = doc.add_page(Mm(215.9), Mm(279.4), "Compliance Status");
        let compliance_layer = doc.get_page(page3).get_layer(layer3);
        self.draw_compliance_status(&compliance_layer, &font, &font_regular, data)?;

        // Network & Assets page
        let (page4, layer4) = doc.add_page(Mm(215.9), Mm(279.4), "Network Assets");
        let assets_layer = doc.get_page(page4).get_layer(layer4);
        self.draw_network_assets(&assets_layer, &font, &font_regular, data)?;

        // Recommendations page
        let (page5, layer5) = doc.add_page(Mm(215.9), Mm(279.4), "Recommendations");
        let recommendations_layer = doc.get_page(page5).get_layer(layer5);
        self.draw_recommendations(&recommendations_layer, &font, &font_regular, data)?;

        // Save PDF
        let file = File::create(output_path)
            .map_err(|e| format!("Failed to create file: {}", e))?;
        let mut writer = BufWriter::new(file);
        doc.save(&mut writer)
            .map_err(|e| format!("Failed to save PDF: {}", e))?;

        // Get file size
        let metadata = std::fs::metadata(output_path)
            .map_err(|e| format!("Failed to get file metadata: {}", e))?;

        Ok(metadata.len())
    }

    fn draw_cover_page(
        &self,
        layer: &PdfLayerReference,
        font_bold: &IndirectFontRef,
        font_regular: &IndirectFontRef,
        data: &ExecutiveReportData,
    ) -> Result<(), String> {
        // Title
        layer.use_text(&data.title, 28.0, Mm(25.0), Mm(230.0), font_bold);

        // Client name
        layer.use_text(
            &format!("Prepared for: {}", data.client_name),
            14.0,
            Mm(25.0),
            Mm(200.0),
            font_regular,
        );

        // Date
        layer.use_text(
            &format!("Date: {}", data.report_date),
            12.0,
            Mm(25.0),
            Mm(185.0),
            font_regular,
        );

        // Key metrics box
        layer.use_text("Executive Overview", 18.0, Mm(25.0), Mm(150.0), font_bold);

        // Network Health Score
        layer.use_text(
            &format!("Network Health Score: {:.0}%", data.network_health_score),
            14.0,
            Mm(30.0),
            Mm(130.0),
            font_regular,
        );

        // Compliance Score
        if let Some(ref compliance) = data.compliance_status {
            layer.use_text(
                &format!("Compliance Score: {:.1}%", compliance.compliance_percentage),
                14.0,
                Mm(30.0),
                Mm(115.0),
                font_regular,
            );
        }

        // Total Assets
        layer.use_text(
            &format!("Total Assets Discovered: {}", data.total_assets),
            14.0,
            Mm(30.0),
            Mm(100.0),
            font_regular,
        );

        // Risk Summary
        layer.use_text("Risk Summary", 18.0, Mm(25.0), Mm(70.0), font_bold);
        layer.use_text(
            &format!(
                "Critical: {} | High: {} | Medium: {} | Low: {}",
                data.risk_summary.critical_count,
                data.risk_summary.high_count,
                data.risk_summary.medium_count,
                data.risk_summary.low_count
            ),
            12.0,
            Mm(30.0),
            Mm(55.0),
            font_regular,
        );
        layer.use_text(
            &format!("Overall Risk Rating: {}", data.risk_summary.overall_risk_rating),
            12.0,
            Mm(30.0),
            Mm(42.0),
            font_regular,
        );

        // Classification footer
        layer.use_text(
            "CONFIDENTIAL",
            10.0,
            Mm(90.0),
            Mm(15.0),
            font_bold,
        );

        Ok(())
    }

    fn draw_executive_summary(
        &self,
        layer: &PdfLayerReference,
        font_bold: &IndirectFontRef,
        font_regular: &IndirectFontRef,
        data: &ExecutiveReportData,
    ) -> Result<(), String> {
        // Header
        layer.use_text("Executive Summary", 22.0, Mm(25.0), Mm(265.0), font_bold);

        // Introduction
        layer.use_text("Assessment Overview", 14.0, Mm(25.0), Mm(245.0), font_bold);
        layer.use_text(
            &format!(
                "This assessment was conducted for {} to evaluate the organization's",
                data.client_name
            ),
            11.0,
            Mm(25.0),
            Mm(232.0),
            font_regular,
        );
        layer.use_text(
            "cybersecurity posture, compliance status, and network infrastructure.",
            11.0,
            Mm(25.0),
            Mm(222.0),
            font_regular,
        );

        // Key Findings
        layer.use_text("Key Findings", 14.0, Mm(25.0), Mm(200.0), font_bold);

        let mut y_pos = 185.0;
        for (i, finding) in data.top_findings.iter().take(5).enumerate() {
            layer.use_text(
                &format!("{}. [{}] {}", i + 1, finding.severity, finding.title),
                10.0,
                Mm(30.0),
                Mm(y_pos),
                font_regular,
            );
            y_pos -= 12.0;
        }

        // Metrics summary
        layer.use_text("Security Metrics", 14.0, Mm(25.0), Mm(100.0), font_bold);

        layer.use_text(
            &format!("- Network Health Score: {:.0}/100", data.network_health_score),
            11.0,
            Mm(30.0),
            Mm(85.0),
            font_regular,
        );

        if let Some(ref compliance) = data.compliance_status {
            layer.use_text(
                &format!(
                    "- Framework Compliance: {:.1}% ({} of {} controls assessed)",
                    compliance.compliance_percentage,
                    compliance.assessed_controls,
                    compliance.total_controls
                ),
                11.0,
                Mm(30.0),
                Mm(72.0),
                font_regular,
            );
        }

        layer.use_text(
            &format!("- Assets Discovered: {} across {} categories",
                data.total_assets,
                data.assets_by_category.len()
            ),
            11.0,
            Mm(30.0),
            Mm(59.0),
            font_regular,
        );

        // Page footer
        layer.use_text("Page 2", 10.0, Mm(100.0), Mm(15.0), font_regular);

        Ok(())
    }

    fn draw_compliance_status(
        &self,
        layer: &PdfLayerReference,
        font_bold: &IndirectFontRef,
        font_regular: &IndirectFontRef,
        data: &ExecutiveReportData,
    ) -> Result<(), String> {
        layer.use_text("Compliance Status", 22.0, Mm(25.0), Mm(265.0), font_bold);

        if let Some(ref compliance) = data.compliance_status {
            layer.use_text(
                &format!("Framework: {}", compliance.framework.display_name()),
                14.0,
                Mm(25.0),
                Mm(245.0),
                font_bold,
            );

            // Overall metrics
            layer.use_text("Overall Compliance Metrics", 12.0, Mm(25.0), Mm(225.0), font_bold);
            layer.use_text(
                &format!("Completion: {:.1}%", compliance.completion_percentage),
                11.0,
                Mm(30.0),
                Mm(212.0),
                font_regular,
            );
            layer.use_text(
                &format!("Compliance: {:.1}%", compliance.compliance_percentage),
                11.0,
                Mm(30.0),
                Mm(200.0),
                font_regular,
            );
            layer.use_text(
                &format!("Total Controls: {}", compliance.total_controls),
                11.0,
                Mm(30.0),
                Mm(188.0),
                font_regular,
            );
            layer.use_text(
                &format!("Assessed: {} | Compliant: {} | Partial: {} | Non-Compliant: {}",
                    compliance.assessed_controls,
                    compliance.compliant_controls,
                    compliance.partially_compliant_controls,
                    compliance.non_compliant_controls
                ),
                10.0,
                Mm(30.0),
                Mm(176.0),
                font_regular,
            );

            // Category breakdown
            layer.use_text("Compliance by Category", 12.0, Mm(25.0), Mm(155.0), font_bold);

            let mut y_pos = 140.0;
            for cat in &compliance.category_breakdown {
                layer.use_text(
                    &format!(
                        "{} ({}): {:.1}% compliance ({}/{} controls)",
                        cat.name,
                        cat.code,
                        cat.compliance_percentage,
                        cat.compliant + cat.partially_compliant,
                        cat.total_controls
                    ),
                    10.0,
                    Mm(30.0),
                    Mm(y_pos),
                    font_regular,
                );
                y_pos -= 11.0;
                if y_pos < 30.0 {
                    break;
                }
            }
        } else {
            layer.use_text(
                "No compliance assessment data available.",
                12.0,
                Mm(25.0),
                Mm(245.0),
                font_regular,
            );
        }

        layer.use_text("Page 3", 10.0, Mm(100.0), Mm(15.0), font_regular);

        Ok(())
    }

    fn draw_network_assets(
        &self,
        layer: &PdfLayerReference,
        font_bold: &IndirectFontRef,
        font_regular: &IndirectFontRef,
        data: &ExecutiveReportData,
    ) -> Result<(), String> {
        layer.use_text("Network Assets", 22.0, Mm(25.0), Mm(265.0), font_bold);

        layer.use_text(
            &format!("Total Assets Discovered: {}", data.total_assets),
            14.0,
            Mm(25.0),
            Mm(245.0),
            font_bold,
        );

        layer.use_text(
            &format!("Network Health Score: {:.0}%", data.network_health_score),
            12.0,
            Mm(25.0),
            Mm(230.0),
            font_regular,
        );

        // Assets by category
        layer.use_text("Assets by Category", 12.0, Mm(25.0), Mm(210.0), font_bold);

        let mut y_pos = 195.0;
        for cat in &data.assets_by_category {
            layer.use_text(
                &format!("{}: {} assets", cat.category, cat.count),
                11.0,
                Mm(30.0),
                Mm(y_pos),
                font_regular,
            );
            y_pos -= 12.0;
            if y_pos < 30.0 {
                break;
            }
        }

        layer.use_text("Page 4", 10.0, Mm(100.0), Mm(15.0), font_regular);

        Ok(())
    }

    fn draw_recommendations(
        &self,
        layer: &PdfLayerReference,
        font_bold: &IndirectFontRef,
        font_regular: &IndirectFontRef,
        data: &ExecutiveReportData,
    ) -> Result<(), String> {
        layer.use_text("Recommendations", 22.0, Mm(25.0), Mm(265.0), font_bold);

        layer.use_text("Strategic Recommendations", 14.0, Mm(25.0), Mm(245.0), font_bold);

        let mut y_pos = 228.0;
        for (i, finding) in data.top_findings.iter().enumerate() {
            if y_pos < 50.0 {
                break;
            }

            layer.use_text(
                &format!("{}. {}", i + 1, finding.title),
                11.0,
                Mm(25.0),
                Mm(y_pos),
                font_bold,
            );
            y_pos -= 12.0;

            layer.use_text(
                &format!("Severity: {}", finding.severity),
                10.0,
                Mm(30.0),
                Mm(y_pos),
                font_regular,
            );
            y_pos -= 10.0;

            // Wrap recommendation text
            let rec_text = &finding.recommendation;
            if rec_text.len() > 80 {
                layer.use_text(
                    &rec_text[..80],
                    10.0,
                    Mm(30.0),
                    Mm(y_pos),
                    font_regular,
                );
                y_pos -= 10.0;
                if rec_text.len() > 80 {
                    layer.use_text(
                        &rec_text[80..rec_text.len().min(160)],
                        10.0,
                        Mm(30.0),
                        Mm(y_pos),
                        font_regular,
                    );
                }
            } else {
                layer.use_text(
                    rec_text,
                    10.0,
                    Mm(30.0),
                    Mm(y_pos),
                    font_regular,
                );
            }
            y_pos -= 18.0;
        }

        layer.use_text("Page 5", 10.0, Mm(100.0), Mm(15.0), font_regular);

        Ok(())
    }
}

/// Generate a demo executive report for testing
pub fn generate_demo_executive_report(
    client_name: &str,
    output_path: &PathBuf,
) -> Result<u64, String> {
    use crate::grc::models::{
        AssetCategoryCount, CategoryComplianceStatus, ExecutiveFinding,
        Framework, RiskSummary,
    };

    let compliance_status = ComplianceStatusReport {
        framework: Framework::NistCsf2,
        completion_percentage: 75.0,
        compliance_percentage: 82.5,
        total_controls: 27,
        assessed_controls: 20,
        compliant_controls: 14,
        partially_compliant_controls: 4,
        non_compliant_controls: 2,
        not_applicable_controls: 0,
        category_breakdown: vec![
            CategoryComplianceStatus {
                code: "GV".to_string(),
                name: "Govern".to_string(),
                description: "Establish and monitor cybersecurity risk management".to_string(),
                color: "#8b5cf6".to_string(),
                total_controls: 4,
                assessed_controls: 3,
                compliant: 2,
                partially_compliant: 1,
                non_compliant: 0,
                completion_percentage: 75.0,
                compliance_percentage: 83.3,
            },
            CategoryComplianceStatus {
                code: "ID".to_string(),
                name: "Identify".to_string(),
                description: "Understand cybersecurity risk posture".to_string(),
                color: "#3b82f6".to_string(),
                total_controls: 5,
                assessed_controls: 4,
                compliant: 3,
                partially_compliant: 1,
                non_compliant: 0,
                completion_percentage: 80.0,
                compliance_percentage: 87.5,
            },
            CategoryComplianceStatus {
                code: "PR".to_string(),
                name: "Protect".to_string(),
                description: "Use safeguards to manage risks".to_string(),
                color: "#22c55e".to_string(),
                total_controls: 7,
                assessed_controls: 5,
                compliant: 3,
                partially_compliant: 1,
                non_compliant: 1,
                completion_percentage: 71.4,
                compliance_percentage: 70.0,
            },
            CategoryComplianceStatus {
                code: "DE".to_string(),
                name: "Detect".to_string(),
                description: "Find and analyze cybersecurity attacks".to_string(),
                color: "#f59e0b".to_string(),
                total_controls: 4,
                assessed_controls: 3,
                compliant: 2,
                partially_compliant: 1,
                non_compliant: 0,
                completion_percentage: 75.0,
                compliance_percentage: 83.3,
            },
            CategoryComplianceStatus {
                code: "RS".to_string(),
                name: "Respond".to_string(),
                description: "Take action regarding incidents".to_string(),
                color: "#ef4444".to_string(),
                total_controls: 3,
                assessed_controls: 2,
                compliant: 2,
                partially_compliant: 0,
                non_compliant: 0,
                completion_percentage: 66.7,
                compliance_percentage: 100.0,
            },
            CategoryComplianceStatus {
                code: "RC".to_string(),
                name: "Recover".to_string(),
                description: "Restore assets and operations".to_string(),
                color: "#06b6d4".to_string(),
                total_controls: 3,
                assessed_controls: 3,
                compliant: 2,
                partially_compliant: 0,
                non_compliant: 1,
                completion_percentage: 100.0,
                compliance_percentage: 66.7,
            },
        ],
        network_health_score: Some(78.0),
        total_assets: Some(247),
        last_updated: chrono::Utc::now().to_rfc3339(),
    };

    let data = ExecutiveReportData {
        client_name: client_name.to_string(),
        title: format!("Security Assessment Report - {}", client_name),
        report_date: chrono::Utc::now().format("%B %d, %Y").to_string(),
        compliance_status: Some(compliance_status),
        network_health_score: 78.0,
        total_assets: 247,
        assets_by_category: vec![
            AssetCategoryCount { category: "Servers".to_string(), count: 42 },
            AssetCategoryCount { category: "Workstations".to_string(), count: 156 },
            AssetCategoryCount { category: "Network Devices".to_string(), count: 28 },
            AssetCategoryCount { category: "Security Appliances".to_string(), count: 12 },
            AssetCategoryCount { category: "Other".to_string(), count: 9 },
        ],
        top_findings: vec![
            ExecutiveFinding {
                id: "FIND-001".to_string(),
                title: "Outdated TLS Configuration".to_string(),
                severity: "High".to_string(),
                description: "Multiple servers using deprecated TLS 1.0/1.1".to_string(),
                recommendation: "Disable TLS 1.0/1.1 and enable TLS 1.2+ with strong cipher suites".to_string(),
            },
            ExecutiveFinding {
                id: "FIND-002".to_string(),
                title: "Missing Multi-Factor Authentication".to_string(),
                severity: "Critical".to_string(),
                description: "Remote access systems lack MFA implementation".to_string(),
                recommendation: "Implement MFA for all remote access and privileged accounts".to_string(),
            },
            ExecutiveFinding {
                id: "FIND-003".to_string(),
                title: "Inadequate Network Segmentation".to_string(),
                severity: "Medium".to_string(),
                description: "Flat network architecture increases lateral movement risk".to_string(),
                recommendation: "Implement network segmentation with appropriate ACLs".to_string(),
            },
            ExecutiveFinding {
                id: "FIND-004".to_string(),
                title: "Incomplete Asset Inventory".to_string(),
                severity: "Medium".to_string(),
                description: "15% of discovered assets not in official CMDB".to_string(),
                recommendation: "Deploy automated asset discovery and maintain accurate CMDB".to_string(),
            },
        ],
        risk_summary: RiskSummary {
            critical_count: 2,
            high_count: 5,
            medium_count: 12,
            low_count: 8,
            overall_risk_rating: "Moderate".to_string(),
        },
    };

    let generator = PdfGenerator::new(data.title.clone());
    generator.generate_executive_report(&data, output_path)
}
